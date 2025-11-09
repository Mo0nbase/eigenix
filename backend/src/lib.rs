//! Eigenix Backend Library
//!
//! This library provides cryptocurrency wallet management, exchange integration,
//! and metrics collection for Bitcoin, Monero, and atomic swap operations.

pub mod config;
pub mod db;
pub mod metrics;
pub mod services;
pub mod wallets;

// Re-export commonly used types
pub use config::Config;
pub use db::MetricsDatabase;
pub use services::{AsbClient, BitcoinRpcClient, KrakenClient, MoneroRpcClient};
pub use wallets::{BitcoinWallet, MoneroWallet, WalletConfig, WalletManager};
