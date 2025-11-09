use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

/// ASB (Automated Swap Backend) JSON-RPC client
///
/// Provides wrappers around the ASB's JSON-RPC API for managing
/// Bitcoin/Monero atomic swaps.
pub struct AsbClient {
    url: String,
    client: reqwest::Client,
}

#[derive(Deserialize)]
struct RpcResponse<T> {
    result: Option<T>,
    error: Option<RpcError>,
}

#[derive(Debug, Deserialize)]
struct RpcError {
    code: i32,
    message: String,
}

/// Bitcoin balance information from ASB
#[derive(Debug, Serialize, Deserialize)]
pub struct BitcoinBalance {
    pub balance: f64, // Balance in BTC
}

/// Bitcoin wallet descriptor (contains private keys!)
#[derive(Debug, Serialize, Deserialize)]
pub struct BitcoinSeed {
    pub descriptor: String, // Wallet descriptor
}

/// Monero balance information from ASB
#[derive(Debug, Serialize, Deserialize)]
pub struct MoneroBalance {
    pub balance: f64, // Balance in XMR
}

/// Monero wallet address
#[derive(Debug, Serialize, Deserialize)]
pub struct MoneroAddress {
    pub address: String,
}

/// Monero seed phrase and restore height (contains private keys!)
#[derive(Debug, Serialize, Deserialize)]
pub struct MoneroSeed {
    pub seed: String,
    pub restore_height: u64,
}

/// ASB multiaddresses including Tor onion address
#[derive(Debug, Serialize, Deserialize)]
pub struct Multiaddresses {
    pub addresses: Vec<String>,
}

/// Active P2P connection count
#[derive(Debug, Serialize, Deserialize)]
pub struct ActiveConnections {
    pub count: u32,
}

/// Swap information
#[derive(Debug, Serialize, Deserialize)]
pub struct SwapInfo {
    pub swap_id: String,
    pub status: String,
    // Add more fields as needed based on actual ASB response
}

/// List of swaps
#[derive(Debug, Serialize, Deserialize)]
pub struct Swaps {
    pub swaps: Vec<SwapInfo>,
}

impl AsbClient {
    /// Create a new ASB JSON-RPC client
    ///
    /// # Arguments
    /// * `url` - URL of the ASB JSON-RPC endpoint (e.g., "http://127.0.0.1:9944")
    pub fn new(url: String) -> Self {
        Self {
            url,
            client: reqwest::Client::new(),
        }
    }

    /// Call an ASB JSON-RPC method
    async fn call<T: for<'de> Deserialize<'de>>(
        &self,
        method: &str,
        params: serde_json::Value,
    ) -> Result<T> {
        let body = serde_json::json!({
            "jsonrpc": "2.0",
            "method": method,
            "params": params,
            "id": 1
        });

        let response = self
            .client
            .post(&self.url)
            .header("Content-Type", "application/json")
            .json(&body)
            .send()
            .await
            .context("Failed to send ASB JSON-RPC request")?;

        let rpc_response: RpcResponse<T> = response
            .json()
            .await
            .context("Failed to parse ASB JSON-RPC response")?;

        if let Some(error) = rpc_response.error {
            anyhow::bail!("ASB JSON-RPC error {}: {}", error.code, error.message);
        }

        rpc_response
            .result
            .context("ASB JSON-RPC response missing result")
    }

    /// Check connection to ASB server
    ///
    /// # Returns
    /// `Ok(())` if connection is successful
    pub async fn check_connection(&self) -> Result<()> {
        // Try to call a simple method to verify connection
        // Use get_swaps as a health check since it should always return something
        let _: serde_json::Value = self.call("get_swaps", serde_json::json!({})).await?;
        Ok(())
    }

    /// Get Bitcoin wallet balance
    ///
    /// # Returns
    /// Bitcoin balance in BTC
    pub async fn get_bitcoin_balance(&self) -> Result<f64> {
        let result: serde_json::Value = self.call("bitcoin_balance", serde_json::json!({})).await?;

        // The result might be just a number or wrapped in an object
        // Adjust based on actual ASB response format
        if let Some(balance) = result.as_f64() {
            Ok(balance)
        } else if let Some(balance) = result.get("balance").and_then(|v| v.as_f64()) {
            Ok(balance)
        } else {
            anyhow::bail!("Unexpected bitcoin_balance response format: {:?}", result)
        }
    }

