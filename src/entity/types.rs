//! Entity type definitions.

use serde::{Deserialize, Serialize};

use crate::git::CommitData;

/// Player class determines starting stats.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PlayerClass {
    CodeWarrior,
    MeetingSurvivor,
    InboxKnight,
    Wanderer,
}

impl PlayerClass {
    /// Auto-detect player class from git patterns.
    ///
    /// - CodeWarrior: >100 commits in period
    /// - MeetingSurvivor: commits spread evenly across days
    /// - InboxKnight: high test file ratio
    /// - Wanderer: default
    pub fn detect(commits: &[CommitData]) -> Self {
        if commits.is_empty() {
            return PlayerClass::Wanderer;
        }

        // CodeWarrior: >100 commits
        if commits.len() > 100 {
            return PlayerClass::CodeWarrior;
        }

        // InboxKnight: high test file ratio (check for "test" in commit messages)
        let test_commits = commits.iter()
            .filter(|c| c.message.to_lowercase().contains("test"))
            .count();
        let test_ratio = test_commits as f32 / commits.len() as f32;
        if test_ratio > 0.3 {
            return PlayerClass::InboxKnight;
        }

        // MeetingSurvivor: commits spread evenly across days
        // Calculate unique days with commits
        let unique_days: std::collections::HashSet<_> = commits.iter()
            .map(|c| c.date_naive())
            .collect();
        let days_with_commits = unique_days.len();
        let total_days = if commits.len() >= 2 {
            let first = commits.first().unwrap().date_naive();
            let last = commits.last().unwrap().date_naive();
            (last - first).num_days().unsigned_abs() as usize + 1
        } else {
            1
        };

        // If commits on >60% of days, they're a meeting survivor (always around)
        if total_days > 5 && days_with_commits as f32 / total_days as f32 > 0.6 {
            return PlayerClass::MeetingSurvivor;
        }

        PlayerClass::Wanderer
    }
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
