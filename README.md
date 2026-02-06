# Penumbra

A roguelike where dungeons generate from your git history. Your commits become corridors. Your merge conflicts become bosses. Your test files become sanctuaries.

## Why This Exists

Procedural generation is compelling because every run is different. But it's also disconnected from anything meaningful - just dice rolls and algorithms.

What if the dungeon reflected something real?

Penumbra transforms your git history into a playable dungeon. Busy weeks spawn more rooms. Large commits create boss encounters. Documentation commits drop map scrolls. Your codebase becomes an adventure.

## Features

- Terminal-based roguelike with ASCII graphics
- Dungeon generated from git commit history
- Turn-based tactical combat
- Permadeath (because roguelikes)
- Enemies spawned from commit types
- Items generated from file categories
- Field of view with shadowcasting
- Save/load for long dungeon runs

## Quick Start

```bash
# Play in current git repo
penumbra play

# Play with specific repo and more history
penumbra play --git ~/projects/myapp --days 60

# Continue a saved game
penumbra continue

# View past runs
penumbra history
```

## How Generation Works

| Git Data | Dungeon Element |
|----------|-----------------|
| Day with commits | Room |
| Lines changed | Room size |
| Merge commit | Boss room |
| Test files | Sanctuary (heal zone) |
| Config files | Treasure room |
| Commit count | Enemy count |
| Commit size | Enemy difficulty |

## Controls

| Key | Action |
|-----|--------|
| Arrow keys / hjkl | Move |
| a + direction | Attack |
| i | Inventory |
| ? | Help |
| q | Quit |

## Enemy Types

- **Bug** (B): Small commits. Weak but common.
- **Regression** (R): Revert commits. Regenerates health.
- **Tech Debt** (D): Old code touched. Grows stronger each turn.
- **Merge Conflict** (M): Merge commits. Splits in two at half health.

## License

MIT

## Author

Katie

---

*Your git history has been here all along. Time to explore it.*
