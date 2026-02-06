//! Tests for world module (tiles, rooms, generator).

use chrono::NaiveDate;
use penumbra::git::CommitData;
use penumbra::world::{
    calculate_room_size, determine_room_type, generate_dungeon, Direction, Room, RoomType, Tile,
    World,
};
use chrono::Utc;

// === Tile Tests (Task 3) ===

#[test]
fn tile_floor_is_walkable() {
    assert!(Tile::Floor.is_walkable());
}

#[test]
fn tile_floor_not_blocking() {
    assert!(!Tile::Floor.is_blocking());
}

#[test]
fn tile_wall_not_walkable() {
    assert!(!Tile::Wall.is_walkable());
}

#[test]
fn tile_wall_is_blocking() {
    assert!(Tile::Wall.is_blocking());
}

#[test]
fn tile_door_is_walkable() {
    assert!(Tile::Door(Direction::North).is_walkable());
}

#[test]
fn tile_exit_is_walkable() {
    assert!(Tile::Exit.is_walkable());
}

#[test]
fn tile_entrance_is_walkable() {
    assert!(Tile::Entrance.is_walkable());
}

#[test]
fn tile_symbols_correct() {
    assert_eq!(Tile::Floor.symbol(), '.');
    assert_eq!(Tile::Wall.symbol(), '#');
    assert_eq!(Tile::Door(Direction::North).symbol(), '+');
    assert_eq!(Tile::Exit.symbol(), '>');
    assert_eq!(Tile::Entrance.symbol(), '<');
}

#[test]
fn direction_opposite() {
    assert_eq!(Direction::North.opposite(), Direction::South);
    assert_eq!(Direction::South.opposite(), Direction::North);
    assert_eq!(Direction::East.opposite(), Direction::West);
    assert_eq!(Direction::West.opposite(), Direction::East);
}

#[test]
fn direction_delta() {
    assert_eq!(Direction::North.delta(), (0, -1));
    assert_eq!(Direction::South.delta(), (0, 1));
    assert_eq!(Direction::East.delta(), (1, 0));
    assert_eq!(Direction::West.delta(), (-1, 0));
}

#[test]
fn room_type_names() {
    assert_eq!(RoomType::Normal.name(), "Room");
    assert_eq!(RoomType::Sanctuary.name(), "Sanctuary");
    assert_eq!(RoomType::Treasure.name(), "Treasury");
    assert_eq!(RoomType::Boss.name(), "Boss Chamber");
}

// === Room Tests (Task 4) ===

#[test]
fn room_new_creates_correct_dimensions() {
    let date = NaiveDate::from_ymd_opt(2026, 1, 1).unwrap();
    let room = Room::new(0, 7, 7, RoomType::Normal, date);
    assert_eq!(room.width, 7);
    assert_eq!(room.height, 7);
}

#[test]
fn room_new_fills_with_floor() {
    let date = NaiveDate::from_ymd_opt(2026, 1, 1).unwrap();
    let room = Room::new(0, 5, 5, RoomType::Normal, date);
    for y in 0..5 {
        for x in 0..5 {
            assert_eq!(*room.get_tile(x, y).unwrap(), Tile::Floor);
        }
    }
}

#[test]
fn room_is_walkable_returns_false_for_out_of_bounds() {
    let date = NaiveDate::from_ymd_opt(2026, 1, 1).unwrap();
    let room = Room::new(0, 5, 5, RoomType::Normal, date);
    assert!(!room.is_walkable(-1, 0));
    assert!(!room.is_walkable(0, -1));
    assert!(!room.is_walkable(5, 0));
    assert!(!room.is_walkable(0, 5));
}

#[test]
fn room_is_walkable_returns_true_for_floor() {
    let date = NaiveDate::from_ymd_opt(2026, 1, 1).unwrap();
    let room = Room::new(0, 5, 5, RoomType::Normal, date);
    assert!(room.is_walkable(2, 2));
}

