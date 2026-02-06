//! Penumbra - A roguelike where dungeons generate from your git history.

use std::path::PathBuf;

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "penumbra")]
#[command(about = "A roguelike where dungeons generate from your git history")]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Start a new game
    Play {
        /// Path to git repository
        #[arg(long, default_value = ".")]
        git: PathBuf,

        /// Days of history to use
        #[arg(long, default_value = "30")]
        days: u32,

        /// RNG seed for reproducibility
        #[arg(long)]
        seed: Option<u64>,
    },

    /// Continue saved game
    Continue,

    /// Show past runs
    History,
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::Play { git, days, seed } => {
            println!("Starting new game...");
            println!("  Git path: {}", git.display());
            println!("  Days: {}", days);
            if let Some(s) = seed {
                println!("  Seed: {}", s);
            }
            // TODO: Implement game start
        }
        Commands::Continue => {
            println!("Continuing saved game...");
            // TODO: Implement continue
        }
        Commands::History => {
            println!("Run history:");
            // TODO: Implement history
        }
    }
}
