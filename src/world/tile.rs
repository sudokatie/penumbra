//! Tile types and properties.

use serde::{Deserialize, Serialize};

/// Direction for doors and movement.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Direction {
    North,
    South,
    East,
    West,
}

/// Door state (open or closed).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum DoorState {
    #[default]
    Closed,
    Open,
}

/// A single tile in the dungeon.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Tile {
    Floor,
    Wall,
    Door(Direction, DoorState),
    Exit,
    Entrance,
    HealingZone, // Sanctuary room special tile
}

impl Tile {
    /// Check if entities can walk on this tile.
    pub fn is_walkable(&self) -> bool {
        match self {
            Tile::Door(_, DoorState::Closed) => false,
            Tile::Floor | Tile::Door(_, DoorState::Open) | Tile::Exit | Tile::Entrance | Tile::HealingZone => true,
            Tile::Wall => false,
        }
    }

    /// Check if this tile blocks vision.
    pub fn is_blocking(&self) -> bool {
        matches!(self, Tile::Wall | Tile::Door(_, DoorState::Closed))
    }

    /// Get the ASCII symbol for this tile.
    pub fn symbol(&self) -> char {
        match self {
            Tile::Floor => '.',
            Tile::Wall => '#',
            Tile::Door(_, DoorState::Closed) => '+',
            Tile::Door(_, DoorState::Open) => '/',
            Tile::Exit => '>',
            Tile::Entrance => '<',
            Tile::HealingZone => '*',
        }
    }

    /// Check if this is a door tile.
    pub fn is_door(&self) -> bool {
        matches!(self, Tile::Door(_, _))
    }

    /// Toggle door state.
    pub fn toggle_door(&mut self) {
        if let Tile::Door(dir, state) = self {
            *self = Tile::Door(*dir, match state {
                DoorState::Open => DoorState::Closed,
                DoorState::Closed => DoorState::Open,
            });
        }
    }
}

impl Direction {
    /// Get the opposite direction.
    pub fn opposite(&self) -> Direction {
        match self {
            Direction::North => Direction::South,
            Direction::South => Direction::North,
            Direction::East => Direction::West,
            Direction::West => Direction::East,
        }
    }

    /// Get delta (dx, dy) for this direction.
    pub fn delta(&self) -> (i32, i32) {
        match self {
            Direction::North => (0, -1),
            Direction::South => (0, 1),
            Direction::East => (1, 0),
            Direction::West => (-1, 0),
        }
    }
}