#[test]
fn room_is_walkable_returns_false_for_wall() {
    let date = NaiveDate::from_ymd_opt(2026, 1, 1).unwrap();
    let mut room = Room::new(0, 5, 5, RoomType::Normal, date);
    room.set_tile(2, 2, Tile::Wall);
    assert!(!room.is_walkable(2, 2));
}

#[test]
fn room_get_tile_returns_none_for_out_of_bounds() {
    let date = NaiveDate::from_ymd_opt(2026, 1, 1).unwrap();
    let room = Room::new(0, 5, 5, RoomType::Normal, date);
    assert!(room.get_tile(-1, 0).is_none());
    assert!(room.get_tile(0, -1).is_none());
    assert!(room.get_tile(5, 0).is_none());
}

#[test]
fn room_set_tile_works() {
    let date = NaiveDate::from_ymd_opt(2026, 1, 1).unwrap();
    let mut room = Room::new(0, 5, 5, RoomType::Normal, date);
    room.set_tile(2, 2, Tile::Wall);
    assert_eq!(*room.get_tile(2, 2).unwrap(), Tile::Wall);
}

#[test]
fn room_stores_date() {
    let date = NaiveDate::from_ymd_opt(2026, 3, 15).unwrap();
    let room = Room::new(0, 5, 5, RoomType::Normal, date);
    assert_eq!(room.source_date, date);
}

#[test]
fn room_is_cleared_when_no_enemies() {
    let date = NaiveDate::from_ymd_opt(2026, 1, 1).unwrap();
    let room = Room::new(0, 5, 5, RoomType::Normal, date);
    assert!(room.is_cleared());
}

// === Generator Tests (Task 5) ===

fn make_commit(lines: u32, is_merge: bool, message: &str) -> CommitData {
    CommitData {
        hash: format!("hash_{}", lines),
        date: Utc::now(),
        message: message.to_string(),
        insertions: lines,
        deletions: 0,
        files_changed: 1,
        author: "Test".to_string(),
        is_merge,
    }
}

#[test]
fn calculate_room_size_small() {
    assert_eq!(calculate_room_size(0), (5, 5));
    assert_eq!(calculate_room_size(49), (5, 5));
}

#[test]
fn calculate_room_size_medium() {
    assert_eq!(calculate_room_size(50), (7, 7));
    assert_eq!(calculate_room_size(199), (7, 7));
}

#[test]
fn calculate_room_size_large() {
    assert_eq!(calculate_room_size(200), (9, 9));
    assert_eq!(calculate_room_size(499), (9, 9));
}

#[test]
fn calculate_room_size_huge() {
    assert_eq!(calculate_room_size(500), (11, 11));
    assert_eq!(calculate_room_size(1000), (11, 11));
}

#[test]
fn determine_room_type_boss_for_merge() {
    let commits = vec![make_commit(100, true, "Merge branch")];
    assert_eq!(determine_room_type(&commits), RoomType::Boss);
}

#[test]
fn determine_room_type_sanctuary_for_tests() {
    let commits = vec![
        make_commit(50, false, "Add test for auth"),
        make_commit(50, false, "Test coverage increase"),
    ];
    assert_eq!(determine_room_type(&commits), RoomType::Sanctuary);
}

#[test]
fn determine_room_type_treasure_for_config() {
    let commits = vec![
        make_commit(30, false, "Update config file"),
        make_commit(30, false, "Add settings"),
    ];
    assert_eq!(determine_room_type(&commits), RoomType::Treasure);
}

#[test]
fn determine_room_type_normal_for_regular() {
    let commits = vec![make_commit(100, false, "Fix bug in auth")];
    assert_eq!(determine_room_type(&commits), RoomType::Normal);
}

#[test]
fn boss_overrides_other_types() {
    let commits = vec![
        make_commit(50, false, "Add test"),
        make_commit(50, true, "Merge feature"),
    ];
    // Merge = Boss, even with test commits
    assert_eq!(determine_room_type(&commits), RoomType::Boss);
}

