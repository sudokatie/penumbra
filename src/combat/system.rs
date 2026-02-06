//! Combat resolution system.

use rand::Rng;

use crate::entity::{Enemy, Player};

/// Result of a combat action.
#[derive(Debug, Clone)]
pub struct CombatResult {
    pub hit: bool,
    pub damage: i32,
    pub killed: bool,
    pub critical: bool,
    pub message: String,
}

/// Resolve a player attack on an enemy.
pub fn player_attack(player: &Player, enemy: &mut Enemy, rng: &mut impl Rng) -> CombatResult {
    let hit_chance = calculate_hit_chance(player.focus);
    let roll: f32 = rng.gen();
    
    if roll > hit_chance {
        return CombatResult {
            hit: false,
            damage: 0,
            killed: false,
            critical: false,
            message: "You missed!".to_string(),
        };
    }

    // Check for critical hit (5% chance)
    let crit_roll: f32 = rng.gen();
    let critical = crit_roll < 0.05;
    
    let base_damage = calculate_damage(player.damage, player.level, false);
    let damage = if critical { base_damage * 2 } else { base_damage };
    
    let killed = !enemy.take_damage(damage);
    
    let message = if killed {
        format!("You dealt {} damage and killed the {}!", damage, enemy.enemy_type.symbol())
    } else if critical {
        format!("Critical hit! You dealt {} damage!", damage)
    } else {
        format!("You dealt {} damage.", damage)
    };

    CombatResult {
        hit: true,
        damage,
        killed,
        critical,
        message,
    }
}

/// Resolve an enemy attack on the player.
pub fn enemy_attack(enemy: &Enemy, player: &mut Player, rng: &mut impl Rng) -> CombatResult {
    // Enemies have 80% base hit chance
    let hit_chance = 0.80;
    let roll: f32 = rng.gen();
    
    if roll > hit_chance {
        return CombatResult {
            hit: false,
            damage: 0,
            killed: false,
            critical: false,
            message: format!("The {} missed!", enemy.enemy_type.symbol()),
        };
    }

    let damage = calculate_damage(enemy.damage, 1, player.defending);
    let killed = !player.take_damage(damage);
    
    let message = if killed {
        format!("The {} dealt {} damage. You died!", enemy.enemy_type.symbol(), damage)
    } else {
        format!("The {} dealt {} damage.", enemy.enemy_type.symbol(), damage)
    };

    CombatResult {
        hit: true,
        damage,
        killed,
        critical: false,
        message,
    }
}

/// Calculate hit chance based on focus stat.
/// Base 80%, +1% per 10 focus. Min 5%, max 95%.
pub fn calculate_hit_chance(focus: i32) -> f32 {
    let chance = 0.80 + (focus as f32 / 1000.0);
    chance.clamp(0.05, 0.95)
}

/// Calculate damage with modifiers.
/// Damage scales with level (+10% per level).
/// Defending reduces damage by 50%.
pub fn calculate_damage(base: i32, level: u32, defending: bool) -> i32 {
    let scaled = (base as f32 * (1.0 + (level as f32 - 1.0) * 0.1)) as i32;
    let final_damage = if defending { scaled / 2 } else { scaled };
    final_damage.max(1) // Always deal at least 1 damage
}
