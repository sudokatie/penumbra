//! Penumbra - A roguelike where dungeons generate from your git history.

use std::path::PathBuf;

use clap::{Parser, Subcommand, ValueEnum};

use penumbra::cli;
use penumbra::entity::PlayerClass;

/// Player class for CLI parsing.
#[derive(Debug, Clone, Copy, ValueEnum)]
enum CliPlayerClass {
    CodeWarrior,
    MeetingSurvivor,
    InboxKnight,
    Wanderer,
}

impl From<CliPlayerClass> for PlayerClass {
    fn from(c: CliPlayerClass) -> Self {
        match c {
            CliPlayerClass::CodeWarrior => PlayerClass::CodeWarrior,
            CliPlayerClass::MeetingSurvivor => PlayerClass::MeetingSurvivor,
            CliPlayerClass::InboxKnight => PlayerClass::InboxKnight,
            CliPlayerClass::Wanderer => PlayerClass::Wanderer,
        }
    }
}

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
        /// Path to git repository (default data source)
        #[arg(long, default_value = ".")]
        git: PathBuf,

        /// Path to ICS calendar file (alternative data source)
        #[arg(long)]
        calendar: Option<PathBuf>,

        /// Path to mbox email file
        #[arg(long)]
        email: Option<PathBuf>,

        /// IMAP server hostname
        #[arg(long)]
        imap: Option<String>,

        /// IMAP username
        #[arg(long)]
        imap_user: Option<String>,

        /// IMAP port (default: 993)
        #[arg(long, default_value = "993")]
        imap_port: u16,

        /// IMAP folder (default: INBOX)
        #[arg(long, default_value = "INBOX")]
        imap_folder: String,

        /// Max emails to fetch (IMAP only)
        #[arg(long, default_value = "100")]
        imap_limit: usize,

        /// City name for weather data source
        #[arg(long)]
        weather_city: Option<String>,

        /// Latitude for weather data source
        #[arg(long)]
        weather_lat: Option<f64>,

        /// Longitude for weather data source
        #[arg(long)]
        weather_lon: Option<f64>,

        /// Days of history to use
        #[arg(long, default_value = "30")]
        days: u32,

        /// RNG seed for reproducibility
        #[arg(long)]
        seed: Option<u64>,

        /// Player class
        #[arg(long, value_enum)]
        class: Option<CliPlayerClass>,
    },

    /// Continue saved game
    Continue,

    /// Show past runs
    History,
}

fn main() {
    let cli = Cli::parse();

    let result = match cli.command {
        Commands::Play { git, calendar, email, imap, imap_user, imap_port, imap_folder, imap_limit, weather_city, weather_lat, weather_lon, days, seed, class } => {
            if let Some(cal_path) = calendar {
                cli::play_calendar(&cal_path, days, seed, class.map(|c| c.into()))
            } else if let Some(email_path) = email {
                cli::play_email(&email_path, seed, class.map(|c| c.into()))
            } else if let Some(imap_host) = imap {
                let imap_config = penumbra::email::ImapConfig {
                    host: imap_host,
                    port: imap_port,
                    username: imap_user.unwrap_or_default(),
                    password: String::new(), // Will prompt
                    folder: imap_folder,
                    use_tls: true,
                };
                cli::play_imap(&imap_config, imap_limit, seed, class.map(|c| c.into()))
            } else if let Some(city) = weather_city {
                cli::play_weather_city(&city, seed, class.map(|c| c.into()))
            } else if let (Some(lat), Some(lon)) = (weather_lat, weather_lon) {
                cli::play_weather_coords(lat, lon, seed, class.map(|c| c.into()))
            } else {
                cli::play(&git, days, seed, class.map(|c| c.into()))
            }
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
