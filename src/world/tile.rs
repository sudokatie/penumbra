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

/// A single tile in the dungeon.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Tile {
    Floor,
    Wall,
    Door(Direction),
    Exit,
    Entrance,
}

impl Tile {
    /// Check if entities can walk on this tile.
    pub fn is_walkable(&self) -> bool {
        matches!(self, Tile::Floor | Tile::Door(_) | Tile::Exit | Tile::Entrance)
    }

    /// Check if this tile blocks vision.
    pub fn is_blocking(&self) -> bool {
        matches!(self, Tile::Wall)
    }

    /// Get the ASCII symbol for this tile.
    pub fn symbol(&self) -> char {
        match self {
            Tile::Floor => '.',
            Tile::Wall => '#',
            Tile::Door(_) => '+',
            Tile::Exit => '>',
            Tile::Entrance => '<',
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
