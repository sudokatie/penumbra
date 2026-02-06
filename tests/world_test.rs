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
