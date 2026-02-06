//! Application wrapper for ratatui.

use std::io;
use std::time::Duration;

use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyModifiers};
use ratatui::prelude::*;
use ratatui::Terminal;

use crate::combat::PlayerAction;
use crate::game::GameState;
use crate::world::Direction;

/// Application state.
pub struct App {
    pub state: GameState,
    pub show_help: bool,
    pub show_inventory: bool,
    pub selected_item: usize,
    pub attack_mode: bool,
    pub quit: bool,
}

impl App {
    /// Create a new app with game state.
    pub fn new(state: GameState) -> Self {
        Self {
            state,
            show_help: false,
            show_inventory: false,
            selected_item: 0,
            attack_mode: false,
            quit: false,
        }
    }

    /// Run the main event loop.
    pub fn run(&mut self, terminal: &mut Terminal<impl Backend>) -> io::Result<()> {
        while !self.quit && !self.state.game_over {
            terminal.draw(|frame| super::render(frame, self))?;

            if event::poll(Duration::from_millis(100))? {
                if let Event::Key(key) = event::read()? {
                    self.handle_input(key);
                }
            }
        }

        // Show final screen
        if self.state.game_over {
            terminal.draw(|frame| super::render(frame, self))?;
            // Wait for quit key
            loop {
                if let Event::Key(key) = event::read()? {
                    if key.code == KeyCode::Char('q') || key.code == KeyCode::Esc {
                        break;
                    }
                    if key.code == KeyCode::Char('r') {
                        // Could restart here
                        break;
                    }
                }
            }
        }

        Ok(())
    }

    /// Handle a key event.
    pub fn handle_input(&mut self, key: KeyEvent) {
        // Check for quit
        if key.code == KeyCode::Char('c') && key.modifiers.contains(KeyModifiers::CONTROL) {
            self.quit = true;
            return;
        }

        // Help overlay
        if self.show_help {
            if key.code == KeyCode::Esc || key.code == KeyCode::Char('?') {
                self.show_help = false;
            }
            return;
        }

        // Inventory overlay
        if self.show_inventory {
            match key.code {
                KeyCode::Esc | KeyCode::Char('i') => {
                    self.show_inventory = false;
                }
                KeyCode::Up | KeyCode::Char('k') => {
                    if self.selected_item > 0 {
                        self.selected_item -= 1;
                    }
                }
                KeyCode::Down | KeyCode::Char('j') => {
                    if self.selected_item + 1 < self.state.player.inventory.len() {
                        self.selected_item += 1;
                    }
                }
                KeyCode::Enter => {
                    if self.selected_item < self.state.player.inventory.len() {
                        let action = PlayerAction::UseItem(self.selected_item);
                        self.state.process_action(action);
                        self.state.process_enemies();
                        self.show_inventory = false;
                    }
                }
                _ => {}
            }
            return;
        }

        // Attack mode - waiting for direction
        if self.attack_mode {
            let direction = match key.code {
                KeyCode::Up | KeyCode::Char('k') => Some(Direction::North),
                KeyCode::Down | KeyCode::Char('j') => Some(Direction::South),
                KeyCode::Left | KeyCode::Char('h') => Some(Direction::West),
                KeyCode::Right | KeyCode::Char('l') => Some(Direction::East),
                KeyCode::Esc => {
                    self.attack_mode = false;
                    None
                }
                _ => None,
            };

            if let Some(dir) = direction {
                let action = PlayerAction::Attack(dir);
                self.state.process_action(action);
                self.state.process_enemies();
                self.attack_mode = false;
            }
            return;
        }

        // Normal input
        match key.code {
            // Movement
            KeyCode::Up | KeyCode::Char('k') => {
                self.state.process_action(PlayerAction::Move(0, -1));
                self.state.process_enemies();
            }
            KeyCode::Down | KeyCode::Char('j') => {
                self.state.process_action(PlayerAction::Move(0, 1));
                self.state.process_enemies();
            }
            KeyCode::Left | KeyCode::Char('h') => {
                self.state.process_action(PlayerAction::Move(-1, 0));
                self.state.process_enemies();
            }
            KeyCode::Right | KeyCode::Char('l') => {
                self.state.process_action(PlayerAction::Move(1, 0));
                self.state.process_enemies();
            }

            // Attack mode
            KeyCode::Char('a') => {
                self.attack_mode = true;
            }

            // Defend
            KeyCode::Char('d') => {
                self.state.process_action(PlayerAction::Defend);
                self.state.process_enemies();
            }

            // Wait
            KeyCode::Char('.') | KeyCode::Char(' ') => {
                self.state.process_action(PlayerAction::Wait);
                self.state.process_enemies();
            }

            // Inventory
            KeyCode::Char('i') => {
                self.show_inventory = true;
                self.selected_item = 0;
            }

            // Help
            KeyCode::Char('?') => {
                self.show_help = true;
            }

            // Quit
            KeyCode::Char('q') | KeyCode::Esc => {
                self.quit = true;
            }

            _ => {}
        }
    }
}
