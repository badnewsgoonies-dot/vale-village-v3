# Vale Village v3 — Current State

**Phase:** Wave 8 integrated — GUI djinn scene interaction and feedback sync
**HEAD:** 5ad1879
**Date:** 2026-03-17

## Spine Status: IN PROGRESS
The CLI path loads full data, save state, progression, and deterministic battle execution end-to-end, and the Bevy battle UI now supports sprite-adjacent djinn controls, synced HUD feedback, and execution/playback state transitions on the live GUI surface.

## Domains

| Domain | LOC | Tests | Wired? | Verified |
|--------|-----|-------|--------|----------|
| shared | 460 | 0 | YES | [Observed] Contract frozen in `src/shared/mod.rs` and checksummed in `.contract.sha256` |
| ai | 581 | 14 | YES | [Observed] Unit tests pass and battle engine imports AI decision logic |
| battle_engine | 3472 | 48 | YES | [Observed] Core execution path covered by domain tests and graduation tests |
| cli_runner | 1630 | 6 | YES | [Observed] `src/main.rs` calls demo battle flow through this domain |
| combat | 637 | 23 | YES | [Observed] Damage, targeting, crit, mana, and ordering tests pass |
| damage_mods | 217 | 14 | YES | [Observed] Penetration, splash, and chain logic tested and used by battle engine |
| data_loader | 546 | 12 | YES | [Observed] Full data load is exercised by `src/main.rs`, UI plugin, and graduation tests |
| djinn | 735 | 27 | YES | [Observed] State machine, summon, and recovery tests pass |
| equipment | 414 | 15 | YES | [Observed] Equipment effects are tested and used in demo battle construction |
| progression | 349 | 17 | YES | [Observed] XP/stat growth tests pass and rewards are applied in `src/main.rs` |
| save | 414 | 10 | YES | [Observed] Save/load roundtrips pass and campaign state is read/written in `src/main.rs` |
| status | 1001 | 31 | YES | [Observed] Status, barrier, HoT, buff, and cleanse logic tested and consumed by battle engine |
| ui | 2775 | 5 | YES | [Observed] Battle scene shows djinn rows beside active units, HUD syncs live state, and execution transitions persist through playback |

## Gate Status
- [x] Contract checksum: OK
- [x] Compile: `cargo check` OK
- [x] Tests: 232 passed, 0 failed
- [x] Lint: `cargo clippy -- -D warnings` OK
- [x] Connectivity: OK

## P0 Debt (blocks shipping)
- [ ] Final interactive harden pass for click-through djinn activation/summon behavior and recovery legibility — [Assumed] until manually exercised on the live surface

## P1 Debt (blocks next milestone)
- [ ] Pre-battle team/equipment/djinn assignment surface is not implemented
- [ ] Top-center battle text duplicates HUD information and could be reduced or restyled
- [ ] `verify-state-claims.sh` is still absent, so persistent claim verification is only partially automated

## P2 Debt (nice to fix)
- [ ] Four-level SPD tiebreaker is not fully implemented — [Assumed] from `status/workers/combat.md`
- [ ] Same-element djinn 2+2 ability-count enforcement needs explicit verification — [Assumed] from prior state and missing targeted tests
- [ ] Equipment content audit for zero-power stub abilities still needs a dedicated pass — [Assumed] from prior state

## Verified Claims
- [Observed] The shared contract remains frozen and matches `.contract.sha256` — `src/shared/mod.rs`, `.contract.sha256`
- [Observed] The CLI spine loads data, save state, progression, and battle execution from the main binary — `src/main.rs`
- [Observed] The battle scene now renders djinn rows beside player sprites and supports scene-side activation/summon affordances — `src/domains/ui/battle_scene.rs`, `src/domains/ui/plugin.rs`
- [Observed] The planning/HUD layer now stays in `Executing` through playback and refreshes HP, mana, crit, and round summary from live state — `src/domains/ui/planning.rs`, `src/domains/ui/hud.rs`, `src/domains/ui/animation.rs`
- [Observed] The integrated GUI launches successfully on `DISPLAY=:0` and renders the new scene-side djinn controls — manual launch plus screenshot on 2026-03-17
- [Observed] The global automated gate set is green after Wave 8 integration — `cargo clippy -- -D warnings`, `cargo test`, `bash scripts/run-gates.sh`
- [Assumed] The current GUI flow is fully understandable and operable by a player without a manual audit — NEEDS VERIFICATION before a GUI shipping decision depends on it

## Open Questions
- Does the GUI currently make djinn activation, summon preemption, and recovery order legible enough to count as a shippable interactive battle slice? — blocks GUI milestone
- Should direct djinn interaction move from planning-panel buttons to battle-scene sprites to match the spec more literally? — blocks final UI fidelity
- Once all downstream tooling has been migrated, should `.memory/STATE.md` be removed entirely instead of mirrored for compatibility? — blocks cleanup only
