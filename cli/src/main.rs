mod config;

use clap::{Parser, Subcommand};
use colored::Colorize;
use config::{get_parameters_path, get_project_root, parameters_exist, DeploymentConfig};
use dialoguer::{Confirm, Input, Select};

#[derive(Parser, Debug)]
#[command(name = "eigenix")]
#[command(about = "Eigenix CLI - Manage deployment configurations and services", long_about = None)]
struct Args {
    /// Command to run
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Initialize deployment configuration
    Init {
        /// Template to use (mainnet or testnet)
        #[arg(short, long)]
        template: Option<String>,
        /// Skip interactive configuration
        #[arg(short, long)]
        yes: bool,
    },
    /// Configure deployment parameters interactively
    Configure {
        /// Configuration section to edit
        section: Option<String>,
    },
    /// Show current configuration
    Show {
        /// Show specific section only
        section: Option<String>,
    },
    /// Validate configuration
    Validate,
    /// Start the backend server
    Server {
        /// Port to bind to
        #[arg(short, long, default_value = "3000")]
        port: u16,
    },
    /// Run a health check
    Health,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    match args.command {
        Commands::Init { template, yes } => init_configuration(template.as_deref(), yes).await,
        Commands::Configure { section } => configure_deployment(section.as_deref()).await,
        Commands::Show { section } => show_configuration(section.as_deref()).await,
        Commands::Validate => validate_configuration().await,
        Commands::Server { port } => {
            println!("Starting server on port {}", port);
            // TODO: Start the Axum server
            Ok(())
        }
        Commands::Health => {
            println!("{}", "Health check: OK".green());
            Ok(())
        }
    }
}

async fn init_configuration(template: Option<&str>, skip_interactive: bool) -> anyhow::Result<()> {
    let project_root = get_project_root()?;
    let params_path = get_parameters_path(&project_root);

    if parameters_exist(&project_root) {
        let overwrite = Confirm::new()
            .with_prompt("Configuration already exists. Overwrite?")
            .default(false)
            .interact()?;

        if !overwrite {
            println!("Cancelled.");
            return Ok(());
        }
    }

    let template_name = template.unwrap_or("mainnet");
    let mut config = DeploymentConfig::from_template(template_name);

    if !skip_interactive {
        println!(
            "{}",
            "=== Eigenix Deployment Configuration ===".bold().cyan()
        );
        println!();

        // Deployment metadata
        println!("{}", "Deployment Metadata".bold());
        config.deployment.name = Input::new()
            .with_prompt("Deployment name")
            .default(config.deployment.name)
            .interact_text()?;

        let environments = vec!["production", "staging", "development", "testnet"];
        let env_idx = Select::new()
            .with_prompt("Environment")
            .items(&environments)
            .default(0)
            .interact()?;
        config.deployment.environment = environments[env_idx].to_string();

        let add_domain = Confirm::new()
            .with_prompt("Set a domain for web services?")
            .default(false)
            .interact()?;

        if add_domain {
            let domain: String = Input::new().with_prompt("Domain name").interact_text()?;
            config.deployment.domain = Some(domain);
        }

        println!();
        println!("{}", "Storage Configuration".bold());
        config.storage.base_data_dir = Input::new()
            .with_prompt("Base data directory")
            .default(config.storage.base_data_dir)
            .interact_text()?;

        println!();
        println!("{}", "Network Configuration".bold());
        let bitcoin_networks = vec!["Mainnet", "Testnet", "Signet", "Regtest"];
        let btc_idx = Select::new()
            .with_prompt("Bitcoin network")
            .items(&bitcoin_networks)
            .default(0)
            .interact()?;
        config.networks.bitcoin = bitcoin_networks[btc_idx].to_string();

        let monero_networks = vec!["Mainnet", "Stagenet", "Testnet"];
        let xmr_idx = Select::new()
            .with_prompt("Monero network")
            .items(&monero_networks)
            .default(0)
            .interact()?;
        config.networks.monero = monero_networks[xmr_idx].to_string();

        println!();
        println!("{}", "Service Components".bold());
        config.asb.enable = Confirm::new()
            .with_prompt("Enable ASB (Automated Swap Backend)?")
            .default(config.asb.enable)
            .interact()?;

        config.mempool.enable = Confirm::new()
            .with_prompt("Enable Mempool Explorer?")
            .default(config.mempool.enable)
            .interact()?;

        config.backend.enable = Confirm::new()
            .with_prompt("Enable Backend API?")
            .default(config.backend.enable)
            .interact()?;

        config.web.enable = Confirm::new()
            .with_prompt("Enable Web Frontend?")
            .default(config.web.enable)
            .interact()?;

        if config.asb.enable {
            println!();
            println!("{}", "ASB Configuration".bold());

            let add_external_addr = Confirm::new()
                .with_prompt("Add external address for libp2p discovery?")
                .default(false)
                .interact()?;

            if add_external_addr {
                loop {
                    let addr: String = Input::new()
                        .with_prompt("External address (libp2p multiaddr)")
                        .interact_text()?;
                    config.asb.external_addresses.push(addr);

                    let add_more = Confirm::new()
                        .with_prompt("Add another address?")
                        .default(false)
                        .interact()?;

                    if !add_more {
                        break;
                    }
                }
            }

            config.asb.enable_tor = Confirm::new()
                .with_prompt("Enable Tor hidden service?")
                .default(config.asb.enable_tor)
                .interact()?;

            config.asb.min_buy_btc = Input::new()
                .with_prompt("Minimum BTC per swap")
                .default(config.asb.min_buy_btc)
                .interact_text()?;

            config.asb.max_buy_btc = Input::new()
                .with_prompt("Maximum BTC per swap")
                .default(config.asb.max_buy_btc)
                .interact_text()?;

            config.asb.ask_spread = Input::new()
                .with_prompt("Ask spread (e.g., 0.02 = 2%)")
                .default(config.asb.ask_spread)
                .interact_text()?;

            let add_btc_addr = Confirm::new()
                .with_prompt("Set fixed external Bitcoin address?")
                .default(false)
                .interact()?;

            if add_btc_addr {
                let btc_addr: String = Input::new()
                    .with_prompt("External Bitcoin address")
                    .interact_text()?;
                config.asb.external_bitcoin_address = Some(btc_addr);
            }
        }
    }

    // Ensure instances directory exists
    if let Some(parent) = params_path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    config.save(&params_path)?;

    println!();
    println!(
        "{}",
        "✓ Configuration initialized successfully!".green().bold()
    );
    println!("  Path: {}", params_path.display().to_string().cyan());
    println!();
    println!("{}", "Next steps:".bold());
    println!("  1. Review configuration: {}", "eigenix show".cyan());
    println!(
        "  2. Configure additional settings: {}",
        "eigenix configure".cyan()
    );
    println!("  3. Validate configuration: {}", "eigenix validate".cyan());
    println!("  4. Import in /etc/nixos/configuration.nix:");
    println!(
        "     {}",
        format!("imports = [ {}/nix/module.nix ];", project_root.display()).yellow()
    );
    println!("     {}", "services.eigenix.enable = true;".yellow());
    println!("  5. Apply: {}", "sudo nixos-rebuild switch".cyan());

    Ok(())
}

