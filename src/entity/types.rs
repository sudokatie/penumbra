//! Entity type definitions.

use serde::{Deserialize, Serialize};

/// Player class determines starting stats.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PlayerClass {
    CodeWarrior,
    MeetingSurvivor,
    InboxKnight,
    Wanderer,
}

/// Enemy type determines behavior and stats.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EnemyType {
    Bug,
    Regression,
    TechDebt,
    MergeConflict,
}

impl EnemyType {
    /// Base HP for this enemy type.
    pub fn base_hp(&self) -> i32 {
        match self {
            EnemyType::Bug => 10,
            EnemyType::Regression => 20,
            EnemyType::TechDebt => 30,
            EnemyType::MergeConflict => 50,
        }
    }

    /// Base damage for this enemy type.
    pub fn base_damage(&self) -> i32 {
        match self {
            EnemyType::Bug => 3,
            EnemyType::Regression => 5,
            EnemyType::TechDebt => 4,
            EnemyType::MergeConflict => 8,
        }
    }

    /// ASCII symbol for this enemy.
    pub fn symbol(&self) -> char {
        match self {
            EnemyType::Bug => 'B',
            EnemyType::Regression => 'R',
            EnemyType::TechDebt => 'D',
            EnemyType::MergeConflict => 'M',
        }
    }
}
