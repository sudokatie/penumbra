//! Item type definitions.

use serde::{Deserialize, Serialize};

/// Category of item.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ItemType {
    Consumable,
    Equipment,
    Scroll,
}

/// Item rarity.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Rarity {
    Common,
    Uncommon,
    Rare,
    Legendary,
}

/// Stat that can be modified.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Stat {
    MaxHP,
    MaxEnergy,
    Focus,
    Damage,
}

/// Effect an item can have.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ItemEffect {
    Heal(i32),
    RestoreEnergy(i32),
    Damage(i32),
    Buff(Stat, i32, u32), // stat, amount, duration
    RevealMap,
}
