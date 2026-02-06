//! Save/load persistence.

use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use super::GameState;

/// Record of a completed run.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunRecord {
    pub started_at: DateTime<Utc>,
    pub ended_at: DateTime<Utc>,
    pub victory: bool,
    pub turns: u32,
    pub rooms_cleared: usize,
    pub enemies_killed: usize,
    pub final_level: u32,
    pub death_cause: Option<String>,
}

/// Get the save directory path.
pub fn save_dir() -> PathBuf {
    dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join(".penumbra")
}

/// Get the save file path.
pub fn save_path() -> PathBuf {
    save_dir().join("save.json")
}

/// Get the history file path.
pub fn history_path() -> PathBuf {
    save_dir().join("history.json")
}

/// Ensure save directory exists.
fn ensure_save_dir() -> Result<()> {
    let dir = save_dir();
    if !dir.exists() {
        fs::create_dir_all(&dir).context("Failed to create save directory")?;
    }
    Ok(())
}

/// Save game state to file.
pub fn save_game(state: &GameState) -> Result<()> {
    ensure_save_dir()?;
    let path = save_path();
    let json = serde_json::to_string_pretty(state).context("Failed to serialize game state")?;
    fs::write(&path, json).context("Failed to write save file")?;
    Ok(())
}

/// Load game state from file.
pub fn load_game() -> Result<GameState> {
    let path = save_path();
    let json = fs::read_to_string(&path).context("Failed to read save file")?;
    let state: GameState = serde_json::from_str(&json).context("Failed to parse save file")?;
    Ok(state)
}

/// Check if a save file exists.
pub fn save_exists() -> bool {
    save_path().exists()
}

/// Delete save file.
pub fn delete_save() -> Result<()> {
    let path = save_path();
    if path.exists() {
        fs::remove_file(&path).context("Failed to delete save file")?;
    }
    Ok(())
}

/// Save a run to history.
pub fn save_run_history(record: RunRecord) -> Result<()> {
    ensure_save_dir()?;
    let path = history_path();

    let mut history = load_run_history().unwrap_or_default();
    history.push(record);

    // Keep only last 100 runs
    if history.len() > 100 {
        history.remove(0);
    }

    let json = serde_json::to_string_pretty(&history).context("Failed to serialize history")?;
    fs::write(&path, json).context("Failed to write history file")?;
    Ok(())
}

/// Load run history.
pub fn load_run_history() -> Result<Vec<RunRecord>> {
    let path = history_path();
    if !path.exists() {
        return Ok(Vec::new());
    }
    let json = fs::read_to_string(&path).context("Failed to read history file")?;
    let history: Vec<RunRecord> = serde_json::from_str(&json).context("Failed to parse history")?;
    Ok(history)
}
