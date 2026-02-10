//! Tests for entity module (player, enemy).

use penumbra::entity::{Enemy, EnemyType, Player, PlayerClass};
use penumbra::item::{Item, ItemEffect, ItemType, Rarity};

// === Player Tests (Task 6) ===

#[test]
fn player_new_with_class() {
    let player = Player::new(PlayerClass::CodeWarrior);
    assert_eq!(player.class, PlayerClass::CodeWarrior);
}

#[test]
fn code_warrior_has_damage_bonus() {
    let player = Player::new(PlayerClass::CodeWarrior);
    assert_eq!(player.damage, 20); // 10 base + 10 bonus
}

#[test]
fn meeting_survivor_has_hp_bonus() {
    let player = Player::new(PlayerClass::MeetingSurvivor);
    assert_eq!(player.max_hp, 70); // 50 base + 20 bonus
    assert_eq!(player.hp, 70);
}

#[test]
fn inbox_knight_has_focus_bonus() {
    let player = Player::new(PlayerClass::InboxKnight);
    assert_eq!(player.max_focus, 60); // 50 base + 10 bonus
}

#[test]
fn wanderer_has_balanced_stats() {
    let player = Player::new(PlayerClass::Wanderer);
    assert_eq!(player.max_hp, 55);
    assert_eq!(player.max_focus, 55);
    assert_eq!(player.damage, 15);
}

#[test]
fn player_take_damage_reduces_hp() {
    let mut player = Player::new(PlayerClass::Wanderer);
    let initial = player.hp;
    player.take_damage(10);
    assert_eq!(player.hp, initial - 10);
}

#[test]
fn player_take_damage_returns_false_when_dead() {
    let mut player = Player::new(PlayerClass::Wanderer);
    player.hp = 10;
    assert!(!player.take_damage(15));
}

#[test]
fn player_take_damage_returns_true_when_alive() {
    let mut player = Player::new(PlayerClass::Wanderer);
    assert!(player.take_damage(10));
}

#[test]
fn player_defending_halves_damage() {
    let mut player = Player::new(PlayerClass::Wanderer);
    player.defending = true;
    let initial = player.hp;
    player.take_damage(20);
    assert_eq!(player.hp, initial - 10); // Half of 20
}

#[test]
fn player_heal_increases_hp() {
    let mut player = Player::new(PlayerClass::Wanderer);
    player.hp = 30;
    player.heal(10);
    assert_eq!(player.hp, 40);
}

#[test]
fn player_heal_caps_at_max() {
    let mut player = Player::new(PlayerClass::Wanderer);
    player.hp = player.max_hp - 5;
    player.heal(20);
    assert_eq!(player.hp, player.max_hp);
}

#[test]
fn player_use_energy_succeeds() {
    let mut player = Player::new(PlayerClass::Wanderer);
    assert!(player.use_energy(10));
    assert_eq!(player.energy, 90);
}

#[test]
fn player_use_energy_fails_when_not_enough() {
    let mut player = Player::new(PlayerClass::Wanderer);
    player.energy = 5;
    assert!(!player.use_energy(10));
    assert_eq!(player.energy, 5); // Unchanged
}

#[test]
fn player_regen_energy_works() {
    let mut player = Player::new(PlayerClass::Wanderer);
    player.energy = 50;
    player.regen_energy(20);
    assert_eq!(player.energy, 70);
}

#[test]
fn player_regen_energy_caps_at_max() {
    let mut player = Player::new(PlayerClass::Wanderer);
    player.energy = player.max_energy - 5;
    player.regen_energy(20);
    assert_eq!(player.energy, player.max_energy);
}

#[test]
fn player_add_xp_increases_xp() {
    let mut player = Player::new(PlayerClass::Wanderer);
    player.add_xp(50);
    assert_eq!(player.xp, 50);
}

#[test]
fn player_add_xp_triggers_level_up() {
    let mut player = Player::new(PlayerClass::Wanderer);
    assert!(player.add_xp(100)); // 100 XP for level 1
    assert_eq!(player.level, 2);
}

#[test]
fn player_level_up_increases_max_hp() {
    let mut player = Player::new(PlayerClass::Wanderer);
    let initial_max = player.max_hp;
    player.add_xp(100);
    assert_eq!(player.max_hp, initial_max + 10);
}

#[test]
fn player_pickup_item_works() {
    let mut player = Player::new(PlayerClass::Wanderer);
    let item = Item::new("Potion", ItemType::Consumable, ItemEffect::Heal(10), Rarity::Common);
    assert!(player.pickup_item(item));
    assert_eq!(player.inventory.len(), 1);
}

