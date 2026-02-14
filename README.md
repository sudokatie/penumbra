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
- Meta-progression: permanent upgrades that persist between runs

## Quick Start

```bash
# Play in current git repo
penumbra play

# Play with specific repo and more history
penumbra play --git ~/projects/myapp --days 60

# Play from your calendar (ICS file)
penumbra play --calendar ~/calendar.ics --days 30

# Continue a saved game
penumbra continue

# View past runs
penumbra history
```

## How Generation Works

### From Git

| Git Data | Dungeon Element |
|----------|-----------------|
| Day with commits | Room |
| Lines changed | Room size |
| Merge commit | Boss room |
| Test files | Sanctuary (heal zone) |
| Config files | Treasure room |
| Commit count | Enemy count |
| Commit size | Enemy difficulty |

### From Calendar

Your calendar becomes the dungeon too. Export your calendar as ICS and watch your meetings transform into monsters.

| Calendar Data | Dungeon Element |
|---------------|-----------------|
| Day with events | Room |
| Meeting duration + attendees | Room size |
| All-hands / 10+ attendees | Boss room |
| Focus time / breaks | Sanctuary |
| 1:1 meetings | Treasure room |
| Event count | Enemy count |
| Meeting length | Enemy difficulty |

Export from Google Calendar: Settings > Import & Export > Export

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

## Meta-Progression

Each run earns Essence based on your performance:
- 1 per enemy killed
- 5 per room cleared
- 20 for victory

Spend Essence on permanent upgrades:
- **HP Bonus**: +5 max HP per level (5 levels)
- **Energy Bonus**: +2 max energy per level (5 levels)
- **Damage Bonus**: +1 base damage per level (3 levels)
- **Starting Weapon**: Better initial gear (2 levels)
- **Loot Luck**: +5% better item chance per level (3 levels)

Progress persists in `~/.penumbra/progression.json`.

## Roadmap

### v0.2 (Partial)
- [ ] More data sources (calendar, email)
- [x] Character progression persistence between runs
- [x] File category analysis for room types (test->Sanctuary, config->Treasure)

See FEATURE-BACKLOG.md in the clawd repo for detailed acceptance criteria.

## License

MIT

## Author

Katie

---

*Your git history has been here all along. Time to explore it.*
