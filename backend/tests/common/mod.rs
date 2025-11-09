/// Common test utilities and configuration
///
/// This module provides helper functions for integration tests,
/// particularly for running tests both locally and within the container network.
use std::env;
use std::sync::Once;

static INIT: Once = Once::new();

/// Initialize test environment (load .env file if present)
fn init_test_env() {
    INIT.call_once(|| {
        // Try to load .env file from backend directory
        // Silently ignore if file doesn't exist
        let _ = dotenvy::dotenv();
    });
}

/// Test configuration for service endpoints
pub struct TestConfig {
    pub bitcoin_rpc_url: String,
    pub bitcoin_cookie_path: String,
    pub monero_wallet_rpc_url: String,
    pub asb_rpc_url: String,
    pub wallet_name: String,
    pub kraken_api_key: String,
    pub kraken_api_secret: String,
}

impl Default for TestConfig {
    fn default() -> Self {
        Self {
            bitcoin_rpc_url: "http://127.0.0.1:8332".to_string(),
            bitcoin_cookie_path: "/mnt/vault/bitcoind-data/.cookie".to_string(),
            monero_wallet_rpc_url: "http://127.0.0.1:18082/json_rpc".to_string(),
            asb_rpc_url: "http://127.0.0.1:9944".to_string(),
            wallet_name: "eigenix".to_string(),
            kraken_api_key: "".to_string(),
            kraken_api_secret: "".to_string(),
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
    /// - KRAKEN_API_KEY: Kraken API key
    /// - KRAKEN_API_SECRET: Kraken API secret
    /// - IN_CONTAINER: Set to "true" when running in container (uses container hostnames)
    ///
    /// Note: Automatically loads `.env` file from backend directory if present.
    pub fn from_env() -> Self {
        // Initialize environment (load .env file)
        init_test_env();
        
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
                kraken_api_key: env::var("KRAKEN_API_KEY").unwrap_or_default(),
                kraken_api_secret: env::var("KRAKEN_API_SECRET").unwrap_or_default(),
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
                kraken_api_key: env::var("KRAKEN_API_KEY").unwrap_or_default(),
                kraken_api_secret: env::var("KRAKEN_API_SECRET").unwrap_or_default(),
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

    /// Get Kraken API credentials
    pub fn kraken() -> (String, String) {
        let config = Self::from_env();
        (config.kraken_api_key, config.kraken_api_secret)
    }
}
