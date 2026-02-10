//! Git repository parsing.

use std::collections::BTreeMap;
use std::path::Path;

use chrono::{Duration, NaiveDate, Utc};
use git2::{Commit, Diff, DiffOptions, Repository};

use super::types::{CommitData, CommitStats, FileCategories, GitError};

/// Parse a git repository and extract commit data.
///
/// Returns commits from the last `days` days, sorted by date (oldest first).
pub fn parse_repository(path: &Path, days: u32) -> Result<Vec<CommitData>, GitError> {
    let repo = Repository::open(path).map_err(|e| {
        if e.code() == git2::ErrorCode::NotFound {
            GitError::NotARepository(path.display().to_string())
        } else {
            GitError::OpenFailed(e.message().to_string())
        }
    })?;

    let cutoff = Utc::now() - Duration::days(days as i64);
    let mut revwalk = repo.revwalk().map_err(|e| GitError::WalkFailed(e.message().to_string()))?;
    revwalk.push_head().map_err(|e| GitError::WalkFailed(e.message().to_string()))?;

    let mut commits = Vec::new();

    for oid in revwalk {
        let oid = oid.map_err(|e| GitError::WalkFailed(e.message().to_string()))?;
        let commit = repo.find_commit(oid)?;

        let time = commit.time();
        let datetime = chrono::DateTime::from_timestamp(time.seconds(), 0)
            .unwrap_or_else(Utc::now);

        if datetime < cutoff {
            break;
        }

        let (stats, categories) = get_commit_stats_and_categories(&repo, &commit)?;
        let is_merge = commit.parent_count() > 1;

        commits.push(CommitData {
            hash: oid.to_string(),
            date: datetime,
            message: commit.message().unwrap_or("").to_string(),
            insertions: stats.insertions,
            deletions: stats.deletions,
            files_changed: stats.files_changed,
            author: commit.author().name().unwrap_or("unknown").to_string(),
            is_merge,
            file_categories: categories,
        });
    }

    if commits.is_empty() {
        return Err(GitError::NoCommits(days));
    }

    // Reverse to get oldest first
    commits.reverse();
    Ok(commits)
}

/// Get statistics and file categories for a single commit.
pub fn get_commit_stats_and_categories(
    repo: &Repository,
    commit: &Commit,
) -> Result<(CommitStats, FileCategories), GitError> {
    let tree = commit.tree()?;

    let parent_tree = if commit.parent_count() > 0 {
        Some(commit.parent(0)?.tree()?)
    } else {
        None
    };

    let mut opts = DiffOptions::new();
    let diff = repo.diff_tree_to_tree(parent_tree.as_ref(), Some(&tree), Some(&mut opts))?;

    let stats = diff.stats()?;
    let categories = categorize_files(&diff);

    Ok((
        CommitStats {
            insertions: stats.insertions() as u32,
            deletions: stats.deletions() as u32,
            files_changed: stats.files_changed() as u32,
        },
        categories,
    ))
}

/// Get statistics for a single commit (for backward compatibility).
pub fn get_commit_stats(repo: &Repository, commit: &Commit) -> Result<CommitStats, GitError> {
    let (stats, _) = get_commit_stats_and_categories(repo, commit)?;
    Ok(stats)
}

/// Group commits by date.
pub fn group_by_date(commits: Vec<CommitData>) -> BTreeMap<NaiveDate, Vec<CommitData>> {
    let mut grouped: BTreeMap<NaiveDate, Vec<CommitData>> = BTreeMap::new();

    for commit in commits {
        let date = commit.date_naive();
        grouped.entry(date).or_default().push(commit);
    }

    grouped
}

/// Categorize files in a diff by type.
pub fn categorize_files(diff: &Diff) -> FileCategories {
    let mut categories = FileCategories::default();

    for delta in diff.deltas() {
        let path = delta.new_file().path().or_else(|| delta.old_file().path());

        if let Some(path) = path {
            let path_str = path.to_string_lossy().to_lowercase();

            if is_test_file(&path_str) {
                categories.test_files += 1;
            } else if is_config_file(&path_str) {
                categories.config_files += 1;
            } else if is_doc_file(&path_str) {
                categories.doc_files += 1;
            } else {
                categories.other_files += 1;
            }
        }
    }

    categories
}

fn is_test_file(path: &str) -> bool {
    path.contains("test") || path.contains("spec") || path.starts_with("tests/")
}

fn is_config_file(path: &str) -> bool {
    path.ends_with(".json")
        || path.ends_with(".toml")
        || path.ends_with(".yaml")
        || path.ends_with(".yml")
        || path.ends_with(".conf")
        || path.ends_with(".config")
}

fn is_doc_file(path: &str) -> bool {
    path.ends_with(".md") || path.ends_with(".txt") || path.ends_with(".rst")
}
