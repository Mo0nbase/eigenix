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

    # Eigenix backend service
    systemd.services.eigenix-backend = {
      description = "Eigenix Backend API";
      after = [
        "network.target"
        "podman-surrealdb.service"
      ];
      requires = [ "podman-surrealdb.service" ];
      wantedBy = [ "eigenix-root.target" ];
      partOf = [ "eigenix-root.target" ];

      environment = {
        BIND_HOST = settings.backend.host;
        BIND_PORT = toString settings.ports.eigenixBackend;
        ASB_RPC_URL = "http://asb:${toString settings.ports.asbRpc}";
        BITCOIN_RPC_URL = "http://bitcoind:${toString settings.ports.bitcoinRpc}";
        MONERO_RPC_URL = "http://monerod:${toString settings.ports.moneroRpc}";
        SURREALDB_URL = "http://surrealdb:${toString settings.ports.surrealdb}";
        SURREALDB_USER = "root";
        SURREALDB_PASS = "root";
        RUST_LOG = settings.backend.logLevel;
      };

      serviceConfig = {
        Type = "simple";
        ExecStart = "${cfg.package}/bin/eigenix-backend";
        Restart = "on-failure";
        RestartSec = "10s";

        # Security hardening
        DynamicUser = true;
        PrivateTmp = true;
        ProtectSystem = "strict";
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
