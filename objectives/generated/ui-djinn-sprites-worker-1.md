TASK: Add live djinn markers beside each player sprite and make the acting unit's djinn directly clickable from the battle scene during planning.

CONTEXT:
- `docs/spec.md:31-33` says djinn activation and summons should come from the djinn menu by clicking the djinn sprite.
- `src/domains/ui/battle_scene.rs:71-108` currently spawns only the player rectangle, name label, and HP text. There is no djinn marker row or sprite-adjacent interaction surface.
- `src/domains/ui/planning.rs:200-248` currently exposes djinn activation and summon buttons only inside the planning panel.
- `src/domains/ui/plugin.rs:65-77` registers no `battle_scene` update system, so the scene cannot react to `BattleRes` or `PlanningState` changes after startup.

WHAT TO DO:
1. In `src/domains/ui/battle_scene.rs`, add battle-scene marker components for player djinn slots. Before: each player unit shows only sprite/name/HP. After: each equipped djinn slot renders as a visible marker anchored immediately beside that player's sprite, using the djinn display name from `GameDataRes` plus a state label:
   - `GOOD` for `DjinnState::Good`
   - `REC` or `REC:n` for `DjinnState::Recovery`
2. Still in `src/domains/ui/battle_scene.rs`, use `planning::PlanningState` to highlight only the current acting unit's marker row. Good-state markers on the acting unit must become clickable controls that reuse the existing planning pathway by emitting `planning::ActionButton(planning::ActionChoice::ActivateDjinn(slot_idx, name.clone()))`. Recovery markers stay visible but non-clickable.
3. Still in `src/domains/ui/battle_scene.rs`, surface summon buttons immediately adjacent to the acting unit's marker row by reusing `planning::ActionChoice::Summon(djinn_indices, tier)`. Mirror the current actor-local summon availability exactly; do not reinterpret summon rules or change combat semantics in this wave.
4. Still in `src/domains/ui/battle_scene.rs`, add a sync system that rebuilds or refreshes the marker overlay whenever `BattleRes` or `PlanningState` changes so marker state, recovery-turn text, and acting-unit highlight stay correct after activation, execution, and `Next Round`.
5. In `src/domains/ui/plugin.rs`, register the new `battle_scene` sync/update system in `Update` after `planning::update_planning_ui`. Keep the planning-panel djinn controls intact as fallback; this wave adds the battle-scene affordance instead of deleting the current one.

SCOPE: ONLY modify `src/domains/ui/battle_scene.rs` and `src/domains/ui/plugin.rs`.

DO NOT:
- Do not modify `src/domains/ui/planning.rs`, `src/domains/ui/hud.rs`, or `src/domains/ui/animation.rs`.
- Do not touch `src/shared/mod.rs`, battle-engine logic, summon/djinn rules, scripts, docs, `STATE.md`, or `MANIFEST.md`.
- Do not replace the existing planning-panel djinn menu.

DRIFT CUE: If you start refactoring planning state, changing summon validation, or building a reusable UI framework, stop. This wave is only the battle-scene marker/interaction layer.

VALIDATION:
- `cargo check`
- `cargo test`
- `cargo clippy -- -D warnings`
- `timeout 8 cargo run -- --gui`
- Manual GUI check in the demo battle:
  - each player sprite shows its equipped djinn marker
  - only the acting unit's Good marker(s) are clickable
  - clicking a Good marker advances planning exactly like the existing panel button
  - the planning-panel djinn buttons still work after the scene-side controls are added

COMMIT: `git add src/domains/ui/battle_scene.rs src/domains/ui/plugin.rs && git commit -m "feat(ui): add battle-scene djinn marker interaction"`
