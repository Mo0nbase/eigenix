# Eigenix Backend API NixOS Module
# Axum-based backend service
{
  config,
  lib,
  pkgs,
  ...
}:

with lib;

let
  cfg = config.services.eigenix-backend;
in
{
  options.services.eigenix-backend = {
    enable = mkEnableOption "Eigenix backend API (Axum)";

    package = mkOption {
      type = types.package;
      default = pkgs.emptyDirectory; # Placeholder - will be replaced with actual build
      description = "The eigenix-backend package to run";
    };

    port = mkOption {
      type = types.port;
      default = config.services.eigenix-ports.eigenixBackend or 3000;
      description = "Port for the backend API";
    };

    host = mkOption {
      type = types.str;
      default = "127.0.0.1";
      description = "Host address to bind to (default localhost only)";
    };

    asbRpcUrl = mkOption {
      type = types.str;
      default = "http://localhost:${toString (config.services.eigenix-ports.asbRpc or 9944)}";
      description = "URL for ASB RPC endpoint";
    };

    bitcoinRpcUrl = mkOption {
      type = types.str;
      default = "http://localhost:${toString (config.services.eigenix-ports.bitcoinRpc or 8332)}";
      description = "URL for Bitcoin RPC endpoint";
    };

    moneroRpcUrl = mkOption {
      type = types.str;
      default = "http://localhost:${toString (config.services.eigenix-ports.moneroRpc or 18081)}";
      description = "URL for Monero RPC endpoint";
    };
  };

  config = mkIf cfg.enable {
    systemd.services.eigenix-backend = {
      description = "Eigenix Backend API";
      after = [ "network.target" ];
      wantedBy = [ "eigenix-root.target" ];
      partOf = [ "eigenix-root.target" ];

      environment = {
        BIND_HOST = cfg.host;
        BIND_PORT = toString cfg.port;
        ASB_RPC_URL = cfg.asbRpcUrl;
        BITCOIN_RPC_URL = cfg.bitcoinRpcUrl;
        MONERO_RPC_URL = cfg.moneroRpcUrl;
        RUST_LOG = "info";
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

    # Only open firewall if binding to non-localhost
    networking.firewall.allowedTCPPorts = mkIf (cfg.host != "127.0.0.1" && cfg.host != "localhost") [
      cfg.port
    ];
  };
}
