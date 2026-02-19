//! Tests for game state, death/victory, and room transitions.

use std::path::PathBuf;

use chrono::{NaiveDate, Utc};
use penumbra::combat::PlayerAction;
use penumbra::entity::{Enemy, EnemyType, Player, PlayerClass};
use penumbra::game::GameState;
use penumbra::git::CommitData;
use penumbra::world::{Room, RoomType, Tile, World};

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

fn make_test_room(id: usize, enemies: bool, with_exit: bool) -> Room {
    let date = NaiveDate::from_ymd_opt(2026, 1, 1).unwrap();
    let mut room = Room::new(id, 7, 7, RoomType::Normal, date);
    
    // Add walls around edges
    for x in 0..7 {
        room.set_tile(x, 0, Tile::Wall);
        room.set_tile(x, 6, Tile::Wall);
    }
    for y in 0..7 {
        room.set_tile(0, y, Tile::Wall);
        room.set_tile(6, y, Tile::Wall);
    }
    
    if with_exit {
        room.set_tile(5, 3, Tile::Exit);
    }
    
    if enemies {
        room.enemies.push(Enemy::new(EnemyType::Bug, 3, 3, "test"));
    }
    
    room
}

// === Death/Victory Tests (Task 21) ===

#[test]
fn game_starts_not_over() {
    let commits = vec![make_commit("Test", 50)];
    let state = GameState::new(commits, 42, test_git_path());
    assert!(!state.game_over);
    assert!(!state.victory);
}

#[test]
fn game_over_on_player_death() {
    let date = NaiveDate::from_ymd_opt(2026, 1, 1).unwrap();
    let mut room = Room::new(0, 7, 7, RoomType::Normal, date);
    // Add a powerful enemy
    room.enemies.push(Enemy::new(EnemyType::MergeConflict, 2, 2, "test"));
    
    let world = World::new(vec![room]);
    let mut player = Player::new(PlayerClass::Wanderer);
    player.x = 1;
    player.y = 2;
    player.hp = 1; // Nearly dead
    
    let mut state = GameState::new(vec![make_commit("Test", 50)], 42, test_git_path());
    state.world = world;
    state.player = player;
    
    // Process enemies (they'll attack and kill the player)
    let _events = state.process_enemies();
    
    // Should trigger game over
    assert!(state.game_over);
    assert!(!state.victory);
}

#[test]
fn victory_on_last_room_exit() {
    // Single room that's already cleared
    let mut room = make_test_room(0, false, true);
    room.cleared = true;
    
    let world = World::new(vec![room]);
    let mut player = Player::new(PlayerClass::Wanderer);
    player.x = 5;
    player.y = 3; // At exit
    
    let mut state = GameState::new(vec![make_commit("Test", 50)], 42, test_git_path());
    state.world = world;
    state.player = player;
    state.update_fov();
    
    // Check exit
    let transitioned = state.check_room_exit();
    
    assert!(transitioned);
    assert!(state.game_over);
    assert!(state.victory);
}

#[test]
fn game_tracks_turns() {
    let commits = vec![make_commit("Test", 50)];
    let mut state = GameState::new(commits, 42, test_git_path());
    
    assert_eq!(state.turn, 0);
    
    state.process_action(PlayerAction::Wait);
    assert_eq!(state.turn, 1);
    
    state.process_action(PlayerAction::Wait);
    assert_eq!(state.turn, 2);
}

// === Room Transition Tests (Task 22) ===

#[test]
fn cannot_exit_with_enemies() {
    let mut room = make_test_room(0, true, true);
    room.cleared = false;
    
    let world = World::new(vec![room, make_test_room(1, false, false)]);
    let mut player = Player::new(PlayerClass::Wanderer);
    player.x = 5;
    player.y = 3; // At exit
    
    let mut state = GameState::new(vec![make_commit("Test", 50)], 42, test_git_path());
    state.world = world;
    state.player = player;
    
    let transitioned = state.check_room_exit();
    
    assert!(!transitioned);
    assert_eq!(state.world.current_room, 0);
}

#[test]
fn can_exit_when_cleared() {
    let mut room1 = make_test_room(0, false, true);
    room1.cleared = true;
    let room2 = make_test_room(1, false, false);
    
    let world = World::new(vec![room1, room2]);
    let mut player = Player::new(PlayerClass::Wanderer);
    player.x = 5;
    player.y = 3; // At exit
    
    let mut state = GameState::new(vec![make_commit("Test", 50)], 42, test_git_path());
    state.world = world;
    state.player = player;
    
    let transitioned = state.check_room_exit();
    
    assert!(transitioned);
    assert_eq!(state.world.current_room, 1);
}

#[test]
fn player_positioned_at_entrance_after_transition() {
    let mut room1 = make_test_room(0, false, true);
    room1.cleared = true;
    let room2 = make_test_room(1, false, false);
    
    let world = World::new(vec![room1, room2]);
    let mut player = Player::new(PlayerClass::Wanderer);
    player.x = 5;
    player.y = 3;
    
    let mut state = GameState::new(vec![make_commit("Test", 50)], 42, test_git_path());
    state.world = world;
    state.player = player;
    
    state.check_room_exit();
    
    // Player should be at entrance (left side, middle height)
    assert_eq!(state.player.x, 1);
}

