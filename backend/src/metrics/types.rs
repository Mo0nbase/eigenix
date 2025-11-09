use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

// Re-export RPC clients from services
pub use crate::services::{BitcoinRpcClient, MoneroRpcClient};

/// Bitcoin blockchain information from getblockchaininfo RPC
#[derive(Debug, Serialize, Deserialize)]
pub struct BitcoinMetrics {
    pub blocks: u64,
    pub headers: u64,
    pub verification_progress: f64,
    pub size_on_disk: u64,
    pub wallet_balance: Option<f64>, // in BTC
}

/// Monero blockchain information
#[derive(Debug, Serialize, Deserialize)]
pub struct MoneroMetrics {
    pub height: u64,
    pub target_height: u64,
    pub difficulty: u64,
    pub tx_count: u64,
    pub wallet_balance: Option<f64>, // in XMR
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
