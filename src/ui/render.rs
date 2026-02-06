//! Main render function.

use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, Paragraph};

use super::colors::*;
use super::App;

/// Main render entry point.
pub fn render(frame: &mut Frame, app: &App) {
    let area = frame.area();

    // Check minimum size
    if area.width < 80 || area.height < 24 {
        let msg = Paragraph::new("Terminal too small (min 80x24)")
            .style(Style::default().fg(Color::Red));
        frame.render_widget(msg, area);
        return;
    }

    // Layout: sidebar on right (30%), map on left (70%), log at bottom
    let main_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(70), Constraint::Percentage(30)])
        .split(area);

    let left_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(10), Constraint::Length(6)])
        .split(main_chunks[0]);

    // Render map
    render_map(frame, left_chunks[0], app);

    // Render message log
    render_log(frame, left_chunks[1], app);

    // Render stats sidebar
    render_stats(frame, main_chunks[1], app);

    // Overlays
    if app.show_help {
        render_help(frame, area);
    }

    if app.show_inventory {
        render_inventory(frame, area, app);
    }

    if app.state.game_over {
        render_game_over(frame, area, app);
    }

    if app.attack_mode {
        let msg = Paragraph::new("Attack mode - press direction")
            .style(Style::default().fg(Color::Yellow));
        let msg_area = Rect::new(0, area.height - 1, area.width, 1);
        frame.render_widget(msg, msg_area);
    }
}

