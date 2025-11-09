use crate::api::ApiClient;
use crate::types::metrics::{WalletBalances, WalletHealth};

/// Fetch combined wallet balances (Bitcoin and Monero)
pub async fn fetch_wallet_balances() -> Result<WalletBalances, String> {
    ApiClient::get("/wallets/balances").await
}

/// Fetch wallet health status
pub async fn fetch_wallet_health() -> Result<WalletHealth, String> {
    ApiClient::get("/wallets/health").await
}

