# Vale Village v3 — Build Manifest

## Phase: 0 — Setup

## Architecture

- **Engine:** Bevy (Rust)
- **Data format:** RON files (converted from v2 JSON)
- **Build method:** AI-orchestrated (Opus 4.6 orchestrator, Sonnet/GPT-5.4 workers)
- **Source of truth:** docs/design/DESIGN_LOCK.md (the contract)

## Domain List

| Domain | Path | Owner | Status |
|--------|------|-------|--------|
| shared | src/shared/ | orchestrator | not started |
| combat | src/domains/combat/ | worker | not started |
| status_effects | src/domains/status/ | worker | not started |
| djinn | src/domains/djinn/ | worker | not started |
| equipment | src/domains/equipment/ | worker | not started |
| mana | src/domains/mana/ | worker | not started |
| ui | src/domains/ui/ | worker | not started |
| data_loader | src/domains/data/ | worker | not started |
| ai | src/domains/ai/ | worker | not started |
| progression | src/domains/progression/ | worker | not started |

## Key Constants (from DESIGN_LOCK.md)

- MAX_PARTY_SIZE = 4
- MAX_EQUIPPED_DJINN = 3
- MAX_LEVEL = 20
- CRIT_THRESHOLD = 10
- CRIT_MULTIPLIER = 2.0
- MANA_GAIN_PER_HIT = 1
- MANA_RESETS_EACH_ROUND = true
- Physical damage: basePower + ATK - (DEF * 0.5), floor 1
- Psynergy damage: basePower + MAG - (DEF * 0.3), floor 1
- Healing: basePower + MAG, floor 1
- No element damage modifiers

## Implementation Phases (from SYSTEMS_FOUNDATION.md)

1. Schema + Data Loader (no gameplay)
2. Core Combat (S01-S06): damage, targeting, multi-hit, queue battle, mana, crit
3. Status Framework (S07-S13): status effects, buffs, shields, HoT, DR, immunity, cleanse
4. Advanced Damage (S14-S16): defense pen, splash, chain
5. Life/Death + Equipment (S17-S19): revive, auto-revive, equipment abilities
6. Djinn Layer: state machine, ability oscillation, summons, recovery
7. Content Polish: djinn ability pairs, equipment bonuses, encounters, balance

## Open Blockers

- Bevy project not initialized
- Type contract not frozen

## Decisions

- Elements do NOT affect damage (djinn compatibility only)
- 6 status effects only: Stun, Null, Incapacitate, Burn, Poison, Freeze
- All deterministic — no randomness anywhere
- Barriers are per-instance damage blockers (not DR)
- Djinn have 2 states: GOOD/RECOVERY (not 3-state SET/STANDBY/RECOVERY)
