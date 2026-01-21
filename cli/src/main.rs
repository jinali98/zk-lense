use clap::{Parser, Subcommand};

mod commands;

#[derive(Parser)]
#[command(name = "zkprof", version, about = "ZK Profiling Tool")]
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
        None => {
            println!("zkprof: ZK Profiling Tool");
            println!("Run `zkprof --help` to see commands.");
        }
    }
}