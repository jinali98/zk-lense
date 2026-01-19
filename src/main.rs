use clap::{Parser, Subcommand};

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
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Some(Commands::Hello { name }) => {
            let who = name.unwrap_or_else(|| "world".to_string());
            println!("Test command: Hello, {}!", who);
        }
        Some(Commands::Version) => {
            println!("zkprof {}", env!("CARGO_PKG_VERSION"));
        }
        None => {
            println!("zkprof: ZK Profiling Tool");
            println!("Run `zkprof --help` to see commands.");
        }
    }
}