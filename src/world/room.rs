//! Room structure and generation.

use chrono::NaiveDate;
use rand::Rng;
use serde::{Deserialize, Serialize};

use crate::entity::{Enemy, EnemyType};
use crate::git::CommitData;
use crate::item::{Item, ItemEffect, ItemType, Rarity};

use super::{RoomType, Tile};

/// A single room in the dungeon.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Room {
    pub id: usize,
    pub tiles: Vec<Vec<Tile>>,
    pub width: u8,
    pub height: u8,
    pub enemies: Vec<Enemy>,
    pub items: Vec<Item>,
    pub source_date: NaiveDate,
    pub source_commits: Vec<CommitData>,
    pub room_type: RoomType,
    pub cleared: bool,
}

impl Room {
    /// Create a new room with floor tiles.
    pub fn new(id: usize, width: u8, height: u8, room_type: RoomType, date: NaiveDate) -> Self {
        let tiles = vec![vec![Tile::Floor; width as usize]; height as usize];
        Self {
            id,
            tiles,
            width,
            height,
            enemies: Vec::new(),
            items: Vec::new(),
            source_date: date,
            source_commits: Vec::new(),
            room_type,
            cleared: false,
        }
    }

    /// Check if a position is walkable.
    pub fn is_walkable(&self, x: i32, y: i32) -> bool {
        if x < 0 || y < 0 {
            return false;
        }
        let (ux, uy) = (x as usize, y as usize);
        if ux >= self.width as usize || uy >= self.height as usize {
            return false;
        }
        self.tiles[uy][ux].is_walkable()
    }

    /// Get tile at position.
    pub fn get_tile(&self, x: i32, y: i32) -> Option<&Tile> {
        if x < 0 || y < 0 {
            return None;
        }
        let (ux, uy) = (x as usize, y as usize);
        self.tiles.get(uy).and_then(|row| row.get(ux))
    }

    /// Set tile at position.
    pub fn set_tile(&mut self, x: i32, y: i32, tile: Tile) {
        if x >= 0 && y >= 0 {
            let (ux, uy) = (x as usize, y as usize);
            if uy < self.tiles.len() && ux < self.tiles[uy].len() {
                self.tiles[uy][ux] = tile;
            }
        }
    }

    /// Get enemy at position.
    pub fn get_enemy_at(&self, x: i32, y: i32) -> Option<&Enemy> {
        self.enemies.iter().find(|e| e.x == x && e.y == y)
    }

    /// Get item at position.
    pub fn get_item_at(&self, x: i32, y: i32) -> Option<&Item> {
        self.items.iter().find(|i| i.x == x && i.y == y)
    }

    /// Check if room is cleared of enemies.
    pub fn is_cleared(&self) -> bool {
        self.enemies.is_empty() || self.cleared
    }

    /// Get walkable positions not occupied by enemies or items.
    fn get_free_positions(&self) -> Vec<(i32, i32)> {
        let mut positions = Vec::new();
        for y in 1..(self.height as i32 - 1) {
            for x in 1..(self.width as i32 - 1) {
                if self.is_walkable(x, y)
                    && self.get_enemy_at(x, y).is_none()
                    && self.get_item_at(x, y).is_none()
                {
                    positions.push((x, y));
                }
            }
        }
        positions
    }

    /// Determine enemy type from commit data.
    /// Spec: Bug (<20 lines), Regression (revert), TechDebt (old code), MergeConflict (merge)
    fn enemy_type_from_commit(commit: &CommitData) -> EnemyType {
        // Merge commits spawn MergeConflict
        if commit.is_merge {
            return EnemyType::MergeConflict;
        }
        
        let msg = commit.message.to_lowercase();
        
        // Revert commits spawn Regression
        if msg.contains("revert") || msg.contains("rollback") {
            return EnemyType::Regression;
        }
        
        // Refactor/cleanup/debt commits spawn TechDebt
        if msg.contains("debt") || msg.contains("refactor") || msg.contains("cleanup") {
            return EnemyType::TechDebt;
        }
        
        // Small commits (<20 lines) spawn Bug per spec
        if commit.lines_changed() < 20 {
            return EnemyType::Bug;
        }
        
        // Larger commits also spawn TechDebt (accumulating complexity)
        EnemyType::TechDebt
    }

