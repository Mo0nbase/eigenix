use axum::{
    extract::{Query, State},
    routing::get,
    Json, Router,
};
use chrono::{DateTime, Duration, Utc};
use clap::Parser;
use serde::{Deserialize, Serialize};
use std::{net::SocketAddr, sync::Arc};
use tokio::time::{interval, Duration as TokioDuration};
use tower_http::cors::{Any, CorsLayer};
use tracing_subscriber;

mod config;
mod db;
mod metrics;

use config::{Cli, Config};
use db::MetricsDatabase;

#[derive(Clone)]
struct AppState {
    config: Arc<Config>,
    db: MetricsDatabase,
}

#[derive(Serialize)]
struct Health {
    status: String,
    version: String,
}

#[derive(Deserialize)]
struct HistoryQuery {
    from: Option<DateTime<Utc>>,
    to: Option<DateTime<Utc>>,
}

async fn health() -> Json<Health> {
    Json(Health {
        status: "healthy".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
    })
}

async fn bitcoin_metrics(
    State(state): State<AppState>,
) -> Result<Json<db::StoredBitcoinMetrics>, String> {
    let metrics = state
        .db
        .get_latest_bitcoin_metrics()
        .await
        .map_err(|e| format!("Failed to get Bitcoin metrics: {}", e))?
        .ok_or_else(|| "No Bitcoin metrics available".to_string())?;

    Ok(Json(metrics))
}

async fn monero_metrics(
    State(state): State<AppState>,
) -> Result<Json<db::StoredMoneroMetrics>, String> {
    let metrics = state
        .db
        .get_latest_monero_metrics()
        .await
        .map_err(|e| format!("Failed to get Monero metrics: {}", e))?
        .ok_or_else(|| "No Monero metrics available".to_string())?;

    Ok(Json(metrics))
}

async fn asb_metrics(State(state): State<AppState>) -> Result<Json<db::StoredAsbMetrics>, String> {
    let metrics = state
        .db
        .get_latest_asb_metrics()
        .await
        .map_err(|e| format!("Failed to get ASB metrics: {}", e))?
        .ok_or_else(|| "No ASB metrics available".to_string())?;

    Ok(Json(metrics))
}

async fn electrs_metrics(
    State(state): State<AppState>,
) -> Result<Json<db::StoredElectrsMetrics>, String> {
    let metrics = state
        .db
        .get_latest_electrs_metrics()
        .await
        .map_err(|e| format!("Failed to get Electrs metrics: {}", e))?
        .ok_or_else(|| "No Electrs metrics available".to_string())?;

    Ok(Json(metrics))
}

async fn container_metrics(
    State(state): State<AppState>,
) -> Result<Json<Vec<db::StoredContainerMetrics>>, String> {
    let metrics = state
        .db
        .get_latest_container_metrics()
        .await
        .map_err(|e| format!("Failed to get container metrics: {}", e))?;

    Ok(Json(metrics))
}

async fn summary_metrics(
    State(state): State<AppState>,
) -> Result<Json<db::MetricsSummary>, String> {
    let summary = state
        .db
        .get_summary()
        .await
        .map_err(|e| format!("Failed to get metrics summary: {}", e))?;

    Ok(Json(summary))
}

async fn bitcoin_history(
    State(state): State<AppState>,
    Query(query): Query<HistoryQuery>,
) -> Result<Json<Vec<db::StoredBitcoinMetrics>>, String> {
    let to = query.to.unwrap_or_else(Utc::now);
    let from = query.from.unwrap_or_else(|| to - Duration::hours(24));

    let history = state
        .db
        .get_bitcoin_history(from, to)
        .await
        .map_err(|e| format!("Failed to get Bitcoin history: {}", e))?;

    Ok(Json(history))
}

async fn monero_history(
    State(state): State<AppState>,
    Query(query): Query<HistoryQuery>,
) -> Result<Json<Vec<db::StoredMoneroMetrics>>, String> {
    let to = query.to.unwrap_or_else(Utc::now);
    let from = query.from.unwrap_or_else(|| to - Duration::hours(24));

    let history = state
        .db
        .get_monero_history(from, to)
        .await
        .map_err(|e| format!("Failed to get Monero history: {}", e))?;

    Ok(Json(history))
}

async fn asb_history(
    State(state): State<AppState>,
    Query(query): Query<HistoryQuery>,
) -> Result<Json<Vec<db::StoredAsbMetrics>>, String> {
    let to = query.to.unwrap_or_else(Utc::now);
    let from = query.from.unwrap_or_else(|| to - Duration::hours(24));

    let history = state
        .db
        .get_asb_history(from, to)
        .await
        .map_err(|e| format!("Failed to get ASB history: {}", e))?;

    Ok(Json(history))
}

async fn electrs_history(
    State(state): State<AppState>,
    Query(query): Query<HistoryQuery>,
) -> Result<Json<Vec<db::StoredElectrsMetrics>>, String> {
    let to = query.to.unwrap_or_else(Utc::now);
    let from = query.from.unwrap_or_else(|| to - Duration::hours(24));

    let history = state
        .db
        .get_electrs_history(from, to)
        .await
        .map_err(|e| format!("Failed to get Electrs history: {}", e))?;

    Ok(Json(history))
}

