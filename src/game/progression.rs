//! Meta-progression system.
//!
//! Persistent unlocks and upgrades that carry over between runs.

use std::collections::HashSet;
use std::fs;
use std::path::PathBuf;

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

use super::save::save_dir;
use crate::entity::PlayerClass;

/// Progression file path.
pub fn progression_path() -> PathBuf {
    save_dir().join("progression.json")
}

/// Persistent progression data.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Progression {
    /// Total runs completed (win or lose)
    pub total_runs: u32,
    /// Total victories
    pub victories: u32,
    /// Total enemies killed across all runs
    pub total_kills: u32,
    /// Total rooms cleared across all runs
    pub total_rooms: u32,
    /// Meta-currency earned (1 per enemy killed, 5 per room cleared, 20 per victory)
    pub essence: u32,
    /// Unlocked classes (by name)
    pub unlocked_classes: HashSet<String>,
    /// Unlocked items (by name)
    pub unlocked_items: HashSet<String>,
    /// Purchased permanent upgrades
    pub upgrades: Upgrades,
    /// Best run (most rooms cleared)
    pub best_rooms: u32,
    /// Fastest victory (fewest turns)
    pub fastest_victory: Option<u32>,
}

/// Permanent upgrades purchasable with essence.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Upgrades {
    /// Bonus starting HP (+5 per level)
    pub hp_bonus: u8,
    /// Bonus starting energy (+2 per level)
    pub energy_bonus: u8,
    /// Bonus starting damage (+1 per level)
    pub damage_bonus: u8,
    /// Start with better weapon tier (0-2)
    pub starting_weapon: u8,
    /// Chance to find better items (0-3, +5% per level)
    pub loot_luck: u8,
}

impl Upgrades {
    /// Max level for each upgrade.
    pub const MAX_HP: u8 = 5;
    pub const MAX_ENERGY: u8 = 5;
    pub const MAX_DAMAGE: u8 = 3;
    pub const MAX_WEAPON: u8 = 2;
    pub const MAX_LUCK: u8 = 3;

    /// Cost for an upgrade at a given level.
    pub fn cost(level: u8) -> u32 {
        match level {
            0 => 10,
            1 => 25,
            2 => 50,
            3 => 100,
            4 => 200,
            _ => 500,
        }
    }

    /// Total bonus HP from upgrades.
    pub fn bonus_hp(&self) -> i32 {
        self.hp_bonus as i32 * 5
    }

    /// Total bonus energy from upgrades.
    pub fn bonus_energy(&self) -> i32 {
        self.energy_bonus as i32 * 2
    }

    /// Total bonus damage from upgrades.
    pub fn bonus_damage(&self) -> i32 {
        self.damage_bonus as i32
    }

    /// Loot luck percentage bonus.
    pub fn loot_luck_bonus(&self) -> i32 {
        self.loot_luck as i32 * 5
    }
}

impl Progression {
    /// Create new progression with starter class unlocked.
    pub fn new() -> Self {
        let mut prog = Self::default();
        prog.unlocked_classes.insert("CodeWarrior".to_string());
        prog
    }

    /// Award essence and update stats from a completed run.
    pub fn complete_run(&mut self, victory: bool, kills: u32, rooms: u32, turns: u32) {
        self.total_runs += 1;
        self.total_kills += kills;
        self.total_rooms += rooms;

        // Calculate essence earned
        let mut essence_earned = kills; // 1 per kill
        essence_earned += rooms * 5; // 5 per room

        if victory {
            self.victories += 1;
            essence_earned += 20; // Victory bonus

            // Track fastest victory
            match self.fastest_victory {
                Some(best) if turns < best => self.fastest_victory = Some(turns),
                None => self.fastest_victory = Some(turns),
                _ => {}
            }
        }

        self.essence += essence_earned;

        // Track best run
        if rooms > self.best_rooms {
            self.best_rooms = rooms;
        }
    }

    /// Check if a class is unlocked.
    pub fn is_class_unlocked(&self, class: &PlayerClass) -> bool {
        let name = format!("{:?}", class);
        self.unlocked_classes.contains(&name)
    }

