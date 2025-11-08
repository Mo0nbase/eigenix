use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use surrealdb::engine::remote::ws::{Client, Ws};
use surrealdb::opt::auth::Root;
use surrealdb::Surreal;

use crate::metrics::{AsbMetrics, BitcoinMetrics, ContainerMetrics, ElectrsMetrics, MoneroMetrics};

/// Database-stored Bitcoin metrics with timestamp
#[derive(Debug, Serialize, Deserialize)]
pub struct StoredBitcoinMetrics {
    pub timestamp: DateTime<Utc>,
    pub blocks: u64,
    pub headers: u64,
    pub verification_progress: f64,
    pub size_on_disk: u64,
}

/// Database-stored Monero metrics with timestamp
#[derive(Debug, Serialize, Deserialize)]
pub struct StoredMoneroMetrics {
    pub timestamp: DateTime<Utc>,
    pub height: u64,
    pub target_height: u64,
    pub difficulty: u64,
    pub tx_count: u64,
}

/// Database-stored ASB metrics with timestamp
#[derive(Debug, Serialize, Deserialize)]
pub struct StoredAsbMetrics {
    pub timestamp: DateTime<Utc>,
    pub balance_btc: f64,
    pub pending_swaps: u64,
    pub completed_swaps: u64,
    pub failed_swaps: u64,
    pub up: bool,
}

/// Database-stored Electrs metrics with timestamp
#[derive(Debug, Serialize, Deserialize)]
pub struct StoredElectrsMetrics {
    pub timestamp: DateTime<Utc>,
    pub up: bool,
    pub indexed_blocks: u64,
}

/// Database-stored Container metrics with timestamp
#[derive(Debug, Serialize, Deserialize)]
pub struct StoredContainerMetrics {
    pub timestamp: DateTime<Utc>,
    pub name: String,
    pub up: bool,
    pub restarts: u64,
    pub uptime_seconds: u64,
}

/// Summary of all latest metrics
#[derive(Debug, Serialize, Deserialize)]
pub struct MetricsSummary {
    pub bitcoin: Option<StoredBitcoinMetrics>,
    pub monero: Option<StoredMoneroMetrics>,
    pub asb: Option<StoredAsbMetrics>,
    pub electrs: Option<StoredElectrsMetrics>,
    pub containers: Vec<StoredContainerMetrics>,
}

/// Metrics database interface
#[derive(Clone)]
pub struct MetricsDatabase {
    db: Surreal<Client>,
}

impl MetricsDatabase {
    /// Connect to SurrealDB
    pub async fn connect(endpoint: &str, namespace: &str, database: &str) -> Result<Self> {
        let db = Surreal::new::<Ws>(endpoint)
            .await
            .context("Failed to connect to SurrealDB")?;

        // Sign in as root user (for local development)
        db.signin(Root {
            username: "root",
            password: "root",
        })
        .await
        .context("Failed to sign in to SurrealDB")?;

        // Use namespace and database
        db.use_ns(namespace)
            .use_db(database)
            .await
            .context("Failed to select namespace and database")?;

        Ok(Self { db })
    }

    /// Store Bitcoin metrics
    pub async fn store_bitcoin_metrics(&self, metrics: &BitcoinMetrics) -> Result<()> {
        let stored = StoredBitcoinMetrics {
            timestamp: Utc::now(),
            blocks: metrics.blocks,
            headers: metrics.headers,
            verification_progress: metrics.verification_progress,
            size_on_disk: metrics.size_on_disk,
        };

        let _: Option<StoredBitcoinMetrics> = self
            .db
            .create("bitcoin_metrics")
            .content(stored)
            .await
            .context("Failed to store Bitcoin metrics")?;

        Ok(())
    }

    /// Store Monero metrics
    pub async fn store_monero_metrics(&self, metrics: &MoneroMetrics) -> Result<()> {
        let stored = StoredMoneroMetrics {
            timestamp: Utc::now(),
            height: metrics.height,
            target_height: metrics.target_height,
            difficulty: metrics.difficulty,
            tx_count: metrics.tx_count,
        };

        let _: Option<StoredMoneroMetrics> = self
            .db
            .create("monero_metrics")
            .content(stored)
            .await
            .context("Failed to store Monero metrics")?;

        Ok(())
    }

