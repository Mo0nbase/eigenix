# Eigenix - Main NixOS Module
# Complete deployment orchestration for ASB, Bitcoin/Monero nodes, mempool.space, and web services
{
  config,
  lib,
  pkgs,
  ...
}:

with lib;

let
  cfg = config.services.eigenix;
in
{
  imports = [
    ./ports.nix
    ./asb.nix
    ./mempool.nix
    ./web.nix
    ./backend.nix
  ];

  options.services.eigenix = {
    enable = mkEnableOption "Eigenix - Complete crypto services stack";

    baseDataDir = mkOption {
      type = types.str;
      default = "/mnt/vault";
      description = "Base directory for all persistent data (blockchains, wallets, logs)";
    };

    # Component toggles
    components = {
      asb = mkEnableOption "ASB (Automated Swap Backend) with Bitcoin/Monero nodes";

      mempool = mkEnableOption "Mempool.space Bitcoin block explorer";

      web = mkEnableOption "Eigenix web frontend (Dioxus)";

      backend = mkEnableOption "Eigenix backend API (Axum)";
    };

    # ASB Configuration
    asb = {
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

      externalBitcoinAddress = mkOption {
        type = types.nullOr types.str;
        default = null;
        example = "bc1qyouraddresshere";
        description = "Fixed Bitcoin address for redeeming/punishing swaps (optional)";
      };
    };

    # Domain configuration for web services
    domain = mkOption {
      type = types.nullOr types.str;
      default = null;
      example = "eigenix.example.com";
      description = "Domain name for web services (optional, for reverse proxy setup)";
    };
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

    # Forward ASB configuration
    services.eigenix-asb = mkIf cfg.components.asb {
      enable = true;
      baseDataDir = cfg.baseDataDir;
      externalAddresses = cfg.asb.externalAddresses;
      enableTor = cfg.asb.enableTor;
      minBuyBtc = cfg.asb.minBuyBtc;
      maxBuyBtc = cfg.asb.maxBuyBtc;
      askSpread = cfg.asb.askSpread;
      externalBitcoinAddress = cfg.asb.externalBitcoinAddress;
    };

    # Forward mempool configuration
    services.eigenix-mempool = mkIf cfg.components.mempool {
      enable = true;
      baseDataDir = cfg.baseDataDir;
    };

    # Forward web configuration
    services.eigenix-web = mkIf cfg.components.web {
      enable = true;
    };

    # Forward backend configuration
    services.eigenix-backend = mkIf cfg.components.backend {
      enable = true;
    };

    # Root systemd target for managing all eigenix services
    systemd.targets."eigenix-root" = {
      unitConfig.Description = "Eigenix services root target";
      wantedBy = [ "multi-user.target" ];
    };
  };
}
