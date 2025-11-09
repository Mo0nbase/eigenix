use axum::{
    extract::State,
    routing::get,
    Json, Router,
};
use anyhow::Context;
use serde::Serialize;

use crate::{services::KrakenClient, ApiError, ApiResult, AppState};

/// Kraken ticker price response
#[derive(Serialize, serde::Deserialize)]
pub struct KrakenTickerResponse {
    /// BTC/USD price
    pub btc_usd: f64,
    /// BTC/USD 24h change percentage
    pub btc_usd_change_24h: f64,
    /// XMR/USD price
    pub xmr_usd: f64,
    /// XMR/USD 24h change percentage
    pub xmr_usd_change_24h: f64,
    /// XMR/BTC price (BTC per XMR)
    pub xmr_btc: f64,
    /// XMR/BTC 24h change percentage
    pub xmr_btc_change_24h: f64,
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

    // Parse current prices
    let btc_usd: f64 = btc_usd_ticker.last_trade[0]
        .parse()
        .context("Failed to parse BTC/USD price")?;

    let xmr_usd: f64 = xmr_usd_ticker.last_trade[0]
        .parse()
        .context("Failed to parse XMR/USD price")?;

    let xmr_btc: f64 = xmr_btc_ticker.last_trade[0]
        .parse()
        .context("Failed to parse XMR/BTC price")?;

    // Parse opening prices for 24h change calculation
    let btc_usd_open: f64 = btc_usd_ticker.open
        .parse()
        .context("Failed to parse BTC/USD opening price")?;

    let xmr_usd_open: f64 = xmr_usd_ticker.open
        .parse()
        .context("Failed to parse XMR/USD opening price")?;

    let xmr_btc_open: f64 = xmr_btc_ticker.open
        .parse()
        .context("Failed to parse XMR/BTC opening price")?;

    // Calculate 24h change percentages
    let btc_usd_change_24h = if btc_usd_open != 0.0 {
        ((btc_usd - btc_usd_open) / btc_usd_open) * 100.0
    } else {
        0.0
    };

    let xmr_usd_change_24h = if xmr_usd_open != 0.0 {
        ((xmr_usd - xmr_usd_open) / xmr_usd_open) * 100.0
    } else {
        0.0
    };

    let xmr_btc_change_24h = if xmr_btc_open != 0.0 {
        ((xmr_btc - xmr_btc_open) / xmr_btc_open) * 100.0
    } else {
        0.0
    };

    tracing::info!(
        "BTC/USD: ${:.2} ({:+.2}%), XMR/USD: ${:.2} ({:+.2}%), XMR/BTC: {:.8} ({:+.2}%)", 
        btc_usd, btc_usd_change_24h,
        xmr_usd, xmr_usd_change_24h,
        xmr_btc, xmr_btc_change_24h
    );

    let response = KrakenTickerResponse {
        btc_usd,
        btc_usd_change_24h,
        xmr_usd,
        xmr_usd_change_24h,
        xmr_btc,
        xmr_btc_change_24h,
    };

    Ok(Json(response))
}

/// Create the Kraken routes router
pub fn kraken_routes() -> Router<AppState> {
    Router::new()
        .route("/tickers", get(get_tickers))
}
