use axum::{
    routing::get,
    Json, Router,
};
use clap::Parser;
use serde::Serialize;
use std::{net::SocketAddr, sync::Arc};
use tower_http::cors::{Any, CorsLayer};
use tracing_subscriber;

use anyhow::Context;
use eigenix_backend::{
    config::{Cli, Config},
    db::MetricsDatabase,
    metrics::MetricsCollector,
    routes,
    wallets::WalletManager,
    AppState,
};

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

    // Initialize wallets from ASB
    tracing::info!("Initializing wallets...");
    let wallet_config = config.to_wallet_config();
    let wallets = WalletManager::initialize_or_connect(wallet_config)
        .await
        .context("Failed to initialize wallets")?;
    let wallets = Arc::new(wallets);

    // Log wallet balances
    match wallets.get_balances().await {
        Ok((btc, xmr)) => {
            tracing::info!("Wallet balances - BTC: {:.8}, XMR: {:.12}", btc, xmr);
        }
        Err(e) => {
            tracing::warn!("Failed to get initial wallet balances: {}", e);
        }
    }

    // Spawn background metrics collection task
    let collector = MetricsCollector::new(config.clone(), db.clone());
    tokio::spawn(async move {
        collector.run().await;
    });
    tracing::info!("Started background metrics collection task");

    // Create application state
    let state = AppState {
        config: config.clone(),
        db,
        wallets,
    };

    // Build our application with routes
    let app = Router::new()
        .route("/health", get(health))
        .nest("/wallets", routes::wallets::wallet_routes())
        .nest("/metrics", routes::metrics::metrics_routes())
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
