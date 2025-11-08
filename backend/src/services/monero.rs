use anyhow::{Context, Result};
use serde::Deserialize;

use crate::metrics::MoneroMetrics;

/// Monero node RPC client for blockchain information
pub struct MoneroRpcClient {
    url: String,
}

#[derive(Deserialize)]
struct MoneroRpcResponse<T> {
    result: Option<T>,
}

#[derive(Deserialize)]
struct MoneroInfo {
    height: u64,
    target_height: u64,
    difficulty: u64,
    tx_count: u64,
}

impl MoneroRpcClient {
    pub fn new(url: String) -> Self {
        Self { url }
    }

    pub async fn get_metrics(&self) -> Result<MoneroMetrics> {
        let client = reqwest::Client::new();

        let body = serde_json::json!({
            "jsonrpc": "2.0",
            "id": "0",
            "method": "get_info"
        });

        let response = client
            .post(&self.url)
            .header("Content-Type", "application/json")
            .json(&body)
            .send()
            .await
            .context("Failed to send Monero RPC request")?;

        let rpc_response: MoneroRpcResponse<MoneroInfo> = response
            .json()
            .await
            .context("Failed to parse Monero RPC response")?;

        let info = rpc_response
            .result
            .context("Monero RPC response missing result")?;

        // Try to get wallet balance (may fail if wallet RPC not available)
        let wallet_balance = self.get_wallet_balance().await.ok();

        Ok(MoneroMetrics {
            height: info.height,
            target_height: info.target_height,
            difficulty: info.difficulty,
            tx_count: info.tx_count,
            wallet_balance,
        })
    }

    /// Get wallet balance in XMR (requires monero-wallet-rpc)
    async fn get_wallet_balance(&self) -> Result<f64> {
        #[derive(Deserialize)]
        struct BalanceResult {
            balance: u64, // Balance in atomic units
        }

        let client = reqwest::Client::new();
        let body = serde_json::json!({
            "jsonrpc": "2.0",
            "id": "0",
            "method": "get_balance"
        });

        let response = client
            .post(&self.url)
            .header("Content-Type", "application/json")
            .json(&body)
            .send()
            .await
            .context("Failed to send Monero wallet RPC request")?;

        let rpc_response: MoneroRpcResponse<BalanceResult> = response
            .json()
            .await
            .context("Failed to parse Monero wallet RPC response")?;

        let balance_result = rpc_response
            .result
            .context("Monero wallet RPC response missing result")?;

        // Convert atomic units to XMR (1 XMR = 10^12 atomic units)
        Ok(balance_result.balance as f64 / 1_000_000_000_000.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    #[ignore] // Only run with actual Monero node
    async fn test_get_monero_metrics() {
        let client = MoneroRpcClient::new("http://127.0.0.1:18081/json_rpc".to_string());
        let metrics = client.get_metrics().await.unwrap();
        assert!(metrics.height > 0);
    }
}
