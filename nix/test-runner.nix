# Integration Test Runner Service
# Runs integration tests as a container in the eigenix-network
# Similar to backend.nix but runs tests instead of the service

{
  config,
  lib,
  pkgs,
  eigenixPackages ? null,
  ...
}:

with lib;

let
  cfg = config.services.eigenix-test-runner;
  settings = config.eigenix.finalSettings;
in
{
  options.services.eigenix-test-runner = {
    enable = mkEnableOption "Eigenix integration test runner container";

    testFilter = mkOption {
      type = types.nullOr types.str;
      default = null;
      description = "Test filter (e.g., 'asb' to run only ASB tests)";
      example = "asb";
    };

    autoStart = mkOption {
      type = types.bool;
      default = false;
      description = "Whether to automatically start tests when services start";
    };
  };

  config = mkIf cfg.enable {
    assertions = [
      {
        assertion = eigenixPackages != null -> eigenixPackages ? eigenix-backend-tests;
        message = "eigenix-backend-tests package not found. Make sure to pass eigenixPackages when importing the eigenix module.";
      }
    ];

    # Build the backend test image
    virtualisation.oci-containers.containers."eigenix-test-runner" = mkIf (eigenixPackages != null) {
      # Use the Rust image with all build tools
      image = "rust:1.75-slim";

      # Mount the backend source code and writable build directory
      volumes = [
        "${eigenixPackages.eigenix-backend-tests}:/app/src:ro"
        # Need writable target directory for cargo builds
        "eigenix-test-target:/app/target:rw"
        # Also need writable workspace
        "eigenix-test-workspace:/app/work:rw"
      ];

      # Environment variables for container network
      environment = {
        IN_CONTAINER = "true";
        RUST_LOG = "info";
        ASB_RPC_URL = "http://asb:${toString settings.ports.asbRpc}";
        BITCOIN_RPC_URL = "http://bitcoind:${toString settings.ports.bitcoinRpc}";
        BITCOIN_COOKIE_PATH = "/bitcoind-data/.cookie";
        MONERO_WALLET_RPC_URL = "http://monerod:${toString settings.ports.moneroRpc}/json_rpc";
        WALLET_NAME = "eigenix";
      };

      workdir = "/app/work";

      # Command to run tests
      cmd = [
        "bash"
        "-c"
        ''
          set -e
          echo "=== Eigenix Integration Tests ==="
          
          # Copy source to writable workspace
          echo "Setting up workspace..."
          cp -r /app/src/* /app/work/
          cd /app/work
          
          echo "Waiting for services to be ready..."
          sleep 10
          
          # Wait for ASB to be responsive
          until curl -f http://asb:${toString settings.ports.asbRpc}/status 2>/dev/null; do
            echo "Waiting for ASB..."
            sleep 5
          done
          
          echo "Services ready, running tests..."
          
          # Install dependencies
          apt-get update && apt-get install -y pkg-config libssl-dev curl
          
          # Install nextest if needed
          if ! command -v cargo-nextest &> /dev/null; then
            cargo install cargo-nextest --locked
          fi
          
          # Run tests with optional filter
          ${if cfg.testFilter != null then ''
            cargo nextest run --test "*${cfg.testFilter}_integration*" --release --no-fail-fast
          '' else ''
            cargo nextest run --test "*integration*" --release --no-fail-fast
          ''}
          
          echo "Tests completed!"
        ''
      ];

      # Connect to eigenix network
      extraOptions = [
        "--entrypoint=[]"
        "--network=eigenix-network"
        # Resource limits
        "--memory=2g"
        "--cpus=2.0"
        # Security
        "--security-opt=no-new-privileges:true"
        # Remove container after completion
        "--rm"
      ];

      # Depend on all required services
      dependsOn = [
        "asb"
        "bitcoind"
        "monerod"
        "electrs"
      ];
    };

    # Systemd service configuration
    systemd.services."podman-eigenix-test-runner" = {
      serviceConfig = {
        Type = lib.mkForce "oneshot";
        RemainAfterExit = lib.mkForce false;  # Don't keep service active after tests complete
        Restart = lib.mkForce "no";  # Don't auto-restart
      };
      after = [
        "podman-network-eigenix.service"
        "podman-asb.service"
        "podman-bitcoind.service"
        "podman-monerod.service"
        "podman-electrs.service"
      ];
      requires = [
        "podman-network-eigenix.service"
        "podman-asb.service"
        "podman-bitcoind.service"
        "podman-monerod.service"
        "podman-electrs.service"
      ];
      
      # Only auto-start if configured
      wantedBy = mkIf cfg.autoStart [ "eigenix-root.target" ];
    };

    # Create the volumes for test artifacts and workspace
    system.activationScripts.eigenix-test-volumes = ''
      ${pkgs.podman}/bin/podman volume create eigenix-test-target 2>/dev/null || true
      ${pkgs.podman}/bin/podman volume create eigenix-test-workspace 2>/dev/null || true
    '';
  };
}

