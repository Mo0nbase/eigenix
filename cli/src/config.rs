use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

/// Full deployment configuration matching parameters.json schema
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DeploymentConfig {
    pub deployment: DeploymentMeta,
    pub storage: StorageConfig,
    pub networks: NetworkConfig,
    pub asb: AsbConfig,
    pub bitcoin: BitcoinConfig,
    pub monero: MoneroConfig,
    pub electrs: ElectrsConfig,
    pub mempool: MempoolConfig,
    pub backend: BackendConfig,
    pub web: WebConfig,
    pub ports: PortsConfig,
    pub resources: ResourcesConfig,
    pub images: ImagesConfig,
    pub monitoring: MonitoringConfig,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DeploymentMeta {
    pub name: String,
    pub environment: String,
    pub domain: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct StorageConfig {
    pub base_data_dir: String,
    pub bitcoind_data_dir: String,
    pub monerod_data_dir: String,
    pub asb_data_dir: String,
    pub mempool_data_dir: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct NetworkConfig {
    pub bitcoin: String,
    pub monero: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct AsbConfig {
    pub enable: bool,
    pub external_addresses: Vec<String>,
    pub enable_tor: bool,
    pub tor_intro_points: u32,
    pub min_buy_btc: f64,
    pub max_buy_btc: f64,
    pub ask_spread: f64,
    pub price_ticker_url: String,
    pub external_bitcoin_address: Option<String>,
    pub developer_tip: f64,
    pub rendezvous_points: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct BitcoinConfig {
    pub enable: bool,
    pub prune: bool,
    pub txindex: bool,
    pub dbcache: u32,
    pub maxconnections: u32,
    pub rpcallowip: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct MoneroConfig {
    pub enable: bool,
    pub restricted_rpc: bool,
    pub enable_dns_blocklist: bool,
    pub max_incoming_connections: u32,
    pub max_outgoing_connections: u32,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ElectrsConfig {
    pub enable: bool,
    pub log_filters: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct MempoolConfig {
    pub enable: bool,
    pub enable_statistics: bool,
    pub backend: String,
    pub database: MempoolDatabaseConfig,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct MempoolDatabaseConfig {
    pub name: String,
    pub user: String,
    pub password: String,
    pub root_password: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct BackendConfig {
    pub enable: bool,
    pub host: String,
    pub log_level: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct WebConfig {
    pub enable: bool,
    pub host: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct PortsConfig {
    pub asb_p2p: u16,
    pub asb_rpc: u16,
    pub bitcoin_rpc: u16,
    pub bitcoin_p2p: u16,
    pub electrs: u16,
    pub monero_rpc: u16,
    pub monero_p2p: u16,
    pub mempool_web: u16,
    pub mempool_api: u16,
    pub eigenix_web: u16,
    pub eigenix_backend: u16,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ResourcesConfig {
    pub bitcoind: ResourceLimit,
    pub monerod: ResourceLimit,
    pub electrs: ResourceLimit,
    pub asb: ResourceLimit,
    #[serde(rename = "mempoolDb")]
    pub mempool_db: ResourceLimit,
    #[serde(rename = "mempoolApi")]
    pub mempool_api: ResourceLimit,
    #[serde(rename = "mempoolWeb")]
    pub mempool_web: ResourceLimit,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ResourceLimit {
    pub memory: String,
    pub cpus: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ImagesConfig {
    pub asb: String,
    pub asb_controller: String,
    pub bitcoind: String,
    pub monerod: String,
    pub electrs: String,
    pub mempool_db: String,
    pub mempool_web: String,
    pub mempool_api: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct MonitoringConfig {
    pub enable_metrics: bool,
    pub enable_tracing: bool,
    pub log_retention_days: u32,
}

impl Default for DeploymentConfig {
    fn default() -> Self {
        Self::mainnet()
    }
}

impl DeploymentConfig {
    /// Load configuration from JSON file
    pub fn load(path: &Path) -> Result<Self> {
        let content = fs::read_to_string(path)
            .context(format!("Failed to read config file: {}", path.display()))?;
        let config: Self =
            serde_json::from_str(&content).context("Failed to parse configuration JSON")?;
        Ok(config)
    }

    /// Save configuration to JSON file
    pub fn save(&self, path: &Path) -> Result<()> {
        let json =
            serde_json::to_string_pretty(self).context("Failed to serialize configuration")?;
        fs::write(path, json)
            .context(format!("Failed to write config file: {}", path.display()))?;
        Ok(())
    }

    /// Create mainnet configuration template
    pub fn mainnet() -> Self {
        Self {
            deployment: DeploymentMeta {
                name: "eigenix".to_string(),
                environment: "production".to_string(),
                domain: None,
            },
            storage: StorageConfig {
                base_data_dir: "/mnt/vault".to_string(),
                bitcoind_data_dir: "/mnt/vault/bitcoind-data".to_string(),
                monerod_data_dir: "/mnt/vault/monerod-data".to_string(),
                asb_data_dir: "/mnt/vault/asb-data".to_string(),
                mempool_data_dir: "/mnt/vault/mempool".to_string(),
            },
            networks: NetworkConfig {
                bitcoin: "Mainnet".to_string(),
                monero: "Mainnet".to_string(),
            },
            asb: AsbConfig {
                enable: true,
                external_addresses: vec![],
                enable_tor: true,
                tor_intro_points: 5,
                min_buy_btc: 0.002,
                max_buy_btc: 0.02,
                ask_spread: 0.02,
                price_ticker_url: "wss://ws.kraken.com/".to_string(),
                external_bitcoin_address: None,
                developer_tip: 0.0,
                rendezvous_points: vec![
                    "/dns4/discover.unstoppableswap.net/tcp/8888/p2p/12D3KooWA6cnqJpVnreBVnoro8midDL9Lpzmg8oJPoAGi7YYaamE".to_string(),
                    "/dns4/discover2.unstoppableswap.net/tcp/8888/p2p/12D3KooWGRvf7qVQDrNR5nfYD6rKrbgeTi9x8RrbdxbmsPvxL4mw".to_string(),
                    "/dns4/darkness.su/tcp/8888/p2p/12D3KooWFQAgVVS9t9UgL6v1sLprJVM7am5hFK7vy9iBCCoCBYmU".to_string(),
                    "/dns4/eigen.center/tcp/8888/p2p/12D3KooWS5RaYJt4ANKMH4zczGVhNcw5W214e2DDYXnjs5Mx5zAT".to_string(),
                    "/dns4/swapanarchy.cfd/tcp/8888/p2p/12D3KooWRtyVpmyvwzPYXuWyakFbRKhyXGrjhq6tP7RrBofpgQGp".to_string(),
                    "/dns4/rendezvous.observer/tcp/8888/p2p/12D3KooWMjceGXrYuGuDMGrfmJxALnSDbK4km6s1i1sJEgDTgGQa".to_string(),
                    "/dns4/aswap.click/tcp/8888/p2p/12D3KooWQzW52mdsLHTMu1EPiz3APumG6vGwpCuyy494MAQoEa5X".to_string(),
                    "/dns4/getxmr.st/tcp/8888/p2p/12D3KooWHHwiz6WDThPT8cEurstomg3kDSxzL2L8pwxfyX2fpxVk".to_string(),
                ],
            },
            bitcoin: BitcoinConfig {
                enable: true,
                prune: false,
                txindex: true,
                dbcache: 16384,
                maxconnections: 125,
                rpcallowip: vec!["127.0.0.1".to_string(), "10.89.0.0/24".to_string()],
            },
            monero: MoneroConfig {
                enable: true,
                restricted_rpc: true,
                enable_dns_blocklist: true,
                max_incoming_connections: 64,
                max_outgoing_connections: 64,
            },
            electrs: ElectrsConfig {
                enable: true,
                log_filters: "INFO".to_string(),
            },
            mempool: MempoolConfig {
                enable: true,
                enable_statistics: true,
                backend: "electrum".to_string(),
                database: MempoolDatabaseConfig {
                    name: "mempool".to_string(),
                    user: "mempool".to_string(),
                    password: "mempool".to_string(),
                    root_password: "admin".to_string(),
                },
            },
            backend: BackendConfig {
                enable: false,
                host: "127.0.0.1".to_string(),
                log_level: "info".to_string(),
            },
            web: WebConfig {
                enable: false,
                host: "0.0.0.0".to_string(),
            },
            ports: PortsConfig {
                asb_p2p: 9939,
                asb_rpc: 9944,
                bitcoin_rpc: 8332,
                bitcoin_p2p: 8333,
                electrs: 50001,
                monero_rpc: 18081,
                monero_p2p: 18080,
                mempool_web: 8999,
                mempool_api: 8998,
                eigenix_web: 8080,
                eigenix_backend: 3000,
            },
            resources: ResourcesConfig {
                bitcoind: ResourceLimit {
                    memory: "8g".to_string(),
                    cpus: "4.0".to_string(),
                },
                monerod: ResourceLimit {
                    memory: "4g".to_string(),
                    cpus: "2.0".to_string(),
                },
                electrs: ResourceLimit {
                    memory: "4g".to_string(),
                    cpus: "2.0".to_string(),
                },
                asb: ResourceLimit {
                    memory: "2g".to_string(),
                    cpus: "2.0".to_string(),
                },
                mempool_db: ResourceLimit {
                    memory: "2g".to_string(),
                    cpus: "2.0".to_string(),
                },
                mempool_api: ResourceLimit {
                    memory: "2g".to_string(),
                    cpus: "2.0".to_string(),
                },
                mempool_web: ResourceLimit {
                    memory: "512m".to_string(),
                    cpus: "1.0".to_string(),
                },
            },
            images: ImagesConfig {
                asb: "ghcr.io/eigenwallet/asb:latest".to_string(),
                asb_controller: "ghcr.io/eigenwallet/asb-controller:latest".to_string(),
                bitcoind: "getumbrel/bitcoind@sha256:c565266ea302c9ab2fc490f04ff14e584210cde3d0d991b8309157e5dfae9e8d".to_string(),
                monerod: "ghcr.io/sethforprivacy/simple-monerod@sha256:f30e5706a335c384e4cf420215cbffd1196f0b3a11d4dd4e819fe3e0bca41ec5".to_string(),
                electrs: "getumbrel/electrs@sha256:622657fbdc7331a69f5b3444e6f87867d51ac27d90c399c8bf25d9aab020052b".to_string(),
                mempool_db: "mariadb@sha256:9e7695800ab8fa72d75053fe536b090d0c9373465b32a073c73bc7940a2e8dbe".to_string(),
                mempool_web: "mempool/frontend@sha256:1f33796b56bb661ac7b417d11d6c44c467f51c808ea3c48748a2428e1bed918c".to_string(),
                mempool_api: "mempool/backend@sha256:edc4cc7b27b8d6267abb74f76b80dd0258803377d7a60fd1d050c4786369d15a".to_string(),
            },
            monitoring: MonitoringConfig {
                enable_metrics: false,
                enable_tracing: true,
                log_retention_days: 30,
            },
        }
    }

    /// Create testnet configuration template
    pub fn testnet() -> Self {
        let mut config = Self::mainnet();
        config.deployment.name = "eigenix-testnet".to_string();
        config.deployment.environment = "testnet".to_string();
        config.storage.base_data_dir = "/mnt/vault-testnet".to_string();
        config.storage.bitcoind_data_dir = "/mnt/vault-testnet/bitcoind-data".to_string();
        config.storage.monerod_data_dir = "/mnt/vault-testnet/monerod-data".to_string();
        config.storage.asb_data_dir = "/mnt/vault-testnet/asb-data".to_string();
        config.storage.mempool_data_dir = "/mnt/vault-testnet/mempool".to_string();
        config.networks.bitcoin = "Testnet".to_string();
        config.networks.monero = "Stagenet".to_string();
        config.asb.enable_tor = false;
        config.asb.min_buy_btc = 0.001;
        config.asb.max_buy_btc = 0.1;
        config.asb.ask_spread = 0.01;
        config
    }

    /// Get from template name
    pub fn from_template(template: &str) -> Self {
        match template {
            "testnet" => Self::testnet(),
            _ => Self::mainnet(),
        }
    }
}

pub fn get_project_root() -> Result<PathBuf> {
    // Start from current directory and walk up to find the project root
    let mut current = std::env::current_dir()?;

    loop {
        // Check if we're in the eigenix project root
        if current.join("nix").exists() && current.join("flake.nix").exists() {
            return Ok(current);
        }

        // Try parent directory
        if let Some(parent) = current.parent() {
            current = parent.to_path_buf();
        } else {
            anyhow::bail!("Could not find eigenix project root. Make sure you're running this command from within the eigenix project directory.");
        }
    }
}

pub fn get_parameters_path(base_path: &Path) -> PathBuf {
    base_path.join("nix").join("settings.json")
}

pub fn parameters_exist(base_path: &Path) -> bool {
    get_parameters_path(base_path).exists()
}
