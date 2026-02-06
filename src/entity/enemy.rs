//! Enemy entities.

use serde::{Deserialize, Serialize};

use super::EnemyType;

/// An enemy in the dungeon.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Enemy {
    pub x: i32,
    pub y: i32,
    pub hp: i32,
    pub max_hp: i32,
    pub damage: i32,
    pub enemy_type: EnemyType,
    pub source_commit: String,
    pub turns_alive: u32,
}

impl Enemy {
    /// Create a new enemy of the given type.
    pub fn new(enemy_type: EnemyType, x: i32, y: i32, commit_hash: &str) -> Self {
        let hp = enemy_type.base_hp();
        Self {
            x,
            y,
            hp,
            max_hp: hp,
            damage: enemy_type.base_damage(),
            enemy_type,
            source_commit: commit_hash.to_string(),
            turns_alive: 0,
        }
    }

    /// Take damage, return true if still alive.
    pub fn take_damage(&mut self, amount: i32) -> bool {
        self.hp -= amount.max(1);
        self.hp > 0
    }

    /// Get ASCII symbol.
    pub fn symbol(&self) -> char {
        self.enemy_type.symbol()
    }

    /// Check if at half health (for MergeConflict split).
    pub fn at_half_health(&self) -> bool {
        self.hp <= self.max_hp / 2
    }
}
