//! Local multiplayer support.
//!
//! Enables co-op dungeon crawling with two players sharing the same terminal.

use crossterm::event::KeyCode;
use serde::{Deserialize, Serialize};

use crate::combat::PlayerAction;
use crate::entity::Player;

/// Multiplayer configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MultiplayerConfig {
    /// Whether multiplayer mode is enabled.
    pub enabled: bool,
    /// Player 1 keybinds (default: arrow keys).
    pub p1_keybinds: PlayerKeybinds,
    /// Player 2 keybinds (default: WASD).
    pub p2_keybinds: PlayerKeybinds,
    /// Whether inventory is shared between players.
    pub shared_inventory: bool,
    /// Whether XP is shared between players.
    pub shared_xp: bool,
}

impl Default for MultiplayerConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            p1_keybinds: PlayerKeybinds::arrows(),
            p2_keybinds: PlayerKeybinds::wasd(),
            shared_inventory: true,
            shared_xp: true,
        }
    }
}

/// Keybinds for a single player.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerKeybinds {
    pub up: KeyInput,
    pub down: KeyInput,
    pub left: KeyInput,
    pub right: KeyInput,
    pub wait: KeyInput,
    pub attack: KeyInput,
    pub defend: KeyInput,
    pub use_item: KeyInput,
}

impl PlayerKeybinds {
    /// Arrow key layout (Player 1 default).
    pub fn arrows() -> Self {
        Self {
            up: KeyInput::Arrow(ArrowKey::Up),
            down: KeyInput::Arrow(ArrowKey::Down),
            left: KeyInput::Arrow(ArrowKey::Left),
            right: KeyInput::Arrow(ArrowKey::Right),
            wait: KeyInput::Char('.'),
            attack: KeyInput::Char('/'),
            defend: KeyInput::Char(';'),
            use_item: KeyInput::Char('\''),
        }
    }

    /// WASD layout (Player 2 default).
    pub fn wasd() -> Self {
        Self {
            up: KeyInput::Char('w'),
            down: KeyInput::Char('s'),
            left: KeyInput::Char('a'),
            right: KeyInput::Char('d'),
            wait: KeyInput::Char('x'),
            attack: KeyInput::Char('e'),
            defend: KeyInput::Char('q'),
            use_item: KeyInput::Char('r'),
        }
    }

    /// Check if a key matches this keybind set and return the action.
    pub fn key_to_action(&self, code: KeyCode) -> Option<PlayerAction> {
        if self.up.matches(code) {
            Some(PlayerAction::Move(0, -1))
        } else if self.down.matches(code) {
            Some(PlayerAction::Move(0, 1))
        } else if self.left.matches(code) {
            Some(PlayerAction::Move(-1, 0))
        } else if self.right.matches(code) {
            Some(PlayerAction::Move(1, 0))
        } else if self.wait.matches(code) {
            Some(PlayerAction::Wait)
        } else if self.defend.matches(code) {
            Some(PlayerAction::Defend)
        } else {
            None
        }
    }
}

/// A key input type.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum KeyInput {
    Char(char),
    Arrow(ArrowKey),
}

impl KeyInput {
    /// Check if a KeyCode matches this input.
    pub fn matches(&self, code: KeyCode) -> bool {
        match (self, code) {
            (KeyInput::Char(c), KeyCode::Char(k)) => *c == k,
            (KeyInput::Arrow(ArrowKey::Up), KeyCode::Up) => true,
            (KeyInput::Arrow(ArrowKey::Down), KeyCode::Down) => true,
            (KeyInput::Arrow(ArrowKey::Left), KeyCode::Left) => true,
            (KeyInput::Arrow(ArrowKey::Right), KeyCode::Right) => true,
            _ => false,
        }
    }
}

/// Arrow key directions.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum ArrowKey {
    Up,
    Down,
    Left,
    Right,
}