async fn configure_deployment(section: Option<&str>) -> anyhow::Result<()> {
    let project_root = get_project_root()?;
    let params_path = get_parameters_path(&project_root);

    if !parameters_exist(&project_root) {
        println!(
            "{}",
            "No configuration found. Run 'eigenix init' first.".yellow()
        );
        return Ok(());
    }

    let mut config = DeploymentConfig::load(&params_path)?;

    let sections = vec![
        "deployment",
        "storage",
        "networks",
        "asb",
        "bitcoin",
        "monero",
        "mempool",
        "backend",
        "web",
        "ports",
    ];

    let section_name = match section {
        Some(s) => s,
        None => {
            let idx = Select::new()
                .with_prompt("Select section to configure")
                .items(&sections)
                .interact()?;
            sections[idx]
        }
    };

    println!();
    println!("{}", format!("Configuring: {}", section_name).bold().cyan());
    println!();

    match section_name {
        "deployment" => {
            config.deployment.name = Input::new()
                .with_prompt("Deployment name")
                .default(config.deployment.name)
                .interact_text()?;

            let environments = vec!["production", "staging", "development", "testnet"];
            let current_idx = environments
                .iter()
                .position(|&e| e == config.deployment.environment)
                .unwrap_or(0);
            let env_idx = Select::new()
                .with_prompt("Environment")
                .items(&environments)
                .default(current_idx)
                .interact()?;
            config.deployment.environment = environments[env_idx].to_string();

            let has_domain = config.deployment.domain.is_some();
            let set_domain = Confirm::new()
                .with_prompt("Set domain?")
                .default(has_domain)
                .interact()?;

            if set_domain {
                let domain: String = Input::new()
                    .with_prompt("Domain name")
                    .default(config.deployment.domain.unwrap_or_default())
                    .interact_text()?;
                config.deployment.domain = Some(domain);
            } else {
                config.deployment.domain = None;
            }
        }
        "storage" => {
            config.storage.base_data_dir = Input::new()
                .with_prompt("Base data directory")
                .default(config.storage.base_data_dir)
                .interact_text()?;
        }
        "asb" => {
            config.asb.enable = Confirm::new()
                .with_prompt("Enable ASB?")
                .default(config.asb.enable)
                .interact()?;

            if config.asb.enable {
                config.asb.enable_tor = Confirm::new()
                    .with_prompt("Enable Tor?")
                    .default(config.asb.enable_tor)
                    .interact()?;

                config.asb.min_buy_btc = Input::new()
                    .with_prompt("Minimum BTC per swap")
                    .default(config.asb.min_buy_btc)
                    .interact_text()?;

                config.asb.max_buy_btc = Input::new()
                    .with_prompt("Maximum BTC per swap")
                    .default(config.asb.max_buy_btc)
                    .interact_text()?;

                config.asb.ask_spread = Input::new()
                    .with_prompt("Ask spread (0.02 = 2%)")
                    .default(config.asb.ask_spread)
                    .interact_text()?;
            }
        }
        "mempool" => {
            config.mempool.enable = Confirm::new()
                .with_prompt("Enable Mempool Explorer?")
                .default(config.mempool.enable)
                .interact()?;

            if config.mempool.enable {
                config.mempool.enable_statistics = Confirm::new()
                    .with_prompt("Enable statistics collection?")
                    .default(config.mempool.enable_statistics)
                    .interact()?;
            }
        }
        "backend" => {
            config.backend.enable = Confirm::new()
                .with_prompt("Enable Backend API?")
                .default(config.backend.enable)
                .interact()?;

            if config.backend.enable {
                config.backend.host = Input::new()
                    .with_prompt("Bind host (127.0.0.1 for localhost only)")
                    .default(config.backend.host)
                    .interact_text()?;
            }
        }
        "web" => {
            config.web.enable = Confirm::new()
                .with_prompt("Enable Web Frontend?")
                .default(config.web.enable)
                .interact()?;

            if config.web.enable {
                config.web.host = Input::new()
                    .with_prompt("Bind host")
                    .default(config.web.host)
                    .interact_text()?;
            }
        }
        "ports" => {
            println!("Configure ports (press Enter to keep current value):");
            config.ports.asb_p2p = Input::new()
                .with_prompt("ASB P2P port")
                .default(config.ports.asb_p2p)
                .interact_text()?;
            config.ports.asb_rpc = Input::new()
                .with_prompt("ASB RPC port")
                .default(config.ports.asb_rpc)
                .interact_text()?;
            config.ports.mempool_web = Input::new()
                .with_prompt("Mempool web port")
                .default(config.ports.mempool_web)
                .interact_text()?;
            config.ports.eigenix_web = Input::new()
                .with_prompt("Eigenix web port")
                .default(config.ports.eigenix_web)
                .interact_text()?;
            config.ports.eigenix_backend = Input::new()
                .with_prompt("Eigenix backend port")
                .default(config.ports.eigenix_backend)
                .interact_text()?;
        }
        _ => {
            println!(
                "{}",
                format!(
                    "Section '{}' configuration not yet implemented",
                    section_name
                )
                .yellow()
            );
            return Ok(());
        }
    }

    config.save(&params_path)?;

    println!();
    println!("{}", "✓ Configuration updated successfully!".green().bold());
    println!("  Run {} to see changes", "eigenix show".cyan());

    Ok(())
}