#[test]
fn generate_dungeon_creates_rooms() {
    let commits = vec![
        make_commit(50, false, "Commit 1"),
        make_commit(100, false, "Commit 2"),
    ];
    let world = generate_dungeon(&commits, 12345);
    assert!(!world.rooms.is_empty());
}

#[test]
fn generate_dungeon_deterministic_with_seed() {
    let commits = vec![
        make_commit(50, false, "Commit 1"),
        make_commit(100, false, "Commit 2"),
    ];
    let world1 = generate_dungeon(&commits, 12345);
    let world2 = generate_dungeon(&commits, 12345);
    assert_eq!(world1.rooms.len(), world2.rooms.len());
}

// === World Tests ===

#[test]
fn world_current_room() {
    let date = NaiveDate::from_ymd_opt(2026, 1, 1).unwrap();
    let rooms = vec![
        Room::new(0, 5, 5, RoomType::Normal, date),
        Room::new(1, 5, 5, RoomType::Normal, date),
    ];
    let world = World::new(rooms);
    assert_eq!(world.current_room, 0);
    assert!(world.current().is_some());
}

#[test]
fn world_next_room_advances() {
    let date = NaiveDate::from_ymd_opt(2026, 1, 1).unwrap();
    let rooms = vec![
        Room::new(0, 5, 5, RoomType::Normal, date),
        Room::new(1, 5, 5, RoomType::Normal, date),
    ];
    let mut world = World::new(rooms);
    assert!(world.next_room());
    assert_eq!(world.current_room, 1);
}

#[test]
fn world_next_room_returns_false_at_end() {
    let date = NaiveDate::from_ymd_opt(2026, 1, 1).unwrap();
    let rooms = vec![Room::new(0, 5, 5, RoomType::Normal, date)];
    let mut world = World::new(rooms);
    assert!(!world.next_room());
}

#[test]
fn world_is_last_room() {
    let date = NaiveDate::from_ymd_opt(2026, 1, 1).unwrap();
    let rooms = vec![
        Room::new(0, 5, 5, RoomType::Normal, date),
        Room::new(1, 5, 5, RoomType::Normal, date),
    ];
    let mut world = World::new(rooms);
    assert!(!world.is_last_room());
    world.next_room();
    assert!(world.is_last_room());
}

// === Enemy Spawning Tests (Task 19) ===

use penumbra::entity::EnemyType;
use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;

fn make_commit_typed(message: &str) -> CommitData {
    CommitData {
        hash: format!("hash_{}", message.len()),
        date: Utc::now(),
        message: message.to_string(),
        insertions: 50,
        deletions: 0,
        files_changed: 1,
        author: "Test".to_string(),
        is_merge: false,
    }
}

#[test]
fn spawn_enemies_creates_enemies() {
    let date = NaiveDate::from_ymd_opt(2026, 1, 1).unwrap();
    let mut room = Room::new(0, 7, 7, RoomType::Normal, date);
    let commits = vec![make_commit_typed("Fix bug"), make_commit_typed("Another fix")];
    let mut rng = ChaCha8Rng::seed_from_u64(42);
    room.spawn_enemies(&commits, &mut rng);
    assert!(!room.enemies.is_empty());
}

#[test]
fn spawn_enemies_respects_count_limit() {
    let date = NaiveDate::from_ymd_opt(2026, 1, 1).unwrap();
    let mut room = Room::new(0, 5, 5, RoomType::Normal, date);
    // Small room (5x5 = 25 tiles, /4 = 6 max enemies)
    let commits: Vec<_> = (0..20).map(|i| make_commit_typed(&format!("Commit {}", i))).collect();
    let mut rng = ChaCha8Rng::seed_from_u64(42);
    room.spawn_enemies(&commits, &mut rng);
    // Should be capped by room size
    assert!(room.enemies.len() <= 6);
}

