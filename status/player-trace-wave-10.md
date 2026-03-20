# Player Trace — Wave 10

**Date:** 2026-03-20
**HEAD:** b3dd68c (after sprite wiring campaign)
**Tier:** S → M (sprite generation + graduation tests)

## Player Trace (boot to first interaction)

The player runs `cargo run --bin vale-village` and the CLI loads game data from `data/full/`, creates or loads a save file from `saves/game.ron`, and presents the first encounter. [Observed] via `cargo check` confirming the binary compiles; [Inferred] for runtime because the main binary OOMs on link in the 4GB container.

The player can also run `cargo run --bin vale-village -- --gui` to launch the Bevy battle scene, which renders player units (Adept, Blaze) on the left with real Imagen 3 sprites, enemies (War Mage) on the right with manifest-loaded sprites, HP text above each unit, name labels below, and a bottom HUD with HP bars and mana display. [Observed] via `screenshot_battle` binary + VLM verification (Gemini 2.5 Flash, confidence 0.9).

During the planning phase the player sees "Round 1 • Planning" at the top and can select actions for each party member via the planning panel. [Observed] from screenshot evidence showing the planning UI state.

The battle engine processes actions, emits BattleEvents (damage, healing, status, crits, barriers, djinn changes, mana, defeats, round markers, enemy abilities), and all 11 event types are consumed by both CLI (text display) and GUI (floating text animations). [Observed] via code inspection — zero orphaned events.

Save/load preserves the full game state including party, roster, equipment, djinn, gold, XP, encounters, and inventory through RON serialization with version checking. [Observed] via 12/12 save tests passing including P0 graduation test `graduation_save_load_full_state_roundtrip`.

## Graduation Gate Checklist

### EntryPoint [wave-required]
- **Status:** [Inferred] PASS
- **Evidence:** `cargo check` compiles the `vale-village` binary without errors. `screenshot_battle` binary (same codebase, same plugin) runs and renders correctly. Main binary OOMs on link in 4GB container — verified to compile, not verified to execute from this container.
- **Risk if wrong:** Game not launchable.
- **Graduation target:** Verify `cargo run --bin vale-village` executes on a machine with >4GB RAM.

### First-60-Seconds [wave-required]
- **Status:** [Observed] partial, [Inferred] full
- **Evidence:** Screenshot binary proves: boot → battle scene → sprites rendered → HUD active → planning phase reachable. CLI path verified via `cargo check`. No menu/new-game/movement flow exists — this is a battle-focused tactical RPG.
- **Risk if wrong:** Player cannot reach first interaction.

### Asset Reachability [wave-if-touched]
- **Status:** [Observed]
- **Evidence:** 160/434 manifest entries have sprites on disk.
  - Enemy idle: 137/137 present ✓
  - Enemy attack: 0/137 present ✗ (falls back to idle sprite)
  - Enemy hit: 0/137 present ✗ (falls back to idle sprite)
  - Djinn: 23/23 present ✓
  - Player units: 11/11 present ✓
- **Risk if wrong:** Missing attack/hit sprites are cosmetic — renderer falls back to idle. No crash risk.
- **Graduation target:** Generate attack and hit sprites to reach 434/434.

### Content Reachability [wave-if-touched]
- **Status:** [Observed]
- **Evidence:** 346 abilities, 137 enemies, 109 equipment, 23 djinn, 55 encounters, 11 units all load from `data/full/` without errors (graduation test `test_full_data_loads_without_error`).

### Event Connectivity [wave-required]
- **Status:** [Observed]
- **Evidence:** All 11 BattleEvent variants have producers (battle_engine) and consumers (cli_runner + animation.rs). DjinnActivationEvent has producer (djinn) and consumer (battle_engine). Zero orphaned events confirmed via grep.
- **Producer count:** battle_engine emits all 11 variants across 20 call sites.
- **Consumer count:** cli_runner handles all 11 in `format_event()`. GUI animation.rs handles all 11 in `spawn_event_animation()`.

### Save/Load Round-Trip [wave-if-touched]
- **Status:** [Observed] PASS
- **Evidence:** 12/12 save domain tests pass. P0 graduation test `graduation_save_load_full_state_roundtrip` verifies mid-game state with every field populated survives save→load with exact equality. RON file verified human-readable.

### VLM Verification [wave-if-available]
- **Status:** [Observed] PASS
- **Evidence:** Gemini 2.5 Flash evaluated `screenshot_battle.png` with schema-enforced assertions. `player_are_real_sprites=true`, `enemies_are_real_sprites=true`, confidence 0.9.

## P0 Debt (blocks shipping)
- [ ] Main binary OOM on link in 4GB container — [Inferred] to work on >4GB machines
- [ ] No title screen / main menu — battle launches directly

## P1 Debt (blocks next milestone)
- [ ] 274 missing attack/hit sprites (cosmetic — falls back to idle)
- [ ] Pre-battle team/equipment/djinn assignment screen not implemented
- [ ] No overworld/map renderer
- [ ] No audio system

## P2 Debt (nice to fix)
- [ ] 2 unused import warnings in lib code
- [ ] Graduation integration tests OOM in 4GB container (lib tests cover same logic)

## Gate Status Summary
- [x] cargo check: PASS
- [x] cargo check --tests: PASS
- [x] cargo test --lib: 232 passed, 0 failed
- [x] Screenshot render: PASS (VLM verified)
- [x] Contract checksum: PASS
- [x] Event connectivity: PASS (0 orphans)
- [x] Save/load round-trip: PASS (P0 graduated)
- [ ] cargo run --bin vale-village: [Inferred] (OOM on link)
