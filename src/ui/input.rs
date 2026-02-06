//! Input handling utilities.

use crossterm::event::KeyCode;

use crate::combat::PlayerAction;
use crate::config::Keybinds;
use crate::world::Direction;

/// Parse a key into a player action based on keybinds.
pub fn key_to_action(code: KeyCode, keybinds: &Keybinds) -> Option<PlayerAction> {
    match code {
        // Movement
        KeyCode::Up => Some(PlayerAction::Move(0, -1)),
        KeyCode::Down => Some(PlayerAction::Move(0, 1)),
        KeyCode::Left => Some(PlayerAction::Move(-1, 0)),
        KeyCode::Right => Some(PlayerAction::Move(1, 0)),
        
        KeyCode::Char(c) => {
            if c.to_string() == keybinds.move_up {
                Some(PlayerAction::Move(0, -1))
            } else if c.to_string() == keybinds.move_down {
                Some(PlayerAction::Move(0, 1))
            } else if c.to_string() == keybinds.move_left {
                Some(PlayerAction::Move(-1, 0))
            } else if c.to_string() == keybinds.move_right {
                Some(PlayerAction::Move(1, 0))
            } else if c.to_string() == keybinds.wait || c == '.' || c == ' ' {
                Some(PlayerAction::Wait)
            } else {
                None
            }
        }
        
        _ => None,
    }
}

/// Check if a key is the quit key.
pub fn is_quit_key(code: KeyCode, keybinds: &Keybinds) -> bool {
    match code {
        KeyCode::Esc => true,
        KeyCode::Char(c) => c.to_string() == keybinds.quit,
        _ => false,
    }
}

/// Check if a key is the help key.
pub fn is_help_key(code: KeyCode, keybinds: &Keybinds) -> bool {
    matches!(code, KeyCode::Char(c) if c.to_string() == keybinds.help)
}

/// Check if a key is the inventory key.
pub fn is_inventory_key(code: KeyCode, keybinds: &Keybinds) -> bool {
    matches!(code, KeyCode::Char(c) if c.to_string() == keybinds.inventory)
}

/// Check if a key is the attack key.
pub fn is_attack_key(code: KeyCode, keybinds: &Keybinds) -> bool {
    matches!(code, KeyCode::Char(c) if c.to_string() == keybinds.attack)
}

/// Convert a direction key to a Direction.
pub fn key_to_direction(code: KeyCode) -> Option<Direction> {
    match code {
        KeyCode::Up | KeyCode::Char('k') => Some(Direction::North),
        KeyCode::Down | KeyCode::Char('j') => Some(Direction::South),
        KeyCode::Left | KeyCode::Char('h') => Some(Direction::West),
        KeyCode::Right | KeyCode::Char('l') => Some(Direction::East),
        _ => None,
    }
}
