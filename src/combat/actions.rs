//! Player actions and costs.

use crate::world::Direction;

/// Energy costs for actions.
pub const MOVE_COST: i32 = 1;
pub const ATTACK_COST: i32 = 5;
pub const DEFEND_COST: i32 = 3;
pub const USE_ITEM_COST: i32 = 2;
pub const WAIT_REGEN: i32 = 2;

/// Actions the player can take.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlayerAction {
    /// Move in a direction.
    Move(i32, i32),
    /// Attack in a direction.
    Attack(Direction),
    /// Defend (reduce incoming damage).
    Defend,
    /// Use an item from inventory.
    UseItem(usize),
    /// Wait and regenerate energy.
    Wait,
}

impl PlayerAction {
    /// Get the energy cost of this action.
    pub fn energy_cost(&self) -> i32 {
        match self {
            PlayerAction::Move(_, _) => MOVE_COST,
            PlayerAction::Attack(_) => ATTACK_COST,
            PlayerAction::Defend => DEFEND_COST,
            PlayerAction::UseItem(_) => USE_ITEM_COST,
            PlayerAction::Wait => 0, // Wait costs nothing, gives regen
        }
    }

    /// Check if this is a movement action.
    pub fn is_movement(&self) -> bool {
        matches!(self, PlayerAction::Move(_, _))
    }

    /// Check if this is an attack action.
    pub fn is_attack(&self) -> bool {
        matches!(self, PlayerAction::Attack(_))
    }
}
