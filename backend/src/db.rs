use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use surrealdb::engine::remote::ws::{Client, Ws};
use surrealdb::opt::auth::Root;
use surrealdb::Surreal;

use crate::metrics::{AsbMetrics, BitcoinMetrics, ContainerMetrics, ElectrsMetrics, MoneroMetrics};

/// Trading transaction type
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum TransactionType {
    /// Bitcoin deposit to exchange
    BitcoinDeposit,
    /// BTC to XMR trade on exchange
    Trade,
    /// Monero withdrawal from exchange
    MoneroWithdrawal,
}

/// Trading transaction status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum TransactionStatus {
    /// Transaction initiated
    Pending,
    /// Transaction confirmed/completed
    Completed,
    /// Transaction failed
    Failed,
    /// Transaction cancelled
    Cancelled,
}

/// Database-stored trading transaction
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoredTradingTransaction {
    #[serde(skip_deserializing)]
    pub id: Option<String>,
    pub timestamp: DateTime<Utc>,
    pub transaction_type: TransactionType,
    pub status: TransactionStatus,
    pub btc_amount: Option<f64>,
    pub xmr_amount: Option<f64>,
    pub exchange_rate: Option<f64>,
    pub txid: Option<String>,
    pub order_id: Option<String>,
    pub refid: Option<String>,
    pub from_address: Option<String>,
    pub to_address: Option<String>,
    pub fee: Option<f64>,
    pub notes: Option<String>,
    pub error_message: Option<String>,
    pub completed_at: Option<DateTime<Utc>>,
}

/// Database-stored Bitcoin metrics with timestamp
#[derive(Debug, Serialize, Deserialize)]
pub struct StoredBitcoinMetrics {
    pub timestamp: DateTime<Utc>,
    pub blocks: u64,
    pub headers: u64,
    pub verification_progress: f64,
    pub size_on_disk: u64,
    pub wallet_balance: Option<f64>,
}

/// Database-stored Monero metrics with timestamp
#[derive(Debug, Serialize, Deserialize)]
pub struct StoredMoneroMetrics {
    pub timestamp: DateTime<Utc>,
    pub height: u64,
    pub target_height: u64,
    pub difficulty: u64,
    pub tx_count: u64,
    pub wallet_balance: Option<f64>,
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
            wallet_balance: metrics.wallet_balance,
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
            wallet_balance: metrics.wallet_balance,
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

    /// Store a trading transaction
    pub async fn store_trading_transaction(
        &self,
        transaction: &StoredTradingTransaction,
    ) -> Result<String> {
        let _result: Option<StoredTradingTransaction> = self
            .db
            .create("trading_transactions")
            .content(transaction.clone())
            .await
            .context("Failed to store trading transaction")?;

        // The response doesn't include the id field due to skip_deserializing
        // So we need to query it back or use a different approach
        // For now, let's use a query that returns the id explicitly
        let mut response = self
            .db
            .query("CREATE trading_transactions CONTENT $transaction RETURN VALUE meta::id(id)")
            .bind(("transaction", transaction.clone()))
            .await
            .context("Failed to store trading transaction")?;

        let ids: Vec<String> = response.take(0).context("Failed to get transaction ID")?;
        let id_string = ids.into_iter().next().context("No ID returned")?;

        Ok(id_string)
    }

    /// Update a trading transaction
    pub async fn update_trading_transaction(
        &self,
        id: &str,
        transaction: &StoredTradingTransaction,
    ) -> Result<()> {
        let _: Option<StoredTradingTransaction> = self
            .db
            .update(("trading_transactions", id))
            .content(transaction.clone())
            .await
            .context("Failed to update trading transaction")?;

        Ok(())
    }

    /// Get a trading transaction by ID
    pub async fn get_trading_transaction(
        &self,
        id: &str,
    ) -> Result<Option<StoredTradingTransaction>> {
        let result: Option<StoredTradingTransaction> = self
            .db
            .select(("trading_transactions", id))
            .await
            .context("Failed to get trading transaction")?;

        Ok(result)
    }

    /// Get all trading transactions within a time range
    pub async fn get_trading_transactions(
        &self,
        from: DateTime<Utc>,
        to: DateTime<Utc>,
    ) -> Result<Vec<StoredTradingTransaction>> {
        let result: Vec<StoredTradingTransaction> = self
            .db
            .query("SELECT * FROM trading_transactions WHERE timestamp >= $from AND timestamp <= $to ORDER BY timestamp DESC")
            .bind(("from", from))
            .bind(("to", to))
            .await
            .context("Failed to query trading transactions")?
            .take(0)
            .context("Failed to parse trading transactions")?;

        Ok(result)
    }

    /// Get recent trading transactions
    pub async fn get_recent_trading_transactions(
        &self,
        limit: usize,
    ) -> Result<Vec<StoredTradingTransaction>> {
        let result: Vec<StoredTradingTransaction> = self
            .db
            .query("SELECT * FROM trading_transactions ORDER BY timestamp DESC LIMIT $limit")
            .bind(("limit", limit))
            .await
            .context("Failed to query recent trading transactions")?
            .take(0)
            .context("Failed to parse trading transactions")?;

        Ok(result)
    }

    /// Get trading transactions by status
    pub async fn get_trading_transactions_by_status(
        &self,
        status: TransactionStatus,
    ) -> Result<Vec<StoredTradingTransaction>> {
        let status_str = format!("{:?}", status);
        let result: Vec<StoredTradingTransaction> = self
            .db
            .query(
                "SELECT * FROM trading_transactions WHERE status = $status ORDER BY timestamp DESC",
            )
            .bind(("status", status_str))
            .await
            .context("Failed to query trading transactions by status")?
            .take(0)
            .context("Failed to parse trading transactions")?;

        Ok(result)
    }

    /// Get trading transactions by type
    pub async fn get_trading_transactions_by_type(
        &self,
        transaction_type: TransactionType,
    ) -> Result<Vec<StoredTradingTransaction>> {
        let type_str = format!("{:?}", transaction_type);
        let result: Vec<StoredTradingTransaction> = self
            .db
            .query("SELECT * FROM trading_transactions WHERE transaction_type = $type ORDER BY timestamp DESC")
            .bind(("type", type_str))
            .await
            .context("Failed to query trading transactions by type")?
            .take(0)
            .context("Failed to parse trading transactions")?;

        Ok(result)
    }

    /// Mark a transaction as completed
    pub async fn complete_trading_transaction(
        &self,
        id: &str,
        xmr_amount: Option<f64>,
        exchange_rate: Option<f64>,
    ) -> Result<()> {
        let mut transaction = self
            .get_trading_transaction(id)
            .await?
            .context("Transaction not found")?;

        transaction.status = TransactionStatus::Completed;
        transaction.completed_at = Some(Utc::now());
        if let Some(amount) = xmr_amount {
            transaction.xmr_amount = Some(amount);
        }
        if let Some(rate) = exchange_rate {
            transaction.exchange_rate = Some(rate);
        }

        self.update_trading_transaction(id, &transaction).await?;
        Ok(())
    }

    /// Mark a transaction as failed
    pub async fn fail_trading_transaction(&self, id: &str, error_message: String) -> Result<()> {
        let mut transaction = self
            .get_trading_transaction(id)
            .await?
            .context("Transaction not found")?;

        transaction.status = TransactionStatus::Failed;
        transaction.error_message = Some(error_message);
        transaction.completed_at = Some(Utc::now());

        self.update_trading_transaction(id, &transaction).await?;
        Ok(())
    }
}
