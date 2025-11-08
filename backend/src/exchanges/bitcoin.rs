use anyhow::{Context, Result};
use base64::{engine::general_purpose, Engine as _};
use serde::{Deserialize, Serialize};
use std::fs;

/// Bitcoin wallet client for sending/receiving BTC
pub struct BitcoinWallet {
    url: String,
    auth: String,
}

#[derive(Deserialize)]
struct RpcResponse<T> {
    result: Option<T>,
    error: Option<RpcError>,
}

#[derive(Deserialize)]
struct RpcError {
    message: String,
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
    /// Create a new Bitcoin wallet client using cookie authentication
    /// First tries BITCOIN_RPC_COOKIE env var, then tries sudo, then direct read
    pub fn new(url: String, cookie_path: &str) -> Result<Self> {
        let cookie = if let Ok(cookie_env) = std::env::var("BITCOIN_RPC_COOKIE") {
            cookie_env
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
                .context("Failed to read Bitcoin RPC cookie file")?
        };

        // Cookie format is "username:password"
        let auth = format!("Basic {}", general_purpose::STANDARD.encode(cookie.trim()));

        Ok(Self { url, auth })
    }

    /// Call a Bitcoin RPC method with parameters
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

        let result: BalancesResult = self.call("getbalances", serde_json::json!([])).await?;

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

        let address: String = self.call("getnewaddress", params).await?;
        Ok(address)
    }

    /// Validate a Bitcoin address
    pub async fn validate_address(&self, address: &str) -> Result<bool> {
        let result: ValidateAddressResult = self
            .call("validateaddress", serde_json::json!([address]))
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

        let txid: String = self.call("sendtoaddress", params).await?;
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
            .call("gettransaction", serde_json::json!([txid]))
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
            .call("listtransactions", serde_json::json!(["*", count]))
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
        // Create a test transaction without broadcasting
        #[derive(Deserialize)]
        struct FundRawResult {
            fee: f64,
        }

        // Create raw transaction
        let raw_tx: String = self
            .call(
                "createrawtransaction",
                serde_json::json!([[], {address: amount}]),
            )
            .await?;

        // Fund it to get fee estimate
        let funded: FundRawResult = self
            .call("fundrawtransaction", serde_json::json!([raw_tx]))
            .await?;

        Ok(funded.fee)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    #[ignore] // Only run with valid Bitcoin node
    async fn test_get_balance() {
        let wallet = BitcoinWallet::new(
            "http://127.0.0.1:8332".to_string(),
            "/mnt/vault/bitcoind-data/.cookie",
        )
        .unwrap();
        let balance = wallet.get_balance().await;
        assert!(balance.is_ok());
    }

    #[tokio::test]
    #[ignore] // Only run with valid Bitcoin node
    async fn test_get_new_address() {
        let wallet = BitcoinWallet::new(
            "http://127.0.0.1:8332".to_string(),
            "/mnt/vault/bitcoind-data/.cookie",
        )
        .unwrap();
        let address = wallet.get_new_address(Some("test")).await;
        assert!(address.is_ok());
    }
}
