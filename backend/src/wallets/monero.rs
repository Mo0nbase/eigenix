use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

/// Monero wallet client for sending/receiving XMR
///
/// This wallet connects to monero-wallet-rpc and manages a wallet created from a seed phrase.
/// It requires a seed phrase and restore height to be provided during initialization.
pub struct MoneroWallet {
    url: String,
    wallet_name: String,
}

#[derive(Deserialize)]
struct RpcResponse<T> {
    result: Option<T>,
    error: Option<RpcError>,
}

#[derive(Deserialize)]
struct RpcError {
    code: i32,
    message: String,
}

/// Monero wallet balance information
#[derive(Debug, Serialize, Deserialize)]
pub struct WalletBalance {
    pub balance: f64,          // Total balance in XMR
    pub unlocked_balance: f64, // Available balance in XMR
}

/// Information about a Monero transfer
#[derive(Debug, Serialize, Deserialize)]
pub struct Transfer {
    pub txid: String,
    pub amount: f64, // Amount in XMR
    pub fee: f64,    // Fee in XMR
    pub height: u64, // Block height
    pub timestamp: u64,
    pub confirmations: u64,
    pub unlock_time: u64,
}

/// Monero subaddress
#[derive(Debug, Serialize, Deserialize)]
pub struct Subaddress {
    pub address: String,
    pub address_index: u32,
    pub label: Option<String>,
    pub used: bool,
}

const ATOMIC_UNITS_PER_XMR: u64 = 1_000_000_000_000;

impl MoneroWallet {
    /// Create and initialize a Monero wallet from seed phrase
    ///
    /// This will:
    /// 1. Connect to monero-wallet-rpc
    /// 2. Restore wallet from seed phrase
    /// 3. Open the wallet for use
    ///
    /// # Arguments
    /// * `url` - monero-wallet-rpc URL (e.g., "http://127.0.0.1:18082/json_rpc")
    /// * `seed` - Monero seed phrase (25 words from ASB)
    /// * `restore_height` - Block height to start scanning from (from ASB)
    /// * `wallet_name` - Name for the wallet file (e.g., "eigenix")
    /// * `password` - Optional password for the wallet (empty string if none)
    pub async fn new_from_seed(
        url: String,
        seed: &str,
        restore_height: u64,
        wallet_name: &str,
        password: &str,
    ) -> Result<Self> {
        let wallet = Self {
            url,
            wallet_name: wallet_name.to_string(),
        };

        // Initialize the wallet from seed
        wallet
            .initialize_wallet(seed, restore_height, password)
            .await?;

        Ok(wallet)
    }

    /// Connect to an existing Monero wallet
    ///
    /// Use this when the wallet has already been created and you just want to open it.
    ///
    /// # Arguments
    /// * `url` - monero-wallet-rpc URL
    /// * `wallet_name` - Name of existing wallet
    /// * `password` - Wallet password (empty string if none)
    pub async fn connect_existing(url: String, wallet_name: &str, password: &str) -> Result<Self> {
        let wallet = Self {
            url,
            wallet_name: wallet_name.to_string(),
        };

        // Try to open the wallet
        wallet.open_wallet(password).await?;

        // Verify wallet is accessible
        wallet
            .get_balance()
            .await
            .context("Failed to connect to existing wallet")?;

        Ok(wallet)
    }

    /// Initialize wallet from seed phrase
    async fn initialize_wallet(
        &self,
        seed: &str,
        restore_height: u64,
        password: &str,
    ) -> Result<()> {
        // Try to restore wallet from seed
        match self
            .restore_wallet_from_seed(seed, restore_height, password)
            .await
        {
            Ok(_) => {
                tracing::info!("Restored Monero wallet from seed: {}", self.wallet_name);
            }
            Err(e) => {
                // Check if wallet already exists
                if e.to_string().contains("already exists")
                    || e.to_string().contains("Cannot create")
                {
                    tracing::info!("Monero wallet already exists: {}", self.wallet_name);
                    // Try to open it
                    self.open_wallet(password).await?;
                } else {
                    return Err(e);
                }
            }
        }

        // Refresh wallet to sync with blockchain
        self.refresh().await?;

        Ok(())
    }

