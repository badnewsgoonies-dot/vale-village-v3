# Vale Village v3 — Current State

**Phase:** Visual layer hardening and state migration
**HEAD:** daad7f5
**Date:** 2026-03-17

## Spine Status: IN PROGRESS
The CLI path loads full data, save state, progression, and deterministic battle execution end-to-end, and the Bevy build now exposes a planning-panel djinn menu, summon tiers, current ability kit, and battle event playback on the GUI surface.

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
| ui | 1761 | 3 | YES | [Observed] Planning panel now surfaces djinn/summon actions and current ability kit; plugin wires event playback |

## Gate Status
- [x] Contract checksum: OK
- [x] Compile: `cargo check` OK
- [x] Tests: 230 passed, 0 failed
- [x] Lint: `cargo clippy -- -D warnings` OK
- [x] Connectivity: OK

## P0 Debt (blocks shipping)
- [ ] Interactive harden pass for the new GUI djinn/summon flow and recovery/ability-swap visibility — [Assumed] until manually verified on the live surface

## P1 Debt (blocks next milestone)
- [ ] Pre-battle team/equipment/djinn assignment surface is not implemented
- [ ] Djinn actions are surfaced through the planning panel, but direct djinn-sprite interaction from the battle scene is still not implemented
- [ ] `verify-state-claims.sh` is still absent, so persistent claim verification is only partially automated

## P2 Debt (nice to fix)
- [ ] Four-level SPD tiebreaker is not fully implemented — [Assumed] from `status/workers/combat.md`
- [ ] Same-element djinn 2+2 ability-count enforcement needs explicit verification — [Assumed] from prior state and missing targeted tests
- [ ] Equipment content audit for zero-power stub abilities still needs a dedicated pass — [Assumed] from prior state

## Verified Claims
- [Observed] The shared contract remains frozen and matches `.contract.sha256` — `src/shared/mod.rs`, `.contract.sha256`
- [Observed] The CLI spine loads data, save state, progression, and battle execution from the main binary — `src/main.rs`
- [Observed] The GUI planning surface now shows current djinn state, current ability kit, djinn activation buttons, and summon tier buttons — `src/domains/ui/planning.rs`
- [Observed] Battle event playback systems are wired into `ValeVillagePlugin` and consume `BattleEvent`s from the planning state — `src/domains/ui/plugin.rs`, `src/domains/ui/animation.rs`
- [Observed] The global automated gate set is green after the GUI planning wave — `cargo clippy -- -D warnings`, `cargo test`, `bash scripts/run-gates.sh`
- [Observed] A launch probe with `timeout 8 cargo run -- --gui` produced no startup error before timeout, but it did not constitute a manual harden pass
- [Assumed] The current GUI flow is fully understandable and operable by a player without a manual audit — NEEDS VERIFICATION before a GUI shipping decision depends on it

## Open Questions
- Does the GUI currently make djinn activation, summon preemption, and recovery order legible enough to count as a shippable interactive battle slice? — blocks GUI milestone
- Should direct djinn interaction move from planning-panel buttons to battle-scene sprites to match the spec more literally? — blocks final UI fidelity
- Once all downstream tooling has been migrated, should `.memory/STATE.md` be removed entirely instead of mirrored for compatibility? — blocks cleanup only
