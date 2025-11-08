# Eigenix Parameters Module (Idiomatic NixOS Pattern)
# Uses the standard "settings" pattern for structured configuration
# See: https://nixos.org/manual/nixos/stable/index.html#sec-settings-attrs

{
  config,
  lib,
  pkgs,
  ...
}:

with lib;

let
  cfg = config.eigenix;

  # Use pkgs.formats for standard JSON handling
  settingsFormat = pkgs.formats.json { };

  # Load user config from JSON if it exists
  userConfigPath = ./settings.json;
  userConfig =
    if builtins.pathExists userConfigPath then
      builtins.fromJSON (builtins.readFile userConfigPath)
    else
      { };

in
{
  options.eigenix = {
    # The idiomatic way: a single "settings" option with freeform attrs
    settings = mkOption {
      type = types.submodule {
        freeformType = settingsFormat.type;

        # Define well-known options with types and defaults
        options = {
          deployment = {
            name = mkOption {
              type = types.str;
              default = "eigenix";
              description = "Deployment identifier";
            };

            environment = mkOption {
              type = types.enum [
                "production"
                "staging"
                "development"
                "testnet"
              ];
              default = "production";
              description = "Deployment environment";
            };

            domain = mkOption {
              type = types.nullOr types.str;
              default = null;
              description = "Primary domain for web services";
            };
          };

          storage = {
            baseDataDir = mkOption {
              type = types.str;
              default = "/mnt/vault";
              description = "Base directory for all persistent data";
            };
          };

          networks = {
            bitcoin = mkOption {
              type = types.enum [
                "Mainnet"
                "Testnet"
                "Signet"
                "Regtest"
              ];
              default = "Mainnet";
              description = "Bitcoin network";
            };

            monero = mkOption {
              type = types.enum [
                "Mainnet"
                "Stagenet"
                "Testnet"
              ];
              default = "Mainnet";
              description = "Monero network";
            };
          };

          asb = {
            enable = mkOption {
              type = types.bool;
              default = true;
              description = "Enable ASB service";
            };

            externalAddresses = mkOption {
              type = types.listOf types.str;
              default = [ ];
              description = "External multiaddresses for P2P discovery";
            };

            enableTor = mkOption {
              type = types.bool;
              default = true;
              description = "Enable Tor hidden service";
            };

            minBuyBtc = mkOption {
              type = types.float;
              default = 0.002;
              description = "Minimum BTC per swap";
            };

            maxBuyBtc = mkOption {
              type = types.float;
              default = 0.02;
              description = "Maximum BTC per swap";
            };

            askSpread = mkOption {
              type = types.float;
              default = 0.02;
              description = "Spread percentage (0.02 = 2%)";
            };

            externalBitcoinAddress = mkOption {
              type = types.nullOr types.str;
              default = null;
              description = "Fixed Bitcoin address for swaps";
            };

            developerTip = mkOption {
              type = types.float;
              default = 0.0;
              description = "Developer donation percentage";
            };
          };

          mempool.enable = mkOption {
            type = types.bool;
            default = true;
            description = "Enable mempool explorer";
          };

          backend = {
            enable = mkOption {
              type = types.bool;
              default = false;
              description = "Enable backend API";
            };

            host = mkOption {
              type = types.str;
              default = "0.0.0.0";
              description = "Backend bind address (use 0.0.0.0 for external access)";
            };

            logLevel = mkOption {
              type = types.str;
              default = "info";
              description = "Backend log level";
            };
          };

          web = {
            enable = mkOption {
              type = types.bool;
              default = false;
              description = "Enable web frontend";
            };

            host = mkOption {
              type = types.str;
              default = "0.0.0.0";
              description = "Web frontend bind address (use 0.0.0.0 for external access)";
            };
          };

          ports = {
            asbP2p = mkOption {
              type = types.port;
              default = 9939;
              description = "ASB P2P port";
            };

            asbRpc = mkOption {
              type = types.port;
              default = 9944;
              description = "ASB RPC port";
            };

            mempoolWeb = mkOption {
              type = types.port;
              default = 8999;
              description = "Mempool web port";
            };

            eigenixWeb = mkOption {
              type = types.port;
              default = 8080;
              description = "Eigenix web port";
            };

            eigenixBackend = mkOption {
              type = types.port;
              default = 3000;
              description = "Eigenix backend port";
            };

            surrealdb = mkOption {
              type = types.port;
              default = 8000;
              description = "SurrealDB port (internal only)";
            };

            bitcoinRpc = mkOption {
              type = types.port;
              default = 8332;
              description = "Bitcoin RPC port (internal only)";
            };

            moneroRpc = mkOption {
              type = types.port;
              default = 18081;
              description = "Monero RPC port (internal only)";
            };

            electrs = mkOption {
              type = types.port;
              default = 50001;
              description = "Electrs RPC port (internal only)";
            };
          };
        };
      };

      default = { };
      description = ''
        Eigenix deployment settings.

        Can be configured via:
        1. JSON file at nix/instances/parameters.json (managed by CLI)
        2. Direct Nix configuration in this option
        3. Both (Nix config takes precedence)

        The JSON file is automatically loaded if present.
      '';
    };

    # Computed final settings (merge JSON + Nix config)
    # JSON values take precedence over Nix defaults, but explicit Nix config overrides JSON
    finalSettings = mkOption {
      type = types.attrs;
      internal = true;
      readOnly = true;
      default = recursiveUpdate cfg.settings userConfig;
      description = "Final merged settings from JSON and Nix config";
    };

    # Alias for backward compatibility with module.nix
    parameters = mkOption {
      type = types.attrs;
      internal = true;
      readOnly = true;
      default = cfg.finalSettings;
      description = "Alias for finalSettings (backward compatibility)";
    };

    # Export the generated config file for services to use
    configFile = mkOption {
      type = types.package;
      internal = true;
      readOnly = true;
      default = settingsFormat.generate "eigenix-config.json" cfg.finalSettings;
      description = "Generated configuration file";
    };
  };

  config = {
    # Services can now access config.eigenix.finalSettings
    # or use config.eigenix.configFile for a JSON file in the store

    # Validation
    assertions = [
      {
        assertion = cfg.finalSettings.asb.minBuyBtc < cfg.finalSettings.asb.maxBuyBtc;
        message = "asb.minBuyBtc must be less than asb.maxBuyBtc";
      }
      {
        assertion =
          let
            ports = with cfg.finalSettings.ports; [
              asbP2p
              asbRpc
              mempoolWeb
              eigenixWeb
              eigenixBackend
              surrealdb
              bitcoinRpc
              moneroRpc
              electrs
            ];
            unique = lib.unique ports;
          in
          length ports == length unique;
        message = "Port conflict detected in eigenix configuration";
      }
    ];
  };
}