    /// Spawn enemies based on commits.
    ///
    /// Count: min(commits.len(), room_size/4)
    /// Type based on commit message keywords.
    /// Sanctuary rooms have no enemies.
    pub fn spawn_enemies<R: Rng>(&mut self, commits: &[CommitData], rng: &mut R) {
        // Sanctuary rooms are safe - no enemies spawn
        if self.room_type == RoomType::Sanctuary {
            return;
        }
        
        let room_size = (self.width as usize * self.height as usize) / 4;
        let count = commits.len().min(room_size).min(10); // Cap at 10 enemies

        let mut positions = self.get_free_positions();
        if positions.is_empty() || count == 0 {
            return;
        }

        for commit in commits.iter().take(count) {
            if positions.is_empty() {
                break;
            }
            let pos_idx = rng.gen_range(0..positions.len());
            let (x, y) = positions.remove(pos_idx);
            let enemy_type = Self::enemy_type_from_commit(commit);
            let enemy = Enemy::new(enemy_type, x, y, &commit.hash);
            self.enemies.push(enemy);
        }
    }

    /// Determine rarity from commit size.
    fn rarity_from_lines(lines: u32) -> Rarity {
        if lines > 500 {
            Rarity::Legendary
        } else if lines > 200 {
            Rarity::Rare
        } else if lines > 50 {
            Rarity::Uncommon
        } else {
            Rarity::Common
        }
    }

    /// Create an item based on commit characteristics.
    fn item_from_commit(commit: &CommitData) -> Item {
        let msg = commit.message.to_lowercase();
        let rarity = Self::rarity_from_lines(commit.lines_changed());

        // Determine item based on commit type
        let (name, item_type, effect) = if msg.contains("doc") || msg.contains("readme") {
            // Doc commits: Map scrolls
            ("Map Scroll".to_string(), ItemType::Scroll, ItemEffect::RevealMap)
        } else if msg.contains("test") {
            // Test commits: Healing
            let heal = match rarity {
                Rarity::Common => 10,
                Rarity::Uncommon => 20,
                Rarity::Rare => 35,
                Rarity::Legendary => 50,
            };
            ("Health Potion".to_string(), ItemType::Consumable, ItemEffect::Heal(heal))
        } else if msg.contains("config") || msg.contains("settings") {
            // Config commits: Buffs
            let amount = match rarity {
                Rarity::Common => 2,
                Rarity::Uncommon => 4,
                Rarity::Rare => 6,
                Rarity::Legendary => 10,
            };
            (
                "Focus Crystal".to_string(),
                ItemType::Consumable,
                ItemEffect::Buff(crate::item::Stat::Focus, amount, 5),
            )
        } else {
            // Default: energy restoration
            let energy = match rarity {
                Rarity::Common => 5,
                Rarity::Uncommon => 10,
                Rarity::Rare => 20,
                Rarity::Legendary => 30,
            };
            ("Energy Vial".to_string(), ItemType::Consumable, ItemEffect::RestoreEnergy(energy))
        };

        Item::new(name, item_type, effect, rarity).from_commit(&commit.hash)
    }

    /// Spawn items based on commits and room type.
    ///
    /// - Doc commits: Map scrolls
    /// - Test commits: Healing items
    /// - Config commits: Buff items
    /// - Treasure rooms: 2-3 items
    pub fn spawn_items<R: Rng>(&mut self, commits: &[CommitData], rng: &mut R) {
        let mut positions = self.get_free_positions();
        if positions.is_empty() {
            return;
        }

        // Treasure rooms get extra items
        let item_count = match self.room_type {
            RoomType::Treasure => rng.gen_range(2..=3),
            _ => {
                // Normal rooms: ~1 item per 3-4 commits
                let base = commits.len() / 4;
                base.clamp(1, 3)
            }
        };

        for commit in commits.iter().take(item_count) {
            if positions.is_empty() {
                break;
            }
            let pos_idx = rng.gen_range(0..positions.len());
            let (x, y) = positions.remove(pos_idx);
            let item = Self::item_from_commit(commit).at(x, y);
            self.items.push(item);
        }
    }
}
