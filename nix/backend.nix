# Eigenix Backend API NixOS Module
# Axum-based backend service with SurrealDB
{
  config,
  lib,
  pkgs,
  ...
}:

with lib;

let
  cfg = config.services.eigenix-backend;
  settings = config.eigenix.finalSettings;
in
{
  options.services.eigenix-backend = {
    enable = mkEnableOption "Eigenix backend API (Axum)";

    package = mkOption {
      type = types.package;
      default = pkgs.emptyDirectory;
      description = "The eigenix-backend package to run";
    };
  };

  config = mkIf cfg.enable {
    # Enable Podman for containers
    virtualisation.podman.enable = true;
    virtualisation.oci-containers.backend = "podman";

    # Ensure data directories exist with secure permissions
    systemd.tmpfiles.rules = [
      "d ${settings.storage.baseDataDir}/surrealdb 0755 1000 1000 -"
      "d ${settings.storage.baseDataDir}/monero-wallets 0755 1000 1000 -"
    ];

    # Create eigenix-network if it doesn't exist (normally created by ASB module)
    systemd.services."podman-network-eigenix" = mkIf (!config.services.eigenix-asb.enable) {
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

    # SurrealDB container with RocksDB backend
    virtualisation.oci-containers.containers."surrealdb" = {
      image = "surrealdb/surrealdb:latest";
      volumes = [
        "${settings.storage.baseDataDir}/surrealdb:/data:rw,Z"
      ];
      ports = [
        "0.0.0.0:${toString settings.ports.surrealdb}:8000/tcp"
      ];
      cmd = [
        "start"
        "--log=info"
        "--user=root"
        "--pass=root"
        "--bind=0.0.0.0:8000"
        "rocksdb:/data/database.db"
      ];
      user = "1000:1000";
      extraOptions = [
        "--network-alias=surrealdb"
        "--network=eigenix-network"
        # Resource limits for security
        "--memory=2g"
        "--cpus=2.0"
        "--pids-limit=512"
        # Security options
        "--security-opt=no-new-privileges:true"
        # Allow container to write to volume
        "--userns=keep-id"
      ];
    };

    systemd.services."podman-surrealdb" = {
      serviceConfig.Restart = lib.mkOverride 90 "always";
      after = [ "podman-network-eigenix.service" ];
      requires = [ "podman-network-eigenix.service" ];
      partOf = [ "eigenix-root.target" ];
      wantedBy = [ "eigenix-root.target" ];
    };

    # Monero wallet RPC container
    virtualisation.oci-containers.containers."monero-wallet-rpc" = {
      image = "ghcr.io/sethforprivacy/simple-monero-wallet-rpc:latest";
      volumes = [
        "${settings.storage.baseDataDir}/monero-wallets:/wallet:rw,Z"
      ];
      ports = [
        "127.0.0.1:18082:18082/tcp"
      ];
      cmd = [
        "--daemon-host=monerod"
        "--daemon-port=${toString settings.ports.moneroRpc}"
        "--rpc-bind-port=18082"
        "--wallet-dir=/wallet"
        "--disable-rpc-login"
        "--log-level=1"
      ];
      extraOptions = [
        "--network-alias=monero-wallet-rpc"
        "--network=eigenix-network"
        # Resource limits
        "--memory=1g"
        "--cpus=1.0"
        "--pids-limit=256"
      ];
    };

    systemd.services."podman-monero-wallet-rpc" = {
      serviceConfig.Restart = lib.mkOverride 90 "always";
      after = [ 
        "podman-network-eigenix.service"
        "podman-monerod.service"
      ];
      requires = [ 
        "podman-network-eigenix.service"
        "podman-monerod.service"
      ];
      partOf = [ "eigenix-root.target" ];
      wantedBy = [ "eigenix-root.target" ];
    };

    # Eigenix backend service
    systemd.services.eigenix-backend = {
      description = "Eigenix Backend API";
      after = [
        "network.target"
        "podman-surrealdb.service"
        "podman-asb.service"
        "podman-monero-wallet-rpc.service"
      ];
      requires = [
        "podman-surrealdb.service"
        "podman-asb.service"
        "podman-monero-wallet-rpc.service"
      ];
      wantedBy = [ "eigenix-root.target" ];
      partOf = [ "eigenix-root.target" ];

      environment = {
        BIND_HOST = settings.backend.host;
        BIND_PORT = toString settings.ports.eigenixBackend;
        ASB_RPC_URL = "http://localhost:${toString settings.ports.asbRpc}";
        BITCOIN_RPC_URL = "http://localhost:${toString settings.ports.bitcoinRpc}";
        MONERO_RPC_URL = "http://localhost:${toString settings.ports.moneroRpc}";
        SURREALDB_URL = "http://localhost:${toString settings.ports.surrealdb}";
        SURREALDB_USER = "root";
        SURREALDB_PASS = "root";
        RUST_LOG = settings.backend.logLevel;
        BITCOIN_COOKIE_PATH = "${settings.storage.baseDataDir}/bitcoind-data/.cookie";
        MONERO_WALLET_RPC_URL = "http://localhost:18082/json_rpc";
        WALLET_NAME = "eigenix";
      };

      serviceConfig = {
        Type = "simple";
        ExecStart = ''
          ${cfg.package}/bin/eigenix-backend \
            --host ${settings.backend.host} \
            --port ${toString settings.ports.eigenixBackend} \
            --bitcoin-rpc-url http://localhost:${toString settings.ports.bitcoinRpc} \
            --monero-rpc-url http://localhost:${toString settings.ports.moneroRpc} \
            --asb-rpc-url http://localhost:${toString settings.ports.asbRpc} \
            --db-endpoint localhost:${toString settings.ports.surrealdb} \
            --bitcoin-cookie-path ${settings.storage.baseDataDir}/bitcoind-data/.cookie
        '';
        Restart = "on-failure";
        RestartSec = "10s";

        # Run as mo0nbase user to access cookie file (containers use same UID)
        User = "mo0nbase";
        Group = "users";

        # Security hardening
        PrivateTmp = true;
        ProtectSystem = "strict";
        # Need access to /mnt/vault for bitcoin cookie
        ReadOnlyPaths = [ settings.storage.baseDataDir ];
        ProtectHome = true;
        NoNewPrivileges = true;
        PrivateDevices = true;
        ProtectKernelTunables = true;
        ProtectKernelModules = true;
        ProtectControlGroups = true;
        RestrictAddressFamilies = [
          "AF_INET"
          "AF_INET6"
        ];
        RestrictNamespaces = true;
        LockPersonality = true;
        RestrictRealtime = true;
        RestrictSUIDSGID = true;
        SystemCallFilter = [
          "@system-service"
          "~@privileged"
        ];
      };
    };

    # Open firewall for backend and SurrealDB
    networking.firewall.allowedTCPPorts = [
      settings.ports.surrealdb
    ]
    ++ (
      if (settings.backend.host != "127.0.0.1" && settings.backend.host != "localhost") then
        [ settings.ports.eigenixBackend ]
      else
        [ ]
    );
  };
}
