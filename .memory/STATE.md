# Vale Village v3 — Current State

**Phase:** Wave 8 complete — P0 debt cleared, spine finished
**HEAD:** d650235 (pre-commit)
**Date:** 2026-03-17

## Spine Status: COMPLETE
Boot → load full data (346 abilities with real stats, 137 enemies, 109 equipment, 23 djinn, 55 encounters, 11 units) → create encounter-based battle → two-sided SPD-ordered combat → victory/defeat.

## Domains (8 domains, 144 tests)

| Domain | Tests | Reachable? | Verified? |
|--------|-------|------------|-----------|
| data_loader | 9 | YES | [Observed] full data loads |
| combat | 23 | YES | [Observed] damage formulas correct |
| status | 27 | YES | [Observed] via battle_engine |
| djinn | 26 | NO — ghost | [Observed] unit tests only |
| equipment | 15 | NO — ghost | [Observed] unit tests only |
| damage_mods | 14 | NO — ghost | [Observed] unit tests only |
| battle_engine | 27 | YES | [Observed] two-sided combat |
| cli_runner | 3 | YES | [Observed] cargo run works |

## Gate Status (all PASS)
Contract, check, test (144), clippy, connectivity, scope clamp, cargo run

## Verified Claims (upgraded from [Assumed])
- [Observed] Djinn recovery delay = 2 turns (matches DESIGN_LOCK "turn after next")
- [Observed] Equipment bonuses applied at battle init (but never exercised — ghost)
- [Observed] SPD tiebreaker = 2 levels (gap: DESIGN_LOCK says 4, filed as P2)

## P0 Debt: NONE

## P1 Debt
- [ ] Djinn exercised in demo (equip djinn on units, use activation/summon)
- [ ] Equipment exercised in demo (equip items on units)
- [ ] Ability usage in demo (use abilities, not just auto-attacks)
- [ ] Bevy visual app
- [ ] Player input (interactive planning)
- [ ] Enemy AI (smarter than "attack first alive")

## P2 Debt
- [ ] SPD tiebreaker depth (2/4 levels implemented)
- [ ] 10 equipment stub abilities still zero-power