async fn show_configuration(section: Option<&str>) -> anyhow::Result<()> {
    let project_root = get_project_root()?;
    let params_path = get_parameters_path(&project_root);

    if !parameters_exist(&project_root) {
        println!(
            "{}",
            "No configuration found. Run 'eigenix init' first.".yellow()
        );
        return Ok(());
    }

    let config = DeploymentConfig::load(&params_path)?;

    println!(
        "{}",
        "=== Eigenix Deployment Configuration ===".bold().cyan()
    );
    println!("Path: {}", params_path.display().to_string().dimmed());
    println!();

    if section.is_none() {
        // Show overview
        println!("{}", "Deployment:".bold());
        println!("  Name: {}", config.deployment.name.cyan());
        println!("  Environment: {}", config.deployment.environment.cyan());
        if let Some(domain) = &config.deployment.domain {
            println!("  Domain: {}", domain.cyan());
        }
        println!();

        println!("{}", "Storage:".bold());
        println!("  Base directory: {}", config.storage.base_data_dir.cyan());
        println!();

        println!("{}", "Networks:".bold());
        println!("  Bitcoin: {}", config.networks.bitcoin.cyan());
        println!("  Monero: {}", config.networks.monero.cyan());
        println!();

        println!("{}", "Enabled Services:".bold());
        println!(
            "  ASB: {}",
            if config.asb.enable {
                "✓".green()
            } else {
                "✗".red()
            }
        );
        println!(
            "  Mempool: {}",
            if config.mempool.enable {
                "✓".green()
            } else {
                "✗".red()
            }
        );
        println!(
            "  Backend: {}",
            if config.backend.enable {
                "✓".green()
            } else {
                "✗".red()
            }
        );
        println!(
            "  Web: {}",
            if config.web.enable {
                "✓".green()
            } else {
                "✗".red()
            }
        );
        println!();

        if config.asb.enable {
            println!("{}", "ASB Configuration:".bold());
            println!(
                "  Min swap: {} BTC",
                config.asb.min_buy_btc.to_string().cyan()
            );
            println!(
                "  Max swap: {} BTC",
                config.asb.max_buy_btc.to_string().cyan()
            );
            println!(
                "  Spread: {}%",
                (config.asb.ask_spread * 100.0).to_string().cyan()
            );
            println!(
                "  Tor: {}",
                if config.asb.enable_tor {
                    "enabled".green()
                } else {
                    "disabled".red()
                }
            );
            if !config.asb.external_addresses.is_empty() {
                println!(
                    "  External addresses: {}",
                    config.asb.external_addresses.len()
                );
            }
            println!();
        }

        println!("{}", "Ports:".bold());
        println!("  ASB P2P: {}", config.ports.asb_p2p.to_string().cyan());
        println!("  ASB RPC: {}", config.ports.asb_rpc.to_string().cyan());
        if config.mempool.enable {
            println!(
                "  Mempool Web: {}",
                config.ports.mempool_web.to_string().cyan()
            );
        }
        if config.web.enable {
            println!(
                "  Eigenix Web: {}",
                config.ports.eigenix_web.to_string().cyan()
            );
        }
        if config.backend.enable {
            println!(
                "  Eigenix Backend: {}",
                config.ports.eigenix_backend.to_string().cyan()
            );
        }
        println!();

        println!(
            "Use {} to see full configuration in JSON",
            "eigenix show <section>".dimmed()
        );
    } else {
        // Show full JSON
        let json = serde_json::to_string_pretty(&config)?;
        println!("{}", json);
    }

    Ok(())
}

