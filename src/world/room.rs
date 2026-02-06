//! Room structure and generation.

use chrono::NaiveDate;
use serde::{Deserialize, Serialize};

use crate::entity::Enemy;
use crate::git::CommitData;
use crate::item::Item;

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
}
