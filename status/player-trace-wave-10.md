# Player Trace — Wave 10 (updated)

**Date:** 2026-03-21
**HEAD:** ea22df6
**Phase:** Wave 10 — sprite generation complete, animation wired

## First-60-Seconds Trace

The player runs `cargo run --bin vale-village` and the CLI boots, loading game data from `data/full/` (11 units, 137 enemies, 23 djinn, 109 equipment, 55 encounters, 346 abilities). [Observed] — data_loader test confirms exact counts.

The CLI loads or creates a save file at `saves/game.ron`, resolves the next encounter, builds a battle, and enters the turn-based battle loop. [Observed] — code traced in `src/main.rs:63-260`.

In `--gui` mode, the battle scene renders with real Imagen 3 sprites for all units and enemies: idle poses during Planning, attack sprites when dealing damage, hit sprites when receiving damage, auto-revert to idle after 0.4s. [Observed] — `screenshot_battle.png` via xvfb, VLM confirmed real sprites at 0.9 confidence.

The bottom HUD shows HP bars, mana orbs, and unit names. Floating damage text, status labels, and KO markers appear during the Executing phase. [Observed] — planning.rs, animation.rs, hud.rs wired in plugin.rs.

Save/load preserves full game state — P0 graduated via `graduation_save_load_full_state_roundtrip`. [Observed] — 12/12 save tests pass.

## Reality Gates

| Gate | Status | Evidence |
|---|---|---|
| EntryPoint | [Inferred] | `cargo check` passes. OOM on link in 4GB container. screenshot_battle runs. |
| First-60-Seconds | [Observed] partial | Boot → data load → battle scene → planning → sprites + HUD + animation. No title/menu screen. |
| Asset Reachability | [Observed] COMPLETE | 336 sprite files: 134 idle + 134 attack + 134 hit (enemies) + 23 djinn + 11 units. Zero missing. |
| Content Reachability | [Observed] | 55 encounters, 11 units, 23 djinn, 109 equipment, 346 abilities all load. |
| Event Connectivity | [Observed] | 1 event type (DjinnActivationEvent) — not orphaned. Direct function calls. |
| Save/Load Round-Trip | [Observed] | P0 graduated. 12/12 tests pass. |
| VLM Verification | [Observed] | Real sprites confirmed via Gemini 2.5 Flash schema-enforced assertion. |

## P0 Debt

- [ ] Main binary OOM on link in 4GB container (works on >4GB machines)

## P1 Debt

- [ ] Pre-battle team/equipment/djinn assignment surface
- [ ] No title screen, menu, or new-game flow
- [ ] No audio system
- [ ] Player unit attack/hit pose sprites (currently reuse idle)

## P2 Debt

- [ ] verify-state-claims.sh absent
- [ ] GUI harden pass for djinn activation/summon legibility

## Gate Status

- [x] Contract checksum: OK
- [x] Compile: `cargo check` clean
- [x] Test compile: `cargo check --tests` clean
- [x] Tests: 232 lib pass (incl P0 save graduation)
- [x] Screenshot render: real sprites confirmed
- [x] VLM verification: PASS
- [x] Asset reachability: 336/336 sprites present — COMPLETE
- [ ] Full binary link: OOM in 4GB container
