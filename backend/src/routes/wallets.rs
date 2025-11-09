use axum::{
    extract::State,
    routing::get,
    Json, Router,
};
use serde::Serialize;

use crate::routes::{bitcoin, monero};
use crate::{ApiError, ApiResult, AppState};

/// Combined wallet balances response
#[derive(Serialize)]
pub struct WalletBalances {
    /// Bitcoin balance in BTC
    bitcoin: f64,
    /// Monero balance in XMR
    monero: f64,
}


/// Wallet health status response
#[derive(Serialize)]
pub struct WalletHealth {
    /// Whether wallets are healthy and operational
    healthy: bool,
    /// Individual wallet health status
    bitcoin_ready: bool,
    monero_ready: bool,
}


/// Get combined balances for both Bitcoin and Monero wallets
pub async fn get_balances(
    State(state): State<AppState>,
) -> ApiResult<Json<WalletBalances>> {
    let (bitcoin, monero) = state
        .wallets
        .get_balances()
        .await
        .map_err(ApiError::Wallet)?;

    Ok(Json(WalletBalances { bitcoin, monero }))
}

/// Check wallet health status
pub async fn get_wallet_health(
    State(state): State<AppState>,
) -> ApiResult<Json<WalletHealth>> {
    let healthy = state.wallets.is_healthy().await;
    let bitcoin_ready = state.wallets.bitcoin.is_ready().await;
    let monero_ready = state.wallets.monero.is_ready().await;

    Ok(Json(WalletHealth {
        healthy,
        bitcoin_ready,
        monero_ready,
    }))
}

/// Create the wallet routes router
pub fn wallet_routes() -> Router<AppState> {
    Router::new()
        .route("/balances", get(get_balances))
        .route("/health", get(get_wallet_health))
        .nest("/bitcoin", bitcoin::bitcoin_routes())
        .nest("/monero", monero::monero_routes())
}