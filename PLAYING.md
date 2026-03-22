# How to Play Vale Village v3

## Requirements
- Rust toolchain (`rustup` installed)
- System packages for Bevy: `apt install libasound2-dev libudev-dev pkg-config` (Linux)

## Build
```bash
cargo build --release
```
First build takes ~15 minutes. Subsequent builds are fast.

## Three Game Modes

### Adventure Mode (full game loop)
```bash
cargo run --release -- --adventure
```
Text-based RPG with world map, towns, shops, NPC dialogue, dungeons, puzzles, and boss fights. Uses the deterministic battle engine for combat. Auto-saves on exit.

**Controls:** Type numbers to select options, type directions (`up`/`down`/`left`/`right`) in dungeons, `m` for menu on the world map, `q` to quit.

**What's in the game:**
- 4 world map locations (Vale Village, Mercury Lighthouse, Kolima Forest, Imil)
- 2 towns with NPCs, shops, and djinn discovery points
- 2 dungeons (Mercury Lighthouse: 3 rooms + boss, Kolima Forest: 5 rooms + puzzles + boss)
- Branching NPC dialogue with quest advancement and gold rewards
- 7 menu screens (Party, Equipment, Djinn, Items, Quest Log, Psynergy, Status)
- Random overworld encounters between locations
- Full save/load with quest state, shop stock, and dungeon progress

### GUI Battle Mode (Bevy)
```bash
cargo run --release -- --gui
```
Graphical battle scene with 434 pixel art sprites, projectile animations, screen shake, flash overlay, and damage numbers. Shows the deterministic combat engine visually.

### CLI Battle Mode (default)
```bash
cargo run --release
```
Runs a single demo battle in the terminal. Prints turn-by-turn combat events.

Flags:
- `--new` — force new game (ignore existing save)

## Save Data
Saves to `saves/game.ron`. Delete this file to start fresh.

## Game Design
Deterministic tactical RPG inspired by Golden Sun:
- **ATTACK** generates mana and advances crit counter
- **ABILITY** spends mana for elemental damage/healing/status effects
- **Djinn** system: equip elemental spirits that grant abilities and enable summons
- **No randomness** — every outcome is determined by player choices
- Planning order = execution order (not speed-sorted)

See `docs/spec.md` for the full game specification.
