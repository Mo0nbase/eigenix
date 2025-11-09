/// Cryptocurrency wallet modules
///
/// This module provides interfaces for:
/// - Bitcoin wallet operations (sending/receiving BTC)
/// - Monero wallet operations (sending/receiving XMR)
/// - Wallet manager for orchestrating initialization from ASB
pub mod bitcoin;
pub mod manager;
pub mod monero;

pub use bitcoin::BitcoinWallet;
pub use manager::{WalletConfig, WalletManager};
pub use monero::MoneroWallet;