    /// Store ASB metrics
    pub async fn store_asb_metrics(&self, metrics: &AsbMetrics) -> Result<()> {
        let stored = StoredAsbMetrics {
            timestamp: Utc::now(),
            balance_btc: metrics.balance_btc,
            pending_swaps: metrics.pending_swaps,
            completed_swaps: metrics.completed_swaps,
            failed_swaps: metrics.failed_swaps,
            up: metrics.up,
        };

        let _: Option<StoredAsbMetrics> = self
            .db
            .create("asb_metrics")
            .content(stored)
            .await
            .context("Failed to store ASB metrics")?;

        Ok(())
    }

    /// Store Electrs metrics
    pub async fn store_electrs_metrics(&self, metrics: &ElectrsMetrics) -> Result<()> {
        let stored = StoredElectrsMetrics {
            timestamp: Utc::now(),
            up: metrics.up,
            indexed_blocks: metrics.indexed_blocks,
        };

        let _: Option<StoredElectrsMetrics> = self
            .db
            .create("electrs_metrics")
            .content(stored)
            .await
            .context("Failed to store Electrs metrics")?;

        Ok(())
    }

    /// Store Container metrics
    pub async fn store_container_metrics(&self, metrics: &[ContainerMetrics]) -> Result<()> {
        for metric in metrics {
            let stored = StoredContainerMetrics {
                timestamp: Utc::now(),
                name: metric.name.clone(),
                up: metric.up,
                restarts: metric.restarts,
                uptime_seconds: metric.uptime_seconds,
            };

            let _: Option<StoredContainerMetrics> = self
                .db
                .create("container_metrics")
                .content(stored)
                .await
                .context("Failed to store container metrics")?;
        }

        Ok(())
    }

    /// Get latest Bitcoin metrics
    pub async fn get_latest_bitcoin_metrics(&self) -> Result<Option<StoredBitcoinMetrics>> {
        let mut result: Vec<StoredBitcoinMetrics> = self
            .db
            .query("SELECT * FROM bitcoin_metrics ORDER BY timestamp DESC LIMIT 1")
            .await
            .context("Failed to query Bitcoin metrics")?
            .take(0)
            .context("Failed to parse Bitcoin metrics")?;

        Ok(result.pop())
    }

    /// Get latest Monero metrics
    pub async fn get_latest_monero_metrics(&self) -> Result<Option<StoredMoneroMetrics>> {
        let mut result: Vec<StoredMoneroMetrics> = self
            .db
            .query("SELECT * FROM monero_metrics ORDER BY timestamp DESC LIMIT 1")
            .await
            .context("Failed to query Monero metrics")?
            .take(0)
            .context("Failed to parse Monero metrics")?;

        Ok(result.pop())
    }

    /// Get latest ASB metrics
    pub async fn get_latest_asb_metrics(&self) -> Result<Option<StoredAsbMetrics>> {
        let mut result: Vec<StoredAsbMetrics> = self
            .db
            .query("SELECT * FROM asb_metrics ORDER BY timestamp DESC LIMIT 1")
            .await
            .context("Failed to query ASB metrics")?
            .take(0)
            .context("Failed to parse ASB metrics")?;

        Ok(result.pop())
    }

    /// Get latest Electrs metrics
    pub async fn get_latest_electrs_metrics(&self) -> Result<Option<StoredElectrsMetrics>> {
        let mut result: Vec<StoredElectrsMetrics> = self
            .db
            .query("SELECT * FROM electrs_metrics ORDER BY timestamp DESC LIMIT 1")
            .await
            .context("Failed to query Electrs metrics")?
            .take(0)
            .context("Failed to parse Electrs metrics")?;

        Ok(result.pop())
    }

