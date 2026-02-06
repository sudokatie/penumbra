# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.1.0] - 2026-02-06

### Added

- Initial release
- Git history parsing with commit data extraction
- Dungeon generation from commit history
  - Room size scales with lines changed
  - Room type determined by commit patterns
- Four enemy types from commit characteristics
  - Bug: Regular commits
  - Regression: Reverts and rollbacks
  - Tech Debt: Refactoring commits
  - Merge Conflict: Merge commits (boss enemies)
- Turn-based combat with hit chance, damage, and defense
- Enemy AI with type-specific behaviors
  - Bug: Random movement chance
  - Regression: Self-healing when low
  - Tech Debt: Growing damage over time
  - Merge Conflict: Splits at half health
- Shadowcasting field of view
- Item system with rarity scaling
  - Map scrolls from doc commits
  - Health potions from test commits
  - Buff items from config commits
- Save/load system with run history
- Terminal UI with ratatui
  - Map display with FOV
  - Stats sidebar
  - Message log
  - Help overlay
  - Inventory management
- CLI with play, continue, and history commands
- Death and victory screens with stats
- Room transition system with clearing requirements

### Technical

- 151 tests covering all major systems
- Rust with ratatui + crossterm
- git2 for repository parsing
- Deterministic generation with seeded RNG
