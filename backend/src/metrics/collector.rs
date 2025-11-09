//! Metrics collection service
//!
//! This module handles periodic collection of metrics from various sources:
//! - Bitcoin node
//! - Monero node
//! - ASB (Atomic Swap Backend)
//! - Electrs
//! - Container health
//!
//! The collector runs as a background task and stores metrics in the database.

use std::sync::Arc;
use tokio::time::{interval, Duration as TokioDuration};

use crate::{
    config::Config,
    db::MetricsDatabase,
    metrics::{
        AsbRpcClient, BitcoinRpcClient, ContainerHealthClient, ElectrsClient, MoneroRpcClient,
    },
};

/// Metrics collector service
pub struct MetricsCollector {
    config: Arc<Config>,
    db: MetricsDatabase,
}

impl MetricsCollector {
    /// Create a new metrics collector
    pub fn new(config: Arc<Config>, db: MetricsDatabase) -> Self {
        Self { config, db }
    }

    /// Run the metrics collection loop
    ///
    /// This function runs indefinitely, collecting metrics every 60 seconds.
    pub async fn run(self) {
        let mut ticker = interval(TokioDuration::from_secs(60));

        loop {
            ticker.tick().await;
            tracing::info!("Collecting metrics...");

            self.collect_all().await;

            tracing::info!("Metrics collection complete");
        }
    }

    /// Collect all metrics from all sources
    async fn collect_all(&self) {
        // Collect metrics in parallel for better performance
        tokio::join!(
            self.collect_bitcoin(),
            self.collect_monero(),
            self.collect_asb(),
            self.collect_electrs(),
            self.collect_containers(),
        );
    }

    /// Collect Bitcoin metrics
    async fn collect_bitcoin(&self) {
        match BitcoinRpcClient::new(
            self.config.bitcoin.rpc_url.clone(),
            &self.config.bitcoin.cookie_path,
        ) {
            Ok(client) => match client.get_metrics().await {
                Ok(metrics) => {
                    if let Err(e) = self.db.store_bitcoin_metrics(&metrics).await {
                        tracing::error!("Failed to store Bitcoin metrics: {}", e);
                    }
                }
                Err(e) => tracing::error!("Failed to collect Bitcoin metrics: {}", e),
            },
            Err(e) => tracing::error!("Failed to create Bitcoin RPC client: {}", e),
        }
    }

    /// Collect Monero metrics
    async fn collect_monero(&self) {
        let client = MoneroRpcClient::new(self.config.monero.rpc_url.clone());
        match client.get_metrics().await {
            Ok(metrics) => {
                if let Err(e) = self.db.store_monero_metrics(&metrics).await {
                    tracing::error!("Failed to store Monero metrics: {}", e);
                }
            }
            Err(e) => tracing::error!("Failed to collect Monero metrics: {}", e),
        }
    }

    /// Collect ASB metrics
    async fn collect_asb(&self) {
        let client = AsbRpcClient::new(self.config.asb.rpc_url.clone());
        match client.get_metrics().await {
            Ok(metrics) => {
                if let Err(e) = self.db.store_asb_metrics(&metrics).await {
                    tracing::error!("Failed to store ASB metrics: {}", e);
                }
            }
            Err(e) => tracing::error!("Failed to collect ASB metrics: {}", e),
        }
    }

    /// Collect Electrs metrics
    async fn collect_electrs(&self) {
        let client = ElectrsClient::new("electrs".to_string());
        match client.get_metrics().await {
            Ok(metrics) => {
                if let Err(e) = self.db.store_electrs_metrics(&metrics).await {
                    tracing::error!("Failed to store Electrs metrics: {}", e);
                }
            }
            Err(e) => tracing::error!("Failed to collect Electrs metrics: {}", e),
        }
    }

    /// Collect container health metrics
    async fn collect_containers(&self) {
        let client = ContainerHealthClient::new();
        let container_refs: Vec<&str> = self
            .config
            .containers
            .names
            .iter()
            .map(|s| s.as_str())
            .collect();

        match client.get_metrics(&container_refs).await {
            Ok(metrics) => {
                if let Err(e) = self.db.store_container_metrics(&metrics).await {
                    tracing::error!("Failed to store container metrics: {}", e);
                }
            }
            Err(e) => tracing::error!("Failed to collect container metrics: {}", e),
        }
    }
}

