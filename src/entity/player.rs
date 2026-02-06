//! Player entity.

use serde::{Deserialize, Serialize};

use crate::item::Item;

use super::PlayerClass;

/// The player character.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Player {
    pub x: i32,
    pub y: i32,
    pub hp: i32,
    pub max_hp: i32,
    pub energy: i32,
    pub max_energy: i32,
    pub focus: i32,
    pub max_focus: i32,
    pub damage: i32,
    pub inventory: Vec<Item>,
    pub class: PlayerClass,
    pub level: u32,
    pub xp: u32,
    pub defending: bool,
}

impl Player {
    /// Create a new player with the given class.
    pub fn new(class: PlayerClass) -> Self {
        let (hp_bonus, focus_bonus, damage_bonus) = match class {
            PlayerClass::CodeWarrior => (0, 0, 10),
            PlayerClass::MeetingSurvivor => (20, 0, 0),
            PlayerClass::InboxKnight => (0, 10, 0),
            PlayerClass::Wanderer => (5, 5, 5),
        };

        Self {
            x: 1,
            y: 1,
            hp: 50 + hp_bonus,
            max_hp: 50 + hp_bonus,
            energy: 100,
            max_energy: 100,
            focus: 50 + focus_bonus,
            max_focus: 50 + focus_bonus,
            damage: 10 + damage_bonus,
            inventory: Vec::new(),
            class,
            level: 1,
            xp: 0,
            defending: false,
        }
    }

    /// Take damage, return true if still alive.
    pub fn take_damage(&mut self, amount: i32) -> bool {
        let actual = if self.defending { amount / 2 } else { amount };
        self.hp -= actual.max(1);
        self.defending = false;
        self.hp > 0
    }

    /// Heal the player.
    pub fn heal(&mut self, amount: i32) {
        self.hp = (self.hp + amount).min(self.max_hp);
    }

    /// Use energy, return true if had enough.
    pub fn use_energy(&mut self, amount: i32) -> bool {
        if self.energy >= amount {
            self.energy -= amount;
            true
        } else {
            false
        }
    }

    /// Regenerate energy.
    pub fn regen_energy(&mut self, amount: i32) {
        self.energy = (self.energy + amount).min(self.max_energy);
    }

    /// Add XP, return true if leveled up.
    pub fn add_xp(&mut self, amount: u32) -> bool {
        self.xp += amount;
        let threshold = self.level * 100;
        if self.xp >= threshold {
            self.xp -= threshold;
            self.level += 1;
            self.max_hp += 10;
            self.hp = self.max_hp;
            true
        } else {
            false
        }
    }

    /// Pick up an item (max 10).
    pub fn pickup_item(&mut self, item: Item) -> bool {
        if self.inventory.len() < 10 {
            self.inventory.push(item);
            true
        } else {
            false
        }
    }
}