#[test]
fn player_pickup_item_fails_when_full() {
    let mut player = Player::new(PlayerClass::Wanderer);
    for i in 0..10 {
        let item = Item::new(
            format!("Item {}", i),
            ItemType::Consumable,
            ItemEffect::Heal(10),
            Rarity::Common,
        );
        player.pickup_item(item);
    }
    let extra = Item::new("Extra", ItemType::Consumable, ItemEffect::Heal(10), Rarity::Common);
    assert!(!player.pickup_item(extra));
}

// === Enemy Tests (Task 7) ===

#[test]
fn enemy_new_with_type() {
    let enemy = Enemy::new(EnemyType::Bug, 5, 5, "abc123");
    assert_eq!(enemy.enemy_type, EnemyType::Bug);
}

#[test]
fn bug_has_correct_stats() {
    let enemy = Enemy::new(EnemyType::Bug, 0, 0, "");
    assert_eq!(enemy.hp, 10);
    assert_eq!(enemy.damage, 3);
}

#[test]
fn regression_has_correct_stats() {
    let enemy = Enemy::new(EnemyType::Regression, 0, 0, "");
    assert_eq!(enemy.hp, 20);
    assert_eq!(enemy.damage, 5);
}

#[test]
fn tech_debt_has_correct_stats() {
    let enemy = Enemy::new(EnemyType::TechDebt, 0, 0, "");
    assert_eq!(enemy.hp, 30);
    assert_eq!(enemy.damage, 4);
}

#[test]
fn merge_conflict_has_correct_stats() {
    let enemy = Enemy::new(EnemyType::MergeConflict, 0, 0, "");
    assert_eq!(enemy.hp, 50);
    assert_eq!(enemy.damage, 8);
}

#[test]
fn enemy_take_damage_reduces_hp() {
    let mut enemy = Enemy::new(EnemyType::Bug, 0, 0, "");
    enemy.take_damage(5);
    assert_eq!(enemy.hp, 5);
}

#[test]
fn enemy_take_damage_returns_false_when_dead() {
    let mut enemy = Enemy::new(EnemyType::Bug, 0, 0, "");
    assert!(!enemy.take_damage(15));
}

#[test]
fn enemy_symbol_correct() {
    assert_eq!(Enemy::new(EnemyType::Bug, 0, 0, "").symbol(), 'B');
    assert_eq!(Enemy::new(EnemyType::Regression, 0, 0, "").symbol(), 'R');
    assert_eq!(Enemy::new(EnemyType::TechDebt, 0, 0, "").symbol(), 'D');
    assert_eq!(Enemy::new(EnemyType::MergeConflict, 0, 0, "").symbol(), 'M');
}

#[test]
fn enemy_at_half_health() {
    let mut enemy = Enemy::new(EnemyType::MergeConflict, 0, 0, "");
    assert!(!enemy.at_half_health());
    enemy.take_damage(25);
    assert!(enemy.at_half_health());
}

#[test]
fn enemy_stores_source_commit() {
    let enemy = Enemy::new(EnemyType::Bug, 0, 0, "abc123");
    assert_eq!(enemy.source_commit, "abc123");
}

#[test]
fn enemy_type_base_hp() {
    assert_eq!(EnemyType::Bug.base_hp(), 10);
    assert_eq!(EnemyType::Regression.base_hp(), 20);
    assert_eq!(EnemyType::TechDebt.base_hp(), 30);
    assert_eq!(EnemyType::MergeConflict.base_hp(), 50);
}

#[test]
fn enemy_type_base_damage() {
    assert_eq!(EnemyType::Bug.base_damage(), 3);
    assert_eq!(EnemyType::Regression.base_damage(), 5);
    assert_eq!(EnemyType::TechDebt.base_damage(), 4);
    assert_eq!(EnemyType::MergeConflict.base_damage(), 8);
}

// === Class Detection Tests ===

use chrono::Utc;
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

#[test]
fn detect_wanderer_for_empty_commits() {
    let commits: Vec<CommitData> = vec![];
    assert_eq!(PlayerClass::detect(&commits), PlayerClass::Wanderer);
}

#[test]
fn detect_wanderer_for_few_commits() {
    let commits = vec![make_commit("Fix bug", 50)];
    assert_eq!(PlayerClass::detect(&commits), PlayerClass::Wanderer);
}

#[test]
fn detect_code_warrior_for_many_commits() {
    let commits: Vec<CommitData> = (0..105)
        .map(|i| make_commit(&format!("Commit {}", i), 50))
        .collect();
    assert_eq!(PlayerClass::detect(&commits), PlayerClass::CodeWarrior);
}

#[test]
fn detect_inbox_knight_high_test_ratio() {
    let commits = vec![
        make_commit("Add test", 50),
        make_commit("Fix test", 50),
        make_commit("Test coverage", 50),
        make_commit("Fix bug", 50),
    ];
    // 75% are test-related
    assert_eq!(PlayerClass::detect(&commits), PlayerClass::InboxKnight);
}
