use anyhow::{Context, Result};
use base64::{engine::general_purpose, Engine as _};
use serde::{Deserialize, Serialize};
use std::fs;

/// Bitcoin wallet client for sending/receiving BTC
///
/// This wallet connects to a Bitcoin Core node and manages a descriptor-based wallet.
/// It requires a descriptor (containing private keys) to be provided during initialization.
pub struct BitcoinWallet {
    url: String,
    auth: String,
    wallet_name: String,
}

#[derive(Deserialize)]
struct RpcResponse<T> {
    result: Option<T>,
    error: Option<RpcError>,
}

#[derive(Deserialize)]
struct RpcError {
    message: String,
    code: Option<i32>,
}

/// Bitcoin wallet balance information
#[derive(Debug, Serialize, Deserialize)]
pub struct WalletBalance {
    pub balance: f64,             // Confirmed balance
    pub unconfirmed_balance: f64, // Unconfirmed balance
    pub immature_balance: f64,    // Immature balance (e.g., from mining)
}

/// Information about a Bitcoin transaction
#[derive(Debug, Serialize, Deserialize)]
pub struct Transaction {
    pub txid: String,
    pub amount: f64,
    pub confirmations: u64,
    pub blockhash: Option<String>,
    pub blockindex: Option<u64>,
    pub blocktime: Option<u64>,
    pub time: u64,
}

/// Address validation result
#[derive(Debug, Deserialize)]
struct ValidateAddressResult {
    isvalid: bool,
}

impl BitcoinWallet {
    /// Create and initialize a Bitcoin wallet from a descriptor
    ///
    /// This will:
    /// 1. Connect to Bitcoin Core RPC
    /// 2. Create a new descriptor wallet (or load if exists)
    /// 3. Import the provided descriptor
    ///
    /// # Arguments
    /// * `url` - Bitcoin Core RPC URL (e.g., "http://127.0.0.1:8332")
    /// * `cookie_path` - Path to Bitcoin Core .cookie file for authentication
    /// * `descriptor` - Wallet descriptor string (from ASB) containing private keys
    /// * `wallet_name` - Name for the wallet in Bitcoin Core (e.g., "eigenix")
    /// * `rescan` - Whether to rescan blockchain for existing transactions
    pub async fn new_from_descriptor(
        url: String,
        cookie_path: &str,
        descriptor: &str,
        wallet_name: &str,
        rescan: bool,
    ) -> Result<Self> {
        let cookie = Self::read_cookie(cookie_path)?;
        let auth = format!("Basic {}", general_purpose::STANDARD.encode(cookie.trim()));

        let wallet = Self {
            url,
            auth,
            wallet_name: wallet_name.to_string(),
        };

        // Initialize the wallet in Bitcoin Core
        wallet.initialize_wallet(descriptor, rescan).await?;

        Ok(wallet)
    }

    /// Connect to an existing Bitcoin wallet
    ///
    /// Use this when the wallet has already been initialized and you just want to reconnect.
    ///
    /// # Arguments
    /// * `url` - Bitcoin Core RPC URL
    /// * `cookie_path` - Path to .cookie file
    /// * `wallet_name` - Name of existing wallet
    pub async fn connect_existing(
        url: String,
        cookie_path: &str,
        wallet_name: &str,
    ) -> Result<Self> {
        let cookie = Self::read_cookie(cookie_path)?;
        let auth = format!("Basic {}", general_purpose::STANDARD.encode(cookie.trim()));

        let wallet = Self {
            url,
            auth,
            wallet_name: wallet_name.to_string(),
        };

        // Try to load the wallet if it exists
        let _ = wallet.load_wallet().await;

        // Verify wallet is accessible
        wallet
            .get_balance()
            .await
            .context("Failed to connect to existing wallet")?;

        Ok(wallet)
    }

    /// Read Bitcoin Core cookie file for authentication
    fn read_cookie(cookie_path: &str) -> Result<String> {
        if let Ok(cookie_env) = std::env::var("BITCOIN_RPC_COOKIE") {
            Ok(cookie_env)
        } else {
            // Try reading with sudo if direct read fails
            std::process::Command::new("sudo")
                .arg("cat")
                .arg(cookie_path)
                .output()
                .ok()
                .and_then(|output| {
                    if output.status.success() {
                        String::from_utf8(output.stdout).ok()
                    } else {
                        None
                    }
                })
                .or_else(|| fs::read_to_string(cookie_path).ok())
                .context("Failed to read Bitcoin RPC cookie file")
        }
    }

