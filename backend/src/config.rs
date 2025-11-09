use clap::Parser;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(name = "eigenix-backend")]
#[command(about = "Eigenix metrics backend server", long_about = None)]
pub struct Cli {
    /// Path to configuration file
    #[arg(short, long, value_name = "FILE")]
    pub config: Option<PathBuf>,

    /// Bitcoin RPC URL
    #[arg(long, default_value = "http://127.0.0.1:8332")]
    pub bitcoin_rpc_url: Option<String>,

    /// Bitcoin RPC cookie file path
    #[arg(long, default_value = "/mnt/vault/bitcoind-data/.cookie")]
    pub bitcoin_cookie_path: Option<String>,

    /// Monero RPC URL
    #[arg(long, default_value = "http://127.0.0.1:18081/json_rpc")]
    pub monero_rpc_url: Option<String>,

    /// ASB RPC URL
    #[arg(long, default_value = "http://127.0.0.1:9944")]
    pub asb_rpc_url: Option<String>,

    /// Server listen address
    #[arg(long, default_value = "127.0.0.1")]
    pub host: Option<String>,

    /// Server listen port
    #[arg(long, default_value = "1235")]
    pub port: Option<u16>,

    /// SurrealDB endpoint
    #[arg(long, default_value = "127.0.0.1:8001")]
    pub db_endpoint: Option<String>,

    /// SurrealDB namespace
    #[arg(long, default_value = "eigenix")]
    pub db_namespace: Option<String>,

    /// SurrealDB database name
    #[arg(long, default_value = "metrics")]
    pub db_database: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub server: ServerConfig,
    pub database: DatabaseConfig,
    pub bitcoin: BitcoinConfig,
    pub monero: MoneroConfig,
    pub asb: AsbConfig,
    pub wallets: WalletsConfig,
    pub containers: ContainerConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseConfig {
    pub endpoint: String,
    pub namespace: String,
    pub database: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BitcoinConfig {
    pub rpc_url: String,
    pub cookie_path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MoneroConfig {
    pub rpc_url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AsbConfig {
    pub rpc_url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WalletsConfig {
    /// Bitcoin wallet name in Bitcoin Core
    pub bitcoin_wallet_name: String,
    /// Whether to rescan blockchain on first initialization
    pub bitcoin_rescan: bool,
    /// Monero wallet name in monero-wallet-rpc
    pub monero_wallet_name: String,
    /// Monero wallet password (empty string for no password)
    pub monero_wallet_password: String,
    /// Monero wallet RPC URL (for wallet operations, different from node RPC)
    pub monero_wallet_rpc_url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContainerConfig {
    pub names: Vec<String>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            server: ServerConfig {
                host: "0.0.0.0".to_string(),
                port: 1235,
            },
            database: DatabaseConfig {
                endpoint: "127.0.0.1:8001".to_string(),
                namespace: "eigenix".to_string(),
                database: "metrics".to_string(),
            },
            bitcoin: BitcoinConfig {
                rpc_url: "http://127.0.0.1:8332".to_string(),
                cookie_path: "/mnt/vault/bitcoind-data/.cookie".to_string(),
            },
            monero: MoneroConfig {
                rpc_url: "http://127.0.0.1:18081/json_rpc".to_string(),
            },
            asb: AsbConfig {
                rpc_url: "http://127.0.0.1:9944".to_string(),
            },
            wallets: WalletsConfig {
                bitcoin_wallet_name: "eigenix".to_string(),
                bitcoin_rescan: false,
                monero_wallet_name: "eigenix".to_string(),
                monero_wallet_password: "".to_string(),
                monero_wallet_rpc_url: "http://127.0.0.1:18082/json_rpc".to_string(),
            },
            containers: ContainerConfig {
                names: vec![
                    "bitcoind".to_string(),
                    "electrs".to_string(),
                    "monerod".to_string(),
                    "asb".to_string(),
                    "asb-controller".to_string(),
                ],
            },
        }
    }
}

impl Config {
    /// Convert to WalletConfig for wallet initialization
    pub fn to_wallet_config(&self) -> crate::wallets::WalletConfig {
        crate::wallets::WalletConfig {
            bitcoin_rpc_url: self.bitcoin.rpc_url.clone(),
            bitcoin_cookie_path: self.bitcoin.cookie_path.clone(),
            bitcoin_wallet_name: self.wallets.bitcoin_wallet_name.clone(),
            bitcoin_rescan: self.wallets.bitcoin_rescan,
            monero_rpc_url: self.wallets.monero_wallet_rpc_url.clone(),
            monero_wallet_name: self.wallets.monero_wallet_name.clone(),
            monero_wallet_password: self.wallets.monero_wallet_password.clone(),
            asb_rpc_url: self.asb.rpc_url.clone(),
        }
    }

    /// Load configuration from CLI arguments and optional config file
    pub fn load(cli: Cli) -> anyhow::Result<Self> {
        let mut config = if let Some(config_path) = &cli.config {
            // Load from config file
            let config_str = std::fs::read_to_string(config_path)?;
            toml::from_str(&config_str)?
        } else {
            // Use defaults
            Config::default()
        };

        // Override with CLI arguments
        if let Some(host) = cli.host {
            config.server.host = host;
        }
        if let Some(port) = cli.port {
            config.server.port = port;
        }
        if let Some(endpoint) = cli.db_endpoint {
            config.database.endpoint = endpoint;
        }
        if let Some(namespace) = cli.db_namespace {
            config.database.namespace = namespace;
        }
        if let Some(database) = cli.db_database {
            config.database.database = database;
        }
        if let Some(url) = cli.bitcoin_rpc_url {
            config.bitcoin.rpc_url = url;
        }
        if let Some(path) = cli.bitcoin_cookie_path {
            config.bitcoin.cookie_path = path;
        }
        if let Some(url) = cli.monero_rpc_url {
            config.monero.rpc_url = url;
        }
        if let Some(url) = cli.asb_rpc_url {
            config.asb.rpc_url = url;
        }

        Ok(config)
    }
}
