# Eigenix Ports Configuration
{ config, lib, ... }:

with lib;

{
  options.services.eigenix-ports = {
    # ASB services
    asbP2p = mkOption {
      type = types.port;
      default = 9939;
      description = "ASB P2P libp2p port";
    };

    asbRpc = mkOption {
      type = types.port;
      default = 9944;
      description = "ASB RPC API port";
    };

    # Blockchain nodes
    bitcoinRpc = mkOption {
      type = types.port;
      default = 8332;
      description = "Bitcoin RPC port";
    };

    bitcoinP2p = mkOption {
      type = types.port;
      default = 8333;
      description = "Bitcoin P2P port";
    };

    electrs = mkOption {
      type = types.port;
      default = 50001;
      description = "Electrs RPC port";
    };

    moneroRpc = mkOption {
      type = types.port;
      default = 18081;
      description = "Monero RPC port";
    };

    moneroP2p = mkOption {
      type = types.port;
      default = 18080;
      description = "Monero P2P port";
    };

    # Mempool services
    mempoolWeb = mkOption {
      type = types.port;
      default = 8999;
      description = "Mempool.space web interface port";
    };

    mempoolApi = mkOption {
      type = types.port;
      default = 8998;
      description = "Mempool.space API port";
    };

    # Eigenix application services
    eigenixWeb = mkOption {
      type = types.port;
      default = 8080;
      description = "Eigenix web frontend port";
    };

    eigenixBackend = mkOption {
      type = types.port;
      default = 3000;
      description = "Eigenix backend API port";
    };
  };

  config = {
    # Port conflict validation
    assertions = [
      {
        assertion =
          let
            ports = with config.services.eigenix-ports; [
              asbP2p
              asbRpc
              bitcoinRpc
              bitcoinP2p
              electrs
              moneroRpc
              moneroP2p
              mempoolWeb
              mempoolApi
              eigenixWeb
              eigenixBackend
            ];
            uniquePorts = lib.unique ports;
          in
          length ports == length uniquePorts;
        message = "Port conflict detected in eigenix-ports configuration. Each port must be unique.";
      }
    ];
  };
}
