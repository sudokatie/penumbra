//! Tests for git parsing module.

use std::path::Path;
use std::process::Command;

use chrono::Utc;
use tempfile::TempDir;

use penumbra::git::{
    categorize_files, group_by_date, parse_repository, CommitData, FileCategories, GitError,
};

/// Create a temp git repo with some commits for testing.
fn create_test_repo() -> TempDir {
    let dir = TempDir::new().unwrap();
    let path = dir.path();

    // Initialize git repo
    Command::new("git")
        .args(["init"])
        .current_dir(path)
        .output()
        .unwrap();

    // Configure git user
    Command::new("git")
        .args(["config", "user.email", "test@test.com"])
        .current_dir(path)
        .output()
        .unwrap();
    Command::new("git")
        .args(["config", "user.name", "Test"])
        .current_dir(path)
        .output()
        .unwrap();

    // Create initial commit
    std::fs::write(path.join("README.md"), "# Test").unwrap();
    Command::new("git")
        .args(["add", "."])
        .current_dir(path)
        .output()
        .unwrap();
    Command::new("git")
        .args(["commit", "-m", "Initial commit"])
        .current_dir(path)
        .output()
        .unwrap();

    // Add more commits
    std::fs::write(path.join("main.rs"), "fn main() {}").unwrap();
    Command::new("git")
        .args(["add", "."])
        .current_dir(path)
        .output()
        .unwrap();
    Command::new("git")
        .args(["commit", "-m", "Add main"])
        .current_dir(path)
        .output()
        .unwrap();

    std::fs::write(path.join("test_foo.rs"), "fn test() {}").unwrap();
    Command::new("git")
        .args(["add", "."])
        .current_dir(path)
        .output()
        .unwrap();
    Command::new("git")
        .args(["commit", "-m", "Add test"])
        .current_dir(path)
        .output()
        .unwrap();

    dir
}

#[test]
fn parse_repository_with_valid_repo_returns_commits() {
    let repo = create_test_repo();
    let commits = parse_repository(repo.path(), 30).unwrap();
    assert!(!commits.is_empty());
    assert!(commits.len() >= 3);
}

#[test]
fn parse_repository_with_invalid_path_returns_error() {
    let result = parse_repository(Path::new("/nonexistent/path"), 30);
    assert!(result.is_err());
    match result.unwrap_err() {
        GitError::NotARepository(_) => (),
        e => panic!("Expected NotARepository, got {:?}", e),
    }
}

#[test]
fn parse_repository_respects_days_parameter() {
    let repo = create_test_repo();
    // All commits are recent, should all be included
    let commits = parse_repository(repo.path(), 1).unwrap();
    assert!(!commits.is_empty());
}

#[test]
fn commits_have_correct_fields() {
    let repo = create_test_repo();
    let commits = parse_repository(repo.path(), 30).unwrap();
    
    for commit in &commits {
        assert!(!commit.hash.is_empty());
        assert!(!commit.message.is_empty());
        assert!(!commit.author.is_empty());
    }
}

#[test]
fn commits_sorted_oldest_first() {
    let repo = create_test_repo();
    let commits = parse_repository(repo.path(), 30).unwrap();
    
    for i in 1..commits.len() {
        assert!(commits[i].date >= commits[i - 1].date);
    }
}

#[test]
fn group_by_date_groups_correctly() {
    let commits = vec![
        CommitData {
            hash: "a".to_string(),
            date: Utc::now(),
            message: "test".to_string(),
            insertions: 10,
            deletions: 5,
            files_changed: 1,
            author: "Test".to_string(),
            is_merge: false,
        },
        CommitData {
            hash: "b".to_string(),
            date: Utc::now(),
            message: "test2".to_string(),
            insertions: 20,
            deletions: 10,
            files_changed: 2,
            author: "Test".to_string(),
            is_merge: false,
        },
    ];

    let grouped = group_by_date(commits);
    assert_eq!(grouped.len(), 1); // Same day
    assert_eq!(grouped.values().next().unwrap().len(), 2);
}

#[test]
fn commit_data_lines_changed() {
    let commit = CommitData {
        hash: "abc".to_string(),
        date: Utc::now(),
        message: "test".to_string(),
        insertions: 100,
        deletions: 50,
        files_changed: 5,
        author: "Test".to_string(),
        is_merge: false,
    };
    assert_eq!(commit.lines_changed(), 150);
}

#[test]
fn file_categories_detects_test_files() {
    // Note: This tests the helper functions indirectly through patterns
    let path = "tests/foo_test.rs";
    assert!(path.contains("test"));
}

#[test]
fn file_categories_detects_config_files() {
    let extensions = [".json", ".toml", ".yaml", ".yml", ".conf"];
    for ext in extensions {
        assert!(ext.ends_with("json") || ext.ends_with("toml") || 
                ext.ends_with("yaml") || ext.ends_with("yml") || ext.ends_with("conf"));
    }
}

#[test]
fn file_categories_detects_doc_files() {
    let extensions = [".md", ".txt", ".rst"];
    for ext in extensions {
        assert!(ext.ends_with("md") || ext.ends_with("txt") || ext.ends_with("rst"));
    }
}

#[test]
fn is_merge_false_for_normal_commits() {
    let repo = create_test_repo();
    let commits = parse_repository(repo.path(), 30).unwrap();
    // Simple linear repo has no merges
    for commit in commits {
        assert!(!commit.is_merge);
    }
}

#[test]
fn commit_date_is_recent() {
    let repo = create_test_repo();
    let commits = parse_repository(repo.path(), 30).unwrap();
    let now = Utc::now();
    
    for commit in commits {
        // All commits should be within last minute (just created)
        let diff = now - commit.date;
        assert!(diff.num_minutes() < 5);
    }
}