    /// Restore wallet from seed phrase
    async fn restore_wallet_from_seed(
        &self,
        seed: &str,
        restore_height: u64,
        password: &str,
    ) -> Result<()> {
        let params = serde_json::json!({
            "filename": self.wallet_name,
            "password": password,
            "seed": seed,
            "restore_height": restore_height,
            "language": "English",
            "autosave_current": true,
        });

        let _: serde_json::Value = self.call("restore_deterministic_wallet", params).await?;
        Ok(())
    }

    /// Open an existing wallet
    async fn open_wallet(&self, password: &str) -> Result<()> {
        // Close any currently opened wallet first
        let _ = self.close_wallet().await;

        let params = serde_json::json!({
            "filename": self.wallet_name,
            "password": password,
        });

        let _: serde_json::Value = self.call("open_wallet", params).await?;
        tracing::info!("Opened Monero wallet: {}", self.wallet_name);
        Ok(())
    }

    /// Close the currently opened wallet
    async fn close_wallet(&self) -> Result<()> {
        let _: serde_json::Value = self.call("close_wallet", serde_json::json!({})).await?;
        Ok(())
    }

    /// Call a Monero wallet RPC method
    async fn call<T: for<'de> Deserialize<'de>>(
        &self,
        method: &str,
        params: serde_json::Value,
    ) -> Result<T> {
        let client = reqwest::Client::new();

        let body = serde_json::json!({
            "jsonrpc": "2.0",
            "id": "0",
            "method": method,
            "params": params
        });

        let response = client
            .post(&self.url)
            .header("Content-Type", "application/json")
            .json(&body)
            .send()
            .await
            .context("Failed to send Monero wallet RPC request")?;

        let rpc_response: RpcResponse<T> = response
            .json()
            .await
            .context("Failed to parse Monero wallet RPC response")?;

        if let Some(error) = rpc_response.error {
            anyhow::bail!("Monero wallet RPC error {}: {}", error.code, error.message);
        }

        rpc_response
            .result
            .context("Monero wallet RPC response missing result")
    }

    /// Convert atomic units to XMR
    fn atomic_to_xmr(atomic: u64) -> f64 {
        atomic as f64 / ATOMIC_UNITS_PER_XMR as f64
    }

    /// Convert XMR to atomic units
    fn xmr_to_atomic(xmr: f64) -> u64 {
        (xmr * ATOMIC_UNITS_PER_XMR as f64) as u64
    }

    /// Get wallet balance
    pub async fn get_balance(&self) -> Result<WalletBalance> {
        #[derive(Deserialize)]
        struct BalanceResult {
            balance: u64,          // in atomic units
            unlocked_balance: u64, // in atomic units
        }

        let result: BalanceResult = self.call("get_balance", serde_json::json!({})).await?;

        Ok(WalletBalance {
            balance: Self::atomic_to_xmr(result.balance),
            unlocked_balance: Self::atomic_to_xmr(result.unlocked_balance),
        })
    }

    /// Get the primary address of the wallet
    pub async fn get_address(&self) -> Result<String> {
        #[derive(Deserialize)]
        struct AddressResult {
            address: String,
        }

        let result: AddressResult = self
            .call("get_address", serde_json::json!({"account_index": 0}))
            .await?;

        Ok(result.address)
    }

