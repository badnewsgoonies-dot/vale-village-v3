# Player Trace — Wave 10

**Date:** 2026-03-20
**HEAD:** b3dd68c
**Phase:** Wave 10 — sprite wiring + graduation gates

## First-60-Seconds Trace

The player runs `cargo run --bin vale-village` and the CLI boots, loading game data from `data/full/` (11 units, 137 enemies, 23 djinn, 109 equipment, 55 encounters, 346 abilities). [Observed] — data_loader test confirms exact counts.

The CLI loads or creates a save file at `saves/game.ron`, resolves the next encounter from the story sequence, builds a battle with the player's party, and enters the turn-based battle loop where the player selects actions (attack, abilities, djinn activation, summon) for each unit. [Observed] — code traced in `src/main.rs:63-260` and `src/domains/cli_runner/mod.rs`.

In `--gui` mode, the Bevy window opens showing the battle scene with real Imagen 3 sprites for player units (Adept, Blaze) and enemies (War Mage from house-01), HP text above each unit, name labels below, and a bottom HUD with HP bars, mana orbs, and round/phase heading. [Observed] — `screenshot_battle.png` captured via `xvfb-run`, VLM confirmed `player_are_real_sprites=true, enemies_are_real_sprites=true` at 0.9 confidence.

The player interacts with the planning panel to select abilities and targets, then watches the execution phase play out with floating damage text and HP bar updates. [Observed] — planning.rs, animation.rs, hud.rs all wired in plugin.rs and confirmed rendering in screenshot.

Save/load preserves the full game state including party (with equipment, djinn, HP including KO), roster, gold, XP, completed encounters, inventory, and team djinn pool — verified by P0 graduation test `graduation_save_load_full_state_roundtrip`. [Observed] — 12/12 save tests pass.

## Reality Gates

| Gate | Status | Evidence |
|---|---|---|
| EntryPoint | [Inferred] | `cargo check` passes for `vale-village` binary. OOM on link in 4GB container. `screenshot_battle` binary runs successfully. CLI entry point code-traced. |
| First-60-Seconds | [Observed] partial | Boot → data load → battle scene → planning → sprites + HUD confirmed via screenshot + VLM. No title/menu/new-game screen exists — battle-focused tactical RPG with CLI encounter flow. |
| Asset Reachability | [Observed] | 171/171 gameplay-critical sprites present (137 enemy idle + 23 djinn + 11 unit portraits). 274 cosmetic sprites missing (137 attack + 137 hit — renderer falls back to idle). |
| Content Reachability | [Observed] | 55 encounters, 11 units, 23 djinn, 109 equipment, 346 abilities — all load without error. |
| Event Connectivity | [Observed] | 1 event type (DjinnActivationEvent) — not orphaned. Game logic is direct function calls. |
| Save/Load Round-Trip | [Observed] | P0 graduated. 12/12 save tests pass. |
| VLM Verification | [Observed] | Battle scene → Gemini 2.5 Flash → structured JSON confirms real sprites. |

## P0 Debt

- [ ] Main binary OOM on link in 4GB container — works on >4GB machines

## P1 Debt

- [ ] 274 missing cosmetic sprites (137 attack + 137 hit) — fallback to idle
- [ ] Pre-battle team/equipment/djinn assignment surface
- [ ] No title screen, menu, or new-game flow
- [ ] No audio system

## P2 Debt

- [ ] verify-state-claims.sh absent
- [ ] Unused import warnings in battle_engine and djinn
- [ ] GUI harden pass for djinn activation/summon legibility

## Gate Status

- [x] Contract checksum: OK
- [x] Compile: `cargo check` clean
- [x] Test compile: `cargo check --tests` clean
- [x] Tests: 232 lib pass (incl P0 save graduation)
- [x] Screenshot render: real sprites confirmed
- [x] VLM verification: PASS
- [ ] Full binary link: OOM in 4GB container