#[derive(Deserialize)]
struct ContainerHistoryQuery {
    name: String,
    from: Option<DateTime<Utc>>,
    to: Option<DateTime<Utc>>,
}

async fn container_history(
    State(state): State<AppState>,
    Query(query): Query<ContainerHistoryQuery>,
) -> Result<Json<Vec<db::StoredContainerMetrics>>, String> {
    let to = query.to.unwrap_or_else(Utc::now);
    let from = query.from.unwrap_or_else(|| to - Duration::hours(24));

    let history = state
        .db
        .get_container_history(&query.name, from, to)
        .await
        .map_err(|e| format!("Failed to get container history: {}", e))?;

    Ok(Json(history))
}

async fn collect_metrics(config: Arc<Config>, db: MetricsDatabase) {
    let mut interval = interval(TokioDuration::from_secs(60));

    loop {
        interval.tick().await;

        tracing::info!("Collecting metrics...");

        // Collect Bitcoin metrics
        if let Ok(client) = metrics::BitcoinRpcClient::new(
            config.bitcoin.rpc_url.clone(),
            &config.bitcoin.cookie_path,
        ) {
            match client.get_metrics().await {
                Ok(metrics) => {
                    if let Err(e) = db.store_bitcoin_metrics(&metrics).await {
                        tracing::error!("Failed to store Bitcoin metrics: {}", e);
                    }
                }
                Err(e) => tracing::error!("Failed to collect Bitcoin metrics: {}", e),
            }
        }

        // Collect Monero metrics
        let monero_client = metrics::MoneroRpcClient::new(config.monero.rpc_url.clone());
        match monero_client.get_metrics().await {
            Ok(metrics) => {
                if let Err(e) = db.store_monero_metrics(&metrics).await {
                    tracing::error!("Failed to store Monero metrics: {}", e);
                }
            }
            Err(e) => tracing::error!("Failed to collect Monero metrics: {}", e),
        }

        // Collect ASB metrics
        let asb_client = metrics::AsbRpcClient::new(config.asb.rpc_url.clone());
        match asb_client.get_metrics().await {
            Ok(metrics) => {
                if let Err(e) = db.store_asb_metrics(&metrics).await {
                    tracing::error!("Failed to store ASB metrics: {}", e);
                }
            }
            Err(e) => tracing::error!("Failed to collect ASB metrics: {}", e),
        }

        // Collect Electrs metrics
        let electrs_client = metrics::ElectrsClient::new("electrs".to_string());
        match electrs_client.get_metrics().await {
            Ok(metrics) => {
                if let Err(e) = db.store_electrs_metrics(&metrics).await {
                    tracing::error!("Failed to store Electrs metrics: {}", e);
                }
            }
            Err(e) => tracing::error!("Failed to collect Electrs metrics: {}", e),
        }

        // Collect Container metrics
        let container_client = metrics::ContainerHealthClient::new();
        let container_refs: Vec<&str> =
            config.containers.names.iter().map(|s| s.as_str()).collect();
        match container_client.get_metrics(&container_refs).await {
            Ok(metrics) => {
                if let Err(e) = db.store_container_metrics(&metrics).await {
                    tracing::error!("Failed to store container metrics: {}", e);
                }
            }
            Err(e) => tracing::error!("Failed to collect container metrics: {}", e),
        }

        tracing::info!("Metrics collection complete");
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_target(false)
        .compact()
        .init();

    // Parse CLI arguments and load configuration
    let cli = Cli::parse();
    let config = Config::load(cli)?;
    let config = Arc::new(config);

    tracing::info!("Configuration loaded: {:?}", config);

    // Connect to SurrealDB
    tracing::info!("Connecting to SurrealDB at {}", config.database.endpoint);
    let db = MetricsDatabase::connect(
        &config.database.endpoint,
        &config.database.namespace,
        &config.database.database,
    )
    .await?;
    tracing::info!("Connected to SurrealDB");

    // Spawn background metrics collection task
    let metrics_config = config.clone();
    let metrics_db = db.clone();
    tokio::spawn(async move {
        collect_metrics(metrics_config, metrics_db).await;
    });
    tracing::info!("Started background metrics collection task");

    // Create application state
    let state = AppState {
        config: config.clone(),
        db,
    };

    // Build our application with routes
    let app = Router::new()
        .route("/health", get(health))
        .route("/metrics/summary", get(summary_metrics))
        .route("/metrics/bitcoin", get(bitcoin_metrics))
        .route("/metrics/bitcoin/history", get(bitcoin_history))
        .route("/metrics/monero", get(monero_metrics))
        .route("/metrics/monero/history", get(monero_history))
        .route("/metrics/asb", get(asb_metrics))
        .route("/metrics/asb/history", get(asb_history))
        .route("/metrics/electrs", get(electrs_metrics))
        .route("/metrics/electrs/history", get(electrs_history))
        .route("/metrics/containers", get(container_metrics))
        .route("/metrics/containers/history", get(container_history))
        .with_state(state)
        .layer(
            CorsLayer::new()
                .allow_origin(Any)
                .allow_methods(Any)
                .allow_headers(Any),
        );

    // Run it
    let addr = SocketAddr::from((
        config.server.host.parse::<std::net::IpAddr>()?,
        config.server.port,
    ));
    tracing::info!("Listening on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
