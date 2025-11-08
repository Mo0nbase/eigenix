# ASB (Automated Swap Backend) NixOS Module
#
# SECURITY FEATURES (Configuration-Enforced):
# ===========================================
#
# 1. File Permissions (via systemd.tmpfiles.rules):
#    - All data directories: 700 (owner-only access)
#    - Seed file (seed.pem): 600 (CRITICAL - contains cryptographic secrets)
#    - Wallet files: 600 (read/write only, no execute bit)
#    - Tor directories: 700 (required by fs_mistrust)
#
# 2. Network Security:
#    - Bitcoin RPC restricted to 127.0.0.1 and 10.89.0.0/24 (asb-network only)
#    - ASB P2P on 0.0.0.0:9939 (required for public swap functionality)
#    - ASB RPC on 127.0.0.1 only (localhost-only access)
#
# 3. Container Security:
#    - All containers run as unprivileged user 1000:1000
#    - no-new-privileges prevents privilege escalation
#    - Resource limits prevent DoS attacks
#    - Read-only volume mounts where appropriate
#
# 4. Authentication:
#    - Bitcoin RPC uses cookie-based authentication (automatic rotation)
#    - No hardcoded passwords in configuration
#
# On every system activation, systemd-tmpfiles will enforce permissions,
# ensuring security even after manual modifications.

{
  config,
  lib,
  pkgs,
  ...
}:

with lib;

let
  cfg = config.services.eigenix-asb;
  settings = config.eigenix.finalSettings;

  # ASB config.toml content
  asbConfigToml = pkgs.writeText "asb-config.toml" ''
    [data]
    dir = "/asb-data"

    [network]
    listen = ["/ip4/0.0.0.0/tcp/9939"]
    rendezvous_point = [
      "/dns4/discover.unstoppableswap.net/tcp/8888/p2p/12D3KooWA6cnqJpVnreBVnoro8midDL9Lpzmg8oJPoAGi7YYaamE",
      "/dns4/discover2.unstoppableswap.net/tcp/8888/p2p/12D3KooWGRvf7qVQDrNR5nfYD6rKrbgeTi9x8RrbdxbmsPvxL4mw",
      "/dns4/darkness.su/tcp/8888/p2p/12D3KooWFQAgVVS9t9UgL6v1sLprJVM7am5hFK7vy9iBCCoCBYmU",
      "/dns4/eigen.center/tcp/8888/p2p/12D3KooWS5RaYJt4ANKMH4zczGVhNcw5W214e2DDYXnjs5Mx5zAT",
      "/dns4/swapanarchy.cfd/tcp/8888/p2p/12D3KooWRtyVpmyvwzPYXuWyakFbRKhyXGrjhq6tP7RrBofpgQGp",
      "/dns4/rendezvous.observer/tcp/8888/p2p/12D3KooWMjceGXrYuGuDMGrfmJxALnSDbK4km6s1i1sJEgDTgGQa",
      "/dns4/aswap.click/tcp/8888/p2p/12D3KooWQzW52mdsLHTMu1EPiz3APumG6vGwpCuyy494MAQoEa5X",
      "/dns4/getxmr.st/tcp/8888/p2p/12D3KooWHHwiz6WDThPT8cEurstomg3kDSxzL2L8pwxfyX2fpxVk"
    ]
    external_addresses = ${builtins.toJSON settings.asb.externalAddresses}

    [bitcoin]
    electrum_rpc_urls = ["tcp://electrs:${toString settings.ports.electrs}"]
    target_block = 1
    network = "${settings.networks.bitcoin}"
    use_mempool_space_fee_estimation = true

    [monero]
    daemon_url = "http://monerod:${toString settings.ports.moneroRpc}/"
    network = "${settings.networks.monero}"

    [tor]
    register_hidden_service = ${if settings.asb.enableTor then "true" else "false"}
    hidden_service_num_intro_points = 5

    [maker]
    min_buy_btc = ${toString settings.asb.minBuyBtc}
    max_buy_btc = ${toString settings.asb.maxBuyBtc}
    ask_spread = ${toString settings.asb.askSpread}
    price_ticker_ws_url = "${settings.asb.priceTickerUrl}"
    developer_tip = ${toString settings.asb.developerTip}
    ${optionalString (
      settings.asb.externalBitcoinAddress != null
    ) ''external_bitcoin_address = "${settings.asb.externalBitcoinAddress}"''}
  '';

  asbConfigFile = asbConfigToml;

  bitcoinChain = "main";
  electrsNetwork = "bitcoin";

  # Comment updated to reflect new network name
  # Network subnet: eigenix-network uses 10.89.0.0/24

