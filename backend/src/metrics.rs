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

/// Monero blockchain information
#[derive(Debug, Serialize, Deserialize)]
pub struct MoneroMetrics {
    pub height: u64,
    pub target_height: u64,
    pub difficulty: u64,
    pub tx_count: u64,
}

/// ASB (Automated Swap Backend) metrics
#[derive(Debug, Serialize, Deserialize)]
pub struct AsbMetrics {
    pub balance_btc: f64,
    pub pending_swaps: u64,
    pub completed_swaps: u64,
    pub failed_swaps: u64,
    pub up: bool,
}

/// Electrs metrics
#[derive(Debug, Serialize, Deserialize)]
pub struct ElectrsMetrics {
    pub up: bool,
    pub indexed_blocks: u64,
}

/// Container health metrics
#[derive(Debug, Serialize, Deserialize)]
pub struct ContainerMetrics {
    pub name: String,
    pub up: bool,
    pub restarts: u64,
    pub uptime_seconds: u64,
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

/// Monero RPC client
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

        Ok(MoneroMetrics {
            height: info.height,
            target_height: info.target_height,
            difficulty: info.difficulty,
            tx_count: info.tx_count,
        })
    }
}

/// ASB RPC client
pub struct AsbRpcClient {
    url: String,
}

impl AsbRpcClient {
    pub fn new(url: String) -> Self {
        Self { url }
    }

    pub async fn get_metrics(&self) -> Result<AsbMetrics> {
        let client = reqwest::Client::new();

        // Check if ASB is up
        let up = client.get(&self.url).send().await.is_ok();

        if !up {
            return Ok(AsbMetrics {
                balance_btc: 0.0,
                pending_swaps: 0,
                completed_swaps: 0,
                failed_swaps: 0,
                up: false,
            });
        }

        // Get balance
        let balance_btc = match client
            .post(&self.url)
            .header("Content-Type", "application/json")
            .json(&serde_json::json!({
                "jsonrpc": "2.0",
                "method": "balance",
                "params": [],
                "id": 1
            }))
            .send()
            .await
        {
            Ok(response) => match response.json::<serde_json::Value>().await {
                Ok(v) => v["result"]["total"].as_f64().unwrap_or(0.0),
                Err(_) => 0.0,
            },
            Err(_) => 0.0,
        };

        // Get swap history
        let (pending, completed, failed) = match client
            .post(&self.url)
            .header("Content-Type", "application/json")
            .json(&serde_json::json!({
                "jsonrpc": "2.0",
                "method": "history",
                "params": [],
                "id": 1
            }))
            .send()
            .await
        {
            Ok(response) => match response.json::<serde_json::Value>().await {
                Ok(v) => {
                    if let Some(swaps) = v["result"]["swaps"].as_array() {
                        let pending = swaps.iter().filter(|s| s["state"] == "pending").count();
                        let completed = swaps.iter().filter(|s| s["state"] == "completed").count();
                        let failed = swaps.iter().filter(|s| s["state"] == "failed").count();
                        (pending as u64, completed as u64, failed as u64)
                    } else {
                        (0, 0, 0)
                    }
                }
                Err(_) => (0, 0, 0),
            },
            Err(_) => (0, 0, 0),
        };

        Ok(AsbMetrics {
            balance_btc,
            pending_swaps: pending,
            completed_swaps: completed,
            failed_swaps: failed,
            up: true,
        })
    }
}

/// Electrs client
pub struct ElectrsClient {
    container_name: String,
}

impl ElectrsClient {
    pub fn new(container_name: String) -> Self {
        Self { container_name }
    }

    pub async fn get_metrics(&self) -> Result<ElectrsMetrics> {
        // Check if container is running
        let output = std::process::Command::new("sudo")
            .arg("podman")
            .arg("ps")
            .arg("--filter")
            .arg(format!("name=^{}$", self.container_name))
            .arg("--format")
            .arg("{{.Status}}")
            .output()
            .context("Failed to check Electrs container status")?;

        let status = String::from_utf8_lossy(&output.stdout);
        let up = status.contains("Up");

        let indexed_blocks = if up {
            // Try to get indexed blocks from logs
            std::process::Command::new("sudo")
                .arg("podman")
                .arg("logs")
                .arg("--tail")
                .arg("50")
                .arg(&self.container_name)
                .output()
                .ok()
                .and_then(|o| String::from_utf8(o.stdout).ok())
                .and_then(|logs| {
                    // Parse "indexed X blocks" from logs
                    logs.lines()
                        .filter_map(|line| {
                            line.split("indexed ")
                                .nth(1)?
                                .split(" blocks")
                                .next()?
                                .parse::<u64>()
                                .ok()
                        })
                        .last()
                })
                .unwrap_or(0)
        } else {
            0
        };

        Ok(ElectrsMetrics { up, indexed_blocks })
    }
}

/// Container health checker
pub struct ContainerHealthClient;

impl ContainerHealthClient {
    pub fn new() -> Self {
        Self
    }

    pub async fn get_metrics(&self, container_names: &[&str]) -> Result<Vec<ContainerMetrics>> {
        let mut metrics = Vec::new();

        for name in container_names {
            let output = std::process::Command::new("sudo")
                .arg("podman")
                .arg("ps")
                .arg("-a")
                .arg("--filter")
                .arg(format!("name=^{}$", name))
                .arg("--format")
                .arg("{{.Status}}")
                .output()
                .context("Failed to check container status")?;

            let status = String::from_utf8_lossy(&output.stdout);
            let up = status.contains("Up");

            let (restarts, uptime_seconds) = if up {
                // Get restart count
                let restart_output = std::process::Command::new("sudo")
                    .arg("podman")
                    .arg("inspect")
                    .arg(name)
                    .arg("--format")
                    .arg("{{.RestartCount}}")
                    .output()
                    .ok()
                    .and_then(|o| String::from_utf8(o.stdout).ok())
                    .and_then(|s| s.trim().parse::<u64>().ok())
                    .unwrap_or(0);

                // Get uptime
                let uptime_output = std::process::Command::new("sudo")
                    .arg("podman")
                    .arg("inspect")
                    .arg(name)
                    .arg("--format")
                    .arg("{{.State.StartedAt}}")
                    .output()
                    .ok()
                    .and_then(|o| String::from_utf8(o.stdout).ok())
                    .and_then(|_started| {
                        // Parse timestamp and calculate uptime
                        // This is a simplified version
                        Some(0) // TODO: Implement proper timestamp parsing
                    })
                    .unwrap_or(0);

                (restart_output, uptime_output)
            } else {
                (0, 0)
            };

            metrics.push(ContainerMetrics {
                name: name.to_string(),
                up,
                restarts,
                uptime_seconds,
            });
        }

        Ok(metrics)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    #[ignore] // Only run with actual Bitcoin node
    async fn test_get_bitcoin_metrics() {
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
