use crate::api::ApiClient;
use crate::types::metrics::KrakenTickers;

/// Fetch current Kraken ticker prices
pub async fn fetch_kraken_tickers() -> Result<KrakenTickers, String> {
    ApiClient::get("/kraken/tickers").await
}
