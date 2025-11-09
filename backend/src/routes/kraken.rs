use axum::{
    extract::State,
    routing::get,
    Json, Router,
};
use anyhow::Context;
use serde::Serialize;

use crate::{services::KrakenClient, ApiError, ApiResult, AppState};

/// Kraken ticker price response
#[derive(Serialize)]
pub struct KrakenTickerResponse {
    /// BTC/USD price
    pub btc_usd: f64,
    /// XMR/USD price
    pub xmr_usd: f64,
    /// BTC/XMR price (XMR per BTC)
    pub btc_xmr: f64,
}

/// Get current Kraken ticker prices
pub async fn get_tickers(State(state): State<AppState>) -> ApiResult<Json<KrakenTickerResponse>> {
    // Create Kraken client using API credentials from config
    let kraken = KrakenClient::new(
        state.config.kraken.api_key.clone(),
        state.config.kraken.api_secret.clone(),
    );

    // Get BTC/USD ticker
    let btc_usd_ticker = kraken
        .get_ticker("XXBTZUSD")
        .await
        .context("Failed to get BTC/USD ticker")?;

    // Get XMR/USD ticker
    let xmr_usd_ticker = kraken
        .get_ticker("XXMRZUSD")
        .await
        .context("Failed to get XMR/USD ticker")?;

    // Get BTC/XMR ticker
    let btc_xmr_ticker = kraken
        .get_ticker("XBTXMR")
        .await
        .context("Failed to get BTC/XMR ticker")?;

    let btc_usd: f64 = btc_usd_ticker.last_trade[0]
        .parse()
        .context("Failed to parse BTC/USD price")?;

    let xmr_usd: f64 = xmr_usd_ticker.last_trade[0]
        .parse()
        .context("Failed to parse XMR/USD price")?;

    let btc_xmr: f64 = btc_xmr_ticker.last_trade[0]
        .parse()
        .context("Failed to parse BTC/XMR price")?;

    let response = KrakenTickerResponse {
        btc_usd,
        xmr_usd,
        btc_xmr,
    };

    Ok(Json(response))
}

/// Create the Kraken routes router
pub fn kraken_routes() -> Router<AppState> {
    Router::new()
        .route("/tickers", get(get_tickers))
}