    /// Initialize wallet in Bitcoin Core with descriptor
    async fn initialize_wallet(&self, descriptor: &str, rescan: bool) -> Result<()> {
        // Try to create wallet (ignore error if already exists)
        let wallet_existed = match self.create_wallet().await {
            Ok(_) => {
                tracing::info!("Created new Bitcoin wallet: {}", self.wallet_name);
                false
            }
            Err(e) => {
                // Check if error is "wallet already exists"
                if e.to_string().contains("already exists") {
                    tracing::info!("Bitcoin wallet already exists: {}", self.wallet_name);
                    // Try to load the existing wallet (ignore if already loaded)
                    match self.load_wallet().await {
                        Ok(_) => tracing::info!("Loaded existing wallet: {}", self.wallet_name),
                        Err(load_err) => {
                            if load_err.to_string().contains("already loaded") {
                                tracing::info!("Bitcoin wallet already loaded: {}", self.wallet_name);
                            } else {
                                return Err(load_err.context("Failed to load existing wallet"));
                            }
                        }
                    }
                    true
                } else {
                    return Err(e);
                }
            }
        };

        // Only import descriptor if this is a new wallet
        if !wallet_existed {
            self.import_descriptors(descriptor, rescan).await?;
        } else {
            tracing::info!("Skipping descriptor import for existing wallet");
        }

        Ok(())
    }

    /// Create a descriptor wallet in Bitcoin Core
    async fn create_wallet(&self) -> Result<()> {
        #[derive(Deserialize)]
        struct CreateWalletResult {
            name: String,
        }

        let params = serde_json::json!([
            self.wallet_name,
            false, // disable_private_keys (false - we need private keys)
            false, // blank (false - we'll import descriptors)
            "",    // passphrase (empty for now)
            false, // avoid_reuse
            true,  // descriptors (use descriptor wallet - required for modern Bitcoin Core)
            true,  // load_on_startup
        ]);

        let _result: CreateWalletResult = self.call("createwallet", params).await?;
        Ok(())
    }

    /// Load wallet in Bitcoin Core
    async fn load_wallet(&self) -> Result<()> {
        #[derive(Deserialize)]
        struct LoadWalletResult {
            name: String,
        }

        let params = serde_json::json!([self.wallet_name]);
        let _result: LoadWalletResult = self.call("loadwallet", params).await?;
        Ok(())
    }

    /// Add checksum to a descriptor using Bitcoin Core's getdescriptorinfo
    async fn add_checksum_to_descriptor(&self, descriptor: &str) -> Result<String> {
        #[derive(Deserialize)]
        struct DescriptorInfo {
            descriptor: String,
        }

        let params = serde_json::json!([descriptor]);
        let info: DescriptorInfo = self.call("getdescriptorinfo", params).await?;
        Ok(info.descriptor)
    }

    /// Import descriptors into Bitcoin Core wallet
    async fn import_descriptors(&self, descriptor: &str, rescan: bool) -> Result<()> {
        #[derive(Deserialize)]
        struct ImportResult {
            success: bool,
            #[serde(default)]
            warnings: Vec<String>,
            #[serde(default)]
            error: Option<serde_json::Value>,
        }

        // Add checksum to descriptor if missing
        let descriptor_with_checksum = if !descriptor.contains('#') {
            self.add_checksum_to_descriptor(descriptor).await?
        } else {
            descriptor.to_string()
        };

        let params = if rescan {
            serde_json::json!([[
                {
                    "desc": descriptor_with_checksum,
                    "timestamp": 0,
                    "active": true,
                }
            ]])
        } else {
            serde_json::json!([[
                {
                    "desc": descriptor_with_checksum,
                    "timestamp": "now",
                    "active": true,
                }
            ]])
        };

        let results: Vec<ImportResult> = self.call_wallet("importdescriptors", params).await?;

        for result in &results {
            if !result.success {
                // Check if the error is because descriptor is already imported
                if let Some(error) = &result.error {
                    let error_str = error.to_string();
                    if error_str.contains("already") || error_str.contains("exists") {
                        tracing::info!("Descriptor already imported, skipping");
                        continue;
                    }
                }
                anyhow::bail!("Failed to import descriptor: {:?}", result.error);
            }
            for warning in &result.warnings {
                tracing::warn!("Descriptor import warning: {}", warning);
            }
        }

        tracing::info!("Successfully imported descriptor into Bitcoin wallet");
        Ok(())
    }

