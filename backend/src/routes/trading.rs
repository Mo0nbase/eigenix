use axum::{
    extract::State,
    routing::{get, post, put},
    Json, Router,
};
use serde::{Deserialize, Serialize};

use crate::{
    trading::{config::TradingConfig, engine::TradingStatus},
    ApiError, ApiResult, AppState,
};

/// Request to enable/disable trading
#[derive(Deserialize)]
pub struct EnableRequest {
    enabled: bool,
}

/// Response for enable/disable operations
#[derive(Serialize)]
pub struct EnableResponse {
    success: bool,
    enabled: bool,
}

/// Get trading engine status
pub async fn get_status(State(state): State<AppState>) -> ApiResult<Json<TradingStatus>> {
    let status = state.trading_engine.get_status().await;

    Ok(Json(status))
}

/// Get current trading configuration
pub async fn get_config(State(state): State<AppState>) -> ApiResult<Json<TradingConfig>> {
    let config = state.trading_engine.config.get();
    Ok(Json(config))
}

/// Update trading configuration
pub async fn update_config(
    State(state): State<AppState>,
    Json(new_config): Json<TradingConfig>,
) -> ApiResult<Json<TradingConfig>> {
    state
        .trading_engine
        .config
        .update(new_config.clone())
        .map_err(|e| ApiError::BadRequest(e))?;

    tracing::info!("Trading configuration updated: {:?}", new_config);
    Ok(Json(new_config))
}

/// Enable or disable the trading engine
pub async fn set_enabled(
    State(state): State<AppState>,
    Json(request): Json<EnableRequest>,
) -> ApiResult<Json<EnableResponse>> {
    if request.enabled {
        state.trading_engine.enable();
        tracing::info!("Trading engine enabled via API");
    } else {
        state.trading_engine.disable();
        tracing::info!("Trading engine disabled via API");
    }

    Ok(Json(EnableResponse {
        success: true,
        enabled: request.enabled,
    }))
}

/// Create the trading engine routes router
pub fn trading_routes() -> Router<AppState> {
    Router::new()
        .route("/status", get(get_status))
        .route("/config", get(get_config))
        .route("/config", put(update_config))
        .route("/enable", post(set_enabled))
}
