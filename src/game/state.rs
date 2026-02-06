//! Game state management.

use rand::prelude::*;
use rand_chacha::ChaCha8Rng;
use serde::{Deserialize, Serialize};

use crate::combat::{enemy_attack, player_attack, EnemyAction, PlayerAction, WAIT_REGEN};
use crate::entity::{Enemy, Player, PlayerClass};
use crate::fov::calculate_fov;
use crate::git::CommitData;
use crate::world::{generate_dungeon, Room, Tile, World};

use std::collections::HashSet;

/// Events that occur during gameplay.
#[derive(Debug, Clone)]
pub enum GameEvent {
    PlayerMoved { x: i32, y: i32 },
    PlayerAttacked { damage: i32, killed: bool },
    PlayerDefending,
    PlayerUsedItem { name: String },
    PlayerLevelUp { level: u32 },
    EnemyAttacked { damage: i32, enemy_type: String },
    EnemyKilled { enemy_type: String, xp: u32 },
    RoomEntered { room_id: usize },
    RoomCleared { room_id: usize },
    GameOver { victory: bool },
    Message(String),
}

/// Complete game state.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameState {
    pub world: World,
    pub player: Player,
    pub turn: u32,
    #[serde(skip)]
    pub visible_tiles: HashSet<(i32, i32)>,
    #[serde(skip)]
    pub messages: Vec<String>,
    pub game_over: bool,
    pub victory: bool,
    pub seed: u64,
}

impl GameState {
    /// Create a new game from git data.
    pub fn new(git_data: Vec<CommitData>, seed: u64) -> Self {
        let world = generate_dungeon(&git_data, seed);
        let player = Player::new(PlayerClass::Wanderer);

        let mut state = Self {
            world,
            player,
            turn: 0,
            visible_tiles: HashSet::new(),
            messages: Vec::new(),
            game_over: false,
            victory: false,
            seed,
        };

        // Position player at entrance of first room
        if let Some(room) = state.world.current() {
            state.player.x = 1;
            state.player.y = room.height as i32 / 2;
        }

        state.update_fov();
        state.log("You enter the dungeon generated from your git history...");
        state
    }

