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
    #[command(name = "hi")]
    Hello {
        name: Option<String>,
    },
    Version,
    /// Initialize a new zkproof project
    #[command(name = "init")]
    Initialize {
        /// Path to initialize zkproof in (relative or absolute). Defaults to current directory.
        path: Option<String>,
    },

    #[command(name = "view")]
    View {
        /// Path to the project directory. Defaults to current directory.
        path: Option<String>,
    },
    /// Example: Display emojis in output
    #[command(name = "testcommand")]
    Emoji,
    /// Example: Show a loading spinner
    Loading,
    /// Example: Display data in a table
    Table,
    /// Example: Show a progress bar
    Progress,
    /// Example: Simulate a transaction
    Simulate,
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    match cli.command {
        Some(Commands::Hello { name }) => {
            commands::run_hello(name);
        }
        Some(Commands::Version) => {
            commands::run_version();
        }
        Some(Commands::Emoji) => {
            commands::run_emoji();
        }
        Some(Commands::Loading) => {
            commands::run_loading();
        }
        Some(Commands::Table) => {
            commands::run_table();
        }
        Some(Commands::Progress) => {
            commands::run_progress();
        }
        Some(Commands::Simulate) => {
            if let Err(e) = commands::run_simulate().await {
                eprintln!("Error: {}", e);
            }
        }
        Some(Commands::Initialize { path }) => {
            commands::run_init(path);
        }
        Some(Commands::View { path }) => {
            commands::run_view(path);
        }
        None => {
            println!("zkprof: ZK Profiling Tool");
            println!("Run `zkprof --help` to see commands.");
        }
    }
}