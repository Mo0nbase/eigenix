/// Common test utilities and configuration
///
/// This module provides helper functions for integration tests,
/// particularly for running tests both locally and within the container network.
use std::env;

/// Test configuration for service endpoints
pub struct TestConfig {
    pub bitcoin_rpc_url: String,
    pub bitcoin_cookie_path: String,
    pub monero_wallet_rpc_url: String,
    pub asb_rpc_url: String,
    pub wallet_name: String,
}

impl Default for TestConfig {
    fn default() -> Self {
        Self {
            bitcoin_rpc_url: "http://127.0.0.1:8332".to_string(),
            bitcoin_cookie_path: "/mnt/vault/bitcoind-data/.cookie".to_string(),
            monero_wallet_rpc_url: "http://127.0.0.1:18082/json_rpc".to_string(),
            asb_rpc_url: "http://127.0.0.1:9944".to_string(),
            wallet_name: "eigenix".to_string(),
        }
    }
}

impl TestConfig {
    /// Load test configuration from environment variables
    ///
    /// Environment variables:
    /// - BITCOIN_RPC_URL: Bitcoin RPC endpoint (default: http://127.0.0.1:8332)
    /// - BITCOIN_COOKIE_PATH: Path to Bitcoin cookie file
    /// - MONERO_WALLET_RPC_URL: Monero wallet RPC endpoint (default: http://127.0.0.1:18082/json_rpc)
    /// - ASB_RPC_URL: ASB RPC endpoint (default: http://127.0.0.1:9944)
    /// - WALLET_NAME: Wallet name to use (default: eigenix)
    /// - IN_CONTAINER: Set to "true" when running in container (uses container hostnames)
    pub fn from_env() -> Self {
        let in_container = env::var("IN_CONTAINER").unwrap_or_default() == "true";

        if in_container {
            // Container network configuration - use container hostnames from eigenix-network
            Self {
                bitcoin_rpc_url: env::var("BITCOIN_RPC_URL")
                    .unwrap_or_else(|_| "http://bitcoind:8332".to_string()),
                bitcoin_cookie_path: env::var("BITCOIN_COOKIE_PATH")
                    .unwrap_or_else(|_| "/bitcoind-data/.cookie".to_string()),
                monero_wallet_rpc_url: env::var("MONERO_WALLET_RPC_URL")
                    .unwrap_or_else(|_| "http://monero-wallet-rpc:18082/json_rpc".to_string()),
                asb_rpc_url: env::var("ASB_RPC_URL")
                    .unwrap_or_else(|_| "http://asb:9944".to_string()),
                wallet_name: env::var("WALLET_NAME").unwrap_or_else(|_| "eigenix".to_string()),
            }
        } else {
            // Local configuration - use localhost (matches default NixOS ports)
            Self {
                bitcoin_rpc_url: env::var("BITCOIN_RPC_URL")
                    .unwrap_or_else(|_| "http://127.0.0.1:8332".to_string()),
                bitcoin_cookie_path: env::var("BITCOIN_COOKIE_PATH")
                    .unwrap_or_else(|_| "/mnt/vault/bitcoind-data/.cookie".to_string()),
                monero_wallet_rpc_url: env::var("MONERO_WALLET_RPC_URL")
                    .unwrap_or_else(|_| "http://127.0.0.1:18082/json_rpc".to_string()),
                asb_rpc_url: env::var("ASB_RPC_URL")
                    .unwrap_or_else(|_| "http://127.0.0.1:9944".to_string()),
                wallet_name: env::var("WALLET_NAME").unwrap_or_else(|_| "eigenix".to_string()),
            }
        }
    }

    /// Get a test configuration for Bitcoin wallet tests
    pub fn bitcoin_wallet() -> (String, String, String) {
        let config = Self::from_env();
        (
            config.bitcoin_rpc_url,
            config.bitcoin_cookie_path,
            config.wallet_name,
        )
    }

    /// Get a test configuration for Monero wallet tests
    pub fn monero_wallet() -> (String, String, String) {
        let config = Self::from_env();
        (
            config.monero_wallet_rpc_url,
            config.wallet_name,
            String::new(), // Empty password
        )
    }

    /// Get ASB RPC URL for tests
    pub fn asb_rpc_url() -> String {
        let config = Self::from_env();
        config.asb_rpc_url
    }
}