    /// Process a player action and return events.
    pub fn process_action(&mut self, action: PlayerAction) -> Vec<GameEvent> {
        if self.game_over {
            return vec![];
        }

        let mut events = Vec::new();

        // Check energy cost
        let cost = action.energy_cost();
        if cost > 0 && !self.player.use_energy(cost) {
            self.log("Not enough energy!");
            return events;
        }

        match action {
            PlayerAction::Move(dx, dy) => {
                let new_x = self.player.x + dx;
                let new_y = self.player.y + dy;

                let can_move = self.world.current().map_or(false, |room| {
                    room.is_walkable(new_x, new_y) && room.get_enemy_at(new_x, new_y).is_none()
                });

                let blocked_by_enemy = self.world.current().map_or(false, |room| {
                    room.get_enemy_at(new_x, new_y).is_some()
                });

                if blocked_by_enemy {
                    self.log("An enemy blocks the way!");
                    self.player.regen_energy(cost);
                } else if can_move {
                    self.player.x = new_x;
                    self.player.y = new_y;
                    events.push(GameEvent::PlayerMoved { x: new_x, y: new_y });
                    self.update_fov();

                    if self.check_room_exit() {
                        events.push(GameEvent::RoomEntered {
                            room_id: self.world.current_room,
                        });
                    }
                } else {
                    self.player.regen_energy(cost);
                }
            }

            PlayerAction::Attack(direction) => {
                let (dx, dy) = direction.delta();
                let target_x = self.player.x + dx;
                let target_y = self.player.y + dy;

                let enemy_idx = self.world.current().and_then(|room| {
                    room.enemies.iter().position(|e| e.x == target_x && e.y == target_y)
                });

                if let Some(idx) = enemy_idx {
                    let mut rng = ChaCha8Rng::seed_from_u64(self.seed + self.turn as u64);
                    
                    let result = {
                        let room = self.world.current_mut().unwrap();
                        player_attack(&self.player, &mut room.enemies[idx], &mut rng)
                    };

                    self.log(&result.message);
                    events.push(GameEvent::PlayerAttacked {
                        damage: result.damage,
                        killed: result.killed,
                    });

                    if result.killed {
                        let room = self.world.current_mut().unwrap();
                        let enemy = room.enemies.remove(idx);
                        let xp = match enemy.enemy_type {
                            crate::entity::EnemyType::Bug => 10,
                            crate::entity::EnemyType::Regression => 20,
                            crate::entity::EnemyType::TechDebt => 30,
                            crate::entity::EnemyType::MergeConflict => 50,
                        };

                        if self.player.add_xp(xp) {
                            events.push(GameEvent::PlayerLevelUp {
                                level: self.player.level,
                            });
                            self.log(&format!("Level up! You are now level {}.", self.player.level));
                        }

                        events.push(GameEvent::EnemyKilled {
                            enemy_type: format!("{:?}", enemy.enemy_type),
                            xp,
                        });

                        let room = self.world.current_mut().unwrap();
                        if room.enemies.is_empty() {
                            room.cleared = true;
                            events.push(GameEvent::RoomCleared { room_id: room.id });
                            self.log("Room cleared!");
                        }
                    }
                } else {
                    self.log("Nothing to attack there.");
                    self.player.regen_energy(cost);
                }
            }

            PlayerAction::Defend => {
                self.player.defending = true;
                events.push(GameEvent::PlayerDefending);
                self.log("You take a defensive stance.");
            }

            PlayerAction::UseItem(index) => {
                if index < self.player.inventory.len() {
                    let item = self.player.inventory.remove(index);
                    let msg = crate::item::apply_effect(&item.effect, &mut self.player);
                    self.log(&msg);
                    events.push(GameEvent::PlayerUsedItem { name: item.name });
                } else {
                    self.player.regen_energy(cost);
                }
            }

            PlayerAction::Wait => {
                self.player.regen_energy(WAIT_REGEN);
                self.log("You wait and recover energy.");
            }
        }

        self.turn += 1;
        events
    }

