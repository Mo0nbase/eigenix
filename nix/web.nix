# Eigenix Web Frontend NixOS Module
# Serves the Dioxus-based web frontend
{
  config,
  lib,
  pkgs,
  ...
}:

with lib;

let
  cfg = config.services.eigenix-web;
in
{
  options.services.eigenix-web = {
    enable = mkEnableOption "Eigenix web frontend (Dioxus)";

    package = mkOption {
      type = types.package;
      default = pkgs.emptyDirectory; # Placeholder - will be replaced with actual build
      description = "The eigenix-web package to serve";
    };

    port = mkOption {
      type = types.port;
      default = config.services.eigenix-ports.eigenixWeb or 8080;
      description = "Port to serve the web frontend on";
    };

    host = mkOption {
      type = types.str;
      default = "0.0.0.0";
      description = "Host address to bind to";
    };
  };

  config = mkIf cfg.enable {
    # Use a simple HTTP server to serve the static Dioxus bundle
    systemd.services.eigenix-web = {
      description = "Eigenix Web Frontend";
      after = [ "network.target" ];
      wantedBy = [ "eigenix-root.target" ];
      partOf = [ "eigenix-root.target" ];

      serviceConfig = {
        Type = "simple";
        ExecStart = "${pkgs.python3}/bin/python3 -m http.server ${toString cfg.port} --bind ${cfg.host} --directory ${cfg.package}";
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
        MemoryDenyWriteExecute = true;
        SystemCallFilter = [
          "@system-service"
          "~@privileged"
        ];
      };
    };

    # Open firewall for web frontend
    networking.firewall.allowedTCPPorts = [ cfg.port ];
  };
}
