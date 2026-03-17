# Vale Village v3 — Current State

**Phase:** Wave 6 complete — full data loads, spine scaffolded
**HEAD:** d84bad2 (pre-commit)
**Date:** 2026-03-17

## Milestone
Full v2 dataset converted and loading: 346 abilities, 137 enemies, 109 equipment, 23 djinn, 55 encounters, 11 units. `cargo run` executes a complete battle with full data loaded.

## Completed Domains

| Domain | Tests | Status |
|--------|-------|--------|
| data_loader | 9 | Full data loads |
| combat | 23 | All formulas working |
| status | 27 | All 6 status types |
| djinn | 26 | State machine + oscillation |
| equipment | 15 | Loadout + bonuses |
| damage_mods | 14 | Pen/splash/chain |
| battle_engine | 21 | Integration layer |
| cli_runner | 3 | cargo run works |
| **Total** | **138** | |

## Data Counts (all [Observed])
- Abilities: 346 (241 base + 105 djinn/equipment stubs)
- Enemies: 137
- Equipment: 109
- Djinn: 23
- Encounters: 55
- Units: 11

## Gate Status (all PASS)
Contract, cargo check, cargo test (138), clippy, connectivity, scope clamp, cargo run

## P0 Debt
- [ ] Enemy actions in battle (enemies don't attack)
- [ ] 105 stub abilities need real stats designed
- [ ] Demo should use a real encounter from encounters.ron

## P1 Debt
- [ ] Bevy app
- [ ] Player input
- [ ] Enemy AI
- [ ] Graduation tests for: full data load, battle completion, retargeting

## [Assumed] on Critical Path
1. Djinn recovery delay = 1 turn (possibly 2 per DESIGN_LOCK)
2. SPD tiebreaker = 2 levels (DESIGN_LOCK says 4)
3. 105 stub abilities are zero-power placeholders — game balance incomplete
