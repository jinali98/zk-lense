use clap::{Parser, Subcommand};

mod commands;
mod ui;

#[derive(Parser)]
#[command(name = "zklense", version, about = "ZK Profiling Tool")]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    Version,

    #[command(name = "init")]
    Initialize {
        path: Option<String>,
    },
    #[command(name = "view")]
    View {
        path: Option<String>,
    },
    #[command(name = "simulate")]
    Simulate {
        /// Program ID to simulate against
        #[arg(short, long)]
        program_id: Option<String>,
    },
    #[command(name = "run")]
    Run {
        path: Option<String>,
    },
    #[command(name = "generate")]
    Generate {
        /// Name of the new Noir project
        #[arg(short, long)]
        name: Option<String>,

        /// Template to use (age_verifier, merkle_inclusion, hash_preimage, range_proof or none)
        #[arg(short, long)]
        template: Option<String>,
    },
    /// Manage zklense configuration
    #[command(name = "config")]
    Config {
        #[command(subcommand)]
        action: ConfigCommands,
    },
}

#[derive(Subcommand)]
enum ConfigCommands {
    /// Show all configuration values
    #[command(name = "show")]
    Show { path: Option<String> },
    /// Get the current Solana network
    #[command(name = "get-network")]
    GetNetwork { path: Option<String> },
    /// Set the Solana network (devnet, testnet, or mainnet)
    #[command(name = "set-network")]
    SetNetwork {
        /// Network to use: devnet, testnet, or mainnet
        network: String,
        path: Option<String>,
    },
    /// List all available Solana networks
    #[command(name = "list-networks")]
    ListNetworks { path: Option<String> },
    /// Get the current Solana RPC URL
    #[command(name = "get-rpc")]
    GetRpc { path: Option<String> },
    /// Set a custom Solana RPC URL
    #[command(name = "set-rpc")]
    SetRpc {
        /// Custom RPC URL (e.g., https://my-rpc.example.com)
        rpc_url: String,
        path: Option<String>,
    },
    /// Reset the RPC URL to the default for the current network
    #[command(name = "reset-rpc")]
    ResetRpc { path: Option<String> },
}

/// Check if the project is initialized, prompting the user if not.
/// Returns true if we should proceed, false otherwise.
fn check_initialized(path: Option<&str>) -> bool {
    match commands::ensure_initialized(path) {
        Ok(true) => true,
        Ok(false) => false, // User declined initialization
        Err(e) => {
            eprintln!("❌ Error: {}", e);
            false
        }
    }
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    match cli.command {
        Some(Commands::Version) => {
            commands::run_version();
        }
        Some(Commands::Simulate { program_id }) => {
            if !check_initialized(None) {
                return;
            }
            if let Err(e) = commands::run_simulate(program_id).await {
                eprintln!("Error: {}", e);
            }
        }
        Some(Commands::Initialize { path }) => {
            commands::run_init(path);
        }
        Some(Commands::View { path }) => {
            if !check_initialized(path.as_deref()) {
                return;
            }
            commands::run_view(path);
        }
        Some(Commands::Run { path }) => {
            if !check_initialized(path.as_deref()) {
                return;
            }
            if let Err(e) = commands::run_pipeline(path) {
                eprintln!("❌ Error: {}", e);
            }
        }
        Some(Commands::Generate { name, template }) => {
            if let Err(e) = commands::run_generate(name, template) {
                eprintln!("❌ Error: {}", e);
            }
        }
        Some(Commands::Config { action }) => {
            let (config_action, path) = match action {
                ConfigCommands::Show { path } => {
                    if !check_initialized(path.as_deref()) {
                        return;
                    }
                    (commands::ConfigAction::Show, path)
                }
                ConfigCommands::GetNetwork { path } => {
                    if !check_initialized(path.as_deref()) {
                        return;
                    }
                    (commands::ConfigAction::GetNetwork, path)
                }
                ConfigCommands::SetNetwork { network, path } => {
                    if !check_initialized(path.as_deref()) {
                        return;
                    }
                    (commands::ConfigAction::SetNetwork(network), path)
                }
                ConfigCommands::ListNetworks { path } => {
                    if !check_initialized(path.as_deref()) {
                        return;
                    }
                    (commands::ConfigAction::ListNetworks, path)
                }
                ConfigCommands::GetRpc { path } => {
                    if !check_initialized(path.as_deref()) {
                        return;
                    }
                    (commands::ConfigAction::GetRpc, path)
                }
                ConfigCommands::SetRpc { rpc_url, path } => {
                    if !check_initialized(path.as_deref()) {
                        return;
                    }
                    (commands::ConfigAction::SetRpc(rpc_url), path)
                }
                ConfigCommands::ResetRpc { path } => {
                    if !check_initialized(path.as_deref()) {
                        return;
                    }
                    (commands::ConfigAction::ResetRpc, path)
                }
            };

            if let Err(e) = commands::run_config(config_action, path) {
                eprintln!("❌ Error: {}", e);
            }
        }
        None => {
            println!("zklense: ZK Profiling Tool");
            println!("Run `zklense --help` to see commands.");
        }
    }
}
