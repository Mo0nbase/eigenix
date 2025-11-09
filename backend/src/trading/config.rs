use serde::{Deserialize, Serialize};
use std::sync::{Arc, RwLock};

/// Trading configuration with runtime-updatable parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TradingConfig {
    /// Minimum Monero balance threshold (in XMR) before triggering rebalance
    pub monero_min_threshold: f64,

    /// Target Monero balance to maintain after rebalancing (in XMR)
    pub monero_target_balance: f64,

    /// Minimum Bitcoin balance to keep (don't trade below this, in BTC)
    pub bitcoin_reserve_minimum: f64,

    /// Maximum amount of Bitcoin to use in a single rebalance operation (in BTC)
    pub max_btc_per_rebalance: f64,

    /// Check interval in seconds (how often to check balances)
    pub check_interval_secs: u64,

    /// Maximum time to wait for Kraken order execution in seconds
    pub order_timeout_secs: u64,

    /// Slippage tolerance percentage (e.g., 0.5 for 0.5%)
    pub slippage_tolerance_percent: f64,

    /// Whether to use limit orders (true) or market orders (false)
    pub use_limit_orders: bool,
}

impl Default for TradingConfig {
    fn default() -> Self {
        Self {
            monero_min_threshold: 1.0,        // Rebalance if XMR drops below 1.0
            monero_target_balance: 5.0,       // Target 5.0 XMR after rebalancing
            bitcoin_reserve_minimum: 0.00001, // Keep at least 0.00001 BTC
            max_btc_per_rebalance: 0.01,      // Max 0.1 BTC per operation
            check_interval_secs: 300,         // Check every 5 minutes
            order_timeout_secs: 600,          // Wait max 10 minutes for order
            slippage_tolerance_percent: 1.0,  // 1% slippage tolerance
            use_limit_orders: true,           // Use limit orders by default
        }
    }
}

impl TradingConfig {
    /// Validate configuration parameters
    pub fn validate(&self) -> Result<(), String> {
        if self.monero_min_threshold >= self.monero_target_balance {
            return Err("monero_min_threshold must be less than monero_target_balance".to_string());
        }

        if self.monero_min_threshold < 0.0 {
            return Err("monero_min_threshold must be positive".to_string());
        }

        if self.bitcoin_reserve_minimum < 0.0 {
            return Err("bitcoin_reserve_minimum must be positive".to_string());
        }

        if self.max_btc_per_rebalance <= 0.0 {
            return Err("max_btc_per_rebalance must be positive".to_string());
        }

        if self.check_interval_secs == 0 {
            return Err("check_interval_secs must be greater than 0".to_string());
        }

        if self.slippage_tolerance_percent < 0.0 || self.slippage_tolerance_percent > 100.0 {
            return Err("slippage_tolerance_percent must be between 0 and 100".to_string());
        }

        Ok(())
    }
}

/// Thread-safe wrapper for trading configuration
#[derive(Debug, Clone)]
pub struct SharedTradingConfig {
    config: Arc<RwLock<TradingConfig>>,
}

impl SharedTradingConfig {
    pub fn new(config: TradingConfig) -> Self {
        Self {
            config: Arc::new(RwLock::new(config)),
        }
    }

    /// Get a copy of the current configuration
    pub fn get(&self) -> TradingConfig {
        self.config.read().unwrap().clone()
    }

    /// Update the configuration
    pub fn update(&self, new_config: TradingConfig) -> Result<(), String> {
        new_config.validate()?;
        *self.config.write().unwrap() = new_config;
        Ok(())
    }
}

impl Default for SharedTradingConfig {
    fn default() -> Self {
        Self::new(TradingConfig::default())
    }
}