    /// Create a new subaddress
    ///
    /// # Arguments
    /// * `account_index` - Account index (usually 0 for main account)
    /// * `label` - Optional label for the subaddress
    pub async fn create_subaddress(
        &self,
        account_index: u32,
        label: Option<&str>,
    ) -> Result<Subaddress> {
        #[derive(Deserialize)]
        struct SubaddressResult {
            address: String,
            address_index: u32,
        }

        let mut params = serde_json::json!({"account_index": account_index});
        if let Some(l) = label {
            params["label"] = serde_json::json!(l);
        }

        let result: SubaddressResult = self.call("create_address", params).await?;

        Ok(Subaddress {
            address: result.address,
            address_index: result.address_index,
            label: label.map(|s| s.to_string()),
            used: false,
        })
    }

    /// Validate a Monero address
    pub async fn validate_address(&self, address: &str) -> Result<bool> {
        #[derive(Deserialize)]
        struct ValidateResult {
            valid: bool,
        }

        let result: ValidateResult = self
            .call("validate_address", serde_json::json!({"address": address}))
            .await?;

        Ok(result.valid)
    }

    /// Transfer XMR to an address
    ///
    /// # Arguments
    /// * `address` - Destination Monero address
    /// * `amount` - Amount in XMR to send
    /// * `priority` - Transaction priority (0=default, 1=unimportant, 2=normal, 3=elevated, 4=priority)
    ///
    /// # Returns
    /// Transaction hash (txid) and fee in XMR
    pub async fn transfer(
        &self,
        address: &str,
        amount: f64,
        priority: u32,
    ) -> Result<(String, f64)> {
        // Validate address first
        if !self.validate_address(address).await? {
            anyhow::bail!("Invalid Monero address: {}", address);
        }

        #[derive(Deserialize)]
        struct TransferResult {
            tx_hash: String,
            fee: u64, // in atomic units
        }

        let amount_atomic = Self::xmr_to_atomic(amount);

        let params = serde_json::json!({
            "destinations": [{
                "amount": amount_atomic,
                "address": address
            }],
            "priority": priority,
            "get_tx_key": true
        });

        let result: TransferResult = self.call("transfer", params).await?;

        Ok((result.tx_hash, Self::atomic_to_xmr(result.fee)))
    }

    /// Transfer all unlocked balance to an address
    ///
    /// # Arguments
    /// * `address` - Destination Monero address
    /// * `priority` - Transaction priority
    ///
    /// # Returns
    /// Transaction hash (txid) and fee in XMR
    pub async fn sweep_all(&self, address: &str, priority: u32) -> Result<(String, f64)> {
        // Validate address first
        if !self.validate_address(address).await? {
            anyhow::bail!("Invalid Monero address: {}", address);
        }

        #[derive(Deserialize)]
        struct SweepResult {
            tx_hash_list: Vec<String>,
            fee_list: Vec<u64>, // in atomic units
        }

        let params = serde_json::json!({
            "address": address,
            "priority": priority,
            "get_tx_keys": true
        });

        let result: SweepResult = self.call("sweep_all", params).await?;

        // Return first transaction (usually there's only one)
        let txid = result
            .tx_hash_list
            .first()
            .context("No transaction in sweep_all result")?
            .clone();
        let fee = result
            .fee_list
            .first()
            .copied()
            .context("No fee in sweep_all result")?;

        Ok((txid, Self::atomic_to_xmr(fee)))
    }

    /// Get transfer details by transaction ID
    ///
    /// # Arguments
    /// * `txid` - Transaction ID to query
    pub async fn get_transfer_by_txid(&self, txid: &str) -> Result<Transfer> {
        #[derive(Deserialize)]
        struct TransferData {
            amount: u64, // in atomic units
            fee: u64,    // in atomic units
            height: u64,
            timestamp: u64,
            confirmations: u64,
            unlock_time: u64,
            txid: String,
        }

        #[derive(Deserialize)]
        struct GetTransferResult {
            transfer: TransferData,
        }

        let result: GetTransferResult = self
            .call("get_transfer_by_txid", serde_json::json!({"txid": txid}))
            .await?;

        Ok(Transfer {
            txid: result.transfer.txid,
            amount: Self::atomic_to_xmr(result.transfer.amount),
            fee: Self::atomic_to_xmr(result.transfer.fee),
            height: result.transfer.height,
            timestamp: result.transfer.timestamp,
            confirmations: result.transfer.confirmations,
            unlock_time: result.transfer.unlock_time,
        })
    }

