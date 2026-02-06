//! CLI command implementations.

use std::io;
use std::path::Path;

use anyhow::{Context, Result};
use crossterm::terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen};
use crossterm::ExecutableCommand;
use ratatui::prelude::*;

use crate::entity::PlayerClass;
use crate::game::{load_game, save_exists, save_game, GameState, load_run_history};
use crate::git::parse_repository;
use crate::ui::App;

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

    // Create game state with optional class
    let state = GameState::new_with_class(commits, seed, class);

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