#[test]
fn spawn_enemies_bug_type_for_regular_commits() {
    let date = NaiveDate::from_ymd_opt(2026, 1, 1).unwrap();
    let mut room = Room::new(0, 7, 7, RoomType::Normal, date);
    let commits = vec![make_commit_typed("Fix something")];
    let mut rng = ChaCha8Rng::seed_from_u64(42);
    room.spawn_enemies(&commits, &mut rng);
    assert_eq!(room.enemies[0].enemy_type, EnemyType::Bug);
}

#[test]
fn spawn_enemies_merge_conflict_for_merge() {
    let date = NaiveDate::from_ymd_opt(2026, 1, 1).unwrap();
    let mut room = Room::new(0, 7, 7, RoomType::Normal, date);
    let commits = vec![make_commit_typed("Merge branch feature")];
    let mut rng = ChaCha8Rng::seed_from_u64(42);
    room.spawn_enemies(&commits, &mut rng);
    assert_eq!(room.enemies[0].enemy_type, EnemyType::MergeConflict);
}

#[test]
fn spawn_enemies_regression_for_revert() {
    let date = NaiveDate::from_ymd_opt(2026, 1, 1).unwrap();
    let mut room = Room::new(0, 7, 7, RoomType::Normal, date);
    let commits = vec![make_commit_typed("Revert bad change")];
    let mut rng = ChaCha8Rng::seed_from_u64(42);
    room.spawn_enemies(&commits, &mut rng);
    assert_eq!(room.enemies[0].enemy_type, EnemyType::Regression);
}

#[test]
fn spawn_enemies_tech_debt_for_refactor() {
    let date = NaiveDate::from_ymd_opt(2026, 1, 1).unwrap();
    let mut room = Room::new(0, 7, 7, RoomType::Normal, date);
    let commits = vec![make_commit_typed("Refactor auth module")];
    let mut rng = ChaCha8Rng::seed_from_u64(42);
    room.spawn_enemies(&commits, &mut rng);
    assert_eq!(room.enemies[0].enemy_type, EnemyType::TechDebt);
}

#[test]
fn spawn_enemies_positions_on_walkable_tiles() {
    let date = NaiveDate::from_ymd_opt(2026, 1, 1).unwrap();
    let mut room = Room::new(0, 7, 7, RoomType::Normal, date);
    let commits = vec![make_commit_typed("Commit 1"), make_commit_typed("Commit 2")];
    let mut rng = ChaCha8Rng::seed_from_u64(42);
    room.spawn_enemies(&commits, &mut rng);
    for enemy in &room.enemies {
        assert!(room.is_walkable(enemy.x, enemy.y));
    }
}

#[test]
fn spawn_enemies_no_overlapping_positions() {
    let date = NaiveDate::from_ymd_opt(2026, 1, 1).unwrap();
    let mut room = Room::new(0, 9, 9, RoomType::Normal, date);
    let commits: Vec<_> = (0..5).map(|i| make_commit_typed(&format!("Commit {}", i))).collect();
    let mut rng = ChaCha8Rng::seed_from_u64(42);
    room.spawn_enemies(&commits, &mut rng);
    
    let mut positions: Vec<_> = room.enemies.iter().map(|e| (e.x, e.y)).collect();
    positions.sort();
    let len_before = positions.len();
    positions.dedup();
    assert_eq!(len_before, positions.len());
}

// === Item Spawning Tests (Task 20) ===

use penumbra::item::{ItemEffect, ItemType, Rarity};

fn make_commit_lines(message: &str, lines: u32) -> CommitData {
    CommitData {
        hash: format!("hash_{}", message.len()),
        date: Utc::now(),
        message: message.to_string(),
        insertions: lines,
        deletions: 0,
        files_changed: 1,
        author: "Test".to_string(),
        is_merge: false,
    }
}

