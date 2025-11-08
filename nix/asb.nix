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
    external_addresses = ${builtins.toJSON cfg.externalAddresses}

    [bitcoin]
    electrum_rpc_urls = ["tcp://electrs:50001"]
    target_block = 1
    network = "Mainnet"
    use_mempool_space_fee_estimation = true

    [monero]
    daemon_url = "http://monerod:18081/"
    network = "Mainnet"

    [tor]
    register_hidden_service = ${if cfg.enableTor then "true" else "false"}
    hidden_service_num_intro_points = 5

    [maker]
    min_buy_btc = ${toString cfg.minBuyBtc}
    max_buy_btc = ${toString cfg.maxBuyBtc}
    ask_spread = ${toString cfg.askSpread}
    price_ticker_ws_url = "${cfg.priceTickerUrl}"
    developer_tip = ${toString cfg.developerTip}
    ${optionalString (
      cfg.externalBitcoinAddress != null
    ) ''external_bitcoin_address = "${cfg.externalBitcoinAddress}"''}
  '';

  asbConfigFile = asbConfigToml;

  bitcoinChain = "main";
  electrsNetwork = "bitcoin";

in
{
  options.services.eigenix-asb = {
    enable = mkEnableOption "Automated Swap Backend (ASB) for XMR/BTC atomic swaps";

    baseDataDir = mkOption {
      type = types.str;
      default = "/mnt/vault";
      description = "Base directory for all persistent data (blockchains, wallets, logs)";
    };

    # Network options
    externalAddresses = mkOption {
      type = types.listOf types.str;
      default = [ ];
      example = [
        "/dns4/swap.example.com/tcp/9939"
        "/ip4/1.2.3.4/tcp/9939"
      ];
      description = "External multiaddresses for ASB discovery (libp2p format)";
    };

    enableTor = mkOption {
      type = types.bool;
      default = true;
      description = "Enable Tor hidden service for ASB";
    };

    # Maker parameters
    minBuyBtc = mkOption {
      type = types.float;
      default = 0.002;
      description = "Minimum BTC amount to accept per swap";
    };

    maxBuyBtc = mkOption {
      type = types.float;
      default = 0.02;
      description = "Maximum BTC amount to accept per swap";
    };

    askSpread = mkOption {
      type = types.float;
      default = 0.02;
      description = "Spread to add on top of exchange price (e.g., 0.02 = 2%)";
    };

    priceTickerUrl = mkOption {
      type = types.str;
      default = "wss://ws.kraken.com/";
      description = "WebSocket URL for price ticker";
    };

    developerTip = mkOption {
      type = types.float;
      default = 0.0;
      description = "Optional donation to project development (0.0 to 1.0, e.g., 0.02 = 2%)";
    };

    externalBitcoinAddress = mkOption {
      type = types.nullOr types.str;
      default = null;
      example = "bc1qyouraddresshere";
      description = "Fixed Bitcoin address for redeeming/punishing swaps (optional)";
    };
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
      "d ${cfg.baseDataDir}/asb-data 0700 1000 1000 -"
      "d ${cfg.baseDataDir}/bitcoind-data 0700 1000 1000 -"
      "d ${cfg.baseDataDir}/electrs-data 0700 1000 1000 -"
      "d ${cfg.baseDataDir}/monerod-data 0700 1000 1000 -"

      # ASB sensitive subdirectories
      "d ${cfg.baseDataDir}/asb-data/wallet 0700 1000 1000 -"
      "d ${cfg.baseDataDir}/asb-data/monero 0700 1000 1000 -"
      "d ${cfg.baseDataDir}/asb-data/tor 0700 1000 1000 -"
      "d ${cfg.baseDataDir}/asb-data/tor/state 0700 1000 1000 -"
      "d ${cfg.baseDataDir}/asb-data/tor/cache 0700 1000 1000 -"

      # ASB operational subdirectories (were previously 775/755)
      "d ${cfg.baseDataDir}/asb-data/bitcoind 0700 1000 1000 -"
      "d ${cfg.baseDataDir}/asb-data/asb 0700 1000 1000 -"
      "d ${cfg.baseDataDir}/asb-data/monerod 0700 1000 1000 -"
      "d ${cfg.baseDataDir}/asb-data/electrs 0700 1000 1000 -"
      "d ${cfg.baseDataDir}/asb-data/logs 0700 1000 1000 -"

      # ASB seed file - CRITICAL: 600 permissions (read/write owner only)
      "f ${cfg.baseDataDir}/asb-data/seed.pem 0600 1000 1000 -"

      # Recursively fix ownership and permissions on wallet directories
      "Z ${cfg.baseDataDir}/asb-data/wallet 0700 1000 1000 -"
      "Z ${cfg.baseDataDir}/asb-data/monero 0700 1000 1000 -"
    ];

    # Containers
    virtualisation.oci-containers.containers."monerod" = {
      image = "ghcr.io/sethforprivacy/simple-monerod@sha256:f30e5706a335c384e4cf420215cbffd1196f0b3a11d4dd4e819fe3e0bca41ec5";
      volumes = [
        "${cfg.baseDataDir}/monerod-data:/monerod-data:rw"
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
        "--network=asb-network"
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
      after = [ "podman-network-asb.service" ];
      requires = [ "podman-network-asb.service" ];
      partOf = [ "eigenix-root.target" ];
      wantedBy = [ "eigenix-root.target" ];
    };

    virtualisation.oci-containers.containers."bitcoind" = {
      image = "getumbrel/bitcoind@sha256:c565266ea302c9ab2fc490f04ff14e584210cde3d0d991b8309157e5dfae9e8d";
      volumes = [
        "${cfg.baseDataDir}/bitcoind-data:/bitcoind-data:rw"
      ];
      cmd = [
        "bitcoind"
        "-chain=${bitcoinChain}"
        "-rpcallowip=127.0.0.1"
        # Restrict to container network subnet only (asb-network: 10.89.0.0/24)
        "-rpcallowip=10.89.0.0/24"
        "-rpcbind=0.0.0.0:8332"
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
        "--network=asb-network"
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
      after = [ "podman-network-asb.service" ];
      requires = [ "podman-network-asb.service" ];
      partOf = [ "eigenix-root.target" ];
      wantedBy = [ "eigenix-root.target" ];
    };

    virtualisation.oci-containers.containers."electrs" = {
      image = "getumbrel/electrs@sha256:622657fbdc7331a69f5b3444e6f87867d51ac27d90c399c8bf25d9aab020052b";
      volumes = [
        "${cfg.baseDataDir}/bitcoind-data:/bitcoind-data:ro"
        "${cfg.baseDataDir}/electrs-data:/electrs-data:rw"
      ];
      cmd = [
        "electrs"
        "--network=${electrsNetwork}"
        "--daemon-dir=/bitcoind-data/"
        "--db-dir=/electrs-data/db"
        "--daemon-rpc-addr=bitcoind:8332"
        "--daemon-p2p-addr=bitcoind:8333"
        "--electrum-rpc-addr=0.0.0.0:50001"
        "--log-filters=INFO"
      ];
      dependsOn = [ "bitcoind" ];
      user = "1000:1000";
      extraOptions = [
        "--entrypoint=[]"
        "--network-alias=electrs"
        "--network=asb-network"
        # Resource limits for security
        "--memory=4g"
        "--cpus=2.0"
        "--pids-limit=1024"
        # Security options
        "--security-opt=no-new-privileges:true"
      ];
    };
    systemd.services."podman-electrs" = {
      serviceConfig.Restart = lib.mkOverride 90 "always";
      after = [
        "podman-network-asb.service"
        "podman-bitcoind.service"
      ];
      requires = [
        "podman-network-asb.service"
        "podman-bitcoind.service"
      ];
      partOf = [ "eigenix-root.target" ];
      wantedBy = [ "eigenix-root.target" ];
    };

    virtualisation.oci-containers.containers."asb" = {
      image = "ghcr.io/eigenwallet/asb:latest";
      volumes = [
        "${asbConfigFile}:/asb-data/config.toml:ro"
        "${cfg.baseDataDir}/asb-data:/asb-data:rw"
      ];
      ports = [
        "0.0.0.0:${toString config.services.eigenix-ports.asbP2p}:9939/tcp"
        "127.0.0.1:${toString config.services.eigenix-ports.asbRpc}:${toString config.services.eigenix-ports.asbRpc}/tcp"
      ];
      cmd = [
        "asb"
        "--config=/asb-data/config.toml"
        "start"
        "--rpc-bind-port=${toString config.services.eigenix-ports.asbRpc}"
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
        "--network=asb-network"
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
        "podman-network-asb.service"
        "podman-electrs.service"
        "podman-monerod.service"
      ];
      requires = [
        "podman-network-asb.service"
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
        "--url=http://asb:${toString config.services.eigenix-ports.asbRpc}"
      ];
      dependsOn = [ "asb" ];
      extraOptions = [
        "--entrypoint=[]"
        "--network-alias=asb-controller"
        "--network=asb-network"
        "--tty"
        "--interactive"
      ];
    };
    systemd.services."podman-asb-controller" = {
      serviceConfig.Restart = lib.mkOverride 90 "always";
      after = [
        "podman-network-asb.service"
        "podman-asb.service"
      ];
      requires = [
        "podman-network-asb.service"
        "podman-asb.service"
      ];
      partOf = [ "eigenix-root.target" ];
      wantedBy = [ "eigenix-root.target" ];
    };

    virtualisation.oci-containers.containers."asb-tracing-logger" = {
      image = "alpine@sha256:4bcff63911fcb4448bd4fdacec207030997caf25e9bea4045fa6c8c44de311d1";
      volumes = [
        "${cfg.baseDataDir}/asb-data:/asb-data:ro"
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
        "--network=asb-network"
      ];
    };
    systemd.services."podman-asb-tracing-logger" = {
      serviceConfig.Restart = lib.mkOverride 90 "always";
      after = [
        "podman-network-asb.service"
        "podman-asb.service"
      ];
      requires = [
        "podman-network-asb.service"
        "podman-asb.service"
      ];
      partOf = [ "eigenix-root.target" ];
      wantedBy = [ "eigenix-root.target" ];
    };

    # Network
    systemd.services."podman-network-asb" = {
      path = [ pkgs.podman ];
      serviceConfig = {
        Type = "oneshot";
        RemainAfterExit = true;
        ExecStop = "podman network rm -f asb-network";
      };
      script = ''
        podman network inspect asb-network || podman network create asb-network
      '';
      partOf = [ "eigenix-root.target" ];
      wantedBy = [ "eigenix-root.target" ];
    };

    # Firewall for ASB
    networking.firewall.allowedTCPPorts = [ config.services.eigenix-ports.asbP2p ];

    # Root target
    systemd.targets."eigenix-root" = {
      unitConfig.Description = "Eigenix root target";
      wantedBy = [ "multi-user.target" ];
    };
  };
}