/// Which player's turn it is.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ActivePlayer {
    Player1,
    Player2,
}

impl ActivePlayer {
    /// Switch to the other player.
    pub fn next(self) -> Self {
        match self {
            ActivePlayer::Player1 => ActivePlayer::Player2,
            ActivePlayer::Player2 => ActivePlayer::Player1,
        }
    }

    /// Get the player number (1 or 2).
    pub fn number(self) -> u8 {
        match self {
            ActivePlayer::Player1 => 1,
            ActivePlayer::Player2 => 2,
        }
    }
}

/// Multiplayer game state extension.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MultiplayerState {
    /// Player 2 (Player 1 is in the main GameState).
    pub player2: Player,
    /// Whose turn it is.
    pub active_player: ActivePlayer,
    /// Configuration.
    pub config: MultiplayerConfig,
}

impl MultiplayerState {
    /// Create a new multiplayer state with a second player.
    pub fn new(config: MultiplayerConfig) -> Self {
        // Player 2 uses a different class by default
        let player2 = Player::new(crate::entity::PlayerClass::Wanderer);

        Self {
            player2,
            active_player: ActivePlayer::Player1,
            config,
        }
    }

    /// Advance to the next player's turn.
    pub fn next_turn(&mut self) {
        self.active_player = self.active_player.next();
    }

    /// Check if it's player 1's turn.
    pub fn is_p1_turn(&self) -> bool {
        self.active_player == ActivePlayer::Player1
    }

    /// Check if it's player 2's turn.
    pub fn is_p2_turn(&self) -> bool {
        self.active_player == ActivePlayer::Player2
    }

    /// Get the active player's keybinds.
    pub fn active_keybinds(&self) -> &PlayerKeybinds {
        match self.active_player {
            ActivePlayer::Player1 => &self.config.p1_keybinds,
            ActivePlayer::Player2 => &self.config.p2_keybinds,
        }
    }

    /// Check if a key belongs to the active player.
    pub fn key_to_action(&self, code: KeyCode) -> Option<(ActivePlayer, PlayerAction)> {
        // Check Player 1's keys
        if let Some(action) = self.config.p1_keybinds.key_to_action(code) {
            return Some((ActivePlayer::Player1, action));
        }

        // Check Player 2's keys
        if let Some(action) = self.config.p2_keybinds.key_to_action(code) {
            return Some((ActivePlayer::Player2, action));
        }

        None
    }

    /// Check if both players are alive.
    pub fn both_alive(&self, p1: &Player) -> bool {
        p1.hp > 0 && self.player2.hp > 0
    }

    /// Check if either player is alive.
    pub fn any_alive(&self, p1: &Player) -> bool {
        p1.hp > 0 || self.player2.hp > 0
    }

    /// Get the player who is closer to a position.
    pub fn closer_player<'a>(&'a self, p1: &'a Player, x: i32, y: i32) -> &'a Player {
        let d1 = (p1.x - x).abs() + (p1.y - y).abs();
        let d2 = (self.player2.x - x).abs() + (self.player2.y - y).abs();
        if d1 <= d2 { p1 } else { &self.player2 }
    }

