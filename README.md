# Eigenix

Eigenix is a NixOS flake that provides a hardened, self-hosted deployment of the EigenLayer protocol for Bitcoin-to-Monero atomic swaps. In an era where governments and state actors suppress privacy-focused cryptocurrencies like Monero—leading to delistings from centralized exchanges and persecution of developers (e.g., Samurai Wallet for Bitcoin coinjoins)—Eigenix empowers users to run their own swap infrastructure, bypassing censorship and enabling private capital exchange.

Built on the EigenLayer protocol, Eigenix integrates atomic swap backends with managed Bitcoin and Monero nodes, plus a trading engine that arbitrages fees against Kraken exchange rates.

## Features

- **Hardened Node Deployment**: Full Bitcoin Core and Monero nodes with security hardening via NixOS modules.
- **Atomic Swap Backend**: Eigenswap service for secure BTC ↔ XMR swaps.
- **Trading Engine**: Automated arbitrage using Kraken API for real-time exchange rates.
- **Web Dashboard**: Modern Dioxus-based UI for monitoring balances, metrics, and swap operations.
- **CLI Tools**: Command-line interface for wallet management and swap initiation.
- **NixOS Integration**: Flake-based configuration for easy deployment and secrets management (via agenix).

## Architecture

```
eigenix/
├── flake.nix          # Main Nix flake definition
├── backend/           # Rust backend: Eigenswap API, Kraken integration, metrics collection
│   ├── src/routes/    # Axum routes for API endpoints (health, metrics, swaps, kraken tickers)
│   └── tests/         # Integration tests for backend services
├── web/               # Dioxus 0.7 web frontend
│   └── src/           # Components: Dashboard, Header (tickers), Modals (deposits), Metrics charts
├── cli/               # Rust CLI binary for user interactions
├── scripts/           # Deployment and maintenance scripts
├── nix/               # NixOS modules: Common configs, ports, services (backend/web systemd units)
└── hosts/             # Host-specific NixOS configurations
```

## Setup

1. **Clone and Enter Directory**:
   ```bash
   git clone https://github.com/Mo0nbase/eigenix.git
   cd eigenix
   ```

2. **Secrets Management**:
   - Use agenix for secrets: Edit `secrets.nix` and decrypt with `agenix -e hosts/{hostname}/{secret}.age`.
   - Required secrets: Wallet RPC passwords, API keys (Kraken), WireGuard keys.

3. **Nix Development Shell** (optional for building):
   ```bash
   nix develop
   ```

## Usage

### Deployment (NixOS)

Deploy to a NixOS host (e.g., hostname `your-host`):

```bash
# Test changes
sudo nixos-rebuild dry-run --flake '.#your-host'

# Apply changes
sudo nixos-rebuild switch --flake '.#your-host'
```

This starts:
- `eigenix-backend.service`: Rust API server (port 3000).
- `eigenix-web.service`: Dioxus web app (default port 80/443 via nginx).
- Managed nodes: `bitcoind`, `monero-wallet-rpc`, metrics collectors.

### Web Dashboard

Access at `http://your-host/` (or domain). Features:
- Real-time balances (BTC/XMR).
- Node health and metrics charts (block height, difficulty, swap stats).
- Kraken tickers (BTC/USD, XMR/USD, XMR/BTC with 24h % changes).
- Deposit modals with QR codes and copy-to-clipboard.

### CLI

Build and run CLI tools:
```bash
cd cli
cargo run -- --help  # View commands
cargo run -- swap init --from btc --to xmr --amount 0.01
```

Commands include: `balance check`, `swap create`, `metrics fetch`.

### Backend API

Interact with REST API at `http://your-host:3000`:
- `/health`: System status.
- `/metrics/{coin}/interval?minutes=5`: Historical data.
- `/kraken/tickers`: Exchange rates.
- `/swaps`: Atomic swap operations.

### Maintenance

- **Update Packages**: Edit `flake.nix` and rebuild.
- **Logs**: `journalctl -u eigenix-backend.service -f`.
- **Testing**: Backend tests via `cargo test` in `backend/`.
- **Nix Init for Hashes**: Use `nix-init` for Rust crate dependencies (e.g., Kraken API crates).

## Contributing

1. Fork and clone the repo.
2. Create a feature branch: `git checkout -b feature/new-metric`.
3. Commit changes and push.
4. Test locally with `nixos-rebuild dry-run`.
5. Open a PR.

For issues or questions, open a GitHub issue.

## License

[MIT](LICENSE) or see `LICENSE` file.
