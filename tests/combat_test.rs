//! Tests for combat system.

use chrono::NaiveDate;
use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;

use penumbra::combat::{
    calculate_damage, calculate_hit_chance, decide_action, enemy_attack, find_path,
    get_adjacent_positions, player_attack, should_use_special, EnemyAction, PlayerAction,
    ATTACK_COST, DEFEND_COST, MOVE_COST, USE_ITEM_COST,
};
use penumbra::entity::{Enemy, EnemyType, Player, PlayerClass};
use penumbra::world::{Room, RoomType};

// === Combat System Tests (Task 9) ===

#[test]
fn player_attack_can_hit() {
    let player = Player::new(PlayerClass::CodeWarrior);
    let _enemy = Enemy::new(EnemyType::Bug, 0, 0, "");
    let mut rng = ChaCha8Rng::seed_from_u64(12345);
    
    // Run multiple attacks, at least one should hit
    let mut hit_count = 0;
    for _ in 0..20 {
        let mut e = Enemy::new(EnemyType::Bug, 0, 0, "");
        let result = player_attack(&player, &mut e, &mut rng);
        if result.hit {
            hit_count += 1;
        }
    }
    assert!(hit_count > 0);
}

#[test]
fn player_attack_applies_damage() {
    let player = Player::new(PlayerClass::CodeWarrior);
    let mut enemy = Enemy::new(EnemyType::MergeConflict, 0, 0, ""); // High HP enemy
    let mut rng = ChaCha8Rng::seed_from_u64(42); // Seed that hits
    
    let initial_hp = enemy.hp;
    let result = player_attack(&player, &mut enemy, &mut rng);
    
    if result.hit {
        assert!(enemy.hp < initial_hp);
        assert!(result.damage > 0);
    }
}

#[test]
fn player_attack_can_miss() {
    let player = Player::new(PlayerClass::Wanderer);
    let _enemy = Enemy::new(EnemyType::Bug, 0, 0, "");
    let mut rng = ChaCha8Rng::seed_from_u64(12345);
    
    let mut miss_count = 0;
    for _ in 0..50 {
        let mut e = Enemy::new(EnemyType::Bug, 0, 0, "");
        let result = player_attack(&player, &mut e, &mut rng);
        if !result.hit {
            miss_count += 1;
        }
    }
    assert!(miss_count > 0);
}

#[test]
fn player_attack_can_kill() {
    let player = Player::new(PlayerClass::CodeWarrior);
    let mut enemy = Enemy::new(EnemyType::Bug, 0, 0, ""); // Low HP
    let mut rng = ChaCha8Rng::seed_from_u64(42);
    
    // Keep attacking until killed
    for _ in 0..20 {
        let result = player_attack(&player, &mut enemy, &mut rng);
        if result.killed {
            assert!(enemy.hp <= 0);
            return;
        }
    }
    // Bug has 10 HP, should die within 20 attacks
    panic!("Expected to kill enemy");
}

#[test]
fn enemy_attack_applies_damage() {
    let enemy = Enemy::new(EnemyType::Bug, 0, 0, "");
    let mut player = Player::new(PlayerClass::Wanderer);
    let mut rng = ChaCha8Rng::seed_from_u64(42);
    
    let initial_hp = player.hp;
    let result = enemy_attack(&enemy, &mut player, &mut rng);
    
    if result.hit {
        assert!(player.hp < initial_hp);
    }
}

#[test]
fn defending_reduces_damage() {
    let damage_normal = calculate_damage(10, 1, false);
    let damage_defending = calculate_damage(10, 1, true);
    assert_eq!(damage_defending, damage_normal / 2);
}

#[test]
fn calculate_hit_chance_base() {
    let chance = calculate_hit_chance(0);
    assert!((chance - 0.80).abs() < 0.01);
}

#[test]
fn calculate_hit_chance_increases_with_focus() {
    let low = calculate_hit_chance(0);
    let high = calculate_hit_chance(100);
    assert!(high > low);
}

#[test]
fn calculate_hit_chance_caps_at_95() {
    let chance = calculate_hit_chance(1000);
    assert!((chance - 0.95).abs() < 0.01);
}

#[test]
fn calculate_damage_scales_with_level() {
    let level1 = calculate_damage(10, 1, false);
    let level5 = calculate_damage(10, 5, false);
    assert!(level5 > level1);
}

#[test]
fn calculate_damage_minimum_is_one() {
    let damage = calculate_damage(1, 1, true);
    assert!(damage >= 1);
}

