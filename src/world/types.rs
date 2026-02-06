//! World types.

use serde::{Deserialize, Serialize};

use super::Room;

/// Type of room based on commit data.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RoomType {
    /// Standard room.
    Normal,
    /// Test file room - healing zone.
    Sanctuary,
    /// Config file room - extra loot.
    Treasure,
    /// Merge commit room - boss encounter.
    Boss,
}

impl RoomType {
    /// Get display name for this room type.
    pub fn name(&self) -> &'static str {
        match self {
            RoomType::Normal => "Room",
            RoomType::Sanctuary => "Sanctuary",
            RoomType::Treasure => "Treasury",
            RoomType::Boss => "Boss Chamber",
        }
    }
}

/// The complete game world.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct World {
    pub rooms: Vec<Room>,
    pub current_room: usize,
}

impl World {
    /// Create a new world with given rooms.
    pub fn new(rooms: Vec<Room>) -> Self {
        Self {
            rooms,
            current_room: 0,
        }
    }

    /// Get the current room.
    pub fn current(&self) -> Option<&Room> {
        self.rooms.get(self.current_room)
    }

    /// Get the current room mutably.
    pub fn current_mut(&mut self) -> Option<&mut Room> {
        self.rooms.get_mut(self.current_room)
    }

    /// Advance to the next room.
    pub fn next_room(&mut self) -> bool {
        if self.current_room + 1 < self.rooms.len() {
            self.current_room += 1;
            true
        } else {
            false
        }
    }

    /// Check if this is the last room.
    pub fn is_last_room(&self) -> bool {
        self.current_room + 1 >= self.rooms.len()
    }
}
