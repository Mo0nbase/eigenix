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
    /// XMR/BTC price (BTC per XMR)
    pub xmr_btc: f64,
}

/// Get current Kraken ticker prices
pub async fn get_tickers(State(state): State<AppState>) -> ApiResult<Json<KrakenTickerResponse>> {
    // Create Kraken client - public endpoints don't need credentials but we provide them anyway
    let kraken = KrakenClient::new(
        state.config.kraken.api_key.clone(),
        state.config.kraken.api_secret.clone(),
    );

    tracing::info!("Fetching Kraken tickers...");

    // Get BTC/USD ticker - use XBTUSD (standard Kraken symbol)
    let btc_usd_ticker = kraken
        .get_ticker("XBTUSD")
        .await
        .context("Failed to get BTC/USD ticker")?;

    // Get XMR/USD ticker - use XMRUSD (standard Kraken symbol)
    let xmr_usd_ticker = kraken
        .get_ticker("XMRUSD")
        .await
        .context("Failed to get XMR/USD ticker")?;

    // Get XMR/BTC ticker (inverted from XBTXMR)
    let xmr_btc_ticker = kraken
        .get_ticker("XMRXBT")
        .await
        .context("Failed to get XMR/BTC ticker")?;

    tracing::info!("Successfully fetched all ticker data");

    let btc_usd: f64 = btc_usd_ticker.last_trade[0]
        .parse()
        .context("Failed to parse BTC/USD price")?;

    let xmr_usd: f64 = xmr_usd_ticker.last_trade[0]
        .parse()
        .context("Failed to parse XMR/USD price")?;

    let xmr_btc: f64 = xmr_btc_ticker.last_trade[0]
        .parse()
        .context("Failed to parse XMR/BTC price")?;

    tracing::info!("BTC/USD: ${:.2}, XMR/USD: ${:.2}, XMR/BTC: {:.8}", btc_usd, xmr_usd, xmr_btc);

    let response = KrakenTickerResponse {
        btc_usd,
        xmr_usd,
        xmr_btc,
    };

    Ok(Json(response))
}

/// Create the Kraken routes router
pub fn kraken_routes() -> Router<AppState> {
    Router::new()
        .route("/tickers", get(get_tickers))
}