    /// Unlock a class (costs essence).
    pub fn unlock_class(&mut self, class: &PlayerClass, cost: u32) -> bool {
        if self.essence < cost {
            return false;
        }
        let name = format!("{:?}", class);
        if self.unlocked_classes.contains(&name) {
            return false;
        }
        self.essence -= cost;
        self.unlocked_classes.insert(name);
        true
    }

    /// Purchase an HP upgrade.
    pub fn upgrade_hp(&mut self) -> bool {
        if self.upgrades.hp_bonus >= Upgrades::MAX_HP {
            return false;
        }
        let cost = Upgrades::cost(self.upgrades.hp_bonus);
        if self.essence < cost {
            return false;
        }
        self.essence -= cost;
        self.upgrades.hp_bonus += 1;
        true
    }

    /// Purchase an energy upgrade.
    pub fn upgrade_energy(&mut self) -> bool {
        if self.upgrades.energy_bonus >= Upgrades::MAX_ENERGY {
            return false;
        }
        let cost = Upgrades::cost(self.upgrades.energy_bonus);
        if self.essence < cost {
            return false;
        }
        self.essence -= cost;
        self.upgrades.energy_bonus += 1;
        true
    }

    /// Purchase a damage upgrade.
    pub fn upgrade_damage(&mut self) -> bool {
        if self.upgrades.damage_bonus >= Upgrades::MAX_DAMAGE {
            return false;
        }
        let cost = Upgrades::cost(self.upgrades.damage_bonus);
        if self.essence < cost {
            return false;
        }
        self.essence -= cost;
        self.upgrades.damage_bonus += 1;
        true
    }

    /// Purchase a starting weapon upgrade.
    pub fn upgrade_weapon(&mut self) -> bool {
        if self.upgrades.starting_weapon >= Upgrades::MAX_WEAPON {
            return false;
        }
        let cost = Upgrades::cost(self.upgrades.starting_weapon);
        if self.essence < cost {
            return false;
        }
        self.essence -= cost;
        self.upgrades.starting_weapon += 1;
        true
    }

    /// Purchase a loot luck upgrade.
    pub fn upgrade_loot_luck(&mut self) -> bool {
        if self.upgrades.loot_luck >= Upgrades::MAX_LUCK {
            return false;
        }
        let cost = Upgrades::cost(self.upgrades.loot_luck);
        if self.essence < cost {
            return false;
        }
        self.essence -= cost;
        self.upgrades.loot_luck += 1;
        true
    }
}

/// Save progression to file.
pub fn save_progression(prog: &Progression) -> Result<()> {
    let dir = save_dir();
    if !dir.exists() {
        fs::create_dir_all(&dir).context("Failed to create save directory")?;
    }
    let path = progression_path();
    let json = serde_json::to_string_pretty(prog).context("Failed to serialize progression")?;
    fs::write(&path, json).context("Failed to write progression file")?;
    Ok(())
}

