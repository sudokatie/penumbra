//! Penumbra - A roguelike where dungeons generate from your git history.

use std::path::PathBuf;

use clap::{Parser, Subcommand};

use penumbra::cli;

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

    let result = match cli.command {
        Commands::Play { git, days, seed } => {
            cli::play(&git, days, seed, None)
        }
        Commands::Continue => {
            cli::continue_game()
        }
        Commands::History => {
            cli::show_history()
        }
    };

    if let Err(e) = result {
        eprintln!("Error: {:#}", e);
        std::process::exit(1);
    }
}