    /// Get Bitcoin wallet descriptor
    ///
    /// **WARNING: Contains private keys! Handle with extreme care.**
    ///
    /// # Returns
    /// Bitcoin wallet descriptor string
    pub async fn get_bitcoin_seed(&self) -> Result<String> {
        let result: serde_json::Value = self.call("bitcoin_seed", serde_json::json!({})).await?;

        // The result might be just a string or wrapped in an object
        if let Some(descriptor) = result.as_str() {
            Ok(descriptor.to_string())
        } else if let Some(descriptor) = result.get("descriptor").and_then(|v| v.as_str()) {
            Ok(descriptor.to_string())
        } else {
            anyhow::bail!("Unexpected bitcoin_seed response format: {:?}", result)
        }
    }

    /// Get Monero wallet balance
    ///
    /// # Returns
    /// Monero balance in XMR
    pub async fn get_monero_balance(&self) -> Result<f64> {
        let result: serde_json::Value = self.call("monero_balance", serde_json::json!({})).await?;

        // The result might be just a number or wrapped in an object
        if let Some(balance) = result.as_f64() {
            Ok(balance)
        } else if let Some(balance) = result.get("balance").and_then(|v| v.as_f64()) {
            Ok(balance)
        } else {
            anyhow::bail!("Unexpected monero_balance response format: {:?}", result)
        }
    }

    /// Get Monero wallet deposit address
    ///
    /// # Returns
    /// Monero address string
    pub async fn get_monero_address(&self) -> Result<String> {
        let result: serde_json::Value = self.call("monero_address", serde_json::json!({})).await?;

        // The result might be just a string or wrapped in an object
        if let Some(address) = result.as_str() {
            Ok(address.to_string())
        } else if let Some(address) = result.get("address").and_then(|v| v.as_str()) {
            Ok(address.to_string())
        } else {
            anyhow::bail!("Unexpected monero_address response format: {:?}", result)
        }
    }

    /// Get Monero seed phrase and restore height
    ///
    /// **WARNING: Contains private keys! Handle with extreme care.**
    ///
    /// # Returns
    /// Tuple of (seed phrase, restore height)
    pub async fn get_monero_seed(&self) -> Result<(String, u64)> {
        let result: serde_json::Value = self.call("monero_seed", serde_json::json!({})).await?;

        let seed = result
            .get("seed")
            .and_then(|v| v.as_str())
            .context("Missing seed in monero_seed response")?
            .to_string();

        let restore_height = result
            .get("restore_height")
            .and_then(|v| v.as_u64())
            .context("Missing restore_height in monero_seed response")?;

        Ok((seed, restore_height))
    }

    /// Get external multiaddresses (including Tor onion address)
    ///
    /// # Returns
    /// Vector of multiaddress strings
    pub async fn get_multiaddresses(&self) -> Result<Vec<String>> {
        let result: serde_json::Value = self.call("multiaddresses", serde_json::json!({})).await?;

        // The result might be an array or wrapped in an object with key "multiaddresses"
        if let Some(addresses) = result.as_array() {
            Ok(addresses
                .iter()
                .filter_map(|v| v.as_str().map(|s| s.to_string()))
                .collect())
        } else if let Some(addresses) = result.get("multiaddresses").and_then(|v| v.as_array()) {
            Ok(addresses
                .iter()
                .filter_map(|v| v.as_str().map(|s| s.to_string()))
                .collect())
        } else if let Some(addresses) = result.get("addresses").and_then(|v| v.as_array()) {
            Ok(addresses
                .iter()
                .filter_map(|v| v.as_str().map(|s| s.to_string()))
                .collect())
        } else {
            anyhow::bail!("Unexpected multiaddresses response format: {:?}", result)
        }
    }