/// Render the map.
fn render_map(frame: &mut Frame, area: Rect, app: &App) {
    let block = Block::default()
        .title(" Map ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(UI_BORDER));

    let inner = block.inner(area);
    frame.render_widget(block, area);

    if let Some(room) = app.state.world.current() {
        for y in 0..room.height as i32 {
            for x in 0..room.width as i32 {
                let screen_x = inner.x + x as u16;
                let screen_y = inner.y + y as u16;

                if screen_x >= inner.x + inner.width || screen_y >= inner.y + inner.height {
                    continue;
                }

                let visible = app.state.visible_tiles.contains(&(x, y));

                // Player
                if app.state.player.x == x && app.state.player.y == y {
                    let span = Span::styled("@", Style::default().fg(PLAYER_COLOR));
                    frame.render_widget(
                        Paragraph::new(span),
                        Rect::new(screen_x, screen_y, 1, 1),
                    );
                    continue;
                }

                // Enemy
                if visible {
                    if let Some(enemy) = room.get_enemy_at(x, y) {
                        let color = match enemy.enemy_type {
                            crate::entity::EnemyType::Bug => BUG_COLOR,
                            crate::entity::EnemyType::Regression => REGRESSION_COLOR,
                            crate::entity::EnemyType::TechDebt => TECH_DEBT_COLOR,
                            crate::entity::EnemyType::MergeConflict => MERGE_CONFLICT_COLOR,
                        };
                        let span = Span::styled(
                            enemy.symbol().to_string(),
                            Style::default().fg(color),
                        );
                        frame.render_widget(
                            Paragraph::new(span),
                            Rect::new(screen_x, screen_y, 1, 1),
                        );
                        continue;
                    }

                    // Item
                    if let Some(item) = room.get_item_at(x, y) {
                        let color = match item.rarity {
                            crate::item::Rarity::Common => ITEM_COMMON,
                            crate::item::Rarity::Uncommon => ITEM_UNCOMMON,
                            crate::item::Rarity::Rare => ITEM_RARE,
                            crate::item::Rarity::Legendary => ITEM_LEGENDARY,
                        };
                        let span = Span::styled("!", Style::default().fg(color));
                        frame.render_widget(
                            Paragraph::new(span),
                            Rect::new(screen_x, screen_y, 1, 1),
                        );
                        continue;
                    }
                }

                // Tile
                if let Some(tile) = room.get_tile(x, y) {
                    let (ch, color) = if visible {
                        let color = match tile {
                            crate::world::Tile::Floor => FLOOR_COLOR,
                            crate::world::Tile::Wall => WALL_COLOR,
                            crate::world::Tile::Door(_) => DOOR_COLOR,
                            crate::world::Tile::Exit => EXIT_COLOR,
                            crate::world::Tile::Entrance => ENTRANCE_COLOR,
                            crate::world::Tile::HealingZone => HEALING_ZONE_COLOR,
                        };
                        (tile.symbol(), color)
                    } else {
                        (' ', FOG_COLOR)
                    };

                    let span = Span::styled(ch.to_string(), Style::default().fg(color));
                    frame.render_widget(
                        Paragraph::new(span),
                        Rect::new(screen_x, screen_y, 1, 1),
                    );
                }
            }
        }
    }
}

/// Render the message log.
fn render_log(frame: &mut Frame, area: Rect, app: &App) {
    let block = Block::default()
        .title(" Messages ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(UI_BORDER));

    let inner = block.inner(area);
    frame.render_widget(block, area);

    let messages: Vec<Line> = app
        .state
        .messages
        .iter()
        .rev()
        .take(inner.height as usize)
        .rev()
        .map(|m| Line::from(m.as_str()))
        .collect();

    let para = Paragraph::new(messages).style(Style::default().fg(UI_TEXT));
    frame.render_widget(para, inner);
}

/// Render the stats sidebar.
fn render_stats(frame: &mut Frame, area: Rect, app: &App) {
    let block = Block::default()
        .title(" Status ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(UI_BORDER));

    let inner = block.inner(area);
    frame.render_widget(block, area);

    let player = &app.state.player;

    // HP bar
    let hp_pct = player.hp as f32 / player.max_hp as f32;
    let hp_color = if hp_pct > 0.6 {
        HP_HIGH
    } else if hp_pct > 0.3 {
        HP_MED
    } else {
        HP_LOW
    };

    let mut lines = vec![
        Line::from(vec![
            Span::raw("HP: "),
            Span::styled(
                format!("{}/{}", player.hp, player.max_hp),
                Style::default().fg(hp_color),
            ),
        ]),
        Line::from(vec![
            Span::raw("EN: "),
            Span::styled(
                format!("{}/{}", player.energy, player.max_energy),
                Style::default().fg(ENERGY_COLOR),
            ),
        ]),
        Line::from(vec![
            Span::raw("FO: "),
            Span::styled(
                format!("{}/{}", player.focus, player.max_focus),
                Style::default().fg(FOCUS_COLOR),
            ),
        ]),
        Line::from(""),
        Line::from(format!("Level: {}", player.level)),
        Line::from(format!("XP: {}/{}", player.xp, player.level * 100)),
        Line::from(format!("Turn: {}", app.state.turn)),
        Line::from(""),
    ];

    // Room info
    if let Some(room) = app.state.world.current() {
        lines.push(Line::from(format!(
            "Room {}/{}",
            app.state.world.current_room + 1,
            app.state.world.rooms.len()
        )));
        lines.push(Line::from(room.room_type.name()));
        lines.push(Line::from(format!("{}", room.source_date)));
        lines.push(Line::from(format!("Enemies: {}", room.enemies.len())));
    }

    let para = Paragraph::new(lines).style(Style::default().fg(UI_TEXT));
    frame.render_widget(para, inner);
}

/// Render help overlay.
fn render_help(frame: &mut Frame, area: Rect) {
    let help_text = vec![
        Line::from("=== PENUMBRA HELP ===").style(Style::default().fg(UI_TITLE)),
        Line::from(""),
        Line::from("Movement: Arrow keys or hjkl"),
        Line::from("Attack:   a + direction"),
        Line::from("Defend:   d"),
        Line::from("Wait:     . or space"),
        Line::from("Inventory: i"),
        Line::from("Help:     ?"),
        Line::from("Quit:     q or Esc"),
        Line::from(""),
        Line::from("Press Esc to close"),
    ];

    let width = 40;
    let height = help_text.len() as u16 + 2;
    let x = (area.width - width) / 2;
    let y = (area.height - height) / 2;

    let block = Block::default()
        .title(" Help ")
        .borders(Borders::ALL)
        .style(Style::default().bg(Color::Black));

    let help_area = Rect::new(x, y, width, height);
    frame.render_widget(block.clone(), help_area);

    let para = Paragraph::new(help_text).block(block);
    frame.render_widget(para, help_area);
}

/// Render inventory overlay.
fn render_inventory(frame: &mut Frame, area: Rect, app: &App) {
    let width = 50;
    let height = 15;
    let x = (area.width - width) / 2;
    let y = (area.height - height) / 2;

    let block = Block::default()
        .title(" Inventory ")
        .borders(Borders::ALL)
        .style(Style::default().bg(Color::Black));

    let inv_area = Rect::new(x, y, width, height);
    frame.render_widget(block, inv_area);

    let inner = Rect::new(x + 1, y + 1, width - 2, height - 2);

    if app.state.player.inventory.is_empty() {
        let para = Paragraph::new("(empty)").style(Style::default().fg(UI_TEXT));
        frame.render_widget(para, inner);
    } else {
        let items: Vec<Line> = app
            .state
            .player
            .inventory
            .iter()
            .enumerate()
            .map(|(i, item)| {
                let prefix = if i == app.selected_item { "> " } else { "  " };
                let style = if i == app.selected_item {
                    Style::default().fg(UI_HIGHLIGHT)
                } else {
                    Style::default().fg(UI_TEXT)
                };
                Line::from(format!("{}{}", prefix, item.name)).style(style)
            })
            .collect();

        let para = Paragraph::new(items);
        frame.render_widget(para, inner);
    }
}

/// Render game over screen.
fn render_game_over(frame: &mut Frame, area: Rect, app: &App) {
    let title = if app.state.victory {
        "=== VICTORY ==="
    } else {
        "=== GAME OVER ==="
    };

    let color = if app.state.victory {
        Color::Green
    } else {
        Color::Red
    };

    let lines = vec![
        Line::from(title).style(Style::default().fg(color)),
        Line::from(""),
        Line::from(format!("Turns: {}", app.state.turn)),
        Line::from(format!(
            "Rooms: {}/{}",
            app.state.world.current_room + 1,
            app.state.world.rooms.len()
        )),
        Line::from(format!("Level: {}", app.state.player.level)),
        Line::from(""),
        Line::from("Press Q to quit"),
    ];

    let width = 40;
    let height = lines.len() as u16 + 2;
    let x = (area.width - width) / 2;
    let y = (area.height - height) / 2;

    let block = Block::default()
        .borders(Borders::ALL)
        .style(Style::default().bg(Color::Black));

    let end_area = Rect::new(x, y, width, height);
    let para = Paragraph::new(lines).block(block).alignment(Alignment::Center);
    frame.render_widget(para, end_area);
}
