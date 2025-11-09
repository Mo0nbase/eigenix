use axum::{
    extract::State,
    routing::get,
    Json, Router,
};
use serde::Serialize;

use crate::AppState;

/// Bitcoin wallet balance response
#[derive(Serialize)]
pub struct BitcoinBalance {
    /// Balance in BTC
    balance: f64,
}

/// Bitcoin wallet health response
#[derive(Serialize)]
pub struct BitcoinHealth {
    /// Whether Bitcoin wallet is ready and operational
    ready: bool,
}

/// Get Bitcoin wallet balance
pub async fn get_balance(
    State(state): State<AppState>,
) -> Result<Json<BitcoinBalance>, String> {
    let balance = state
        .wallets
        .get_bitcoin_balance()
        .await
        .map_err(|e| format!("Failed to get Bitcoin balance: {}", e))?;

    Ok(Json(BitcoinBalance { balance }))
}

/// Check Bitcoin wallet health
pub async fn get_health(
    State(state): State<AppState>,
) -> Result<Json<BitcoinHealth>, String> {
    let ready = state.wallets.bitcoin.is_ready().await;

    Ok(Json(BitcoinHealth { ready }))
}

/// Create the Bitcoin wallet routes router
pub fn bitcoin_routes() -> Router<AppState> {
    Router::new()
        .route("/balance", get(get_balance))
        .route("/health", get(get_health))
}
