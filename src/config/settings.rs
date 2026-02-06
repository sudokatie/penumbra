//! User configuration settings.

use std::fs;
use std::path::PathBuf;

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

/// Complete application settings.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Settings {
    pub display: DisplaySettings,
    pub gameplay: GameplaySettings,
    pub keybinds: Keybinds,
}

/// Display-related settings.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DisplaySettings {
    /// Enable terminal colors.
    pub color: bool,
    /// Use unicode box-drawing characters.
    pub unicode: bool,
}

/// Gameplay-related settings.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameplaySettings {
    /// Default days of git history to use.
    pub default_days: u32,
    /// Automatically pick up items when walking over them.
    pub auto_pickup: bool,
    /// Confirm before attacking.
    pub confirm_attacks: bool,
}

/// Custom keybindings.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Keybinds {
    pub move_up: String,
    pub move_down: String,
    pub move_left: String,
    pub move_right: String,
    pub attack: String,
    pub inventory: String,
    pub wait: String,
    pub help: String,
    pub quit: String,
}

impl Default for Settings {
    fn default() -> Self {
        default_settings()
    }
}

impl Default for DisplaySettings {
    fn default() -> Self {
        Self {
            color: true,
            unicode: true,
        }
    }
}

impl Default for GameplaySettings {
    fn default() -> Self {
        Self {
            default_days: 30,
            auto_pickup: true,
            confirm_attacks: false,
        }
    }
}

impl Default for Keybinds {
    fn default() -> Self {
        Self {
            move_up: "k".to_string(),
            move_down: "j".to_string(),
            move_left: "h".to_string(),
            move_right: "l".to_string(),
            attack: "a".to_string(),
            inventory: "i".to_string(),
            wait: ".".to_string(),
            help: "?".to_string(),
            quit: "q".to_string(),
        }
    }
}

/// Get the config file path.
pub fn config_path() -> PathBuf {
    dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join(".penumbra")
        .join("config.toml")
}

/// Get default settings.
pub fn default_settings() -> Settings {
    Settings {
        display: DisplaySettings::default(),
        gameplay: GameplaySettings::default(),
        keybinds: Keybinds::default(),
    }
}

/// Load settings from config file, falling back to defaults.
pub fn load_settings() -> Settings {
    let path = config_path();
    
    if !path.exists() {
        return default_settings();
    }

    match fs::read_to_string(&path) {
        Ok(content) => match toml::from_str(&content) {
            Ok(settings) => settings,
            Err(_) => default_settings(),
        },
        Err(_) => default_settings(),
    }
}

/// Save settings to config file.
pub fn save_settings(settings: &Settings) -> Result<()> {
    let path = config_path();
    
    // Ensure parent directory exists
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).context("Failed to create config directory")?;
    }

    let toml = toml::to_string_pretty(settings).context("Failed to serialize settings")?;
    fs::write(&path, toml).context("Failed to write config file")?;
    
    Ok(())
}
