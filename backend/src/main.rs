use axum::{extract::State, routing::get, Json, Router};
use clap::Parser;
use serde::Serialize;
use std::{net::SocketAddr, sync::Arc};
use tower_http::cors::{Any, CorsLayer};
use tracing_subscriber;

mod config;
mod metrics;

use config::{Cli, Config};

#[derive(Serialize)]
struct Health {
    status: String,
    version: String,
}

async fn health() -> Json<Health> {
    Json(Health {
        status: "healthy".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
    })
}

async fn bitcoin_metrics(
    State(config): State<Arc<Config>>,
) -> Result<Json<metrics::BitcoinMetrics>, String> {
    let client =
        metrics::BitcoinRpcClient::new(config.bitcoin.rpc_url.clone(), &config.bitcoin.cookie_path)
            .map_err(|e| format!("Failed to create Bitcoin RPC client: {}", e))?;

    let metrics = client
        .get_metrics()
        .await
        .map_err(|e| format!("Failed to get Bitcoin metrics: {}", e))?;

    Ok(Json(metrics))
}

async fn monero_metrics(
    State(config): State<Arc<Config>>,
) -> Result<Json<metrics::MoneroMetrics>, String> {
    let client = metrics::MoneroRpcClient::new(config.monero.rpc_url.clone());

    let metrics = client
        .get_metrics()
        .await
        .map_err(|e| format!("Failed to get Monero metrics: {}", e))?;

    Ok(Json(metrics))
}

async fn asb_metrics(
    State(config): State<Arc<Config>>,
) -> Result<Json<metrics::AsbMetrics>, String> {
    let client = metrics::AsbRpcClient::new(config.asb.rpc_url.clone());

    let metrics = client
        .get_metrics()
        .await
        .map_err(|e| format!("Failed to get ASB metrics: {}", e))?;

    Ok(Json(metrics))
}

async fn electrs_metrics() -> Result<Json<metrics::ElectrsMetrics>, String> {
    let client = metrics::ElectrsClient::new("electrs".to_string());

    let metrics = client
        .get_metrics()
        .await
        .map_err(|e| format!("Failed to get Electrs metrics: {}", e))?;

    Ok(Json(metrics))
}

async fn container_metrics(
    State(config): State<Arc<Config>>,
) -> Result<Json<Vec<metrics::ContainerMetrics>>, String> {
    let client = metrics::ContainerHealthClient::new();
    let container_refs: Vec<&str> = config.containers.names.iter().map(|s| s.as_str()).collect();

    let metrics = client
        .get_metrics(&container_refs)
        .await
        .map_err(|e| format!("Failed to get container metrics: {}", e))?;

    Ok(Json(metrics))
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

    // Build our application with routes
    let app = Router::new()
        .route("/health", get(health))
        .route("/metrics/bitcoin", get(bitcoin_metrics))
        .route("/metrics/monero", get(monero_metrics))
        .route("/metrics/asb", get(asb_metrics))
        .route("/metrics/electrs", get(electrs_metrics))
        .route("/metrics/containers", get(container_metrics))
        .with_state(config.clone())
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
