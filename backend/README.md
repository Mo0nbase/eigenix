# Eigenix Backend

The Eigenix backend is an Axum-based REST API that provides programmatic access to Bitcoin and Monero wallets managed by the ASB (Atomic Swap Backend) service. It serves as the bridge between the Eigenix web frontend and the underlying swap infrastructure.

## Architecture

```
┌─────────────────┐
│  Eigenix Web    │
│   (Frontend)    │
└────────┬────────┘
         │ HTTP
         ▼
┌─────────────────┐
│ Eigenix Backend │◄──── You are here
│   (Axum API)    │
└────────┬────────┘
         │
    ┌────┼────┬──────────┬────────────┐
    ▼    ▼    ▼          ▼            ▼
  ┌───┐ ┌──┐ ┌───┐   ┌──────┐   ┌─────────┐
  │ASB│ │BTC│ │XMR│   │Surreal│  │bitcoind │
  │RPC│ │RPC│ │RPC│   │  DB   │  │monerod  │
  └───┘ └──┘ └───┘   └──────┘   └─────────┘
```

## Core Responsibilities

1. **Wallet Management**: Initializes and manages Bitcoin and Monero wallets using descriptors/seeds from ASB
2. **Metrics Collection**: Collects and stores blockchain metrics (prices, balances, fees) in SurrealDB
3. **API Endpoints**: Exposes REST endpoints for wallet operations, container management, and metrics
4. **Health Monitoring**: Provides health checks and service status information

## Key Components

### Wallet Initialization

The backend uses a smart initialization strategy:

1. **Try existing wallets first**: Attempts to connect to already-loaded wallets
2. **Initialize from ASB**: If connection fails, retrieves wallet data from ASB and initializes
3. **Handle edge cases**: 
   - Bitcoin: Handles "wallet already exists" and "already loaded" errors
   - Monero: Closes existing wallet before opening to avoid conflicts
   - Descriptors: Automatically adds checksums if missing

### Important Files

- `src/main.rs`: Application entry point, server setup, route definitions
- `src/wallets/`: Wallet management logic (Bitcoin, Monero)
  - `bitcoin.rs`: Bitcoin Core RPC integration
  - `monero.rs`: Monero wallet RPC integration
  - `manager.rs`: Wallet lifecycle management
- `src/metrics.rs`: Background metrics collection task
- `tests/`: Integration tests (see Testing section)

## Configuration

The backend is configured via environment variables:

```bash
# Server
BIND_HOST=127.0.0.1
BIND_PORT=3000

# Database
SURREALDB_URL=http://127.0.0.1:8001
SURREALDB_USER=root
SURREALDB_PASS=root

# Services
ASB_RPC_URL=http://127.0.0.1:9944
BITCOIN_RPC_URL=http://127.0.0.1:8332
BITCOIN_COOKIE_PATH=/mnt/vault/bitcoind-data/.cookie
MONERO_WALLET_RPC_URL=http://127.0.0.1:18082/json_rpc
WALLET_NAME=eigenix

# Logging
RUST_LOG=info
```

## Testing

Integration tests are located in `tests/` and require running services:

- `asb_integration.rs`: Tests ASB RPC communication
- `bitcoin_wallet_integration.rs`: Tests Bitcoin wallet operations
- `monero_wallet_integration.rs`: Tests Monero wallet operations

**Running tests:**

```bash
# Set test environment
export IN_CONTAINER=false  # or true if running in container
export ASB_RPC_URL=http://localhost:9944
export BITCOIN_RPC_URL=http://localhost:8332
# ... other vars

# Run with nextest
cargo nextest run --test "*integration*"
```

The `common/mod.rs` module provides `TestConfig` that automatically switches between localhost and container hostnames based on `IN_CONTAINER`.

## Development Tips

### Wallet State Management

- **Bitcoin wallets persist** across restarts (stored in bitcoind)
- **Monero wallets persist** in `/wallet` directory (mounted volume)
- If wallet initialization fails, check that:
  1. ASB service is running and accessible
  2. Cookie file exists and is readable (Bitcoin)
  3. monerod is synced (Monero)

### Common Issues

1. **"Wallet already loaded"**: The backend handles this gracefully now, but if you see errors:
   - Bitcoin: Restart bitcoind or unload wallet manually
   - Monero: Wallet RPC automatically closes before opening

2. **"Missing checksum"**: Fixed - descriptors from ASB are now automatically given checksums

3. **Connection refused**: Ensure all services are up:
   ```bash
   systemctl status podman-{surrealdb,asb,monero-wallet-rpc}
   ```

### Adding New Endpoints

1. Define handler in appropriate module (e.g., `src/wallets/bitcoin.rs`)
2. Add route in `src/main.rs` router setup
3. Update `AppState` if new dependencies needed
4. Add integration test in `tests/`

## Deployment

The backend is deployed as a systemd service via NixOS:

```nix
services.eigenix-backend.enable = true;
```

Configuration is managed in `nix/backend.nix` which:
- Sets up SurrealDB and monero-wallet-rpc containers
- Configures networking within `eigenix-network`
- Manages service dependencies and restart policies
- Sets resource limits and security options

## Dependencies

### Runtime Services
- **SurrealDB**: Metrics storage
- **ASB**: Wallet seed/descriptor source
- **bitcoind**: Bitcoin blockchain node
- **monero-wallet-rpc**: Monero wallet operations
- **monerod**: Monero blockchain node (via wallet RPC)

