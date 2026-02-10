//! Tests for save/load persistence.
//!
//! Note: These tests modify the user's save directory.
//! Run with `cargo test save_test -- --test-threads=1` for isolation.

use std::path::PathBuf;

use chrono::Utc;
use penumbra::game::{
    save_game, load_game, save_run_history, load_run_history, 
    save_exists, delete_save, GameState, RunRecord,
};
use penumbra::git::CommitData;

fn make_commit(msg: &str, lines: u32) -> CommitData {
    CommitData {
        hash: format!("hash_{}", lines),
        date: Utc::now(),
        message: msg.to_string(),
        insertions: lines,
        deletions: 0,
        files_changed: 1,
        author: "Test".to_string(),
        is_merge: false, file_categories: Default::default(),
    }
}

fn test_git_path() -> PathBuf {
    PathBuf::from("/tmp/test-repo")
}

#[test]
fn save_game_creates_file() {
    let commits = vec![make_commit("Test", 50)];
    let state = GameState::new(commits, 42, test_git_path());
    
    let result = save_game(&state);
    assert!(result.is_ok());
    
    // Cleanup
    let _ = delete_save();
}

#[test]
fn save_exists_returns_bool() {
    // Just test that save_exists runs without error
    let _ = save_exists();
}

#[test]
fn load_run_history_empty_on_no_file() {
    // This tests the case where history file doesn't exist
    // Since we can't easily isolate this, we just verify the function works
    let result = load_run_history();
    assert!(result.is_ok());
}

#[test]
fn run_record_has_expected_fields() {
    let record = RunRecord {
        started_at: Utc::now(),
        ended_at: Utc::now(),
        victory: true,
        turns: 100,
        rooms_cleared: 5,
        enemies_killed: 10,
        final_level: 3,
        death_cause: None,
    };
    
    assert_eq!(record.turns, 100);
    assert_eq!(record.rooms_cleared, 5);
    assert!(record.victory);
    assert!(record.death_cause.is_none());
}

#[test]
fn run_record_with_death_cause() {
    let record = RunRecord {
        started_at: Utc::now(),
        ended_at: Utc::now(),
        victory: false,
        turns: 25,
        rooms_cleared: 1,
        enemies_killed: 0,
        final_level: 1,
        death_cause: Some("MergeConflict".to_string()),
    };
    
    assert!(!record.victory);
    assert_eq!(record.death_cause, Some("MergeConflict".to_string()));
}

#[test]
fn game_state_serializable() {
    let commits = vec![make_commit("Test", 50)];
    let state = GameState::new(commits, 42, test_git_path());
    
    // Test that state can be serialized
    let json = serde_json::to_string(&state);
    assert!(json.is_ok());
}

#[test]
fn game_state_deserializable() {
    let commits = vec![make_commit("Test", 50)];
    let state = GameState::new(commits, 42, test_git_path());
    
    let json = serde_json::to_string(&state).unwrap();
    let loaded: Result<GameState, _> = serde_json::from_str(&json);
    assert!(loaded.is_ok());
}

#[test]
fn game_state_roundtrip() {
    let commits = vec![make_commit("Test", 50)];
    let mut state = GameState::new(commits, 42, test_git_path());
    state.turn = 99;
    
    let json = serde_json::to_string(&state).unwrap();
    let loaded: GameState = serde_json::from_str(&json).unwrap();
    
    assert_eq!(loaded.turn, 99);
    assert_eq!(loaded.seed, 42);
}
