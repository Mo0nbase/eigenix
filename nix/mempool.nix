# Eigenix Mempool Module (mempool.space Bitcoin Explorer)
#
# SECURITY FEATURES (Configuration-Enforced):
# ===========================================
#
# 1. File Permissions (via systemd.tmpfiles.rules):
#    - Mempool directory: 700 (owner-only access)
#    - MySQL directory: 700 (owned by UID 999 - MariaDB container user)
#    - Cache directory: 700 (owned by UID 1000 - mempool-api user)
#
# 2. Network Security:
#    - Mempool Web exposed on configured port (consider reverse proxy with TLS)
#    - Database accessible only within container network
#
# 3. Container Security:
#    - All containers run unprivileged
#    - no-new-privileges prevents privilege escalation
#    - Resource limits prevent DoS attacks
#
# 4. Authentication:
#    - Bitcoin RPC uses cookie-based authentication
#    - Database credentials: Hardcoded for now (TODO: implement secrets management)
#
# WARNING: Database passwords are currently hardcoded in environment variables.
# For production deployments, implement proper secrets management.

{
  config,
  lib,
  pkgs,
  ...
}:

with lib;

let
  cfg = config.services.eigenix-mempool;
  settings = config.eigenix.finalSettings;
in
{
  options.services.eigenix-mempool = {
    enable = mkEnableOption "Eigenix mempool.space Bitcoin block explorer";
  };

  config = mkIf cfg.enable {
    # Runtime
    virtualisation.podman.enable = true;

    # Ensure data directories exist with secure permissions
    systemd.tmpfiles.rules = [
      # Mempool data directory - 700 for maximum security (owner-only access)
      "d ${settings.storage.baseDataDir}/mempool 0700 1000 1000 -"
      # MariaDB uses UID 999 in the container
      "d ${settings.storage.baseDataDir}/mempool/mysql 0700 999 999 -"
      # Cache owned by mempool-api user
      "d ${settings.storage.baseDataDir}/mempool/cache 0700 1000 1000 -"
    ];

    # Mempool.space stack
    virtualisation.oci-containers.containers = {
      "mempool-db" = {
        image = "mariadb@sha256:9e7695800ab8fa72d75053fe536b090d0c9373465b32a073c73bc7940a2e8dbe";
        volumes = [
          "${settings.storage.baseDataDir}/mempool/mysql:/var/lib/mysql:rw"
        ];
        environment = {
          MYSQL_DATABASE = "mempool";
          MYSQL_USER = "mempool";
          MYSQL_PASSWORD = "mempool";
          MYSQL_ROOT_PASSWORD = "admin";
        };
        extraOptions = [
          "--network-alias=mempool-db"
          "--network=eigenix-network"
          # Resource limits for security
          "--memory=2g"
          "--cpus=2.0"
          "--pids-limit=512"
          # Security options
          "--security-opt=no-new-privileges:true"
        ];
      };

      "mempool-web" = {
        image = "mempool/frontend@sha256:1f33796b56bb661ac7b417d11d6c44c467f51c808ea3c48748a2428e1bed918c";
        user = "1000:1000";
        environment = {
          FRONTEND_HTTP_PORT = "8080";
          BACKEND_MAINNET_HTTP_HOST = "mempool-api";
        };
        ports = [
          "0.0.0.0:${toString config.eigenix.finalSettings.ports.mempoolWeb}:8080"
        ];
        dependsOn = [ "mempool-api" ];
        extraOptions = [
          "--network-alias=mempool-web"
          "--network=eigenix-network"
          # Resource limits for security
          "--memory=512m"
          "--cpus=1.0"
          "--pids-limit=256"
          # Security options
          "--security-opt=no-new-privileges:true"
        ];
      };

      "mempool-api" = {
        image = "mempool/backend@sha256:edc4cc7b27b8d6267abb74f76b80dd0258803377d7a60fd1d050c4786369d15a";
        dependsOn = [
          "mempool-db"
          "bitcoind"
        ];
        volumes = [
          "${settings.storage.baseDataDir}/mempool/cache:/backend/cache:rw"
          "${settings.storage.baseDataDir}/bitcoind-data:/bitcoind-data:ro"
        ];
        environment = {
          MEMPOOL_BACKEND = "electrum";
          ELECTRUM_HOST = "electrs";
          ELECTRUM_PORT = toString settings.ports.electrs;
          ELECTRUM_TLS_ENABLED = "false";
          CORE_RPC_HOST = "bitcoind";
          CORE_RPC_PORT = toString settings.ports.bitcoinRpc;
          CORE_RPC_COOKIE = "true";
          CORE_RPC_COOKIE_PATH = "/bitcoind-data/.cookie";
          DATABASE_ENABLED = "true";
          DATABASE_HOST = "mempool-db";
          DATABASE_DATABASE = "mempool";
          DATABASE_USERNAME = "mempool";
          DATABASE_PASSWORD = "mempool";
          STATISTICS_ENABLED = "true";
        };
        user = "1000:1000";
        extraOptions = [
          "--network-alias=mempool-api"
          "--network=eigenix-network"
          # Resource limits for security
          "--memory=2g"
          "--cpus=2.0"
          "--pids-limit=512"
          # Security options
          "--security-opt=no-new-privileges:true"
        ];
      };
    };

    # Systemd service configurations for mempool stack
    systemd.services = {
      "podman-mempool-db" = {
        serviceConfig.Restart = lib.mkOverride 90 "always";
        after = [
          "podman-network-eigenix.service"
        ];
        requires = [
          "podman-network-eigenix.service"
        ];
        partOf = [ "eigenix-root.target" ];
        wantedBy = [ "eigenix-root.target" ];
      };

      "podman-mempool-api" = {
        serviceConfig.Restart = lib.mkOverride 90 "always";
        after = [
          "podman-network-eigenix.service"
          "podman-mempool-db.service"
        ];
        requires = [
          "podman-network-eigenix.service"
          "podman-mempool-db.service"
        ];
        # Make this optional - don't fail if bitcoind/electrs aren't available
        wants = [
          "podman-bitcoind.service"
          "podman-electrs.service"
        ];
        partOf = [ "eigenix-root.target" ];
        wantedBy = [ "eigenix-root.target" ];
      };

      "podman-mempool-web" = {
        serviceConfig.Restart = lib.mkOverride 90 "always";
        after = [
          "podman-network-eigenix.service"
          "podman-mempool-api.service"
        ];
        requires = [
          "podman-network-eigenix.service"
          "podman-mempool-api.service"
        ];
        partOf = [ "eigenix-root.target" ];
        wantedBy = [ "eigenix-root.target" ];
      };
    };

    # Firewall
    networking.firewall.allowedTCPPorts = [
      config.eigenix.finalSettings.ports.mempoolWeb
    ];

    # Packages for management
    environment.systemPackages = with pkgs; [
      curl
      jq
    ];
  };
}
