# Rotation Wave 8 — UI Djinn Sprite Interaction

You are running in an isolated lane worktree. Work from disk truth, not conversation.

## Read First
1. orchestration/FOREMAN_PLAYBOOK.md
2. docs/SUB_AGENT_PLAYBOOK.md
3. MANIFEST.md
4. STATE.md
5. docs/spec.md
6. src/shared/mod.rs

## Tier / Surface / Phase
- Tier: M
- Surface: `src/domains/ui/`
- Phase: Feature → Gate → Document → Harden → Graduate

## Critical-Path Facts
- [Observed] The current GUI planning panel already exposes ATTACK, ABILITY, djinn activation, summon tiers, and event playback.
- [Observed] The current game spec wants djinn interaction to live on the battle scene itself, not only in a menu/panel.
- [Assumed] The next highest-value reachable improvement is direct djinn interaction on or near the player sprites.

## Constraints
- Scope: ONLY modify `src/domains/ui/`, `status/workers/`, and state docs if you complete a verified wave.
- After each coding wave, run `bash scripts/clamp-scope.sh src/domains/ui/`.
- Do NOT edit `src/shared/`.
- Do NOT create orchestration infrastructure.
- Do NOT stop after one wave and ask for approval. Continue until exhausted or genuinely blocked.
- Commit after each completed wave in this lane.

## Wave Plan

### Wave 1
- Render each player unit's equipped djinn as visible markers anchored near the battle sprite.
- Show djinn state on or near those markers:
  - `Good`
  - `Recovery`
  - remaining recovery turns when present
- Keep the current planning panel operational as a fallback path.

### Wave 2
- Make djinn markers clickable during Planning mode.
- Clicking a `Good` djinn should plan `ActivateDjinn` for the current acting unit when valid.
- If enough `Good` djinn exist, expose summon selection from the battle-scene interaction surface or an immediately adjacent overlay anchored to the markers.
- Preserve ATTACK / ABILITY planning flow.

### Wave 3
- Keep battle scene, HUD, and planning state visually in sync after round execution for:
  - HP
  - mana
  - djinn state changes
- Reuse existing `BattleEvent` playback where possible instead of inventing duplicate feedback systems.

## Validation After Each Wave
- `cargo check`
- `cargo test`
- `cargo clippy -- -D warnings`
- `LINT_CMD='cargo clippy -- -D warnings' bash scripts/run-gates.sh`

## When Done
- Write `status/workers/ui-djinn-sprites.md` with:
  - files changed
  - completed waves
  - validation results
  - observed remaining gaps
- Update `STATE.md` and `MANIFEST.md` only if a wave is verified by the commands above.
