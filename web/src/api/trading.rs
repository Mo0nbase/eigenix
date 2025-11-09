use crate::api::ApiClient;
use crate::types::metrics::{TradingConfig, TradingStatus};
use serde::Serialize;

/// Fetch trading engine status
pub async fn fetch_trading_status() -> Result<TradingStatus, String> {
    ApiClient::get("/trading/status").await
}

/// Fetch current trading configuration
pub async fn fetch_trading_config() -> Result<TradingConfig, String> {
    ApiClient::get("/trading/config").await
}

/// Update trading configuration
pub async fn update_trading_config(config: &TradingConfig) -> Result<TradingConfig, String> {
    ApiClient::put("/trading/config", config).await
}

#[derive(Serialize)]
struct EnableRequest {
    enabled: bool,
}

#[derive(serde::Deserialize)]
struct EnableResponse {
    success: bool,
    enabled: bool,
}

/// Enable or disable the trading engine
pub async fn set_trading_enabled(enabled: bool) -> Result<bool, String> {
    let response: EnableResponse = ApiClient::post("/trading/enable", &EnableRequest { enabled }).await?;
    Ok(response.enabled)
}

