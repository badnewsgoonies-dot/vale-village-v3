# UI Djinn Sprites Foreman Plan

## Current Observed UI Behavior

- `src/domains/ui/battle_scene.rs:61-177` spawns the battle scene once at startup: player/enemy colored rectangles, name labels, player HP text, and top-center mana/round/phase labels. It does not render djinn markers, clickable sprite-adjacent controls, or any battle-scene refresh system.
- `src/domains/ui/plugin.rs:65-77` only wires `battle_scene::setup_battle_scene` on `Startup`; no `battle_scene` update system runs during planning or after round execution.
- `src/domains/ui/planning.rs:139-258` is the only current djinn surface. The acting unit gets a text djinn summary plus `ACTIVATE ...` and `SUMMON Tn` buttons inside the planning panel.
- `src/domains/ui/hud.rs:79-203` builds a bottom HUD from the initial `BattleRes` only. HP bars, HP text, crit counters, and mana pips are spawned once and never refreshed.
- `src/domains/ui/animation.rs:57-203` provides transient floating feedback for damage, healing, crits, status, mana, djinn changes, and KOs, but it does not hand execution playback back to a persistent HUD/planning-state sync path.

## Top 2 Worker Tasks

### Worker 1

- Task: Add battle-scene djinn markers and direct interaction near player sprites.
- Why this is next: `docs/spec.md:31-33` says djinn activation and summons should come from clicking the djinn sprite. Right now that interaction exists only in the planning panel, so the battle scene is missing the spec-facing affordance entirely.
- Own these files:
  - `src/domains/ui/battle_scene.rs`
  - `src/domains/ui/plugin.rs`
- Must not touch:
  - `src/domains/ui/planning.rs`
  - `src/domains/ui/hud.rs`
  - `src/domains/ui/animation.rs`
  - anything under `src/shared/`
  - any non-UI domain, scripts, or docs

### Worker 2

- Task: Keep HUD and planning feedback in sync with executed state so persistent UI matches the round that just resolved.
- Why this is next: after a round resolves, the player only gets transient floating text. The static HUD and instant `Round complete!` panel can drift from the actual battle state, which is the highest-value experiential gap after direct sprite-adjacent djinn interaction exists.
- Own these files:
  - `src/domains/ui/hud.rs`
  - `src/domains/ui/planning.rs`
  - `src/domains/ui/animation.rs`
- Must not touch:
  - `src/domains/ui/battle_scene.rs`
  - `src/domains/ui/plugin.rs`
  - anything under `src/shared/`
  - any non-UI domain, scripts, or docs

## Why These 2 Waves

- Wave 1 closes the literal spec gap first without leaving `src/domains/ui/`: the battle scene becomes the djinn interaction surface instead of a passive backdrop.
- Wave 2 then fixes the player-trust problem created by static UI surfaces. Once the battle scene affordance exists, the next best value is making HUD and planning state tell the same truth as execution playback.
- The split is mechanical: Worker 1 owns scene-side rendering/interaction, and Worker 2 owns persistent HUD/planning feedback. That keeps the write sets narrow and avoids routing work into `src/shared/` or combat logic.
