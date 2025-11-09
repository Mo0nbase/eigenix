use super::{BitcoinWallet, MoneroWallet};
use crate::services::AsbClient;
use anyhow::{Context, Result};

/// Wallet manager for initializing and managing Bitcoin and Monero wallets
///
/// This manager orchestrates the wallet initialization process by:
/// 1. Retrieving seeds/descriptors from the ASB service
/// 2. Initializing Bitcoin and Monero wallets
/// 3. Providing access to both wallets
pub struct WalletManager {
    pub bitcoin: BitcoinWallet,
    pub monero: MoneroWallet,
}

/// Configuration for wallet initialization
pub struct WalletConfig {
    // Bitcoin configuration
    pub bitcoin_rpc_url: String,
    pub bitcoin_cookie_path: String,
    pub bitcoin_wallet_name: String,
    pub bitcoin_rescan: bool,

    // Monero configuration
    pub monero_rpc_url: String,
    pub monero_wallet_name: String,
    pub monero_wallet_password: String,

    // ASB configuration
    pub asb_rpc_url: String,
}

impl WalletManager {
    /// Initialize wallets from ASB service
    ///
    /// This method will:
    /// 1. Connect to ASB and retrieve Bitcoin descriptor and Monero seed
    /// 2. Initialize Bitcoin wallet from descriptor
    /// 3. Initialize Monero wallet from seed phrase
    ///
    /// # Arguments
    /// * `config` - Wallet configuration
    ///
    /// # Returns
    /// Initialized WalletManager with both wallets ready
    pub async fn initialize_from_asb(config: WalletConfig) -> Result<Self> {
        tracing::info!("Initializing wallet manager from ASB...");

        // Connect to ASB
        let asb_client = AsbClient::new(config.asb_rpc_url.clone());

        // Check ASB connection
        asb_client
            .check_connection()
            .await
            .context("Failed to connect to ASB service")?;

        tracing::info!("Connected to ASB service");

        // Retrieve Bitcoin descriptor from ASB
        tracing::info!("Retrieving Bitcoin descriptor from ASB...");
        let bitcoin_descriptor = asb_client
            .get_bitcoin_seed()
            .await
            .context("Failed to retrieve Bitcoin descriptor from ASB")?;

        tracing::info!("Retrieved Bitcoin descriptor from ASB");

        // Retrieve Monero seed from ASB
        tracing::info!("Retrieving Monero seed from ASB...");
        let (monero_seed, restore_height) = asb_client
            .get_monero_seed()
            .await
            .context("Failed to retrieve Monero seed from ASB")?;

        tracing::info!(
            "Retrieved Monero seed from ASB (restore height: {})",
            restore_height
        );

        // Initialize Bitcoin wallet
        tracing::info!("Initializing Bitcoin wallet...");
        let bitcoin = BitcoinWallet::new_from_descriptor(
            config.bitcoin_rpc_url,
            &config.bitcoin_cookie_path,
            &bitcoin_descriptor,
            &config.bitcoin_wallet_name,
            config.bitcoin_rescan,
        )
        .await
        .context("Failed to initialize Bitcoin wallet")?;

        tracing::info!("Bitcoin wallet initialized successfully");

        // Initialize Monero wallet
        tracing::info!("Initializing Monero wallet...");
        let monero = MoneroWallet::new_from_seed(
            config.monero_rpc_url,
            &monero_seed,
            restore_height,
            &config.monero_wallet_name,
            &config.monero_wallet_password,
        )
        .await
        .context("Failed to initialize Monero wallet")?;

        tracing::info!("Monero wallet initialized successfully");

        // Verify wallets are ready
        if !bitcoin.is_ready().await {
            anyhow::bail!("Bitcoin wallet is not ready after initialization");
        }

        if !monero.is_ready().await {
            anyhow::bail!("Monero wallet is not ready after initialization");
        }

        tracing::info!("All wallets initialized and ready");

        Ok(Self { bitcoin, monero })
    }

    /// Connect to existing wallets without re-initializing from ASB
    ///
    /// Use this when wallets have already been initialized and you just want to reconnect.
    ///
    /// # Arguments
    /// * `config` - Wallet configuration
    ///
    /// # Returns
    /// WalletManager connected to existing wallets
    pub async fn connect_existing(config: WalletConfig) -> Result<Self> {
        tracing::info!("Connecting to existing wallets...");

        // Connect to existing Bitcoin wallet
        let bitcoin = BitcoinWallet::connect_existing(
            config.bitcoin_rpc_url,
            &config.bitcoin_cookie_path,
            &config.bitcoin_wallet_name,
        )
        .await
        .context("Failed to connect to existing Bitcoin wallet")?;

        tracing::info!("Connected to existing Bitcoin wallet");

        // Connect to existing Monero wallet
        let monero = MoneroWallet::connect_existing(
            config.monero_rpc_url,
            &config.monero_wallet_name,
            &config.monero_wallet_password,
        )
        .await
        .context("Failed to connect to existing Monero wallet")?;

        tracing::info!("Connected to existing Monero wallet");

        Ok(Self { bitcoin, monero })
    }

