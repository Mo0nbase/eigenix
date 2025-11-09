use axum::{
    extract::State,
    routing::{get, post},
    Json, Router,
};
use serde::Serialize;

use crate::AppState;

/// Monero wallet balance response
#[derive(Serialize)]
pub struct MoneroBalance {
    /// Balance in XMR
    balance: f64,
}

/// Monero wallet health response
#[derive(Serialize)]
pub struct MoneroHealth {
    /// Whether Monero wallet is ready and operational
    ready: bool,
}

/// Refresh Monero wallet response
#[derive(Serialize)]
pub struct RefreshResponse {
    /// New wallet height after refresh
    height: u64,
}

/// Get Monero wallet balance
pub async fn get_balance(
    State(state): State<AppState>,
) -> Result<Json<MoneroBalance>, String> {
    let balance = state
        .wallets
        .get_monero_balance()
        .await
        .map_err(|e| format!("Failed to get Monero balance: {}", e))?;

    Ok(Json(MoneroBalance { balance }))
}

/// Check Monero wallet health
pub async fn get_health(
    State(state): State<AppState>,
) -> Result<Json<MoneroHealth>, String> {
    let ready = state.wallets.monero.is_ready().await;

    Ok(Json(MoneroHealth { ready }))
}

/// Refresh Monero wallet to sync with blockchain
pub async fn refresh_wallet(
    State(state): State<AppState>,
) -> Result<Json<RefreshResponse>, String> {
    let height = state
        .wallets
        .refresh_monero()
        .await
        .map_err(|e| format!("Failed to refresh Monero wallet: {}", e))?;

    Ok(Json(RefreshResponse { height }))
}

/// Create the Monero wallet routes router
pub fn monero_routes() -> Router<AppState> {
    Router::new()
        .route("/balance", get(get_balance))
        .route("/health", get(get_health))
        .route("/refresh", post(refresh_wallet))
}
