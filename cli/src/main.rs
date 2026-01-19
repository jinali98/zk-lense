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
    /// Example: Display emojis in output
    #[command(name = "testcommand")]
    Emoji,
    /// Example: Show a loading spinner
    Loading,
    /// Example: Display data in a table
    Table,
    /// Example: Show a progress bar
    Progress,
}

fn main() {
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
        None => {
            println!("zkprof: ZK Profiling Tool");
            println!("Run `zkprof --help` to see commands.");
        }
    }
}