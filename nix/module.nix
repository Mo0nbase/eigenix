# Eigenix - Main NixOS Module
# Provides the main enable switch and sets up shared infrastructure
# Individual services read their configuration directly from eigenix.finalSettings

{
  config,
  lib,
  pkgs,
  eigenixPackages ? null,
  ...
}:

with lib;

let
  cfg = config.services.eigenix;
  settings = config.eigenix.finalSettings;
in
{
  imports = [
    ./settings.nix
    ./asb.nix
    ./mempool.nix
    ./web.nix
    ./backend.nix
    ./test-runner.nix
  ];

  options.services.eigenix = {
    enable = mkEnableOption "Eigenix - Complete crypto services stack";
  };

  config = mkIf cfg.enable {
    # Enable Podman for all container-based services
    virtualisation.podman = {
      enable = true;
      autoPrune.enable = true;
      dockerCompat = true;
    };

    # Enable container name DNS for all Podman networks
    networking.firewall.interfaces =
      let
        matchAll = if !config.networking.nftables.enable then "podman+" else "podman*";
      in
      {
        "${matchAll}".allowedUDPPorts = [ 53 ];
      };

    virtualisation.oci-containers.backend = "podman";

    # Enable individual services based on settings
    services.eigenix-asb.enable = settings.asb.enable;
    services.eigenix-mempool.enable = settings.mempool.enable;
    services.eigenix-backend = {
      enable = settings.backend.enable;
      package = mkIf (eigenixPackages != null) eigenixPackages.eigenix-backend;
    };
    services.eigenix-web = {
      enable = settings.web.enable;
      package = mkIf (eigenixPackages != null) eigenixPackages.eigenix-web;
    };

    # Root systemd target for managing all eigenix services
    systemd.targets."eigenix-root" = {
      unitConfig.Description = "Eigenix services root target";
      wantedBy = [ "multi-user.target" ];
    };
  };
}
