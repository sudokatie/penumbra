//! Git data types.

use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Data extracted from a single git commit.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommitData {
    pub hash: String,
    pub date: DateTime<Utc>,
    pub message: String,
    pub insertions: u32,
    pub deletions: u32,
    pub files_changed: u32,
    pub author: String,
    pub is_merge: bool,
}

/// Statistics for a commit diff.
#[derive(Debug, Clone, Default)]
pub struct CommitStats {
    pub insertions: u32,
    pub deletions: u32,
    pub files_changed: u32,
}

/// Categorized file counts from a commit.
#[derive(Debug, Clone, Default)]
pub struct FileCategories {
    pub test_files: u32,
    pub config_files: u32,
    pub doc_files: u32,
    pub other_files: u32,
}

/// Errors that can occur during git parsing.
#[derive(Error, Debug)]
pub enum GitError {
    #[error("Not a git repository: {0}")]
    NotARepository(String),

    #[error("Failed to open repository: {0}")]
    OpenFailed(String),

    #[error("Failed to walk commits: {0}")]
    WalkFailed(String),

    #[error("No commits found in last {0} days")]
    NoCommits(u32),

    #[error("Git error: {0}")]
    Git(#[from] git2::Error),
}

impl CommitData {
    /// Total lines changed (insertions + deletions).
    pub fn lines_changed(&self) -> u32 {
        self.insertions + self.deletions
    }

    /// Get the date portion only.
    pub fn date_naive(&self) -> NaiveDate {
        self.date.date_naive()
    }
}
