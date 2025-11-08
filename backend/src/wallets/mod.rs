/// Cryptocurrency wallet modules
///
/// This module provides interfaces for:
/// - Bitcoin wallet operations (sending/receiving BTC)
/// - Monero wallet operations (sending/receiving XMR)
pub mod bitcoin;
pub mod monero;

pub use bitcoin::BitcoinWallet;
pub use monero::MoneroWallet;
