# Vale Village v3 — Current State

**Phase:** Wave 9 complete — P0 debt clear, orchestrator takeover
**HEAD:** 098f524
**Date:** 2026-03-17

## Engine Status: VERIFIED
Headless combat engine fully audited by 3 independent Codex workers.
All DESIGN_LOCK rules verified correct. Both audit gaps patched.

10,740 LOC | 227 tests (217 unit + 10 graduation) | 46 commits

## All Gates GREEN
- [x] Contract checksum: OK
- [x] cargo check: clean
- [x] cargo test: 227 passed, 0 failed
- [x] Audit: all DESIGN_LOCK rules confirmed in code

## P0 Debt: CLEAR
- [x] Djinn activation immediate effect — FIXED (098f524)
- [x] Projected mana from planned ATTACKs — FIXED (098f524)

## P1 Debt (next wave targets)
- [ ] End-to-end demo: equip djinn + items, activate, summon, recover, verify ability swap visible
- [ ] Bevy visual layer (rendering, sprites, UI)
- [ ] Interactive planning UI (click to select actions)
- [ ] Pre-battle screen (team select + equipment + djinn assignment)

## P2 Debt
- [ ] SPD tiebreaker 4-level (2 implemented)
- [ ] Same-element 2+2 ability count enforcement
- [ ] 10 equipment stub abilities zero-power