/// Load progression from file.
pub fn load_progression() -> Result<Progression> {
    let path = progression_path();
    if !path.exists() {
        return Ok(Progression::new());
    }
    let json = fs::read_to_string(&path).context("Failed to read progression file")?;
    let prog: Progression = serde_json::from_str(&json).context("Failed to parse progression")?;
    Ok(prog)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_progression() {
        let prog = Progression::new();
        assert_eq!(prog.total_runs, 0);
        assert_eq!(prog.essence, 0);
        assert!(prog.unlocked_classes.contains("CodeWarrior"));
    }

    #[test]
    fn test_complete_run_loss() {
        let mut prog = Progression::new();
        prog.complete_run(false, 10, 3, 50);

        assert_eq!(prog.total_runs, 1);
        assert_eq!(prog.victories, 0);
        assert_eq!(prog.total_kills, 10);
        assert_eq!(prog.total_rooms, 3);
        // 10 kills + 15 rooms = 25 essence
        assert_eq!(prog.essence, 25);
    }

    #[test]
    fn test_complete_run_victory() {
        let mut prog = Progression::new();
        prog.complete_run(true, 20, 5, 100);

        assert_eq!(prog.total_runs, 1);
        assert_eq!(prog.victories, 1);
        // 20 kills + 25 rooms + 20 victory = 65 essence
        assert_eq!(prog.essence, 65);
        assert_eq!(prog.fastest_victory, Some(100));
    }

    #[test]
    fn test_fastest_victory_tracking() {
        let mut prog = Progression::new();
        prog.complete_run(true, 10, 5, 100);
        assert_eq!(prog.fastest_victory, Some(100));

        prog.complete_run(true, 10, 5, 80);
        assert_eq!(prog.fastest_victory, Some(80));

        prog.complete_run(true, 10, 5, 90);
        assert_eq!(prog.fastest_victory, Some(80)); // Didn't beat best
    }

    #[test]
    fn test_upgrade_hp() {
        let mut prog = Progression::new();
        prog.essence = 100;

        assert!(prog.upgrade_hp()); // Cost 10
        assert_eq!(prog.upgrades.hp_bonus, 1);
        assert_eq!(prog.upgrades.bonus_hp(), 5);
        assert_eq!(prog.essence, 90);

        assert!(prog.upgrade_hp()); // Cost 25
        assert_eq!(prog.upgrades.hp_bonus, 2);
        assert_eq!(prog.upgrades.bonus_hp(), 10);
        assert_eq!(prog.essence, 65);
    }

    #[test]
    fn test_upgrade_insufficient_essence() {
        let mut prog = Progression::new();
        prog.essence = 5;

        assert!(!prog.upgrade_hp()); // Costs 10, we have 5
        assert_eq!(prog.upgrades.hp_bonus, 0);
        assert_eq!(prog.essence, 5);
    }

    #[test]
    fn test_upgrade_max_level() {
        let mut prog = Progression::new();
        prog.essence = 10000;
        prog.upgrades.hp_bonus = Upgrades::MAX_HP;

        assert!(!prog.upgrade_hp()); // Already maxed
    }

    #[test]
    fn test_upgrade_damage() {
        let mut prog = Progression::new();
        prog.essence = 100;

        assert!(prog.upgrade_damage());
        assert_eq!(prog.upgrades.damage_bonus, 1);
        assert_eq!(prog.upgrades.bonus_damage(), 1);
    }

    #[test]
    fn test_upgrade_energy() {
        let mut prog = Progression::new();
        prog.essence = 100;

        assert!(prog.upgrade_energy());
        assert_eq!(prog.upgrades.energy_bonus, 1);
        assert_eq!(prog.upgrades.bonus_energy(), 2);
    }

    #[test]
    fn test_upgrade_loot_luck() {
        let mut prog = Progression::new();
        prog.essence = 100;

        assert!(prog.upgrade_loot_luck());
        assert_eq!(prog.upgrades.loot_luck, 1);
        assert_eq!(prog.upgrades.loot_luck_bonus(), 5);
    }

    #[test]
    fn test_unlock_class() {
        let mut prog = Progression::new();
        prog.essence = 50;

        let class = PlayerClass::MeetingSurvivor;
        assert!(!prog.is_class_unlocked(&class));

        assert!(prog.unlock_class(&class, 30));
        assert!(prog.is_class_unlocked(&class));
        assert_eq!(prog.essence, 20);

        // Can't unlock again
        assert!(!prog.unlock_class(&class, 30));
    }

    #[test]
    fn test_best_rooms_tracking() {
        let mut prog = Progression::new();
        prog.complete_run(false, 5, 3, 50);
        assert_eq!(prog.best_rooms, 3);

        prog.complete_run(false, 5, 5, 50);
        assert_eq!(prog.best_rooms, 5);

        prog.complete_run(false, 5, 4, 50);
        assert_eq!(prog.best_rooms, 5); // Didn't beat best
    }

    #[test]
    fn test_upgrade_costs() {
        assert_eq!(Upgrades::cost(0), 10);
        assert_eq!(Upgrades::cost(1), 25);
        assert_eq!(Upgrades::cost(2), 50);
        assert_eq!(Upgrades::cost(3), 100);
        assert_eq!(Upgrades::cost(4), 200);
        assert_eq!(Upgrades::cost(5), 500);
    }
}