#[test]
fn spawn_items_creates_items() {
    let date = NaiveDate::from_ymd_opt(2026, 1, 1).unwrap();
    let mut room = Room::new(0, 7, 7, RoomType::Normal, date);
    let commits = vec![make_commit_typed("Some commit")];
    let mut rng = ChaCha8Rng::seed_from_u64(42);
    room.spawn_items(&commits, &mut rng);
    assert!(!room.items.is_empty());
}

#[test]
fn spawn_items_doc_commit_creates_scroll() {
    let date = NaiveDate::from_ymd_opt(2026, 1, 1).unwrap();
    let mut room = Room::new(0, 7, 7, RoomType::Normal, date);
    let commits = vec![make_commit_typed("Update documentation")];
    let mut rng = ChaCha8Rng::seed_from_u64(42);
    room.spawn_items(&commits, &mut rng);
    assert_eq!(room.items[0].item_type, ItemType::Scroll);
    assert!(matches!(room.items[0].effect, ItemEffect::RevealMap));
}

#[test]
fn spawn_items_test_commit_creates_health() {
    let date = NaiveDate::from_ymd_opt(2026, 1, 1).unwrap();
    let mut room = Room::new(0, 7, 7, RoomType::Normal, date);
    let commits = vec![make_commit_typed("Add test for login")];
    let mut rng = ChaCha8Rng::seed_from_u64(42);
    room.spawn_items(&commits, &mut rng);
    assert!(matches!(room.items[0].effect, ItemEffect::Heal(_)));
}

#[test]
fn spawn_items_config_commit_creates_buff() {
    let date = NaiveDate::from_ymd_opt(2026, 1, 1).unwrap();
    let mut room = Room::new(0, 7, 7, RoomType::Normal, date);
    let commits = vec![make_commit_typed("Update config file")];
    let mut rng = ChaCha8Rng::seed_from_u64(42);
    room.spawn_items(&commits, &mut rng);
    assert!(matches!(room.items[0].effect, ItemEffect::Buff(_, _, _)));
}

#[test]
fn spawn_items_treasure_room_gets_more() {
    let date = NaiveDate::from_ymd_opt(2026, 1, 1).unwrap();
    let mut room = Room::new(0, 9, 9, RoomType::Treasure, date);
    let commits: Vec<_> = (0..5).map(|i| make_commit_typed(&format!("Commit {}", i))).collect();
    let mut rng = ChaCha8Rng::seed_from_u64(42);
    room.spawn_items(&commits, &mut rng);
    // Treasure rooms get 2-3 items
    assert!(room.items.len() >= 2);
}

#[test]
fn spawn_items_rarity_scales_with_lines() {
    let date = NaiveDate::from_ymd_opt(2026, 1, 1).unwrap();
    
    // Common: <50 lines
    let mut room1 = Room::new(0, 7, 7, RoomType::Normal, date);
    let commits1 = vec![make_commit_lines("Small fix", 30)];
    let mut rng = ChaCha8Rng::seed_from_u64(42);
    room1.spawn_items(&commits1, &mut rng);
    assert_eq!(room1.items[0].rarity, Rarity::Common);
    
    // Legendary: >500 lines
    let mut room2 = Room::new(1, 7, 7, RoomType::Normal, date);
    let commits2 = vec![make_commit_lines("Huge feature", 600)];
    let mut rng = ChaCha8Rng::seed_from_u64(42);
    room2.spawn_items(&commits2, &mut rng);
    assert_eq!(room2.items[0].rarity, Rarity::Legendary);
}

#[test]
fn spawn_items_positions_on_walkable_tiles() {
    let date = NaiveDate::from_ymd_opt(2026, 1, 1).unwrap();
    let mut room = Room::new(0, 7, 7, RoomType::Normal, date);
    let commits = vec![make_commit_typed("Commit 1")];
    let mut rng = ChaCha8Rng::seed_from_u64(42);
    room.spawn_items(&commits, &mut rng);
    for item in &room.items {
        assert!(room.is_walkable(item.x, item.y));
    }
}
