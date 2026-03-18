TASK: Make the HUD and planning surface reflect live execution state so persistent UI stays consistent while a round resolves and after playback finishes.

CONTEXT:
- `src/domains/ui/hud.rs:79-203` builds the HUD once from startup `BattleRes`. HP bars, HP text, crit counters, and mana readout never refresh afterward.
- `src/domains/ui/hud.rs:166-199` only shows filled vs empty mana circles from `current_mana`; projected mana from queued attacks is not surfaced.
- `src/domains/ui/planning.rs:330-352` has `PlanningMode::Executing` and a minimal `RoundComplete` view, but `src/domains/ui/planning.rs:489-505` never enters `Executing`; it jumps straight to `RoundComplete` or `BattleOver`.
- `src/domains/ui/animation.rs:57-91` plays `state.last_events`, but when the queue drains it only sets `queue.playing = false`; nothing hands the planning surface back from execution to post-round state.

WHAT TO DO:
1. In `src/domains/ui/hud.rs`, add the marker components and helper functions needed to mutate the existing HUD nodes instead of leaving them at startup values. Before: HP bars, HP text, crit counters, and mana readout stay stale after execution. After: those nodes can be refreshed from live `BattleRes` and `PlanningState`.
2. Still in `src/domains/ui/hud.rs`, change the mana display from simple filled/empty pips to guaranteed-versus-projected pips:
   - `0..current_mana` stays the current solid-blue mana
   - `current_mana..current_mana + projected_mana` uses a distinct projected color
   - remaining capacity stays dark/inactive
3. In `src/domains/ui/planning.rs`, extend `PlanningState` so it can remember the post-playback destination (`RoundComplete` or `BattleOver`) and make `PlanningMode::Executing` real. After the last player action is planned, keep the panel in `Executing` until playback finishes instead of jumping directly to `RoundComplete`.
4. Still in `src/domains/ui/planning.rs`, update `update_planning_ui` so it refreshes the HUD helper from Step 1 and renders better execution/post-round feedback:
   - during `Executing`, show that the round is resolving instead of immediately showing `Round complete!`
   - after playback, replace the current bare `Round complete!` block with a compact summary sourced from `state.last_events`
   - include at least mana delta, any `DjinnChanged` entries, any `UnitDefeated` entries, and the `Next Round` button below the summary
5. In `src/domains/ui/animation.rs`, when the final queued event finishes, move `PlanningState` out of `Executing` into the stored post-playback mode. Also expose playback progress to the planning surface so the user can tell the round is resolving rather than frozen.

SCOPE: ONLY modify `src/domains/ui/hud.rs`, `src/domains/ui/planning.rs`, and `src/domains/ui/animation.rs`.

DO NOT:
- Do not modify `src/domains/ui/battle_scene.rs` or `src/domains/ui/plugin.rs`.
- Do not touch `src/shared/mod.rs`, battle-engine events/rules, scripts, docs, `STATE.md`, or `MANIFEST.md`.
- Do not redesign the overall planning layout beyond the execution-state and post-round summary work above.

DRIFT CUE: If you start adding new combat events, moving logic into `src/domains/battle_engine/`, or rebuilding the whole UI tree from scratch, stop. This wave is only HUD sync plus execution/post-round feedback.

VALIDATION:
- `cargo check`
- `cargo test`
- `cargo clippy -- -D warnings`
- `timeout 8 cargo run -- --gui`
- Manual GUI check in the demo battle:
  - queue at least one ATTACK so projected mana appears in the HUD
  - resolve a round and verify the planning panel stays in `Executing` until floating event playback ends
  - after playback, confirm HUD HP/mana/crit values and the round summary match the executed events before pressing `Next Round`

COMMIT: `git add src/domains/ui/hud.rs src/domains/ui/planning.rs src/domains/ui/animation.rs && git commit -m "feat(ui): sync hud and execution feedback"`