    /// Get latest Container metrics for all containers
    pub async fn get_latest_container_metrics(&self) -> Result<Vec<StoredContainerMetrics>> {
        // Get the latest timestamp
        let latest: Vec<StoredContainerMetrics> = self
            .db
            .query(
                "SELECT * FROM container_metrics
                 WHERE timestamp = (SELECT VALUE timestamp FROM container_metrics ORDER BY timestamp DESC LIMIT 1)[0]",
            )
            .await
            .context("Failed to query container metrics")?
            .take(0)
            .context("Failed to parse container metrics")?;

        Ok(latest)
    }

    /// Get Bitcoin metrics history within time range
    pub async fn get_bitcoin_history(
        &self,
        from: DateTime<Utc>,
        to: DateTime<Utc>,
    ) -> Result<Vec<StoredBitcoinMetrics>> {
        let result: Vec<StoredBitcoinMetrics> = self
            .db
            .query("SELECT * FROM bitcoin_metrics WHERE timestamp >= $from AND timestamp <= $to ORDER BY timestamp ASC")
            .bind(("from", from))
            .bind(("to", to))
            .await
            .context("Failed to query Bitcoin history")?
            .take(0)
            .context("Failed to parse Bitcoin history")?;

        Ok(result)
    }

    /// Get Monero metrics history within time range
    pub async fn get_monero_history(
        &self,
        from: DateTime<Utc>,
        to: DateTime<Utc>,
    ) -> Result<Vec<StoredMoneroMetrics>> {
        let result: Vec<StoredMoneroMetrics> = self
            .db
            .query("SELECT * FROM monero_metrics WHERE timestamp >= $from AND timestamp <= $to ORDER BY timestamp ASC")
            .bind(("from", from))
            .bind(("to", to))
            .await
            .context("Failed to query Monero history")?
            .take(0)
            .context("Failed to parse Monero history")?;

        Ok(result)
    }

    /// Get ASB metrics history within time range
    pub async fn get_asb_history(
        &self,
        from: DateTime<Utc>,
        to: DateTime<Utc>,
    ) -> Result<Vec<StoredAsbMetrics>> {
        let result: Vec<StoredAsbMetrics> = self
            .db
            .query("SELECT * FROM asb_metrics WHERE timestamp >= $from AND timestamp <= $to ORDER BY timestamp ASC")
            .bind(("from", from))
            .bind(("to", to))
            .await
            .context("Failed to query ASB history")?
            .take(0)
            .context("Failed to parse ASB history")?;

        Ok(result)
    }

    /// Get Electrs metrics history within time range
    pub async fn get_electrs_history(
        &self,
        from: DateTime<Utc>,
        to: DateTime<Utc>,
    ) -> Result<Vec<StoredElectrsMetrics>> {
        let result: Vec<StoredElectrsMetrics> = self
            .db
            .query("SELECT * FROM electrs_metrics WHERE timestamp >= $from AND timestamp <= $to ORDER BY timestamp ASC")
            .bind(("from", from))
            .bind(("to", to))
            .await
            .context("Failed to query Electrs history")?
            .take(0)
            .context("Failed to parse Electrs history")?;

        Ok(result)
    }

    /// Get Container metrics history within time range for a specific container
    pub async fn get_container_history(
        &self,
        container_name: &str,
        from: DateTime<Utc>,
        to: DateTime<Utc>,
    ) -> Result<Vec<StoredContainerMetrics>> {
        let name = container_name.to_string();
        let result: Vec<StoredContainerMetrics> = self
            .db
            .query("SELECT * FROM container_metrics WHERE name = $name AND timestamp >= $from AND timestamp <= $to ORDER BY timestamp ASC")
            .bind(("name", name))
            .bind(("from", from))
            .bind(("to", to))
            .await
            .context("Failed to query container history")?
            .take(0)
            .context("Failed to parse container history")?;

        Ok(result)
    }

    /// Get summary of all latest metrics
    pub async fn get_summary(&self) -> Result<MetricsSummary> {
        Ok(MetricsSummary {
            bitcoin: self.get_latest_bitcoin_metrics().await?,
            monero: self.get_latest_monero_metrics().await?,
            asb: self.get_latest_asb_metrics().await?,
            electrs: self.get_latest_electrs_metrics().await?,
            containers: self.get_latest_container_metrics().await?,
        })
    }
}
