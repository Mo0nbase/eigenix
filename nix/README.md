# Eigenix NixOS Module

Complete deployment orchestration for crypto services including ASB (Automated Swap Backend), Bitcoin/Monero nodes, mempool.space block explorer, and Eigenix web/backend services.

## Directory Structure

```
nix/
├── module.nix           # Main Eigenix module (services.eigenix)
├── asb.nix             # ASB with Bitcoin/Monero nodes (services.eigenix-asb)
├── mempool.nix         # Mempool.space block explorer (services.eigenix-mempool)
├── web.nix             # Web frontend service (services.eigenix-web)
├── backend.nix         # Backend API service (services.eigenix-backend)
├── ports.nix           # Port configuration (services.eigenix-ports)
├── templates/
│   └── mainnet.nix     # Production mainnet template
├── instances/           # Your Eigenix instance configurations
│   └── (your configs go here)
└── README.md           # This file
```

## Quick Start

### 1. Create a Configuration

Use the eigenix CLI to generate a new configuration:

```bash
cd projects/eigenix
eigenix config new my-eigenix --template mainnet
```

Or create one interactively:

```bash
eigenix config new --interactive
```

### 2. Apply the Configuration

Import the configuration in your `/etc/nixos/configuration.nix`:

```nix
{
  imports = [
    /path/to/eigenix/nix/instances/my-eigenix.nix
  ];
}
```

Rebuild your system:

```bash
sudo nixos-rebuild switch
```

### 3. Monitor Services

Check service status:

```bash
sudo systemctl status eigenix-root
```

View logs:

```bash
# ASB logs
journalctl -u podman-asb -f

# Bitcoin sync
journalctl -u podman-bitcoind -f

# Monero sync
journalctl -u podman-monerod -f

# Mempool.space
journalctl -u podman-mempool-web -f
```

## Configuration

### Main Service Options

```nix
services.eigenix = {
  enable = true;
  
  # Base directory for all persistent data
  baseDataDir = "/mnt/vault";
  
  # Enable/disable individual components
  components = {
    asb = true;        # ASB + Bitcoin/Monero nodes
    mempool = true;    # Mempool.space block explorer
    web = false;       # Eigenix web frontend
    backend = false;   # Eigenix backend API
  };
  
  # ASB configuration
  asb = {
    externalAddresses = [
      "/dns4/swap.example.com/tcp/9939"
    ];
    enableTor = true;
    minBuyBtc = 0.002;
    maxBuyBtc = 0.02;
    askSpread = 0.02;
    externalBitcoinAddress = null;
  };
  
  # Optional domain for web services
  domain = "eigenix.example.com";
};
```

### Components

#### ASB (Automated Swap Backend)

When enabled (`components.asb = true`), includes:

- **Bitcoin node** (bitcoind) - Full mainnet node with txindex
- **Monero node** (monerod) - Full mainnet node
- **Electrs** - Bitcoin blockchain indexer
- **ASB** - Atomic swap maker service
- **ASB Controller** - Management interface
- **ASB Tracing Logger** - Detailed logging

Configuration options:
- `externalAddresses`: Libp2p multiaddresses for ASB discovery
- `enableTor`: Enable Tor hidden service (requires `services.tor.enable = true`)
- `minBuyBtc` / `maxBuyBtc`: Swap amount limits
- `askSpread`: Price spread percentage (e.g., 0.02 = 2%)
- `externalBitcoinAddress`: Fixed BTC address for swaps (optional)

Data storage:
- Bitcoin blockchain: `${baseDataDir}/bitcoind-data` (~550GB)
- Monero blockchain: `${baseDataDir}/monerod-data` (~170GB)
- Electrs index: `${baseDataDir}/electrs-data` (~100GB)
- ASB wallets/keys: `${baseDataDir}/asb-data` (secure with 700 permissions)

#### Mempool.space

When enabled (`components.mempool = true`), includes:

- **Mempool Web** - Frontend (port 8999)
- **Mempool API** - Backend API
- **MariaDB** - Database for mempool data

Automatically connects to the Bitcoin node from ASB.

#### Web Frontend

When enabled (`components.web = true`):

- Serves the Dioxus-based web interface
- Default port: 8080
- Requires building the web package first

#### Backend API

When enabled (`components.backend = true`):

- Axum-based REST API
- Default port: 3000 (localhost only)
- Provides endpoints for ASB, Bitcoin, and Monero status

### Port Configuration

Default ports (customizable via `services.eigenix-ports`):

