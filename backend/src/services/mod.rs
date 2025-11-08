/// Service modules for exchange and swap operations
///
/// This module provides interfaces for:
/// - Bitcoin node RPC (blockchain info, metrics)
/// - Monero node RPC (blockchain info, metrics)
/// - Kraken exchange operations (trading, deposits, withdrawals)
/// - ASB (Automated Swap Backend) operations (atomic swaps)
pub mod asb;
pub mod bitcoin;
pub mod kraken;
pub mod monero;

pub use asb::AsbClient;
pub use bitcoin::BitcoinRpcClient;
pub use kraken::KrakenClient;
pub use monero::MoneroRpcClient;
