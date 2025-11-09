use axum::{
    routing::get,
    Json, Router,
};
use clap::Parser;
use serde::Serialize;
use std::{net::SocketAddr, sync::Arc};
use tokio::time::{interval, Duration as TokioDuration};
use tower_http::cors::{Any, CorsLayer};
use tracing_subscriber;

use anyhow::Context;
use eigenix_backend::{
    config::{Cli, Config},
    db::MetricsDatabase,
    metrics,
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
