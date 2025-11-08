# Eigenix Nix Configuration System

This directory contains the Nix configuration for deploying Eigenix services using NixOS. The configuration system uses a centralized parameters file that can be managed via the `eigenix` CLI tool.

## Architecture

```
nix/
├── parameters.nix          # Schema definition for all configurable parameters
├── module.nix              # Main module that orchestrates all services
├── ports.nix               # Port allocation and conflict detection
├── asb.nix                 # ASB (Automated Swap Backend) service
├── backend.nix             # Eigenix backend API service
├── mempool.nix             # Mempool.space block explorer service
├── web.nix                 # Eigenix web frontend service
├── templates/              # Template configurations
│   └── mainnet.nix         # Production mainnet template
└── instances/              # User configuration instances
    └── parameters.json     # Active deployment parameters (managed by CLI)
```

## Quick Start

### 1. Initialize Configuration

Use the CLI to create your deployment configuration interactively:

```bash
# Interactive setup with prompts
eigenix init

# Or use a template without prompts
eigenix init --template mainnet --yes
eigenix init --template testnet --yes
```

This creates `nix/instances/parameters.json` with your deployment settings.

### 2. Review Configuration

```bash
# Show overview
eigenix show

# Show full JSON configuration
eigenix show deployment
```

### 3. Configure Additional Settings

Adjust specific configuration sections:

```bash
# Interactive section configuration
eigenix configure

# Configure specific section
eigenix configure asb
eigenix configure mempool
eigenix configure ports
```

### 4. Validate Configuration

Check for errors before deployment:

```bash
eigenix validate
```

### 5. Deploy with NixOS

Add to your `/etc/nixos/configuration.nix`:

```nix
{
  imports = [
    /path/to/eigenix/nix/module.nix
  ];

  services.eigenix.enable = true;
  
  # All parameters are read from nix/instances/parameters.json
  # You can override specific settings here if needed:
  # eigenix.parameters.asb.minBuyBtc = 0.005;
}
```

Then apply:

```bash
sudo nixos-rebuild switch
```

## Configuration System

### Parameters Schema

All configurable parameters are defined in `parameters.nix` and organized into sections:

#### Deployment Metadata
- **name**: Deployment identifier
- **environment**: production | staging | development | testnet
- **domain**: Optional domain for web services

#### Storage Configuration
- **baseDataDir**: Base directory for all data (default: `/mnt/vault`)
- **bitcoindDataDir**: Bitcoin blockchain data
- **monerodDataDir**: Monero blockchain data
- **asbDataDir**: ASB wallet and swap data
- **mempoolDataDir**: Mempool explorer data

#### Network Configuration
- **bitcoin**: Mainnet | Testnet | Signet | Regtest
- **monero**: Mainnet | Stagenet | Testnet

#### ASB (Automated Swap Backend)
- **enable**: Enable/disable ASB service
- **externalAddresses**: Libp2p multiaddrs for P2P discovery
- **enableTor**: Enable Tor hidden service
- **minBuyBtc**: Minimum BTC amount per swap
- **maxBuyBtc**: Maximum BTC amount per swap
- **askSpread**: Price spread percentage (e.g., 0.02 = 2%)
- **priceTickerUrl**: WebSocket URL for price feed
- **externalBitcoinAddress**: Fixed Bitcoin address for redeems (optional)
- **developerTip**: Donation percentage to developers
- **rendezvousPoints**: Libp2p rendezvous servers

#### Bitcoin Node
- **enable**: Enable Bitcoin Core node
- **prune**: Enable pruning mode
- **txindex**: Maintain transaction index
- **dbcache**: Database cache size (MB)
- **maxconnections**: Maximum peer connections
- **rpcallowip**: Allowed IPs for RPC access

#### Monero Node
- **enable**: Enable Monero daemon
- **restrictedRpc**: Run RPC in restricted mode
- **enableDnsBlocklist**: Block malicious peers via DNS
- **maxIncomingConnections**: Max incoming peer connections
- **maxOutgoingConnections**: Max outgoing peer connections

#### Electrs (Electrum Server)
- **enable**: Enable Electrs service
- **logFilters**: Log level (TRACE | DEBUG | INFO | WARN | ERROR)

#### Mempool Explorer
- **enable**: Enable mempool.space block explorer
- **enableStatistics**: Enable statistics collection
- **backend**: Backend type (electrum | esplora)
- **database**: Database credentials (TODO: migrate to secrets)

#### Backend API
- **enable**: Enable Eigenix backend API
- **host**: Bind address (127.0.0.1 for localhost only)
- **logLevel**: Rust log level

