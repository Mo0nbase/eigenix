mod config;

use clap::{Parser, Subcommand};
use colored::Colorize;
use config::{config_exists, get_config_path, get_project_root, list_configs, AsbConfig};
use dialoguer::{Confirm, Input, Select};

#[derive(Parser, Debug)]
#[command(name = "eigenix")]
#[command(about = "Eigenix CLI - Manage ASB configurations and services", long_about = None)]
struct Args {
    /// Command to run
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Manage ASB configurations
    Config {
        #[command(subcommand)]
        action: ConfigAction,
    },
    /// Start the backend server
    Server {
        /// Port to bind to
        #[arg(short, long, default_value = "3000")]
        port: u16,
    },
    /// Run a health check
    Health,
}

#[derive(Subcommand, Debug)]
enum ConfigAction {
    /// Create a new ASB configuration
    New {
        /// Name of the configuration
        name: Option<String>,
        /// Template to use (mainnet or testnet)
        #[arg(short, long)]
        template: Option<String>,
        /// Create configuration interactively
        #[arg(short, long)]
        interactive: bool,
    },
    /// List all configurations
    List,
    /// Show configuration details
    Show {
        /// Name of the configuration
        name: String,
    },
    /// Validate a configuration
    Validate {
        /// Name of the configuration
        name: String,
    },
    /// Delete a configuration
    Delete {
        /// Name of the configuration
        name: String,
        /// Skip confirmation prompt
        #[arg(short, long)]
        yes: bool,
    },
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    match args.command {
        Commands::Config { action } => handle_config(action).await,
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

async fn handle_config(action: ConfigAction) -> anyhow::Result<()> {
    let project_root = get_project_root()?;

    match action {
        ConfigAction::New {
            name,
            template,
            interactive,
        } => {
            if interactive {
                create_config_interactive(&project_root).await?;
            } else {
                let config_name = name.ok_or_else(|| {
                    anyhow::anyhow!("Name is required. Use --interactive for guided setup.")
                })?;
                let template_name = template.unwrap_or_else(|| "mainnet".to_string());
                create_config(&project_root, config_name, &template_name).await?;
            }
        }
        ConfigAction::List => {
            list_configurations(&project_root)?;
        }
        ConfigAction::Show { name } => {
            show_configuration(&project_root, &name)?;
        }
        ConfigAction::Validate { name } => {
            validate_configuration(&project_root, &name)?;
        }
        ConfigAction::Delete { name, yes } => {
            delete_configuration(&project_root, &name, yes)?;
        }
    }

    Ok(())
}

async fn create_config_interactive(project_root: &std::path::Path) -> anyhow::Result<()> {
    println!("{}", "=== Create New ASB Configuration ===".bold().cyan());
    println!();

    let name: String = Input::new()
        .with_prompt("Configuration name")
        .validate_with(|input: &String| -> Result<(), String> {
            if input.is_empty() {
                return Err("Name cannot be empty".to_string());
            }
            if config_exists(project_root, input) {
                return Err(format!("Configuration '{}' already exists", input));
            }
            Ok(())
        })
        .interact_text()?;

    let templates = vec!["mainnet", "testnet"];
    let template_idx = Select::new()
        .with_prompt("Select template")
        .items(&templates)
        .default(0)
        .interact()?;
    let template = templates[template_idx];

    let mut config = AsbConfig::from_template(name.clone(), template);

    println!();
    println!("{}", "Network Configuration".bold());

    config.base_data_dir = Input::new()
        .with_prompt("Base data directory")
        .default(config.base_data_dir)
        .interact_text()?;

    let add_external_addr = Confirm::new()
        .with_prompt("Add external address for libp2p discovery?")
        .default(false)
        .interact()?;

    if add_external_addr {
        loop {
            let addr: String = Input::new()
                .with_prompt("External address (libp2p multiaddr)")
                .interact_text()?;
            config.external_addresses.push(addr);

            let add_more = Confirm::new()
                .with_prompt("Add another address?")
                .default(false)
                .interact()?;

            if !add_more {
                break;
            }
        }
    }

    config.enable_tor = Confirm::new()
        .with_prompt("Enable Tor hidden service?")
        .default(config.enable_tor)
        .interact()?;

    println!();
    println!("{}", "Maker Parameters".bold());

    config.min_buy_btc = Input::new()
        .with_prompt("Minimum BTC per swap")
        .default(config.min_buy_btc)
        .interact_text()?;

    config.max_buy_btc = Input::new()
        .with_prompt("Maximum BTC per swap")
        .default(config.max_buy_btc)
        .interact_text()?;

    config.ask_spread = Input::new()
        .with_prompt("Ask spread (e.g., 0.02 = 2%)")
        .default(config.ask_spread)
        .interact_text()?;

    config.developer_tip = Input::new()
        .with_prompt("Developer tip (0.0 to 1.0)")
        .default(config.developer_tip)
        .interact_text()?;

    let add_btc_addr = Confirm::new()
        .with_prompt("Set fixed external Bitcoin address?")
        .default(false)
        .interact()?;

    if add_btc_addr {
        let btc_addr: String = Input::new()
            .with_prompt("External Bitcoin address")
            .interact_text()?;
        config.external_bitcoin_address = Some(btc_addr);
    }

    let config_path = config.save(project_root)?;

    println!();
    println!("{}", "✓ Configuration created successfully!".green().bold());
    println!("  Path: {}", config_path.display().to_string().cyan());
    println!();
    println!("{}", "Next steps:".bold());
    println!(
        "  1. Review the configuration: {}",
        format!("eigenix config show {}", name).cyan()
    );
    println!("  2. Import in /etc/nixos/configuration.nix:");
    println!(
        "     {}",
        format!("imports = [ {} ];", config_path.display()).yellow()
    );
    println!("  3. Apply: {}", "sudo nixos-rebuild switch".cyan());

    Ok(())
}

async fn create_config(
    project_root: &std::path::Path,
    name: String,
    template: &str,
) -> anyhow::Result<()> {
    if config_exists(project_root, &name) {
        anyhow::bail!("Configuration '{}' already exists", name);
    }

    if template != "mainnet" && template != "testnet" {
        anyhow::bail!(
            "Invalid template '{}'. Use 'mainnet' or 'testnet'",
            template
        );
    }

    let config = AsbConfig::from_template(name.clone(), template);
    let config_path = config.save(project_root)?;

    println!("{}", "✓ Configuration created successfully!".green());
    println!("  Path: {}", config_path.display().to_string().cyan());
    println!();
    println!("Edit the configuration and add your external addresses:");
    println!("  {}", format!("$EDITOR {}", config_path.display()).cyan());

    Ok(())
}

fn list_configurations(project_root: &std::path::Path) -> anyhow::Result<()> {
    let configs = list_configs(project_root)?;

    if configs.is_empty() {
        println!("{}", "No configurations found.".yellow());
        println!();
        println!("Create a new configuration:");
        println!(
            "  {}",
            "eigenix config new <name> --template mainnet".cyan()
        );
        println!("  {}", "eigenix config new --interactive".cyan());
        return Ok(());
    }

    println!("{}", "ASB Configurations:".bold());
    println!();
    for config in configs {
        println!("  • {}", config.cyan());
    }
    println!();
    println!("View details: {}", "eigenix config show <name>".dimmed());

    Ok(())
}

fn show_configuration(project_root: &std::path::Path, name: &str) -> anyhow::Result<()> {
    let config_path = get_config_path(project_root, name);

    if !config_path.exists() {
        anyhow::bail!("Configuration '{}' not found", name);
    }

    let content = std::fs::read_to_string(&config_path)?;
    println!("{}", format!("Configuration: {}", name).bold().cyan());
    println!("{}", format!("Path: {}", config_path.display()).dimmed());
    println!();
    println!("{}", content);

    Ok(())
}

fn validate_configuration(project_root: &std::path::Path, name: &str) -> anyhow::Result<()> {
    let config_path = get_config_path(project_root, name);

    if !config_path.exists() {
        anyhow::bail!("Configuration '{}' not found", name);
    }

    // Basic validation - check if file is readable and not empty
    let content = std::fs::read_to_string(&config_path)?;

    if content.is_empty() {
        anyhow::bail!("Configuration file is empty");
    }

    // Check for common issues
    let mut warnings = vec![];

    if !content.contains("services.asb") {
        warnings.push("Missing services.asb configuration");
    }

    if content.contains("your-domain.com") || content.contains("YOUR.PUBLIC.IP.ADDRESS") {
        warnings.push("Contains placeholder external addresses - update before deploying");
    }

    if warnings.is_empty() {
        println!("{}", format!("✓ Configuration '{}' is valid", name).green());
    } else {
        println!(
            "{}",
            format!("⚠ Configuration '{}' has warnings:", name).yellow()
        );
        for warning in warnings {
            println!("  • {}", warning);
        }
    }

    Ok(())
}

fn delete_configuration(
    project_root: &std::path::Path,
    name: &str,
    skip_confirm: bool,
) -> anyhow::Result<()> {
    let config_path = get_config_path(project_root, name);

    if !config_path.exists() {
        anyhow::bail!("Configuration '{}' not found", name);
    }

    if !skip_confirm {
        let confirm = Confirm::new()
            .with_prompt(format!("Delete configuration '{}'?", name))
            .default(false)
            .interact()?;

        if !confirm {
            println!("Cancelled.");
            return Ok(());
        }
    }

    std::fs::remove_file(&config_path)?;
    println!("{}", format!("✓ Configuration '{}' deleted", name).green());

    Ok(())
}