    /// Process all enemy turns.
    pub fn process_enemies(&mut self) -> Vec<GameEvent> {
        if self.game_over {
            return vec![];
        }

        let mut events = Vec::new();
        let mut rng = ChaCha8Rng::seed_from_u64(self.seed + self.turn as u64);

        // Get enemy count first
        let enemy_count = self.world.current().map_or(0, |r| r.enemies.len());

        for i in 0..enemy_count {
            // Re-check bounds each iteration (enemies might be removed)
            let enemy_exists = self.world.current().map_or(false, |r| i < r.enemies.len());
            if !enemy_exists {
                continue;
            }

            // Get enemy data for AI decision
            let (enemy_x, enemy_y, enemy_type, enemy_hp, enemy_max_hp, enemy_damage, turns_alive) = {
                let room = self.world.current().unwrap();
                let e = &room.enemies[i];
                (e.x, e.y, e.enemy_type, e.hp, e.max_hp, e.damage, e.turns_alive)
            };

            // Create a temporary enemy for AI decision
            let temp_enemy = Enemy::new(enemy_type, enemy_x, enemy_y, "");
            
            // Decide action based on enemy type and position
            let player_x = self.player.x;
            let player_y = self.player.y;
            let dist = (enemy_x - player_x).abs() + (enemy_y - player_y).abs();

            let action = if dist == 1 {
                // Adjacent - check for special or attack
                if enemy_type == crate::entity::EnemyType::Regression && enemy_hp < enemy_max_hp / 2 {
                    EnemyAction::Regenerate(2)
                } else if enemy_type == crate::entity::EnemyType::MergeConflict && enemy_hp <= enemy_max_hp / 2 {
                    EnemyAction::Split
                } else {
                    EnemyAction::Attack
                }
            } else {
                // Not adjacent - move toward player or use special
                if enemy_type == crate::entity::EnemyType::TechDebt && turns_alive > 0 && enemy_damage < enemy_type.base_damage() * 2 {
                    EnemyAction::Grow(1)
                } else {
                    // Simple move toward player
                    let dx = (player_x - enemy_x).signum();
                    let dy = (player_y - enemy_y).signum();
                    if dx != 0 {
                        EnemyAction::Move { dx, dy: 0 }
                    } else {
                        EnemyAction::Move { dx: 0, dy }
                    }
                }
            };

            // Apply action
            match action {
                EnemyAction::Move { dx, dy } => {
                    let new_x = enemy_x + dx;
                    let new_y = enemy_y + dy;
                    let can_move = self.world.current().map_or(false, |r| r.is_walkable(new_x, new_y));
                    if can_move {
                        if let Some(room) = self.world.current_mut() {
                            room.enemies[i].x = new_x;
                            room.enemies[i].y = new_y;
                        }
                    }
                }
                EnemyAction::Attack => {
                    let result = {
                        let room = self.world.current_mut().unwrap();
                        room.enemies[i].turns_alive += 1;
                        enemy_attack(&room.enemies[i], &mut self.player, &mut rng)
                    };
                    
                    self.messages.push(result.message.clone());
                    events.push(GameEvent::EnemyAttacked {
                        damage: result.damage,
                        enemy_type: format!("{:?}", enemy_type),
                    });

                    if result.killed {
                        self.game_over = true;
                        self.victory = false;
                        events.push(GameEvent::GameOver { victory: false });
                        self.log("You have been defeated!");
                        return events;
                    }
                }
                EnemyAction::Regenerate(amount) => {
                    if let Some(room) = self.world.current_mut() {
                        room.enemies[i].hp = (room.enemies[i].hp + amount).min(room.enemies[i].max_hp);
                        room.enemies[i].turns_alive += 1;
                    }
                }
                EnemyAction::Grow(amount) => {
                    if let Some(room) = self.world.current_mut() {
                        room.enemies[i].damage += amount;
                        room.enemies[i].turns_alive += 1;
                    }
                }
                EnemyAction::Split | EnemyAction::Wait => {
                    if let Some(room) = self.world.current_mut() {
                        room.enemies[i].turns_alive += 1;
                    }
                }
            }
        }

        events
    }

    /// Check if player is at room exit and handle transition.
    pub fn check_room_exit(&mut self) -> bool {
        let at_exit = self.world.current().map_or(false, |room| {
            matches!(room.get_tile(self.player.x, self.player.y), Some(Tile::Exit))
        });

        if !at_exit {
            return false;
        }

        let is_cleared = self.world.current().map_or(false, |room| room.is_cleared());
        if !is_cleared {
            self.log("You must defeat all enemies before leaving!");
            return false;
        }

        if self.world.is_last_room() {
            self.game_over = true;
            self.victory = true;
            self.log("Victory! You have conquered the dungeon!");
            return true;
        }

        if self.world.next_room() {
            let (room_name, room_date) = self.world.current().map_or(
                ("Room".to_string(), "".to_string()),
                |r| (r.room_type.name().to_string(), r.source_date.to_string())
            );
            
            if let Some(room) = self.world.current() {
                self.player.x = 1;
                self.player.y = room.height as i32 / 2;
            }
            
            self.update_fov();
            self.log(&format!("You enter {} ({})", room_name, room_date));
            return true;
        }

        false
    }

    /// Update field of view.
    pub fn update_fov(&mut self) {
        let (origin, blocking_tiles) = if let Some(room) = self.world.current() {
            let tiles = room.tiles.clone();
            ((self.player.x, self.player.y), tiles)
        } else {
            return;
        };

        self.visible_tiles = calculate_fov(origin, 5, |x, y| {
            if x < 0 || y < 0 {
                return true;
            }
            let (ux, uy) = (x as usize, y as usize);
            blocking_tiles
                .get(uy)
                .and_then(|row| row.get(ux))
                .map(|t| t.is_blocking())
                .unwrap_or(true)
        });
    }

    /// Add a message to the log.
    pub fn log(&mut self, message: impl Into<String>) {
        let msg = message.into();
        self.messages.push(msg);
        
        // Keep only last 100 messages
        if self.messages.len() > 100 {
            self.messages.remove(0);
        }
    }
}
