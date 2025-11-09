use axum::{extract::State, routing::get, Json, Router};
use serde::Serialize;

use crate::{ApiError, ApiResult, AppState};

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

/// Bitcoin deposit address response
#[derive(Serialize)]
pub struct BitcoinAddress {
    /// Bitcoin deposit address
    address: String,
}

/// Get Bitcoin wallet balance
pub async fn get_balance(State(state): State<AppState>) -> ApiResult<Json<BitcoinBalance>> {
    let balance = state
        .wallets
        .get_bitcoin_balance()
        .await
        .map_err(ApiError::Wallet)?;

    Ok(Json(BitcoinBalance { balance }))
}

/// Check Bitcoin wallet health
pub async fn get_health(State(state): State<AppState>) -> ApiResult<Json<BitcoinHealth>> {
    let ready = state.wallets.bitcoin.is_ready().await;

    Ok(Json(BitcoinHealth { ready }))
}

/// Get a new Bitcoin deposit address
pub async fn get_deposit_address(State(state): State<AppState>) -> ApiResult<Json<BitcoinAddress>> {
    let address = state
        .wallets
        .bitcoin
        .get_new_address(Some("eigenix-deposit"))
        .await
        .map_err(ApiError::Wallet)?;

    Ok(Json(BitcoinAddress { address }))
}

/// Create the Bitcoin wallet routes router
pub fn bitcoin_routes() -> Router<AppState> {
    Router::new()
        .route("/balance", get(get_balance))
        .route("/health", get(get_health))
        .route("/address", get(get_deposit_address))
}
