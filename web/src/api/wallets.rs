use crate::api::ApiClient;
use crate::types::metrics::{WalletBalances, WalletHealth};
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct DepositAddress {
    pub address: String,
}

/// Fetch combined wallet balances (Bitcoin and Monero)
pub async fn fetch_wallet_balances() -> Result<WalletBalances, String> {
    ApiClient::get("/wallets/balances").await
}

/// Fetch wallet health status
pub async fn fetch_wallet_health() -> Result<WalletHealth, String> {
    ApiClient::get("/wallets/health").await
}

/// Fetch Bitcoin deposit address
pub async fn fetch_bitcoin_address() -> Result<DepositAddress, String> {
    ApiClient::get("/wallets/bitcoin/address").await
}

/// Fetch Monero deposit address
pub async fn fetch_monero_address() -> Result<DepositAddress, String> {
    ApiClient::get("/wallets/monero/address").await
}

