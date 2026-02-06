//! Color scheme for the UI.

use ratatui::style::Color;

// Tile colors
pub const FLOOR_COLOR: Color = Color::DarkGray;
pub const WALL_COLOR: Color = Color::Gray;
pub const DOOR_COLOR: Color = Color::Yellow;
pub const EXIT_COLOR: Color = Color::Green;
pub const ENTRANCE_COLOR: Color = Color::Cyan;
pub const FOG_COLOR: Color = Color::Rgb(40, 40, 40);

// Entity colors
pub const PLAYER_COLOR: Color = Color::White;
pub const BUG_COLOR: Color = Color::Red;
pub const REGRESSION_COLOR: Color = Color::Magenta;
pub const TECH_DEBT_COLOR: Color = Color::LightRed;
pub const MERGE_CONFLICT_COLOR: Color = Color::LightMagenta;

// Item colors
pub const ITEM_COMMON: Color = Color::Gray;
pub const ITEM_UNCOMMON: Color = Color::Green;
pub const ITEM_RARE: Color = Color::Blue;
pub const ITEM_LEGENDARY: Color = Color::Yellow;

// UI colors
pub const UI_BORDER: Color = Color::DarkGray;
pub const UI_TITLE: Color = Color::White;
pub const UI_TEXT: Color = Color::Gray;
pub const UI_HIGHLIGHT: Color = Color::Cyan;
pub const HP_HIGH: Color = Color::Green;
pub const HP_MED: Color = Color::Yellow;
pub const HP_LOW: Color = Color::Red;
pub const ENERGY_COLOR: Color = Color::Cyan;
pub const FOCUS_COLOR: Color = Color::Magenta;