    /// Initialize or connect to wallets (smart initialization)
    ///
    /// This method will:
    /// 1. Try to connect to existing wallets
    /// 2. If that fails, initialize from ASB
    ///
    /// # Arguments
    /// * `config` - Wallet configuration
    ///
    /// # Returns
    /// WalletManager with wallets ready
    pub async fn initialize_or_connect(config: WalletConfig) -> Result<Self> {
        tracing::info!("Attempting to connect to existing wallets...");

        // Try to connect to existing wallets first
        match Self::connect_existing(config.clone()).await {
            Ok(manager) => {
                tracing::info!("Successfully connected to existing wallets");
                Ok(manager)
            }
            Err(e) => {
                tracing::warn!(
                    "Failed to connect to existing wallets: {}. Initializing from ASB...",
                    e
                );
                Self::initialize_from_asb(config).await
            }
        }
    }

    /// Get Bitcoin balance
    pub async fn get_bitcoin_balance(&self) -> Result<f64> {
        let balance = self.bitcoin.get_balance().await?;
        Ok(balance.balance)
    }

    /// Get Monero balance
    pub async fn get_monero_balance(&self) -> Result<f64> {
        let balance = self.monero.get_balance().await?;
        Ok(balance.unlocked_balance)
    }

    /// Get balances for both Bitcoin and Monero
    pub async fn get_balances(&self) -> Result<(f64, f64)> {
        let btc_balance = self.get_bitcoin_balance().await?;
        let xmr_balance = self.get_monero_balance().await?;
        Ok((btc_balance, xmr_balance))
    }

    /// Check if both wallets are healthy and operational
    pub async fn is_healthy(&self) -> bool {
        self.bitcoin.is_ready().await && self.monero.is_ready().await
    }

    /// Refresh Monero wallet to sync with blockchain
    pub async fn refresh_monero(&self) -> Result<u64> {
        self.monero.refresh().await
    }
}

// Make WalletConfig cloneable for the initialize_or_connect pattern
impl Clone for WalletConfig {
    fn clone(&self) -> Self {
        Self {
            bitcoin_rpc_url: self.bitcoin_rpc_url.clone(),
            bitcoin_cookie_path: self.bitcoin_cookie_path.clone(),
            bitcoin_wallet_name: self.bitcoin_wallet_name.clone(),
            bitcoin_rescan: self.bitcoin_rescan,
            monero_rpc_url: self.monero_rpc_url.clone(),
            monero_wallet_name: self.monero_wallet_name.clone(),
            monero_wallet_password: self.monero_wallet_password.clone(),
            asb_rpc_url: self.asb_rpc_url.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    #[ignore] // Only run with valid ASB and node infrastructure
    async fn test_initialize_from_asb() {
        let config = WalletConfig {
            bitcoin_rpc_url: "http://127.0.0.1:8332".to_string(),
            bitcoin_cookie_path: "/mnt/vault/bitcoind-data/.cookie".to_string(),
            bitcoin_wallet_name: "eigenix_test".to_string(),
            bitcoin_rescan: false,
            monero_rpc_url: "http://127.0.0.1:18082/json_rpc".to_string(),
            monero_wallet_name: "eigenix_test".to_string(),
            monero_wallet_password: "".to_string(),
            asb_rpc_url: "http://127.0.0.1:9944".to_string(),
        };

        let manager = WalletManager::initialize_from_asb(config).await.unwrap();
        assert!(manager.is_healthy().await);
    }

    #[tokio::test]
    #[ignore] // Only run with existing wallets
    async fn test_connect_existing() {
        let config = WalletConfig {
            bitcoin_rpc_url: "http://127.0.0.1:8332".to_string(),
            bitcoin_cookie_path: "/mnt/vault/bitcoind-data/.cookie".to_string(),
            bitcoin_wallet_name: "eigenix".to_string(),
            bitcoin_rescan: false,
            monero_rpc_url: "http://127.0.0.1:18082/json_rpc".to_string(),
            monero_wallet_name: "eigenix".to_string(),
            monero_wallet_password: "".to_string(),
            asb_rpc_url: "http://127.0.0.1:9944".to_string(),
        };

        let manager = WalletManager::connect_existing(config).await.unwrap();
        assert!(manager.is_healthy().await);
    }
}