#### Web Frontend
- **enable**: Enable Eigenix web frontend
- **host**: Bind address

#### Ports
All service ports are configurable to avoid conflicts:
- **asbP2p**: ASB P2P port (default: 9939)
- **asbRpc**: ASB RPC port (default: 9944)
- **bitcoinRpc**: Bitcoin RPC (default: 8332)
- **bitcoinP2p**: Bitcoin P2P (default: 8333)
- **electrs**: Electrs RPC (default: 50001)
- **moneroRpc**: Monero RPC (default: 18081)
- **moneroP2p**: Monero P2P (default: 18080)
- **mempoolWeb**: Mempool web (default: 8999)
- **mempoolApi**: Mempool API (default: 8998)
- **eigenixWeb**: Web frontend (default: 8080)
- **eigenixBackend**: Backend API (default: 3000)

#### Resources
Container resource limits for each service:
- **memory**: Memory limit (e.g., "8g")
- **cpus**: CPU limit (e.g., "4.0")

#### Container Images
Pinned container image references for reproducibility.

#### Monitoring
- **enableMetrics**: Enable Prometheus metrics
- **enableTracing**: Enable distributed tracing
- **logRetentionDays**: Log retention period

## CLI Commands

### `eigenix init`

Initialize deployment configuration from a template.

```bash
# Interactive mode (recommended for first-time setup)
eigenix init

# Quick setup with template
eigenix init --template mainnet --yes
eigenix init --template testnet --yes
```

Templates:
- **mainnet**: Production Bitcoin/Monero mainnet deployment
- **testnet**: Testing with Bitcoin testnet and Monero stagenet

### `eigenix configure [section]`

Configure deployment parameters interactively.

```bash
# Select section interactively
eigenix configure

# Configure specific section
eigenix configure deployment   # Metadata and domain
eigenix configure storage      # Data directories
eigenix configure networks     # Bitcoin/Monero networks
eigenix configure asb          # ASB parameters
eigenix configure mempool      # Mempool explorer
eigenix configure backend      # Backend API
eigenix configure web          # Web frontend
eigenix configure ports        # Port allocations
```

### `eigenix show [section]`

Display current configuration.

```bash
# Show overview with key settings
eigenix show

# Show full JSON configuration
eigenix show all

# Show specific section
eigenix show deployment
```

### `eigenix validate`

Validate configuration for errors.

```bash
eigenix validate
```

Checks for:
- Empty required fields
- Invalid value ranges (e.g., min > max)
- Port conflicts
- Network misconfigurations

## Deployment Scenarios

### Production Mainnet ASB

```bash
eigenix init --template mainnet
eigenix configure asb
# Set external addresses, swap limits, spread
eigenix validate
```

Key settings:
- Bitcoin: Mainnet
- Monero: Mainnet
- Tor: Enabled
- External addresses: Required for P2P discovery
- Swap limits: Conservative defaults (0.002-0.02 BTC)

### Development/Testing

```bash
eigenix init --template testnet
eigenix configure
# Adjust as needed for testing
```

Key settings:
- Bitcoin: Testnet
- Monero: Stagenet
- Tor: Disabled (faster testing)
- Swap limits: Higher for testing (0.001-0.1 BTC)

### Web-Only Deployment

```bash
eigenix init
eigenix configure
# Enable: mempool, backend, web
# Disable: asb
```

Deploy mempool explorer and Eigenix web UI without running ASB.

## Advanced Configuration

### Manual Parameter Editing

While the CLI is recommended, you can directly edit `nix/instances/parameters.json`:

```json
{
  "asb": {
    "enable": true,
    "minBuyBtc": 0.005,
    "maxBuyBtc": 0.05,
    ...
  }
}
```

After editing, always validate:

```bash
eigenix validate
```

### Overriding Parameters in NixOS Config

You can override any parameter in your NixOS configuration:

```nix
{
  services.eigenix.enable = true;
  
  # Override parameters from parameters.json
  eigenix.parameters = {
    asb.minBuyBtc = 0.005;
    asb.maxBuyBtc = 0.05;
    ports.asbP2p = 19939;  # Use non-standard port
  };
}
```

### Multi-Instance Deployments

To run multiple instances on the same machine:

1. Create separate configurations:
   ```bash
   cp nix/instances/parameters.json nix/instances/instance1.json
   cp nix/instances/parameters.json nix/instances/instance2.json
   ```

2. Edit each with different:
   - Data directories
   - Ports
   - External addresses

3. Import in NixOS config:
   ```nix
   # TODO: Multi-instance support to be implemented
   ```

## Service Management

### Systemd Targets

