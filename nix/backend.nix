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
    systemd.services.eigenix-backend = {
      description = "Eigenix Backend API";
      after = [ "network.target" ];
      wantedBy = [ "eigenix-root.target" ];
      partOf = [ "eigenix-root.target" ];

      environment = {
        BIND_HOST = settings.backend.host;
        BIND_PORT = toString settings.ports.eigenixBackend;
        ASB_RPC_URL = "http://localhost:${toString settings.ports.asbRpc}";
        BITCOIN_RPC_URL = "http://localhost:${toString settings.ports.bitcoinRpc}";
        MONERO_RPC_URL = "http://localhost:${toString settings.ports.moneroRpc}";
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

    # Only open firewall if binding to non-localhost
    networking.firewall.allowedTCPPorts =
      mkIf (settings.backend.host != "127.0.0.1" && settings.backend.host != "localhost")
        [
          settings.ports.eigenixBackend
        ];
  };
}