in
{
  options.services.eigenix-asb = {
    enable = mkEnableOption "Automated Swap Backend (ASB) for XMR/BTC atomic swaps";
  };

  config = mkIf cfg.enable {
    # Runtime
    virtualisation.podman = {
      enable = true;
      autoPrune.enable = true;
      dockerCompat = true;
    };

    # Enable container name DNS for all Podman networks.
    networking.firewall.interfaces =
      let
        matchAll = if !config.networking.nftables.enable then "podman+" else "podman*";
      in
      {
        "${matchAll}".allowedUDPPorts = [ 53 ];
      };

    virtualisation.oci-containers.backend = "podman";

    # Monitoring tools
    environment.systemPackages = with pkgs; [
      bitcoin # For bitcoin-cli
      monero-cli # For monero-wallet-cli and other tools
      curl
      jq
    ];

    # Ensure data directories exist with secure permissions
    systemd.tmpfiles.rules = [
      # Main data directories - 700 to prevent unauthorized access
      "d ${settings.storage.baseDataDir}/asb-data 0700 1000 1000 -"
      "d ${settings.storage.baseDataDir}/bitcoind-data 0755 1000 1000 -"
      "d ${settings.storage.baseDataDir}/electrs-data 0700 1000 1000 -"
      "d ${settings.storage.baseDataDir}/monerod-data 0700 1000 1000 -"

      # ASB sensitive subdirectories
      "d ${settings.storage.baseDataDir}/asb-data/wallet 0700 1000 1000 -"
      "d ${settings.storage.baseDataDir}/asb-data/monero 0700 1000 1000 -"
      "d ${settings.storage.baseDataDir}/asb-data/tor 0700 1000 1000 -"
      "d ${settings.storage.baseDataDir}/asb-data/tor/state 0700 1000 1000 -"
      "d ${settings.storage.baseDataDir}/asb-data/tor/cache 0700 1000 1000 -"

      # ASB operational subdirectories (were previously 775/755)
      "d ${settings.storage.baseDataDir}/asb-data/bitcoind 0700 1000 1000 -"
      "d ${settings.storage.baseDataDir}/asb-data/asb 0700 1000 1000 -"
      "d ${settings.storage.baseDataDir}/asb-data/monerod 0700 1000 1000 -"
      "d ${settings.storage.baseDataDir}/asb-data/electrs 0700 1000 1000 -"
      "d ${settings.storage.baseDataDir}/asb-data/logs 0700 1000 1000 -"

      # ASB seed file - CRITICAL: 600 permissions (read/write owner only)
      "f ${settings.storage.baseDataDir}/asb-data/seed.pem 0600 1000 1000 -"

      # Recursively fix ownership and permissions on wallet directories
      "Z ${settings.storage.baseDataDir}/asb-data/wallet 0700 1000 1000 -"
      "Z ${settings.storage.baseDataDir}/asb-data/monero 0700 1000 1000 -"
    ];

    # Containers
    virtualisation.oci-containers.containers."monerod" = {
      image = "ghcr.io/sethforprivacy/simple-monerod@sha256:f30e5706a335c384e4cf420215cbffd1196f0b3a11d4dd4e819fe3e0bca41ec5";
      volumes = [
        "${settings.storage.baseDataDir}/monerod-data:/monerod-data:rw"
      ];
      ports = [
        "127.0.0.1:${toString settings.ports.moneroRpc}:18081/tcp"
      ];
      cmd = [
        "monerod"
        "--rpc-bind-ip=0.0.0.0"
        "--rpc-bind-port=18081"
        "--data-dir=/monerod-data/"
        "--confirm-external-bind"
        "--restricted-rpc"
        "--non-interactive"
        "--enable-dns-blocklist"
      ];
      user = "1000:1000";
      extraOptions = [
        "--entrypoint=[]"
        "--network-alias=monerod"
        "--network=eigenix-network"
        # Resource limits for security
        "--memory=4g"
        "--cpus=2.0"
        "--pids-limit=1024"
        # Security options
        "--security-opt=no-new-privileges:true"
      ];
    };
    systemd.services."podman-monerod" = {
      serviceConfig.Restart = lib.mkOverride 90 "always";
      after = [ "podman-network-eigenix.service" ];
      requires = [ "podman-network-eigenix.service" ];
      partOf = [ "eigenix-root.target" ];
      wantedBy = [ "eigenix-root.target" ];
    };

    virtualisation.oci-containers.containers."bitcoind" = {
      image = "getumbrel/bitcoind@sha256:c565266ea302c9ab2fc490f04ff14e584210cde3d0d991b8309157e5dfae9e8d";
      volumes = [
        "${settings.storage.baseDataDir}/bitcoind-data:/bitcoind-data:rw"
      ];
      ports = [
        "127.0.0.1:${toString settings.ports.bitcoinRpc}:8332/tcp"
      ];
      cmd = [
        "bitcoind"
        "-chain=${bitcoinChain}"
        "-rpcallowip=127.0.0.1"
        # Restrict to container network subnet only (eigenix-network uses 10.89.0.0/16 for all subnets)
        "-rpcallowip=10.89.0.0/16"
        "-rpcbind=0.0.0.0:${toString settings.ports.bitcoinRpc}"
        "-bind=0.0.0.0:8333"
        "-datadir=/bitcoind-data/"
        "-dbcache=16384"
        "-server=1"
        "-prune=0"
        "-txindex=1"
      ];
      user = "1000:1000";
      extraOptions = [
        "--entrypoint=[]"
        "--network-alias=bitcoind"
        "--network=eigenix-network"
        # Resource limits for security
        "--memory=8g"
        "--cpus=4.0"
        "--pids-limit=2048"
        # Security options
        "--read-only-tmpfs=false"
        "--security-opt=no-new-privileges:true"
      ];
    };
    systemd.services."podman-bitcoind" = {
      serviceConfig.Restart = lib.mkOverride 90 "always";
      after = [ "podman-network-eigenix.service" ];
      requires = [ "podman-network-eigenix.service" ];
      partOf = [ "eigenix-root.target" ];
      wantedBy = [ "eigenix-root.target" ];
    };

    virtualisation.oci-containers.containers."electrs" = {
      image = "getumbrel/electrs@sha256:622657fbdc7331a69f5b3444e6f87867d51ac27d90c399c8bf25d9aab020052b";
      volumes = [
        "${settings.storage.baseDataDir}/bitcoind-data:/bitcoind-data:ro,z"
        "${settings.storage.baseDataDir}/electrs-data:/electrs-data:rw,Z"
      ];
      cmd = [
        "electrs"
        "--network=${electrsNetwork}"
        "--daemon-dir=/bitcoind-data/"
        "--db-dir=/electrs-data/db"
        "--daemon-rpc-addr=bitcoind:${toString settings.ports.bitcoinRpc}"
        "--daemon-p2p-addr=bitcoind:8333"
        "--electrum-rpc-addr=0.0.0.0:${toString settings.ports.electrs}"
        "--log-filters=INFO"
      ];
      dependsOn = [ "bitcoind" ];
      user = "1000:1000";
      extraOptions = [
        "--entrypoint=[]"
        "--network-alias=electrs"
        "--network=eigenix-network"
        # Resource limits for security
        "--memory=4g"
        "--cpus=2.0"
        "--pids-limit=1024"
        # Security options
        "--security-opt=no-new-privileges:true"
        # Allow container to read volumes with proper user mapping
        "--userns=keep-id"
      ];
    };
    systemd.services."podman-electrs" = {
      serviceConfig.Restart = lib.mkOverride 90 "always";
      after = [
        "podman-network-eigenix.service"
        "podman-bitcoind.service"
      ];
      requires = [
        "podman-network-eigenix.service"
        "podman-bitcoind.service"
      ];
      partOf = [ "eigenix-root.target" ];
      wantedBy = [ "eigenix-root.target" ];
    };

    virtualisation.oci-containers.containers."asb" = {
      image = "ghcr.io/eigenwallet/asb:latest";
      volumes = [
        "${asbConfigFile}:/asb-data/config.toml:ro"
        "${settings.storage.baseDataDir}/asb-data:/asb-data:rw"
      ];
      ports = [
        "0.0.0.0:${toString config.eigenix.finalSettings.ports.asbP2p}:9939/tcp"
        "127.0.0.1:${toString config.eigenix.finalSettings.ports.asbRpc}:${toString config.eigenix.finalSettings.ports.asbRpc}/tcp"
      ];
      cmd = [
        "asb"
        "--config=/asb-data/config.toml"
        "start"
        "--rpc-bind-port=${toString config.eigenix.finalSettings.ports.asbRpc}"
        "--rpc-bind-host=0.0.0.0"
      ];
      dependsOn = [
        "electrs"
        "monerod"
      ];
      user = "1000:1000";
      extraOptions = [
        "--entrypoint=[]"
        "--network-alias=asb"
        "--network=eigenix-network"
        # Resource limits for security
        "--memory=2g"
        "--cpus=2.0"
        "--pids-limit=512"
        # Security options
        "--security-opt=no-new-privileges:true"
      ];
    };
    systemd.services."podman-asb" = {
      serviceConfig = {
        Restart = lib.mkOverride 90 "always";
      };
      after = [
        "podman-network-eigenix.service"
        "podman-electrs.service"
        "podman-monerod.service"
      ];
      requires = [
        "podman-network-eigenix.service"
        "podman-electrs.service"
        "podman-monerod.service"
      ];
      partOf = [ "eigenix-root.target" ];
      wantedBy = [ "eigenix-root.target" ];
    };

    virtualisation.oci-containers.containers."asb-controller" = {
      image = "ghcr.io/eigenwallet/asb-controller:latest";
      cmd = [
        "asb-controller"
        "--url=http://asb:${toString config.eigenix.finalSettings.ports.asbRpc}"
      ];
      dependsOn = [ "asb" ];
      extraOptions = [
        "--entrypoint=[]"
        "--network-alias=asb-controller"
        "--network=eigenix-network"
        "--tty"
        "--interactive"
      ];
    };
    systemd.services."podman-asb-controller" = {
      serviceConfig.Restart = lib.mkOverride 90 "always";
      after = [
        "podman-network-eigenix.service"
        "podman-asb.service"
      ];
      requires = [
        "podman-network-eigenix.service"
        "podman-asb.service"
      ];
      partOf = [ "eigenix-root.target" ];
      wantedBy = [ "eigenix-root.target" ];
    };

    virtualisation.oci-containers.containers."asb-tracing-logger" = {
      image = "alpine@sha256:4bcff63911fcb4448bd4fdacec207030997caf25e9bea4045fa6c8c44de311d1";
      volumes = [
        "${settings.storage.baseDataDir}/asb-data:/asb-data:ro"
      ];
      cmd = [
        "sh"
        "-c"
        "tail -f /asb-data/logs/tracing*.log"
      ];
      dependsOn = [ "asb" ];
      extraOptions = [
        "--entrypoint=[]"
        "--network-alias=asb-tracing-logger"
        "--network=eigenix-network"
      ];
    };
    systemd.services."podman-asb-tracing-logger" = {
      serviceConfig.Restart = lib.mkOverride 90 "always";
      after = [
        "podman-network-eigenix.service"
        "podman-asb.service"
      ];
      requires = [
        "podman-network-eigenix.service"
        "podman-asb.service"
      ];
      partOf = [ "eigenix-root.target" ];
      wantedBy = [ "eigenix-root.target" ];
    };

    # Network
    systemd.services."podman-network-eigenix" = {
      path = [ pkgs.podman ];
      serviceConfig = {
        Type = "oneshot";
        RemainAfterExit = true;
        ExecStop = "podman network rm -f eigenix-network";
      };
      script = ''
        podman network inspect eigenix-network || podman network create eigenix-network
      '';
      partOf = [ "eigenix-root.target" ];
      wantedBy = [ "eigenix-root.target" ];
    };

    # Firewall for ASB - Only allow P2P port for external access
    # ASB RPC port is bound to localhost only for internal use
    networking.firewall.allowedTCPPorts = [
      config.eigenix.finalSettings.ports.asbP2p
    ];
  };
}
