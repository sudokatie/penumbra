//! CLI command implementations.

use std::io;
use std::path::Path;

use anyhow::{Context, Result};
use crossterm::terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen};
use crossterm::ExecutableCommand;
use ratatui::prelude::*;

use crate::calendar::parse_ics_file;
use crate::email::{ImapConfig, parse_mbox_file, fetch_emails};
use crate::entity::PlayerClass;
use crate::game::{load_game, save_exists, save_game, GameState, load_run_history};
use crate::git::parse_repository;
use crate::ui::App;
use crate::weather::{fetch_weather, fetch_weather_by_city};
use crate::world::{generate_dungeon_from_calendar, generate_dungeon_from_email, generate_dungeon_from_weather};

/// Start a new game.
pub fn play(git_path: &Path, days: u32, seed: Option<u64>, class: Option<PlayerClass>) -> Result<()> {
    // Parse git repository
    let commits = parse_repository(git_path, days)
        .context("Failed to parse git repository")?;

    println!("Found {} commits over {} days", commits.len(), days);
    println!("Generating dungeon...");

    // Generate seed if not provided
    let seed = seed.unwrap_or_else(|| {
        use std::time::{SystemTime, UNIX_EPOCH};
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs()
    });

    // Create game state with optional class (auto-detects if None)
    let state = GameState::new_with_class(commits, seed, class, git_path.to_path_buf());

    println!("Created {} rooms", state.world.rooms.len());
    println!("Starting game...");

    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    stdout.execute(EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Run game
    let mut app = App::new(state);
    let result = app.run(&mut terminal);

    // Save if game ended properly
    if !app.quit {
        let _ = save_game(&app.state);
    }

    // Restore terminal
    disable_raw_mode()?;
    io::stdout().execute(LeaveAlternateScreen)?;

    result.context("Game error")?;

    if app.state.victory {
        println!("Congratulations! You conquered the dungeon!");
    } else if app.state.game_over {
        println!("Game over. Better luck next time!");
    }

    Ok(())
}

/// Start a new game from calendar data.
pub fn play_calendar(calendar_path: &Path, days: u32, seed: Option<u64>, class: Option<PlayerClass>) -> Result<()> {
    // Parse calendar file
    let events = parse_ics_file(calendar_path, days)
        .context("Failed to parse calendar file")?;

    println!("Found {} events over {} days", events.len(), days);
    println!("Generating dungeon from calendar...");

    // Generate seed if not provided
    let seed = seed.unwrap_or_else(|| {
        use std::time::{SystemTime, UNIX_EPOCH};
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs()
    });

    // Generate world from calendar
    let world = generate_dungeon_from_calendar(&events, seed);

    // Create game state
    let state = GameState::new_from_world(world, seed, class, calendar_path.to_path_buf());

    println!("Created {} rooms", state.world.rooms.len());
    println!("Starting game...");

    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    stdout.execute(EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Run game
    let mut app = App::new(state);
    let result = app.run(&mut terminal);

    // Save if game ended properly
    if !app.quit {
        let _ = save_game(&app.state);
    }

    // Restore terminal
    disable_raw_mode()?;
    io::stdout().execute(LeaveAlternateScreen)?;

    result.context("Game error")?;

    if app.state.victory {
        println!("Congratulations! You conquered the calendar dungeon!");
    } else if app.state.game_over {
        println!("Game over. Your schedule defeated you!");
    }

    Ok(())
}

/// Continue a saved game.
pub fn continue_game() -> Result<()> {
    if !save_exists() {
        println!("No saved game found. Start a new game with 'penumbra play'");
        return Ok(());
    }

    let state = load_game().context("Failed to load save file")?;
    println!("Loading saved game (Turn {})...", state.turn);

    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    stdout.execute(EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Run game
    let mut app = App::new(state);
    let result = app.run(&mut terminal);

    // Save progress
    if !app.quit && !app.state.game_over {
        let _ = save_game(&app.state);
    }

    // Restore terminal
    disable_raw_mode()?;
    io::stdout().execute(LeaveAlternateScreen)?;

    result.context("Game error")?;

    Ok(())
}

/// Start a new game from mbox email file.
pub fn play_email(email_path: &Path, seed: Option<u64>, class: Option<PlayerClass>) -> Result<()> {
    // Parse email file
    let emails = parse_mbox_file(email_path)
        .context("Failed to parse mbox file")?;

    println!("Found {} emails", emails.len());
    println!("Generating dungeon from inbox...");

    // Generate seed if not provided
    let seed = seed.unwrap_or_else(|| {
        use std::time::{SystemTime, UNIX_EPOCH};
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs()
    });

    // Generate world from emails
    let world = generate_dungeon_from_email(&emails, seed);

    // Create game state
    let state = GameState::new_from_world(world, seed, class, email_path.to_path_buf());

    println!("Created {} rooms", state.world.rooms.len());
    println!("Starting game...");

    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    stdout.execute(EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Run game
    let mut app = App::new(state);
    let result = app.run(&mut terminal);

    // Save if game ended properly
    if !app.quit {
        let _ = save_game(&app.state);
    }

    // Restore terminal
    disable_raw_mode()?;
    io::stdout().execute(LeaveAlternateScreen)?;

    result.context("Game error")?;

    if app.state.victory {
        println!("Congratulations! You conquered the inbox dungeon!");
    } else if app.state.game_over {
        println!("Game over. Your inbox defeated you!");
    }

    Ok(())
}

/// Start a new game from IMAP email server.
pub fn play_imap(config: &ImapConfig, limit: usize, seed: Option<u64>, class: Option<PlayerClass>) -> Result<()> {
    // Prompt for password if not provided
    let mut config = config.clone();
    if config.password.is_empty() {
        print!("IMAP Password: ");
        io::Write::flush(&mut io::stdout())?;
        let mut password = String::new();
        io::stdin().read_line(&mut password)?;
        config.password = password.trim().to_string();
    }

    println!("Connecting to {}...", config.host);

    // Fetch emails
    let emails = fetch_emails(&config, limit)
        .context("Failed to fetch emails from IMAP")?;

    println!("Fetched {} emails", emails.len());
    println!("Generating dungeon from inbox...");

    // Generate seed if not provided
    let seed = seed.unwrap_or_else(|| {
        use std::time::{SystemTime, UNIX_EPOCH};
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs()
    });

    // Generate world from emails
    let world = generate_dungeon_from_email(&emails, seed);

    // Create game state (use host as source path)
    let source_path = std::path::PathBuf::from(&config.host);
    let state = GameState::new_from_world(world, seed, class, source_path);

    println!("Created {} rooms", state.world.rooms.len());
    println!("Starting game...");

    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    stdout.execute(EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Run game
    let mut app = App::new(state);
    let result = app.run(&mut terminal);

    // Save if game ended properly
    if !app.quit {
        let _ = save_game(&app.state);
    }

    // Restore terminal
    disable_raw_mode()?;
    io::stdout().execute(LeaveAlternateScreen)?;

    result.context("Game error")?;

    if app.state.victory {
        println!("Congratulations! You conquered the inbox dungeon!");
    } else if app.state.game_over {
        println!("Game over. Your inbox defeated you!");
    }

    Ok(())
}

/// Start a new game from weather data by city name.
pub fn play_weather_city(city: &str, seed: Option<u64>, class: Option<PlayerClass>) -> Result<()> {
    println!("Fetching weather for {}...", city);

    // Fetch weather data
    let weather = fetch_weather_by_city(city)
        .context("Failed to fetch weather data")?;

    play_weather_internal(weather, seed, class)
}

/// Start a new game from weather data by coordinates.
pub fn play_weather_coords(lat: f64, lon: f64, seed: Option<u64>, class: Option<PlayerClass>) -> Result<()> {
    println!("Fetching weather for ({:.2}, {:.2})...", lat, lon);

    // Fetch weather data
    let weather = fetch_weather(lat, lon)
        .context("Failed to fetch weather data")?;

    play_weather_internal(weather, seed, class)
}

/// Internal function to run game from weather data.
fn play_weather_internal(weather: crate::weather::WeatherData, seed: Option<u64>, class: Option<PlayerClass>) -> Result<()> {
    println!("Weather in {}: {} ({:.1}C, {}% humidity, {:.1} km/h wind)",
        weather.location, weather.description,
        weather.temperature_c, weather.humidity, weather.wind_speed_kph);

    let atmosphere = crate::weather::generate_atmosphere(&weather);
    println!("Dungeon atmosphere: {}", atmosphere.description());
    println!("Difficulty multiplier: {:.2}x", weather.difficulty_multiplier());
    println!("Generating dungeon from weather...");

    // Generate seed if not provided
    let seed = seed.unwrap_or_else(|| {
        use std::time::{SystemTime, UNIX_EPOCH};
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs()
    });

    // Generate world from weather
    let world = generate_dungeon_from_weather(&weather, seed);

    // Create game state (use location as source path)
    let source_path = std::path::PathBuf::from(&weather.location);
    let state = GameState::new_from_world(world, seed, class, source_path);

    println!("Created {} rooms", state.world.rooms.len());
    println!("Starting game...");

    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    stdout.execute(EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Run game
    let mut app = App::new(state);
    let result = app.run(&mut terminal);

    // Save if game ended properly
    if !app.quit {
        let _ = save_game(&app.state);
    }

    // Restore terminal
    disable_raw_mode()?;
    io::stdout().execute(LeaveAlternateScreen)?;

    result.context("Game error")?;

    if app.state.victory {
        println!("Congratulations! You conquered the weather dungeon!");
    } else if app.state.game_over {
        println!("Game over. The elements defeated you!");
    }

    Ok(())
}

/// Show run history.
pub fn show_history() -> Result<()> {
    let history = load_run_history().context("Failed to load history")?;

    if history.is_empty() {
        println!("No run history yet. Start a game with 'penumbra play'");
        return Ok(());
    }

    println!("=== Run History ===\n");

    for (i, run) in history.iter().rev().take(10).enumerate() {
        let status = if run.victory { "Victory" } else { "Defeat" };
        println!(
            "{}. {} - {} turns, {} rooms, level {}",
            i + 1,
            status,
            run.turns,
            run.rooms_cleared,
            run.final_level
        );
        if let Some(cause) = &run.death_cause {
            println!("   Cause: {}", cause);
        }
        println!();
    }

    Ok(())
}
