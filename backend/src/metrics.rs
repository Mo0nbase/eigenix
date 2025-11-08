use anyhow::{Context, Result};
use base64::{engine::general_purpose, Engine as _};
use serde::{Deserialize, Serialize};
use std::fs;

/// Bitcoin blockchain information from getblockchaininfo RPC
#[derive(Debug, Serialize, Deserialize)]
pub struct BitcoinMetrics {
    pub blocks: u64,
    pub headers: u64,
    pub verification_progress: f64,
    pub size_on_disk: u64,
}

/// Bitcoin RPC client
pub struct BitcoinRpcClient {
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

#[derive(Deserialize)]
struct BlockchainInfo {
    blocks: u64,
    headers: u64,
    #[serde(rename = "verificationprogress")]
    verification_progress: f64,
    size_on_disk: u64,
}

impl BitcoinRpcClient {
    /// Create a new Bitcoin RPC client using cookie authentication
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

    /// Call a Bitcoin RPC method
    async fn call<T: for<'de> Deserialize<'de>>(&self, method: &str) -> Result<T> {
        let client = reqwest::Client::new();

        let body = serde_json::json!({
            "jsonrpc": "1.0",
            "id": "eigenix",
            "method": method,
            "params": []
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
            anyhow::bail!("RPC error: {}", error.message);
        }

        rpc_response
            .result
            .context("RPC response missing result field")
    }

    /// Get Bitcoin blockchain metrics
    pub async fn get_metrics(&self) -> Result<BitcoinMetrics> {
        let info: BlockchainInfo = self.call("getblockchaininfo").await?;

        Ok(BitcoinMetrics {
            blocks: info.blocks,
            headers: info.headers,
            verification_progress: info.verification_progress,
            size_on_disk: info.size_on_disk,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    #[ignore] // Only run with actual Bitcoin node
    async fn test_get_metrics() {
        let client = BitcoinRpcClient::new(
            "http://127.0.0.1:8332".to_string(),
            "/mnt/vault/bitcoind-data/.cookie",
        )
        .unwrap();

        let metrics = client.get_metrics().await.unwrap();
        assert!(metrics.blocks > 0);
        assert!(metrics.headers >= metrics.blocks);
    }
}