All services are grouped under the `eigenix-root.target`:

```bash
# Start all services
sudo systemctl start eigenix-root.target

# Stop all services
sudo systemctl stop eigenix-root.target

# Check status
sudo systemctl status eigenix-root.target
```

### Individual Service Control

```bash
# ASB services
sudo systemctl status podman-asb
sudo systemctl status podman-bitcoind
sudo systemctl status podman-monerod
sudo systemctl status podman-electrs

# Mempool services
sudo systemctl status podman-mempool-web
sudo systemctl status podman-mempool-api
sudo systemctl status podman-mempool-db

# Eigenix services
sudo systemctl status eigenix-backend
sudo systemctl status eigenix-web
```

### Logs

```bash
# View ASB logs
sudo journalctl -u podman-asb -f

# View all eigenix services
sudo journalctl -u 'podman-*' -u 'eigenix-*' -f

# ASB tracing logs (on disk)
sudo tail -f /mnt/vault/asb-data/logs/tracing*.log
```

## Security Considerations

### File Permissions

All data directories use `700` permissions (owner-only access):
- `/mnt/vault/asb-data` - ASB wallet and seed
- `/mnt/vault/bitcoind-data` - Bitcoin blockchain
- `/mnt/vault/monerod-data` - Monero blockchain
- `/mnt/vault/mempool` - Mempool database

The ASB seed file (`seed.pem`) has `600` permissions.

### Network Security

- Bitcoin RPC: Restricted to `127.0.0.1` and container network
- ASB RPC: Localhost only (`127.0.0.1`)
- ASB P2P: Public on `0.0.0.0:9939` (required for swaps)
- Tor: Enabled by default for ASB hidden service

### Container Security

All containers run:
- As unprivileged users
- With `no-new-privileges` flag
- With resource limits (memory/CPU)
- On isolated `asb-network` network

### Secrets Management

⚠️ **WARNING**: Database passwords are currently hardcoded in `parameters.json`.

For production deployments, implement proper secrets management:
- Option 1: Use [agenix](https://github.com/ryantm/agenix) (age-encrypted secrets)
- Option 2: Use [sops-nix](https://github.com/Mic92/sops-nix) (SOPS-encrypted secrets)

TODO: Add secrets management support to CLI and modules.

## Firewall

Firewall rules are automatically configured based on enabled services:
- ASB P2P port (9939): Always open when ASB enabled
- Mempool web port (8999): Open when mempool enabled
- Web frontend port (8080): Open when web enabled

Additional ports (RPC, internal services) remain closed by default.

## Monitoring

### Health Checks

```bash
# CLI health check
eigenix health

# Check ASB RPC
curl http://localhost:9944/health

# Check backend API
curl http://localhost:3000/health
```

### Metrics (TODO)

Prometheus metrics collection can be enabled:

```bash
eigenix configure monitoring
# Enable metrics collection
```

## Troubleshooting

### Configuration Not Found

```
Error: Could not find eigenix project root
```

**Solution**: Run commands from within the eigenix project directory, or any subdirectory.

### Port Conflicts

```
Error: Port conflict detected
```

**Solution**: Edit ports in configuration:
```bash
eigenix configure ports
```

### Container Start Failures

**Check logs**:
```bash
sudo journalctl -u podman-<service> -n 50
```

**Common issues**:
- Data directory permissions
- Port already in use
- Insufficient disk space
- Container image pull failures

### Blockchain Sync Issues

**Bitcoin/Monero sync takes a long time** (days/weeks for initial sync).

**Check sync status**:
```bash
# Bitcoin
bitcoin-cli -datadir=/mnt/vault/bitcoind-data getblockchaininfo

# Monero
curl http://localhost:18081/get_info
```

## Migration from Old Configs

If you have existing crypto module configurations in `/modules/crypto/`:

1. Review old settings in `modules/crypto/asb.nix`
2. Initialize new config: `eigenix init`
3. Apply old settings: `eigenix configure`
4. Update NixOS config to import `eigenix/nix/module.nix`
5. Remove old crypto module imports

The new system consolidates ASB, mempool, backend, and web into a unified configuration.

## Contributing

When adding new configurable parameters:

1. Add to `parameters.nix` with proper type and description
2. Update module files to read from parameters
3. Add CLI support in `cli/src/config.rs` and `cli/src/main.rs`
4. Update this README with the new parameter
5. Update default values in `instances/parameters.json`

## Support

For issues or questions:
- Check logs: `sudo journalctl -u eigenix-root.target`
- Validate config: `eigenix validate`
- Review security settings in module files
- Open an issue on GitHub

## License

See project root for license information.
