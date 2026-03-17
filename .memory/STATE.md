# Vale Village v3 — Current State

**Phase:** Alignment tranche complete. Safe to scale.
**HEAD:** db20ed9
**Date:** 2026-03-17

## Game Status
Interactive deterministic tactical RPG with:
- SPD-derived planning order (mana economy safe)
- Full data (346 abilities, 137 enemies, 109 equipment, 23 djinn, 55 encounters, 11 units)
- Two-sided combat with equipment, djinn activation/recovery, abilities, mana
- Enemy AI (aggressive/defensive/balanced strategies)
- Save/load with story progression through 50 encounters
- All DESIGN_LOCK rules verified: no randomness, planning order = execution order, 6 status types, barriers, HoT, crit on 10th hit

## Domains (11 domains, 224 tests + 10 graduation = 234 total)

| Domain | Tests | Status |
|--------|-------|--------|
| data_loader | 12 | Full data + validations |
| combat | 23 | S01-S06 |
| status | 31 | S07-S13 + config stacks |
| djinn | 29 | State machine + correct recovery timing |
| equipment | 15 | Loadout + bonuses |
| damage_mods | 14 | Pen/splash/chain |
| battle_engine | 37 | Full integration + audit fixes |
| cli_runner | 9 | Interactive + auto + SPD planning |
| ai | 14 | 3 strategies |
| save | 10 | Roundtrip + versioning |
| progression | 17 | XP/leveling/ability unlock |

## Audit Status
- Audit round 1: 1 CRITICAL + 5 ERROR + 8 WARNING → all fixed
- Audit round 2: 1 CRITICAL + 3 ERROR + 5 WARNING → all fixed
- GPT external audit: 4 items → all resolved
- Schema verification: clean (no dead mechanics)

## Verified [Observed] Claims
- Planning order = SPD order = execution order
- Djinn recovery = 2 turns (turn after next)
- Damage formulas match spec (physical + psynergy + healing with MAG)
- Freeze breaks by damage accumulation
- Barriers block per-instance, don't block status
- Immunity ticks and expires
- Chain/splash/penetration wired
- Revive restores dead allies
- Buffs/debuffs affect damage calculation
- Save IDs match data IDs

## Remaining P1
- [ ] Bevy visual app (blocked by environment — needs display libs)
- [ ] always_first_turn equipment flag not enforced
- [ ] Integration tests for advanced ability fields
- [ ] Save migration path (currently strict version match)

## P2
- [ ] SPD tiebreaker depth (2/4 levels)
- [ ] Poison at full HP = 0 damage (correct per formula, design choice)
