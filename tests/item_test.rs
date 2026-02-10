//! Tests for item module.

use chrono::Utc;
use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;

use penumbra::entity::Player;
use penumbra::git::CommitData;
use penumbra::item::{
    apply_effect, calculate_rarity, generate_item, Item, ItemEffect, ItemType, Rarity,
};
use penumbra::entity::PlayerClass;

// === Item Tests (Task 8) ===

#[test]
fn item_new_creates_item() {
    let item = Item::new("Test", ItemType::Consumable, ItemEffect::Heal(10), Rarity::Common);
    assert_eq!(item.name, "Test");
    assert_eq!(item.item_type, ItemType::Consumable);
    assert_eq!(item.rarity, Rarity::Common);
}

#[test]
fn item_at_sets_position() {
    let item = Item::new("Test", ItemType::Consumable, ItemEffect::Heal(10), Rarity::Common)
        .at(5, 10);
    assert_eq!(item.x, 5);
    assert_eq!(item.y, 10);
}

#[test]
fn item_from_commit_sets_hash() {
    let item = Item::new("Test", ItemType::Consumable, ItemEffect::Heal(10), Rarity::Common)
        .from_commit("abc123");
    assert_eq!(item.source_commit, Some("abc123".to_string()));
}

#[test]
fn apply_heal_effect() {
    let mut player = Player::new(PlayerClass::Wanderer);
    player.hp = 30;
    let msg = apply_effect(&ItemEffect::Heal(20), &mut player);
    assert_eq!(player.hp, 50);
    assert!(msg.contains("Healed"));
}

#[test]
fn apply_restore_energy_effect() {
    let mut player = Player::new(PlayerClass::Wanderer);
    player.energy = 50;
    let msg = apply_effect(&ItemEffect::RestoreEnergy(30), &mut player);
    assert_eq!(player.energy, 80);
    assert!(msg.contains("Restored"));
}

#[test]
fn apply_reveal_map_effect() {
    let mut player = Player::new(PlayerClass::Wanderer);
    let msg = apply_effect(&ItemEffect::RevealMap, &mut player);
    assert!(msg.contains("revealed"));
}

#[test]
fn calculate_rarity_common() {
    assert_eq!(calculate_rarity(0), Rarity::Common);
    assert_eq!(calculate_rarity(49), Rarity::Common);
}

#[test]
fn calculate_rarity_uncommon() {
    assert_eq!(calculate_rarity(50), Rarity::Uncommon);
    assert_eq!(calculate_rarity(199), Rarity::Uncommon);
}

#[test]
fn calculate_rarity_rare() {
    assert_eq!(calculate_rarity(200), Rarity::Rare);
    assert_eq!(calculate_rarity(499), Rarity::Rare);
}

#[test]
fn calculate_rarity_legendary() {
    assert_eq!(calculate_rarity(500), Rarity::Legendary);
    assert_eq!(calculate_rarity(1000), Rarity::Legendary);
}

fn make_commit(message: &str, lines: u32) -> CommitData {
    CommitData {
        hash: "test123".to_string(),
        date: Utc::now(),
        message: message.to_string(),
        insertions: lines,
        deletions: 0,
        files_changed: 1,
        author: "Test".to_string(),
        is_merge: false, file_categories: Default::default(),
    }
}

#[test]
fn generate_item_from_doc_commit() {
    let commit = make_commit("Update README documentation", 50);
    let mut rng = ChaCha8Rng::seed_from_u64(12345);
    let item = generate_item(&commit, &mut rng);
    assert!(item.name.contains("Scroll") || item.name.contains("Map"));
}

#[test]
fn generate_item_from_test_commit() {
    let commit = make_commit("Add unit test for auth", 100);
    let mut rng = ChaCha8Rng::seed_from_u64(12345);
    let item = generate_item(&commit, &mut rng);
    assert!(item.name.contains("Heal") || item.name.contains("Commit"));
}

#[test]
fn generate_item_from_config_commit() {
    let commit = make_commit("Update config settings", 50);
    let mut rng = ChaCha8Rng::seed_from_u64(12345);
    let item = generate_item(&commit, &mut rng);
    assert!(item.name.contains("Config") || item.name.contains("Scroll"));
}

#[test]
fn generate_item_stores_commit_hash() {
    let commit = make_commit("Any commit", 50);
    let mut rng = ChaCha8Rng::seed_from_u64(12345);
    let item = generate_item(&commit, &mut rng);
    assert_eq!(item.source_commit, Some("test123".to_string()));
}
