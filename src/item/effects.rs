//! Item effect application.

use rand::Rng;

use crate::entity::Player;
use crate::git::CommitData;

use super::{Item, ItemEffect, ItemType, Rarity};

/// Apply an item effect to the player.
pub fn apply_effect(effect: &ItemEffect, player: &mut Player) -> String {
    match effect {
        ItemEffect::Heal(amount) => {
            player.heal(*amount);
            format!("Healed for {} HP", amount)
        }
        ItemEffect::RestoreEnergy(amount) => {
            player.regen_energy(*amount);
            format!("Restored {} energy", amount)
        }
        ItemEffect::Damage(_) => "Damage items target enemies".to_string(),
        ItemEffect::Buff(stat, amount, duration) => {
            format!("Buffed {:?} by {} for {} turns", stat, amount, duration)
        }
        ItemEffect::RevealMap => "Map revealed".to_string(),
    }
}

/// Calculate item rarity from lines changed.
pub fn calculate_rarity(lines_changed: u32) -> Rarity {
    match lines_changed {
        0..=49 => Rarity::Common,
        50..=199 => Rarity::Uncommon,
        200..=499 => Rarity::Rare,
        _ => Rarity::Legendary,
    }
}

/// Generate an item from commit data.
pub fn generate_item(commit: &CommitData, rng: &mut impl Rng) -> Item {
    let rarity = calculate_rarity(commit.lines_changed());
    let msg = commit.message.to_lowercase();

    let (name, effect, item_type) = if msg.contains("doc") || msg.contains("readme") {
        ("Map Scroll".to_string(), ItemEffect::RevealMap, ItemType::Scroll)
    } else if msg.contains("test") || msg.contains("spec") {
        let heal = match rarity {
            Rarity::Common => 10,
            Rarity::Uncommon => 20,
            Rarity::Rare => 35,
            Rarity::Legendary => 50,
        };
        ("Healing Commit".to_string(), ItemEffect::Heal(heal), ItemType::Consumable)
    } else if msg.contains("config") || msg.contains("setting") {
        let energy = match rarity {
            Rarity::Common => 20,
            Rarity::Uncommon => 40,
            Rarity::Rare => 60,
            Rarity::Legendary => 100,
        };
        ("Config Scroll".to_string(), ItemEffect::RestoreEnergy(energy), ItemType::Scroll)
    } else {
        // Random effect
        let roll: u8 = rng.gen_range(0..3);
        match roll {
            0 => ("Small Heal".to_string(), ItemEffect::Heal(10), ItemType::Consumable),
            1 => ("Energy Drink".to_string(), ItemEffect::RestoreEnergy(20), ItemType::Consumable),
            _ => ("Mystery Scroll".to_string(), ItemEffect::RevealMap, ItemType::Scroll),
        }
    };

    Item::new(name, item_type, effect, rarity).from_commit(&commit.hash)
}
