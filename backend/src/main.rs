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
