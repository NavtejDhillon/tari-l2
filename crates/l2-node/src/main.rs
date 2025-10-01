use tari_l2_node::{L2Node, NodeConfig};
use tracing::{info, error};
use tracing_subscriber;
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "tari-l2-node")]
#[command(about = "Tari L2 Marketplace Node", long_about = None)]
struct Cli {
    /// Path to configuration file
    #[arg(short, long, default_value = "config.toml")]
    config: String,

    /// Set log level (trace, debug, info, warn, error)
    #[arg(short, long, default_value = "info")]
    log_level: String,

    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Initialize a new node configuration
    Init {
        /// Output path for config file
        #[arg(short, long, default_value = "config.toml")]
        output: String,
    },
    /// Start the L2 node
    Start,
    /// Show node version and info
    Version,
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    // Set up logging
    let log_level = match cli.log_level.to_lowercase().as_str() {
        "trace" => tracing::Level::TRACE,
        "debug" => tracing::Level::DEBUG,
        "info" => tracing::Level::INFO,
        "warn" => tracing::Level::WARN,
        "error" => tracing::Level::ERROR,
        _ => tracing::Level::INFO,
    };

    tracing_subscriber::fmt()
        .with_max_level(log_level)
        .init();

    match &cli.command {
        Some(Commands::Init { output }) => {
            info!("Creating default configuration at: {}", output);
            let config = NodeConfig::default();
            if let Err(e) = config.save_to_file(output) {
                error!("Failed to save config: {}", e);
                std::process::exit(1);
            }
            info!("Configuration created successfully");
            info!("Edit {} and run 'tari-l2-node start' to begin", output);
        }
        Some(Commands::Version) => {
            println!("Tari L2 Marketplace Node");
            println!("Version: 0.1.0");
            println!("Network: Testnet");
        }
        Some(Commands::Start) | None => {
            info!("╔══════════════════════════════════════╗");
            info!("║   Tari L2 Marketplace Node v0.1.0   ║");
            info!("╚══════════════════════════════════════╝");

            // Load config
            let config = match NodeConfig::load_from_file(&cli.config) {
                Ok(config) => {
                    info!("✓ Loaded configuration from {}", cli.config);
                    config
                }
                Err(_) => {
                    info!("⚠ Config file not found, using defaults");
                    let config = NodeConfig::default();

                    if let Err(e) = config.save_to_file(&cli.config) {
                        error!("Failed to save default config: {}", e);
                    }

                    config
                }
            };

            // Validate config
            if let Err(e) = config.validate() {
                error!("Invalid configuration: {}", e);
                std::process::exit(1);
            }

            info!("");
            info!("Configuration:");
            info!("  • Data directory: {:?}", config.data_dir);
            info!("  • RPC server: {}:{}", config.rpc.listen_addr, config.rpc.port);
            info!("  • P2P listen: {}", config.network.listen_addr);
            info!("  • Tari node: {}:{}", config.tari_node.address, config.tari_node.port);
            info!("");

            // Create and start the node
            match L2Node::new(config).await {
                Ok(node) => {
                    info!("✓ Node initialized");
                    info!("✓ Public key: {}", node.public_key());
                    info!("");
                    info!("Starting services...");

                    if let Err(e) = node.start().await {
                        error!("Node error: {}", e);
                        std::process::exit(1);
                    }
                }
                Err(e) => {
                    error!("Failed to initialize node: {}", e);
                    std::process::exit(1);
                }
            }
        }
    }
}