    /// Get incoming transfers
    ///
    /// # Arguments
    /// * `min_height` - Optional minimum block height to query from
    pub async fn get_incoming_transfers(&self, min_height: Option<u64>) -> Result<Vec<Transfer>> {
        #[derive(Deserialize)]
        struct TransferData {
            amount: u64,
            tx_hash: String,
            block_height: u64,
            unlock_time: u64,
        }

        #[derive(Deserialize)]
        struct IncomingResult {
            transfers: Option<Vec<TransferData>>,
        }

        let mut params = serde_json::json!({
            "transfer_type": "available"
        });

        if let Some(height) = min_height {
            params["min_height"] = serde_json::json!(height);
        }

        let result: IncomingResult = self.call("get_transfers", params).await?;

        let transfers = result.transfers.unwrap_or_default();

        Ok(transfers
            .into_iter()
            .map(|t| Transfer {
                txid: t.tx_hash,
                amount: Self::atomic_to_xmr(t.amount),
                fee: 0.0, // Incoming transfers don't have fees
                height: t.block_height,
                timestamp: 0,     // Not available in this call
                confirmations: 0, // Would need current height to calculate
                unlock_time: t.unlock_time,
            })
            .collect())
    }

    /// Refresh the wallet to check for new transactions
    ///
    /// This syncs the wallet with the Monero blockchain
    pub async fn refresh(&self) -> Result<u64> {
        #[derive(Deserialize)]
        struct RefreshResult {
            blocks_fetched: u64,
        }

        let result: RefreshResult = self.call("refresh", serde_json::json!({})).await?;

        if result.blocks_fetched > 0 {
            tracing::info!(
                "Refreshed Monero wallet, fetched {} blocks",
                result.blocks_fetched
            );
}

        Ok(result.blocks_fetched)
    }

    /// Get wallet height (current block height the wallet is synced to)
    pub async fn get_height(&self) -> Result<u64> {
        #[derive(Deserialize)]
        struct HeightResult {
            height: u64,
        }

        let result: HeightResult = self.call("get_height", serde_json::json!({})).await?;
        Ok(result.height)
    }

    /// Check if wallet is ready and operational
    pub async fn is_ready(&self) -> bool {
        self.get_balance().await.is_ok()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_atomic_conversion() {
        assert_eq!(MoneroWallet::atomic_to_xmr(1_000_000_000_000), 1.0);
        assert_eq!(MoneroWallet::xmr_to_atomic(1.0), 1_000_000_000_000);
        assert_eq!(MoneroWallet::atomic_to_xmr(500_000_000_000), 0.5);
        assert_eq!(MoneroWallet::xmr_to_atomic(0.5), 500_000_000_000);
    }

    #[tokio::test]
    #[ignore] // Only run with valid Monero wallet RPC
    async fn test_connect_existing() {
        let wallet = MoneroWallet::connect_existing(
            "http://127.0.0.1:18082/json_rpc".to_string(),
            "eigenix",
            "",
        )
        .await
        .unwrap();

        let balance = wallet.get_balance().await;
        assert!(balance.is_ok());
    }

    #[tokio::test]
    #[ignore] // Only run with valid seed and wallet RPC
    async fn test_new_from_seed() {
        let seed = "your 25 word seed phrase here..."; // Replace with actual seed
        let restore_height = 0;

        let wallet = MoneroWallet::new_from_seed(
            "http://127.0.0.1:18082/json_rpc".to_string(),
            seed,
            restore_height,
            "eigenix_test",
            "",
        )
        .await
        .unwrap();

        assert!(wallet.is_ready().await);
    }
}
