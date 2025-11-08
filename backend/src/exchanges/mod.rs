/// Exchange and wallet modules for cryptocurrency trading
///
/// This module provides interfaces for:
/// - Bitcoin wallet operations (sending/receiving BTC)
/// - Monero wallet operations (sending/receiving XMR)
/// - Kraken exchange operations (trading, deposits, withdrawals)
pub mod bitcoin;
pub mod kraken;
pub mod monero;

pub use bitcoin::BitcoinWallet;
pub use kraken::KrakenClient;
pub use monero::MoneroWallet;