    /// Call a Bitcoin RPC method (no wallet context)
    async fn call<T: for<'de> Deserialize<'de>>(
        &self,
        method: &str,
        params: serde_json::Value,
    ) -> Result<T> {
        let client = reqwest::Client::new();

        let body = serde_json::json!({
            "jsonrpc": "1.0",
            "id": "eigenix",
            "method": method,
            "params": params
        });

        let response = client
            .post(&self.url)
            .header("Authorization", &self.auth)
            .header("Content-Type", "text/plain")
            .json(&body)
            .send()
            .await
            .context("Failed to send RPC request")?;

        let rpc_response: RpcResponse<T> = response
            .json()
            .await
            .context("Failed to parse RPC response")?;

        if let Some(error) = rpc_response.error {
            anyhow::bail!("Bitcoin RPC error: {}", error.message);
        }

        rpc_response
            .result
            .context("RPC response missing result field")
    }

    /// Call a Bitcoin wallet RPC method (with wallet context)
    async fn call_wallet<T: for<'de> Deserialize<'de>>(
        &self,
        method: &str,
        params: serde_json::Value,
    ) -> Result<T> {
        let client = reqwest::Client::new();

        // Use wallet-specific endpoint
        let wallet_url = format!("{}/wallet/{}", self.url, self.wallet_name);

        let body = serde_json::json!({
            "jsonrpc": "1.0",
            "id": "eigenix",
            "method": method,
            "params": params
        });

        let response = client
            .post(&wallet_url)
            .header("Authorization", &self.auth)
            .header("Content-Type", "text/plain")
            .json(&body)
            .send()
            .await
            .context("Failed to send wallet RPC request")?;

        let rpc_response: RpcResponse<T> = response
            .json()
            .await
            .context("Failed to parse wallet RPC response")?;

        if let Some(error) = rpc_response.error {
            anyhow::bail!("Bitcoin wallet RPC error: {}", error.message);
        }

        rpc_response
            .result
            .context("Wallet RPC response missing result field")
    }

    /// Get wallet balance
    pub async fn get_balance(&self) -> Result<WalletBalance> {
        #[derive(Deserialize)]
        struct BalancesResult {
            mine: MineBalance,
        }

        #[derive(Deserialize)]
        struct MineBalance {
            trusted: f64,
            untrusted_pending: f64,
            immature: f64,
        }

        let result: BalancesResult = self
            .call_wallet("getbalances", serde_json::json!([]))
            .await?;

        Ok(WalletBalance {
            balance: result.mine.trusted,
            unconfirmed_balance: result.mine.untrusted_pending,
            immature_balance: result.mine.immature,
        })
    }

    /// Get a new receiving address from the wallet
    ///
    /// # Arguments
    /// * `label` - Optional label for the address
    pub async fn get_new_address(&self, label: Option<&str>) -> Result<String> {
        let params = if let Some(l) = label {
            serde_json::json!([l])
        } else {
            serde_json::json!([])
        };

        let address: String = self.call_wallet("getnewaddress", params).await?;
        Ok(address)
    }

    /// Validate a Bitcoin address
    pub async fn validate_address(&self, address: &str) -> Result<bool> {
        let result: ValidateAddressResult = self
            .call_wallet("validateaddress", serde_json::json!([address]))
            .await?;
        Ok(result.isvalid)
    }

