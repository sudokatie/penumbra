//! Item struct.

use serde::{Deserialize, Serialize};

use super::{ItemEffect, ItemType, Rarity};

/// An item in the dungeon.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Item {
    pub name: String,
    pub item_type: ItemType,
    pub effect: ItemEffect,
    pub rarity: Rarity,
    pub source_commit: Option<String>,
    pub x: i32,
    pub y: i32,
}

impl Item {
    /// Create a new item.
    pub fn new(
        name: impl Into<String>,
        item_type: ItemType,
        effect: ItemEffect,
        rarity: Rarity,
    ) -> Self {
        Self {
            name: name.into(),
            item_type,
            effect,
            rarity,
            source_commit: None,
            x: 0,
            y: 0,
        }
    }

    /// Set position.
    pub fn at(mut self, x: i32, y: i32) -> Self {
        self.x = x;
        self.y = y;
        self
    }

    /// Set source commit.
    pub fn from_commit(mut self, hash: &str) -> Self {
        self.source_commit = Some(hash.to_string());
        self
    }
}