async fn validate_configuration() -> anyhow::Result<()> {
    let project_root = get_project_root()?;
    let params_path = get_parameters_path(&project_root);

    if !parameters_exist(&project_root) {
        println!(
            "{}",
            "✗ No configuration found. Run 'eigenix init' first.".red()
        );
        return Ok(());
    }

    let config = DeploymentConfig::load(&params_path)?;
    let mut errors = vec![];
    let mut warnings = vec![];

    // Validation checks
    if config.deployment.name.is_empty() {
        errors.push("Deployment name cannot be empty");
    }

    if config.asb.enable {
        if config.asb.min_buy_btc >= config.asb.max_buy_btc {
            errors.push("ASB min_buy_btc must be less than max_buy_btc");
        }
        if config.asb.ask_spread < 0.0 {
            errors.push("ASB ask_spread cannot be negative");
        }
        if config.asb.external_addresses.is_empty() {
            warnings.push("ASB has no external addresses configured - may not be discoverable");
        }
    }

    // Port conflict checks
    let ports = vec![
        config.ports.asb_p2p,
        config.ports.asb_rpc,
        config.ports.mempool_web,
        config.ports.eigenix_web,
        config.ports.eigenix_backend,
    ];
    let unique_ports: std::collections::HashSet<_> = ports.iter().collect();
    if ports.len() != unique_ports.len() {
        errors.push("Port conflict detected - some ports are assigned to multiple services");
    }

    // Display results
    if errors.is_empty() && warnings.is_empty() {
        println!("{}", "✓ Configuration is valid!".green().bold());
    } else {
        if !errors.is_empty() {
            println!("{}", "Errors:".red().bold());
            for error in errors {
                println!("  ✗ {}", error.red());
            }
        }
        if !warnings.is_empty() {
            println!("{}", "Warnings:".yellow().bold());
            for warning in warnings {
                println!("  ⚠ {}", warning.yellow());
            }
        }
    }

    Ok(())
}