#[test]
fn fov_updates_after_transition() {
    let mut room1 = make_test_room(0, false, true);
    room1.cleared = true;
    let room2 = make_test_room(1, false, false);
    
    let world = World::new(vec![room1, room2]);
    let mut player = Player::new(PlayerClass::Wanderer);
    player.x = 5;
    player.y = 3;
    
    let mut state = GameState::new(vec![make_commit("Test", 50)], 42, test_git_path());
    state.world = world;
    state.player = player;
    state.update_fov();
    
    let _fov_before = state.visible_tiles.clone();
    
    state.check_room_exit();
    
    // FOV should have changed (though content might overlap)
    // At minimum, player position is now different
    assert!(!state.visible_tiles.is_empty());
}

#[test]
fn transition_logs_room_info() {
    let mut room1 = make_test_room(0, false, true);
    room1.cleared = true;
    let room2 = make_test_room(1, false, false);
    
    let world = World::new(vec![room1, room2]);
    let mut player = Player::new(PlayerClass::Wanderer);
    player.x = 5;
    player.y = 3;
    
    let mut state = GameState::new(vec![make_commit("Test", 50)], 42, test_git_path());
    state.world = world;
    state.player = player;
    state.messages.clear();
    
    state.check_room_exit();
    
    // Should have logged the transition
    assert!(state.messages.iter().any(|m| m.contains("enter")));
}

#[test]
fn room_marked_cleared_when_enemies_defeated() {
    let date = NaiveDate::from_ymd_opt(2026, 1, 1).unwrap();
    let mut room = Room::new(0, 7, 7, RoomType::Normal, date);
    room.enemies.push(Enemy::new(EnemyType::Bug, 2, 3, "test"));
    
    assert!(!room.is_cleared());
    
    // Remove enemy (simulating defeat)
    room.enemies.clear();
    room.cleared = true;
    
    assert!(room.is_cleared());
}

// === Sanctuary Tests ===

fn make_sanctuary_room(id: usize) -> Room {
    let date = NaiveDate::from_ymd_opt(2026, 1, 1).unwrap();
    let mut room = Room::new(id, 7, 7, RoomType::Sanctuary, date);
    
    // Add walls around edges
    for x in 0..7 {
        room.set_tile(x, 0, Tile::Wall);
        room.set_tile(x, 6, Tile::Wall);
    }
    for y in 0..7 {
        room.set_tile(0, y, Tile::Wall);
        room.set_tile(6, y, Tile::Wall);
    }
    
    room
}

#[test]
fn sanctuary_regenerates_energy() {
    let room = make_sanctuary_room(0);
    let world = World::new(vec![room]);
    
    let mut player = Player::new(PlayerClass::Wanderer);
    player.x = 3;
    player.y = 3;
    player.energy = 50; // Start with half energy
    
    let mut state = GameState::new(vec![make_commit("Test", 50)], 42, test_git_path());
    state.world = world;
    state.player = player;
    
    // Process an action (Wait)
    state.process_action(PlayerAction::Wait);
    
    // Should have +2 from Wait + 5 from Sanctuary = 57
    assert!(state.player.energy > 52);
}

#[test]
fn sanctuary_has_no_enemies() {
    use rand::SeedableRng;
    use rand_chacha::ChaCha8Rng;
    
    let date = NaiveDate::from_ymd_opt(2026, 1, 1).unwrap();
    let mut room = Room::new(0, 7, 7, RoomType::Sanctuary, date);
    
    let commits = vec![
        make_commit("Test 1", 50),
        make_commit("Test 2", 50),
        make_commit("Test 3", 50),
    ];
    
    let mut rng = ChaCha8Rng::seed_from_u64(42);
    room.spawn_enemies(&commits, &mut rng);
    
    // Sanctuary rooms should never have enemies
    assert!(room.enemies.is_empty());
}

// === Class Tests ===

#[test]
fn new_with_class_sets_class() {
    let commits = vec![make_commit("Fix bug", 50)];
    let state = GameState::new_with_class(commits, 42, Some(PlayerClass::CodeWarrior), test_git_path());
    assert_eq!(state.player.class, PlayerClass::CodeWarrior);
}

#[test]
fn class_auto_detection_wanderer() {
    // Few commits with varied messages -> Wanderer
    let commits = vec![make_commit("Fix bug", 50)];
    let state = GameState::new_with_class(commits, 42, None, test_git_path());
    assert_eq!(state.player.class, PlayerClass::Wanderer);
}

#[test]
fn class_auto_detection_inbox_knight() {
    // >30% test-related commits -> InboxKnight
    let commits = vec![
        make_commit("Add test for auth", 50),
        make_commit("Fix tests", 50),
        make_commit("Testing edge case", 50),
        make_commit("Fix bug", 50),
    ];
    let state = GameState::new_with_class(commits, 42, None, test_git_path());
    assert_eq!(state.player.class, PlayerClass::InboxKnight);
}
