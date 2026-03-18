# Foreman Rotation — UI Djinn Sprite Workers

You are the foreman for the next rotating cadence wave.

You do NOT write game code directly.
You do NOT modify any file under `src/`.
Your job is to read the current UI code, write exact worker objectives, and stop.

## Read First
1. orchestration/FOREMAN_PLAYBOOK.md
2. docs/SUB_AGENT_PLAYBOOK.md
3. MANIFEST.md
4. STATE.md
5. docs/spec.md
6. src/shared/mod.rs
7. src/domains/ui/battle_scene.rs
8. src/domains/ui/planning.rs
9. src/domains/ui/plugin.rs
10. src/domains/ui/hud.rs
11. src/domains/ui/animation.rs

## Hard Constraints
- Do NOT edit any file under `src/`
- Do NOT edit `scripts/`
- Do NOT edit `docs/spec.md`
- Do NOT dispatch code changes yourself
- Do NOT stop after a vague summary

## What To Produce
Create these files and only these files:

1. `status/foreman/ui-djinn-sprites-plan.md`
2. `objectives/generated/ui-djinn-sprites-worker-1.md`
3. `objectives/generated/ui-djinn-sprites-worker-2.md`

## Plan File Requirements
In `status/foreman/ui-djinn-sprites-plan.md`, write:
- the current observed UI behavior
- the top 2 worker tasks to close the highest-value gap
- why these 2 tasks are the right next waves
- which files each worker should own
- what each worker must NOT touch

## Worker Objective Requirements
For each worker objective:
- use exact file paths
- use exact before/after changes when possible
- keep scope inside `src/domains/ui/`
- include:
  - `TASK`
  - `CONTEXT`
  - `WHAT TO DO`
  - `SCOPE`
  - `DO NOT`
  - `DRIFT CUE`
  - `VALIDATION`
  - `COMMIT`

## Worker Split
Worker 1 should focus on battle-scene djinn markers and direct interaction near player sprites.
Worker 2 should focus on state-sync and feedback so the battle scene, HUD, and planning surface stay consistent after execution.

## Validation
Before stopping, verify that:
- all 3 files exist
- no file under `src/` was modified
- your output is concrete enough that a worker could execute it without asking clarifying questions

When done, stop.
