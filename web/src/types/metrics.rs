use serde::{Deserialize, Serialize};

/// A single metric data point with timestamp and value
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct MetricValue {
    pub timestamp: String,
    pub value: f64,
}

/// Bitcoin metrics from the backend
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct BitcoinMetrics {
    pub timestamp: String,
    pub blocks: u64,
    pub headers: u64,
    pub verification_progress: f64,
    pub size_on_disk: u64,
    pub wallet_balance: Option<f64>,
}

/// Monero metrics from the backend
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct MoneroMetrics {
    pub timestamp: String,
    pub height: u64,
    pub target_height: u64,
    pub difficulty: u64,
    pub tx_count: u64,
    pub wallet_balance: Option<f64>,
}

/// ASB (Atomic Swap Bot) metrics from the backend
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct AsbMetrics {
    pub timestamp: String,
    pub balance_btc: f64,
    pub pending_swaps: u64,
    pub completed_swaps: u64,
    pub failed_swaps: u64,
    pub up: bool,
}

/// Wallet balances response
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct WalletBalances {
    pub bitcoin: f64,
    pub monero: f64,
}

/// Wallet health status response
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct WalletHealth {
    pub healthy: bool,
    pub bitcoin_ready: bool,
    pub monero_ready: bool,
}

/// Trading engine status
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct TradingStatus {
    pub enabled: bool,
    pub last_check: Option<String>,
    pub last_trade: Option<String>,
}

/// Trading configuration
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct TradingConfig {
    pub target_btc_percentage: f64,
    pub rebalance_threshold_percentage: f64,
    pub max_trade_size_btc: f64,
    pub min_trade_size_btc: f64,
    pub check_interval_seconds: u64,
}

/// Kraken ticker prices response
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct KrakenTickers {
    pub btc_usd: f64,
    pub xmr_usd: f64,
    pub btc_xmr: f64,
}

