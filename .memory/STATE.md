# Vale Village v3 — Current State

**Phase:** Visual Wave 5 complete — ready for Wave 6 (djinn interaction)
**HEAD:** 8fcddbb
**Date:** 2026-03-17

## Build Status
12,115 LOC | 227+ tests | 53 commits

### Headless Engine: VERIFIED
- All DESIGN_LOCK rules confirmed by 3 independent auditors
- 217 unit tests + 10 graduation tests, all green (last verified at cb955bd)
- Contract checksum passes

### Visual Layer: 5 WAVES COMPLETE (unverified in container)
Bevy compile requires GPU headers — cannot gate-check from Claude container.
Must verify on local machine with `cargo run -- --gui`.

| Wave | Feature | LOC | Status |
|------|---------|-----|--------|
| 1 | Bevy bootstrap, camera, placeholder sprites | ~170 | Committed |
| 2 | Data-driven battle scene, element colors, name labels | ~177 | Committed |
| 3 | HUD: HP bars, mana circles, crit counters | ~279 | Committed |
| 4 | Planning phase: action/target selection, live mana | ~414 | Committed |
| 5 | Animation: floating damage, status icons, event playback | ~316 | Committed |

### UI Module (src/domains/ui/)
- plugin.rs (170 lines) — Bevy plugin, window, camera, demo battle setup
- battle_scene.rs (177 lines) — spawn units/enemies with element colors
- hud.rs (279 lines) — HP bars, mana circles, crit counter badges
- planning.rs (414 lines) — action selection state machine, target picking
- animation.rs (316 lines) — event queue playback, floating numbers, status icons

## P0 Debt: NONE (engine)
- [x] Djinn activation immediate effect — FIXED
- [x] Projected mana from planned ATTACKs — FIXED

## P1 Debt
- [ ] VERIFY: Visual waves 1-5 compile and render on local machine
- [ ] Wave 6: Djinn interaction (click sprite → menu → activate/summon)
- [ ] Wave 7: Pre-battle screen (team select + equipment + djinn assignment)
- [ ] Wave 8: Out-of-battle screens (shop, character details, abilities)

## P2 Debt
- [ ] Feature-gate Bevy (allow headless build without GPU)
- [ ] SPD tiebreaker 4-level (2 implemented)
- [ ] Same-element 2+2 ability count enforcement
- [ ] Sprite pipeline (GIF → PNG atlas)
- [ ] Sound system
