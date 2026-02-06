//! Dungeon generation from git data.

use chrono::NaiveDate;
use rand::prelude::*;
use rand_chacha::ChaCha8Rng;

use crate::git::CommitData;

use super::{Room, RoomType, Tile, World};

/// Generate a complete dungeon from git commit data.
pub fn generate_dungeon(git_data: &[CommitData], seed: u64) -> World {
    let mut rng = ChaCha8Rng::seed_from_u64(seed);
    let grouped = crate::git::group_by_date(git_data.to_vec());

    let mut rooms = Vec::new();

    for (index, (date, commits)) in grouped.iter().enumerate() {
        let room = generate_room(*date, commits, index, &mut rng);
        rooms.push(room);
    }

    // Place connections between rooms
    place_connections(&mut rooms);

    World::new(rooms)
}

/// Generate a single room from a day's commits.
pub fn generate_room(
    date: NaiveDate,
    commits: &[CommitData],
    index: usize,
    rng: &mut impl Rng,
) -> Room {
    let total_lines: u32 = commits.iter().map(|c| c.lines_changed()).sum();
    let (width, height) = calculate_room_size(total_lines);
    let room_type = determine_room_type(commits);

    let mut room = Room::new(index, width, height, room_type, date);
    room.source_commits = commits.to_vec();

    generate_layout(&mut room, rng);

    room
}

/// Calculate room dimensions from total lines changed.
pub fn calculate_room_size(total_lines: u32) -> (u8, u8) {
    match total_lines {
        0..=19 => (3, 3),
        20..=49 => (5, 5),
        50..=199 => (7, 7),
        200..=499 => (9, 9),
        _ => (11, 11),
    }
}

/// Determine room type from commit data.
pub fn determine_room_type(commits: &[CommitData]) -> RoomType {
    // Merge commits always create boss rooms
    if commits.iter().any(|c| c.is_merge) {
        return RoomType::Boss;
    }

    // TODO: Analyze file categories when we have that data
    // For now, use message-based heuristics
    let test_commits = commits
        .iter()
        .filter(|c| {
            let msg = c.message.to_lowercase();
            msg.contains("test") || msg.contains("spec")
        })
        .count();

    let config_commits = commits
        .iter()
        .filter(|c| {
            let msg = c.message.to_lowercase();
            msg.contains("config") || msg.contains("setting")
        })
        .count();

    let total = commits.len();
    if total > 0 {
        if test_commits * 2 > total {
            return RoomType::Sanctuary;
        }
        if config_commits * 2 > total {
            return RoomType::Treasure;
        }
    }

    RoomType::Normal
}

/// Generate the room layout (walls, doors).
fn generate_layout(room: &mut Room, _rng: &mut impl Rng) {
    let w = room.width as i32;
    let h = room.height as i32;

    // Add perimeter walls
    for x in 0..w {
        room.set_tile(x, 0, Tile::Wall);
        room.set_tile(x, h - 1, Tile::Wall);
    }
    for y in 0..h {
        room.set_tile(0, y, Tile::Wall);
        room.set_tile(w - 1, y, Tile::Wall);
    }

    // Boss rooms are larger with clear center
    if room.room_type == RoomType::Boss && room.width >= 9 {
        // Add some internal structure later
    }
}

/// Place entrance and exit doors connecting rooms.
pub fn place_connections(rooms: &mut [Room]) {
    let room_count = rooms.len();
    for (i, room) in rooms.iter_mut().enumerate() {
        let mid_y = room.height as i32 / 2;

        // First room: entrance only on west
        // Middle rooms: entrance on west, exit on east
        // Last room: entrance on west, no exit

        if i > 0 {
            // Has entrance from previous room
            room.set_tile(0, mid_y, Tile::Entrance);
        }

        if i < room_count - 1 {
            // Has exit to next room
            room.set_tile(room.width as i32 - 1, mid_y, Tile::Exit);
        }
    }
}
