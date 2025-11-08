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
  settings = config.eigenix.finalSettings;
in
{
  options.services.eigenix-web = {
    enable = mkEnableOption "Eigenix web frontend (Dioxus)";

    package = mkOption {
      type = types.package;
      default = pkgs.emptyDirectory;
      description = "The eigenix-web package to serve";
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
        ExecStart = "${pkgs.python3}/bin/python3 -m http.server ${toString settings.ports.eigenixWeb} --bind ${settings.web.host} --directory ${cfg.package}";
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
    networking.firewall.allowedTCPPorts = [ settings.ports.eigenixWeb ];
  };
}
