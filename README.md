# Vale Village v3

Golden Sun-inspired deterministic tactical RPG. Fresh Rust/Bevy build.

> "Vale Village is a deterministic planning RPG where ATTACK prints mana and crit, ABILITY spends them, enemy intent is hidden, and djinn activation temporarily rewires each unit's available kit."

## Status: Pre-Build (Design Locked)

All combat mechanics locked. Data extracted and validated. Ready for vertical slice.

## Docs

| Document | Purpose |
|----------|---------|
| `docs/design/DESIGN_LOCK.md` | Every confirmed game rule. The contract. |
| `docs/design/SYSTEMS_FOUNDATION.md` | 19 mechanical systems mapped to v2 data |
| `docs/design/DATA_MANIFEST.md` | Complete v2 content inventory |
| `docs/data/COMPLETE_DATA.md` | All data tables (human-readable) |
| `docs/data/*.json` | Worker-validated JSON (machine-readable) |

## Data Files (Codex worker-extracted, validated)

| File | Entries | Description |
|------|---------|-------------|
| `data_enemies.json` | 137 | All enemies with stats |
| `data_equipment.json` | 109 | All items with stat bonuses |
| `data_abilities.json` | 241 | All abilities with full properties |
| `data_djinn.json` | 23 | Djinn stats and summon data |
| `data_djinn_abilities.json` | 23 | Ability pairs per compatibility |
| `data_encounters.json` | 55 | Encounters with rewards |
| `data_unit_abilities.json` | 11 | Unit ability progressions |

## Architecture (planned)

- Engine: Bevy (Rust)
- Data: RON files (converted from JSON)
- Sprites: GIF → PNG atlas pipeline from vale-village (original repo)
- Build method: AI-orchestrated (Codex workers + Claude orchestrator)
