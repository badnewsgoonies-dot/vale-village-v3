# Vale Village v3 — Current State

**Phase:** Wave 7 complete — two-sided combat works, spine scaffolded
**HEAD:** 9808d33 (pre-commit)
**Date:** 2026-03-17

## Spine Status
Boot → load full data (346 abilities, 137 enemies, 109 equipment, 23 djinn, 55 encounters, 11 units) → create battle from encounter → plan actions (both sides) → execute rounds (SPD-ordered, interleaved) → victory/defeat. **Working end-to-end.**

## Domains (8 complete, 144 tests)

| Domain | Tests | Reachable? |
|--------|-------|------------|
| data_loader | 9 | YES |
| combat | 23 | YES |
| status | 27 | YES (via battle_engine) |
| djinn | 26 | Unit tests only |
| equipment | 15 | Unit tests only |
| damage_mods | 14 | Unit tests only |
| battle_engine | 27 | YES |
| cli_runner | 3 | YES |

## Gate Status (all PASS)
Contract, check, test (144), clippy, connectivity, scope clamp, cargo run (two-sided battle)

## P0 Debt
- [ ] 105 stub abilities need real stats (djinn/equipment abilities are zero-power placeholders)

## P1 Debt
- [ ] Ability usage in demo (only auto-attacks exercised)
- [ ] Bevy app with visual rendering
- [ ] Player input (interactive planning)
- [ ] Smarter enemy AI
- [ ] Djinn/equipment exercised in battle demo
- [ ] Graduation tests

## [Assumed] on Critical Path
1. Djinn recovery delay = 1 turn (possibly 2)
2. SPD tiebreaker = 2 levels (DESIGN_LOCK says 4)
3. 105 stub abilities are placeholders — game balance incomplete