#[test]
fn action_energy_costs() {
    assert_eq!(PlayerAction::Move(0, 1).energy_cost(), MOVE_COST);
    assert_eq!(PlayerAction::Attack(penumbra::world::Direction::North).energy_cost(), ATTACK_COST);
    assert_eq!(PlayerAction::Defend.energy_cost(), DEFEND_COST);
    assert_eq!(PlayerAction::UseItem(0).energy_cost(), USE_ITEM_COST);
    assert_eq!(PlayerAction::Wait.energy_cost(), 0);
}

#[test]
fn action_is_movement() {
    assert!(PlayerAction::Move(1, 0).is_movement());
    assert!(!PlayerAction::Attack(penumbra::world::Direction::North).is_movement());
}

#[test]
fn action_is_attack() {
    assert!(PlayerAction::Attack(penumbra::world::Direction::North).is_attack());
    assert!(!PlayerAction::Move(1, 0).is_attack());
}

// === AI Tests (Task 10) ===

fn make_test_room() -> Room {
    let date = NaiveDate::from_ymd_opt(2026, 1, 1).unwrap();
    Room::new(0, 10, 10, RoomType::Normal, date)
}

#[test]
fn decide_action_attacks_when_adjacent() {
    let room = make_test_room();
    let enemy = Enemy::new(EnemyType::Bug, 5, 5, "");
    let mut player = Player::new(PlayerClass::Wanderer);
    player.x = 5;
    player.y = 4; // Adjacent
    
    let action = decide_action(&enemy, &player, &room);
    assert_eq!(action, EnemyAction::Attack);
}

#[test]
fn decide_action_moves_when_not_adjacent() {
    let room = make_test_room();
    let enemy = Enemy::new(EnemyType::Bug, 5, 5, "");
    let mut player = Player::new(PlayerClass::Wanderer);
    player.x = 5;
    player.y = 2; // Not adjacent
    
    let action = decide_action(&enemy, &player, &room);
    assert!(matches!(action, EnemyAction::Move { .. }));
}

#[test]
fn find_path_returns_valid_path() {
    let room = make_test_room();
    let path = find_path((1, 1), (3, 1), &room);
    
    assert!(path.is_some());
    let path = path.unwrap();
    assert_eq!(path[0], (1, 1));
    assert_eq!(*path.last().unwrap(), (3, 1));
}

#[test]
fn find_path_avoids_walls() {
    let date = NaiveDate::from_ymd_opt(2026, 1, 1).unwrap();
    let mut room = Room::new(0, 5, 5, RoomType::Normal, date);
    room.set_tile(2, 1, penumbra::world::Tile::Wall);
    
    let path = find_path((1, 1), (3, 1), &room);
    
    if let Some(p) = &path {
        // Path should not go through wall at (2,1)
        assert!(!p.contains(&(2, 1)));
    }
}

#[test]
fn find_path_returns_none_if_blocked() {
    let date = NaiveDate::from_ymd_opt(2026, 1, 1).unwrap();
    let mut room = Room::new(0, 5, 5, RoomType::Normal, date);
    // Block all paths
    for x in 0..5 {
        room.set_tile(x, 2, penumbra::world::Tile::Wall);
    }
    
    let path = find_path((1, 1), (1, 3), &room);
    assert!(path.is_none());
}

#[test]
fn regression_prioritizes_healing() {
    let mut enemy = Enemy::new(EnemyType::Regression, 5, 5, "");
    enemy.hp = enemy.max_hp / 4; // Below 50%
    
    let special = should_use_special(&enemy);
    assert!(matches!(special, Some(EnemyAction::Regenerate(_))));
}

#[test]
fn tech_debt_grows_damage() {
    let mut enemy = Enemy::new(EnemyType::TechDebt, 5, 5, "");
    enemy.turns_alive = 1;
    
    let special = should_use_special(&enemy);
    assert!(matches!(special, Some(EnemyAction::Grow(_))));
}

#[test]
fn merge_conflict_splits() {
    let mut enemy = Enemy::new(EnemyType::MergeConflict, 5, 5, "");
    enemy.hp = enemy.max_hp / 2; // At 50%
    
    let special = should_use_special(&enemy);
    assert!(matches!(special, Some(EnemyAction::Split)));
}

#[test]
fn get_adjacent_positions_returns_four() {
    let positions = get_adjacent_positions((5, 5));
    assert_eq!(positions.len(), 4);
    assert!(positions.contains(&(5, 4)));
    assert!(positions.contains(&(5, 6)));
    assert!(positions.contains(&(4, 5)));
    assert!(positions.contains(&(6, 5)));
}
