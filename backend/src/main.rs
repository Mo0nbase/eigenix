use axum::{routing::get, Json, Router};
use serde::Serialize;
use std::net::SocketAddr;
use tower_http::cors::{Any, CorsLayer};
use tracing_subscriber;

mod metrics;

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

async fn bitcoin_metrics() -> Result<Json<metrics::BitcoinMetrics>, String> {
    let client = metrics::BitcoinRpcClient::new(
        "http://127.0.0.1:8332".to_string(),
        "/mnt/vault/bitcoind-data/.cookie",
    )
    .map_err(|e| format!("Failed to create Bitcoin RPC client: {}", e))?;

    let metrics = client
        .get_metrics()
        .await
        .map_err(|e| format!("Failed to get Bitcoin metrics: {}", e))?;

    Ok(Json(metrics))
}

async fn monero_metrics() -> Result<Json<metrics::MoneroMetrics>, String> {
    let client = metrics::MoneroRpcClient::new("http://127.0.0.1:18081/json_rpc".to_string());

    let metrics = client
        .get_metrics()
        .await
        .map_err(|e| format!("Failed to get Monero metrics: {}", e))?;

    Ok(Json(metrics))
}

async fn asb_metrics() -> Result<Json<metrics::AsbMetrics>, String> {
    let client = metrics::AsbRpcClient::new("http://127.0.0.1:9944".to_string());

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

async fn container_metrics() -> Result<Json<Vec<metrics::ContainerMetrics>>, String> {
    let client = metrics::ContainerHealthClient::new();
    let containers = vec!["bitcoind", "electrs", "monerod", "asb", "asb-controller"];

    let metrics = client
        .get_metrics(&containers)
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

    // Build our application with routes
    let app = Router::new()
        .route("/health", get(health))
        .route("/metrics/bitcoin", get(bitcoin_metrics))
        .route("/metrics/monero", get(monero_metrics))
        .route("/metrics/asb", get(asb_metrics))
        .route("/metrics/electrs", get(electrs_metrics))
        .route("/metrics/containers", get(container_metrics))
        .layer(
            CorsLayer::new()
                .allow_origin(Any)
                .allow_methods(Any)
                .allow_headers(Any),
        );

    // Run it
    let addr = SocketAddr::from(([127, 0, 0, 1], 1234));
    tracing::info!("Listening on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