    /// Share XP between players if configured.
    pub fn share_xp(&mut self, p1: &mut Player, xp: u32) {
        if self.config.shared_xp {
            // Both players get the XP
            p1.xp += xp;
            self.player2.xp += xp;
        } else {
            // Only active player gets XP
            match self.active_player {
                ActivePlayer::Player1 => p1.xp += xp,
                ActivePlayer::Player2 => self.player2.xp += xp,
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_multiplayer_config_default() {
        let config = MultiplayerConfig::default();
        assert!(!config.enabled);
        assert!(config.shared_inventory);
        assert!(config.shared_xp);
    }

    #[test]
    fn test_arrow_keybinds() {
        let keybinds = PlayerKeybinds::arrows();
        assert!(keybinds.up.matches(KeyCode::Up));
        assert!(keybinds.down.matches(KeyCode::Down));
        assert!(keybinds.left.matches(KeyCode::Left));
        assert!(keybinds.right.matches(KeyCode::Right));
    }

    #[test]
    fn test_wasd_keybinds() {
        let keybinds = PlayerKeybinds::wasd();
        assert!(keybinds.up.matches(KeyCode::Char('w')));
        assert!(keybinds.down.matches(KeyCode::Char('s')));
        assert!(keybinds.left.matches(KeyCode::Char('a')));
        assert!(keybinds.right.matches(KeyCode::Char('d')));
    }

    #[test]
    fn test_key_to_action_arrows() {
        let keybinds = PlayerKeybinds::arrows();
        let action = keybinds.key_to_action(KeyCode::Up);
        assert!(matches!(action, Some(PlayerAction::Move(0, -1))));
    }

    #[test]
    fn test_key_to_action_wasd() {
        let keybinds = PlayerKeybinds::wasd();
        let action = keybinds.key_to_action(KeyCode::Char('w'));
        assert!(matches!(action, Some(PlayerAction::Move(0, -1))));
    }

    #[test]
    fn test_active_player_next() {
        let p1 = ActivePlayer::Player1;
        assert_eq!(p1.next(), ActivePlayer::Player2);
        assert_eq!(p1.next().next(), ActivePlayer::Player1);
    }

    #[test]
    fn test_active_player_number() {
        assert_eq!(ActivePlayer::Player1.number(), 1);
        assert_eq!(ActivePlayer::Player2.number(), 2);
    }

    #[test]
    fn test_multiplayer_state_new() {
        let config = MultiplayerConfig::default();
        let state = MultiplayerState::new(config);
        assert_eq!(state.active_player, ActivePlayer::Player1);
        assert!(state.player2.hp > 0);
    }

    #[test]
    fn test_multiplayer_state_next_turn() {
        let config = MultiplayerConfig::default();
        let mut state = MultiplayerState::new(config);
        assert!(state.is_p1_turn());
        state.next_turn();
        assert!(state.is_p2_turn());
        state.next_turn();
        assert!(state.is_p1_turn());
    }

    #[test]
    fn test_key_to_action_identifies_player() {
        let config = MultiplayerConfig::default();
        let state = MultiplayerState::new(config);

        // Arrow up is Player 1
        let (player, _) = state.key_to_action(KeyCode::Up).unwrap();
        assert_eq!(player, ActivePlayer::Player1);

        // W is Player 2
        let (player, _) = state.key_to_action(KeyCode::Char('w')).unwrap();
        assert_eq!(player, ActivePlayer::Player2);
    }

    #[test]
    fn test_both_alive() {
        let config = MultiplayerConfig::default();
        let state = MultiplayerState::new(config);
        let p1 = Player::new(crate::entity::PlayerClass::Wanderer);

        assert!(state.both_alive(&p1));
    }

    #[test]
    fn test_share_xp_enabled() {
        let config = MultiplayerConfig::default();
        let mut state = MultiplayerState::new(config);
        let mut p1 = Player::new(crate::entity::PlayerClass::Wanderer);

        p1.xp = 0;
        state.player2.xp = 0;

        state.share_xp(&mut p1, 100);

        assert_eq!(p1.xp, 100);
        assert_eq!(state.player2.xp, 100);
    }

    #[test]
    fn test_share_xp_disabled() {
        let mut config = MultiplayerConfig::default();
        config.shared_xp = false;
        let mut state = MultiplayerState::new(config);
        let mut p1 = Player::new(crate::entity::PlayerClass::Wanderer);

        p1.xp = 0;
        state.player2.xp = 0;
        state.active_player = ActivePlayer::Player1;

        state.share_xp(&mut p1, 100);

        assert_eq!(p1.xp, 100);
        assert_eq!(state.player2.xp, 0);
    }
}
