# Vale Village v3 — Current State

**Phase:** Wave 9 complete — Engine verified, ready for visual layer
**HEAD:** cb955bd
**Date:** 2026-03-17

## Engine Status: VERIFIED & AUDITED
Headless combat engine complete. 3 independent Codex auditors confirmed all
DESIGN_LOCK rules implemented correctly. Both audit gaps (djinn activation
effect, projected attack mana) patched and tested.

10,740 LOC | 227 tests | 47 commits | All gates green

## Domains (12 source files)

| Domain | LOC | Tests | Wired | Verified |
|--------|-----|-------|-------|----------|
| shared (contract) | 460 | — | all | [Observed] checksum passes |
| data_loader | 546 | 9 | main | [Observed] full data loads |
| combat | 637 | 23 | battle_engine | [Observed] formulas correct |
| status | 1001 | 27 | battle_engine | [Observed] 6 types, zero randomness |
| djinn | 707 | 26 | battle_engine, cli | [Observed] activate, recover, swap |
| equipment | 414 | 15 | battle_engine, cli | [Observed] equip, bonuses, validation |
| damage_mods | 217 | 14 | battle_engine | [Observed] pen, splash, chain |
| battle_engine | 2650 | 45 | main via cli | [Observed] full round lifecycle |
| cli_runner | 1615 | 3 | main | [Observed] cargo run works |
| ai | 581 | — | battle_engine | [Observed] 3 strategies |
| progression | 349 | — | main | [Observed] XP, leveling |
| save | 414 | — | main | [Observed] RON roundtrip |

## Gates: ALL GREEN
- [x] Contract checksum: OK
- [x] cargo check: clean
- [x] cargo test: 227 passed, 0 failed
- [x] Connectivity: all domains import shared

## P0 Debt: CLEAR

## P1 Debt (visual layer)
- [ ] Bevy rendering (sprites, UI, battle screen)
- [ ] Interactive planning UI
- [ ] Pre-battle screen
- [ ] Sprite pipeline (GIF→PNG atlas)

## P2 Debt
- [ ] SPD tiebreaker 4-level (2 implemented)
- [ ] Same-element 2+2 ability count enforcement
- [ ] 10 equipment stub abilities zero-power

## Research Data on Disk
- docs/research/POISONING_DEFENSE_MULTI_REPO.md — 125 trials, 5 models, 2 codebases
- docs/research/FINDINGS_PLAIN_ENGLISH.md — accessible summary of all findings
