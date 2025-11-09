//! Eigenix Backend Library
//!
//! This library provides cryptocurrency wallet management, exchange integration,
//! and metrics collection for Bitcoin, Monero, and atomic swap operations.

use std::sync::Arc;

pub mod config;
pub mod db;
pub mod metrics;
pub mod routes;
pub mod services;
pub mod wallets;

// Re-export commonly used types
pub use config::Config;
pub use db::MetricsDatabase;
pub use services::{AsbClient, BitcoinRpcClient, KrakenClient, MoneroRpcClient};
pub use wallets::{BitcoinWallet, MoneroWallet, WalletConfig, WalletManager};

/// Application state shared across all route handlers
#[derive(Clone)]
pub struct AppState {
    pub config: Arc<Config>,
    pub db: MetricsDatabase,
    pub wallets: Arc<WalletManager>,
}