    /// Send Bitcoin to an address
    ///
    /// # Arguments
    /// * `address` - Destination Bitcoin address
    /// * `amount` - Amount in BTC to send
    /// * `subtract_fee` - If true, subtract fee from amount (default: false)
    ///
    /// # Returns
    /// Transaction ID (txid) of the sent transaction
    pub async fn send_to_address(
        &self,
        address: &str,
        amount: f64,
        subtract_fee: bool,
    ) -> Result<String> {
        // Validate address first
        if !self.validate_address(address).await? {
            anyhow::bail!("Invalid Bitcoin address: {}", address);
        }

        let params = serde_json::json!([
            address,
            amount,
            "", // comment
            "", // comment_to
            subtract_fee
        ]);

        let txid: String = self.call_wallet("sendtoaddress", params).await?;
        Ok(txid)
    }

    /// Get transaction details
    ///
    /// # Arguments
    /// * `txid` - Transaction ID to query
    pub async fn get_transaction(&self, txid: &str) -> Result<Transaction> {
        #[derive(Deserialize)]
        struct TxResult {
            amount: f64,
            confirmations: u64,
            blockhash: Option<String>,
            blockindex: Option<u64>,
            blocktime: Option<u64>,
            txid: String,
            time: u64,
        }

        let result: TxResult = self
            .call_wallet("gettransaction", serde_json::json!([txid]))
            .await?;

        Ok(Transaction {
            txid: result.txid,
            amount: result.amount,
            confirmations: result.confirmations,
            blockhash: result.blockhash,
            blockindex: result.blockindex,
            blocktime: result.blocktime,
            time: result.time,
        })
    }

    /// List recent transactions
    ///
    /// # Arguments
    /// * `count` - Number of transactions to return (default: 10)
    pub async fn list_transactions(&self, count: u32) -> Result<Vec<Transaction>> {
        #[derive(Deserialize)]
        struct TxListItem {
            amount: f64,
            confirmations: u64,
            blockhash: Option<String>,
            blockindex: Option<u64>,
            blocktime: Option<u64>,
            txid: String,
            time: u64,
        }

        let result: Vec<TxListItem> = self
            .call_wallet("listtransactions", serde_json::json!(["*", count]))
            .await?;

        Ok(result
            .into_iter()
            .map(|tx| Transaction {
                txid: tx.txid,
                amount: tx.amount,
                confirmations: tx.confirmations,
                blockhash: tx.blockhash,
                blockindex: tx.blockindex,
                blocktime: tx.blocktime,
                time: tx.time,
            })
            .collect())
    }

    /// Estimate transaction fee for sending to an address
    ///
    /// # Arguments
    /// * `address` - Destination address
    /// * `amount` - Amount to send in BTC
    ///
    /// # Returns
    /// Estimated fee in BTC
    pub async fn estimate_fee(&self, address: &str, amount: f64) -> Result<f64> {
        #[derive(Deserialize)]
        struct FundRawResult {
            fee: f64,
        }

        // Create raw transaction
        let raw_tx: String = self
            .call_wallet(
                "createrawtransaction",
                serde_json::json!([[], {address: amount}]),
            )
            .await?;

        // Fund it to get fee estimate
        let funded: FundRawResult = self
            .call_wallet("fundrawtransaction", serde_json::json!([raw_tx]))
            .await?;

        Ok(funded.fee)
    }

    /// Check if wallet is loaded and operational
    pub async fn is_ready(&self) -> bool {
        self.get_balance().await.is_ok()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    #[ignore] // Only run with valid Bitcoin node
    async fn test_connect_existing() {
        let wallet = BitcoinWallet::connect_existing(
            "http://127.0.0.1:8332".to_string(),
            "/mnt/vault/bitcoind-data/.cookie",
            "eigenix",
        )
        .await
        .unwrap();

        let balance = wallet.get_balance().await;
        assert!(balance.is_ok());
    }

    #[tokio::test]
    #[ignore] // Only run with valid Bitcoin node and descriptor
    async fn test_new_from_descriptor() {
        let descriptor = "wpkh([fingerprint/84h/0h/0h]xpub...)"; // Replace with actual descriptor

        let wallet = BitcoinWallet::new_from_descriptor(
            "http://127.0.0.1:8332".to_string(),
            "/mnt/vault/bitcoind-data/.cookie",
            descriptor,
            "eigenix_test",
            false,
        )
        .await
        .unwrap();

        assert!(wallet.is_ready().await);
    }
}
