use clap::Parser;

#[derive(Parser, Debug)]
#[command(name = "eigenix")]
#[command(about = "Eigenix CLI", long_about = None)]
struct Args {
    /// Command to run
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Parser, Debug)]
enum Commands {
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
        Some(Commands::Server { port }) => {
            println!("Starting server on port {}", port);
            // TODO: Start the Axum server
            Ok(())
        }
        Some(Commands::Health) => {
            println!("Health check: OK");
            Ok(())
        }
        None => {
            println!("No command provided. Use --help for usage information.");
            Ok(())
        }
    }
}