```
ASB P2P:          9939  (public)
ASB RPC:          9944  (localhost)
Bitcoin RPC:      8332  (localhost)
Bitcoin P2P:      8333  (public)
Electrs:          50001 (localhost)
Monero RPC:       18081 (localhost)
Monero P2P:       18080 (public)
Mempool Web:      8999  (public)
Eigenix Web:      8080  (public)
Eigenix Backend:  3000  (localhost)
```

## Managing Configurations

### List Configurations

```bash
eigenix config list
```

### Show Configuration

```bash
eigenix config show my-eigenix
```

### Validate Configuration

```bash
eigenix config validate my-eigenix
```

### Delete Configuration

```bash
eigenix config delete my-eigenix
```

## Security

### Permissions

All data directories use restrictive permissions:
- Directories: 700 (owner-only)
- ASB seed file: 600 (CRITICAL - contains cryptographic secrets)
- Wallet files: 700

### Network Security

- Bitcoin RPC: Restricted to localhost and container network
- ASB P2P: Public (required for swap functionality)
- ASB RPC: Localhost only
- All containers run unprivileged (user 1000:1000)
- `no-new-privileges` prevents privilege escalation

### Best Practices

1. **Backup your data**: Regularly backup `${baseDataDir}/asb-data`
2. **Enable Tor**: For anonymity on mainnet
3. **Use firewall**: Only expose necessary ports
4. **Secure external addresses**: Use HTTPS/TLS for public endpoints
5. **Monitor logs**: Watch for unusual activity
6. **Keep secrets safe**: Never commit keys or passwords to version control

## Blockchain Sync

Initial sync times (approximate):

- **Bitcoin**: 1-2 days (550GB, ~860k blocks)
- **Monero**: 1-3 days (170GB, ~3.1M blocks)
- **Electrs**: Additional 1-2 days after Bitcoin sync

Monitor sync progress:

```bash
# Bitcoin
bitcoin-cli -rpcconnect=localhost -rpcport=8332 getblockchaininfo

# Monero
curl -s -X POST http://localhost:18081/json_rpc \
  -d '{"jsonrpc":"2.0","id":"0","method":"get_info"}' \
  -H 'Content-Type: application/json' | jq
```

## Hardware Requirements

Minimum recommended:

- **CPU**: 8+ cores
- **RAM**: 16+ GB
- **Storage**: 1TB+ SSD
  - Bitcoin: ~550GB
  - Monero: ~170GB
  - Electrs: ~100GB
  - System/overhead: ~180GB

## Troubleshooting

### Services won't start

```bash
# Check systemd target
sudo systemctl status eigenix-root

# Check individual services
sudo systemctl status podman-bitcoind
sudo systemctl status podman-monerod
sudo systemctl status podman-asb

# Check network
sudo systemctl status podman-network-asb
```

### Disk space issues

```bash
# Check usage
du -sh /mnt/vault/*

# Clean Docker/Podman cache
sudo podman system prune -a
```

### Permission errors

```bash
# Fix ASB data permissions
sudo chown -R 1000:1000 /mnt/vault/asb-data
sudo chmod -R 700 /mnt/vault/asb-data
```

### Container logs

```bash
# View specific container logs
sudo podman logs bitcoind
sudo podman logs monerod
sudo podman logs asb

# Follow logs in real-time
sudo podman logs -f asb
```

## Development

### Building Web/Backend

The web and backend components require building first:

```bash
# Build web frontend
cd projects/eigenix/web
dx bundle --release

# Build backend
cd projects/eigenix/backend
cargo build --release
```

Then update your configuration:

```nix
services.eigenix.components = {
  asb = true;
  mempool = true;
  web = true;       # Enable after building
  backend = true;   # Enable after building
};
```

## Related Resources

- [Eigenwallet Core](https://github.com/eigenwallet/core) - ASB source
- [UnstoppableSwap](https://unstoppableswap.net/) - Atomic swap protocol
- [Mempool.space](https://mempool.space/) - Block explorer
- [Dioxus](https://dioxuslabs.com/) - Web framework
- [Axum](https://github.com/tokio-rs/axum) - Backend framework

## Support

For issues with:
- **ASB/Swap protocol**: [Eigenwallet Core Issues](https://github.com/eigenwallet/core/issues)
- **Eigenix module**: File an issue in your monorepo
- **Bitcoin/Monero nodes**: Check respective documentation

## License

See LICENSE file in the project root.
