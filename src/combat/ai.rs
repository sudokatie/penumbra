//! Enemy AI and pathfinding.

use std::collections::{HashSet, VecDeque};

use crate::entity::{Enemy, EnemyType, Player};
use crate::world::Room;

/// Action an enemy can take.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EnemyAction {
    /// Move in direction.
    Move { dx: i32, dy: i32 },
    /// Attack the player.
    Attack,
    /// Regenerate HP (Regression).
    Regenerate(i32),
    /// Split into two (MergeConflict).
    Split,
    /// Grow damage (TechDebt).
    Grow(i32),
    /// Do nothing.
    Wait,
}

/// Decide what action an enemy should take.
pub fn decide_action(enemy: &Enemy, player: &Player, room: &Room) -> EnemyAction {
    let dist = manhattan_distance((enemy.x, enemy.y), (player.x, player.y));

    // Adjacent to player - attack
    if dist == 1 {
        // Check for special abilities first
        if let Some(special) = should_use_special(enemy) {
            return special;
        }
        return EnemyAction::Attack;
    }

    // Check for special abilities
    if let Some(special) = should_use_special(enemy) {
        return special;
    }

    // Find path to player
    if let Some(path) = find_path((enemy.x, enemy.y), (player.x, player.y), room) {
        if path.len() > 1 {
            let next = path[1];
            return EnemyAction::Move {
                dx: next.0 - enemy.x,
                dy: next.1 - enemy.y,
            };
        }
    }

    EnemyAction::Wait
}

/// Check if enemy should use special ability.
pub fn should_use_special(enemy: &Enemy) -> Option<EnemyAction> {
    match enemy.enemy_type {
        EnemyType::Regression => {
            // Regenerate when below 50% HP
            if enemy.hp < enemy.max_hp / 2 {
                return Some(EnemyAction::Regenerate(2));
            }
        }
        EnemyType::TechDebt => {
            // Grow damage every turn
            if enemy.turns_alive > 0 && enemy.damage < enemy.enemy_type.base_damage() * 2 {
                return Some(EnemyAction::Grow(1));
            }
        }
        EnemyType::MergeConflict => {
            // Split at 50% HP
            if enemy.at_half_health() {
                return Some(EnemyAction::Split);
            }
        }
        EnemyType::Bug => {
            // Bugs have no special
        }
    }
    None
}

/// Find path from start to goal using BFS.
pub fn find_path(from: (i32, i32), to: (i32, i32), room: &Room) -> Option<Vec<(i32, i32)>> {
    if from == to {
        return Some(vec![from]);
    }

    let mut visited: HashSet<(i32, i32)> = HashSet::new();
    let mut queue: VecDeque<Vec<(i32, i32)>> = VecDeque::new();

    queue.push_back(vec![from]);
    visited.insert(from);

    while let Some(path) = queue.pop_front() {
        let current = *path.last().unwrap();

        for (dx, dy) in get_adjacent_deltas() {
            let next = (current.0 + dx, current.1 + dy);

            if next == to {
                let mut new_path = path.clone();
                new_path.push(next);
                return Some(new_path);
            }

            if !visited.contains(&next) && room.is_walkable(next.0, next.1) {
                visited.insert(next);
                let mut new_path = path.clone();
                new_path.push(next);
                
                // Limit path length to prevent performance issues
                if new_path.len() < 50 {
                    queue.push_back(new_path);
                }
            }
        }
    }

    None
}

/// Get adjacent position deltas (cardinal directions).
pub fn get_adjacent_deltas() -> [(i32, i32); 4] {
    [(0, -1), (0, 1), (-1, 0), (1, 0)]
}

/// Get adjacent positions.
pub fn get_adjacent_positions(pos: (i32, i32)) -> Vec<(i32, i32)> {
    get_adjacent_deltas()
        .iter()
        .map(|(dx, dy)| (pos.0 + dx, pos.1 + dy))
        .collect()
}

/// Manhattan distance between two points.
fn manhattan_distance(a: (i32, i32), b: (i32, i32)) -> i32 {
    (a.0 - b.0).abs() + (a.1 - b.1).abs()
}
