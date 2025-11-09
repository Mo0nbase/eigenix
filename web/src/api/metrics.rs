use crate::api::ApiClient;
use crate::types::metrics::{AsbMetrics, BitcoinMetrics, MoneroMetrics};

/// Fetch Bitcoin metrics for the given time interval (in minutes)
pub async fn fetch_bitcoin_interval(minutes: i64) -> Result<Vec<BitcoinMetrics>, String> {
    ApiClient::get(&format!("/metrics/bitcoin/interval?minutes={}", minutes)).await
}

/// Fetch Monero metrics for the given time interval (in minutes)
pub async fn fetch_monero_interval(minutes: i64) -> Result<Vec<MoneroMetrics>, String> {
    ApiClient::get(&format!("/metrics/monero/interval?minutes={}", minutes)).await
}

/// Fetch ASB metrics for the given time interval (in minutes)
pub async fn fetch_asb_interval(minutes: i64) -> Result<Vec<AsbMetrics>, String> {
    ApiClient::get(&format!("/metrics/asb/interval?minutes={}", minutes)).await
}