    /// Get active P2P connection count
    ///
    /// # Returns
    /// Number of active connections
    pub async fn get_active_connections(&self) -> Result<u32> {
        let result: serde_json::Value = self
            .call("active_connections", serde_json::json!({}))
            .await?;

        // The result might be just a number or wrapped in an object with key "connections" or "count"
        if let Some(count) = result.as_u64() {
            Ok(count as u32)
        } else if let Some(count) = result.get("connections").and_then(|v| v.as_u64()) {
            Ok(count as u32)
        } else if let Some(count) = result.get("count").and_then(|v| v.as_u64()) {
            Ok(count as u32)
        } else {
            anyhow::bail!(
                "Unexpected active_connections response format: {:?}",
                result
            )
        }
    }

    /// Get list of swaps
    ///
    /// # Returns
    /// Vector of swap information
    pub async fn get_swaps(&self) -> Result<Vec<SwapInfo>> {
        let result: serde_json::Value = self.call("get_swaps", serde_json::json!({})).await?;

        // The result might be an array or wrapped in an object
        if let Some(swaps) = result.as_array() {
            // Parse each swap - this is a simplified version
            // You may need to adjust based on actual swap structure
            let swap_infos: Vec<SwapInfo> = swaps
                .iter()
                .filter_map(|v| {
                    let swap_id = v.get("swap_id")?.as_str()?.to_string();
                    let status = v.get("status")?.as_str()?.to_string();
                    Some(SwapInfo { swap_id, status })
                })
                .collect();
            Ok(swap_infos)
        } else if let Some(swaps) = result.get("swaps").and_then(|v| v.as_array()) {
            let swap_infos: Vec<SwapInfo> = swaps
                .iter()
                .filter_map(|v| {
                    let swap_id = v.get("swap_id")?.as_str()?.to_string();
                    let status = v.get("status")?.as_str()?.to_string();
                    Some(SwapInfo { swap_id, status })
                })
                .collect();
            Ok(swap_infos)
        } else {
            // If the result is empty or in a different format, return empty vec
            Ok(Vec::new())
        }
    }

    /// Check if ASB is healthy and reachable
    ///
    /// This is a convenience method that tries to check connection
    /// and returns a boolean instead of an error.
    pub async fn is_healthy(&self) -> bool {
        self.check_connection().await.is_ok()
    }

    /// Get comprehensive ASB status
    ///
    /// Returns a summary of ASB health, balances, and connection info
    pub async fn get_status(&self) -> Result<AsbStatus> {
        let is_up = self.is_healthy().await;

        if !is_up {
            return Ok(AsbStatus {
                up: false,
                bitcoin_balance: 0.0,
                monero_balance: 0.0,
                active_connections: 0,
                multiaddresses: Vec::new(),
            });
        }

        let bitcoin_balance = self.get_bitcoin_balance().await.unwrap_or(0.0);
        let monero_balance = self.get_monero_balance().await.unwrap_or(0.0);
        let active_connections = self.get_active_connections().await.unwrap_or(0);
        let multiaddresses = self.get_multiaddresses().await.unwrap_or_default();

        Ok(AsbStatus {
            up: true,
            bitcoin_balance,
            monero_balance,
            active_connections,
            multiaddresses,
        })
    }
}

/// Comprehensive ASB status
#[derive(Debug, Serialize, Deserialize)]
pub struct AsbStatus {
    pub up: bool,
    pub bitcoin_balance: f64,
    pub monero_balance: f64,
    pub active_connections: u32,
    pub multiaddresses: Vec<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    #[ignore] // Only run with actual ASB instance
    async fn test_check_connection() {
        let client = AsbClient::new("http://127.0.0.1:9944".to_string());
        let result = client.check_connection().await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    #[ignore] // Only run with actual ASB instance
    async fn test_get_bitcoin_balance() {
        let client = AsbClient::new("http://127.0.0.1:9944".to_string());
        let balance = client.get_bitcoin_balance().await;
        assert!(balance.is_ok());
    }

    #[tokio::test]
    #[ignore] // Only run with actual ASB instance
    async fn test_get_monero_balance() {
        let client = AsbClient::new("http://127.0.0.1:9944".to_string());
        let balance = client.get_monero_balance().await;
        assert!(balance.is_ok());
    }

    #[tokio::test]
    #[ignore] // Only run with actual ASB instance
    async fn test_get_status() {
        let client = AsbClient::new("http://127.0.0.1:9944".to_string());
        let status = client.get_status().await;
        assert!(status.is_ok());
    }
}
