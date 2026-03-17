# AI Orchestration Kit — Quick Start

## What's in here

```
docs/
  AI_ORCHESTRATION_PLAYBOOK.md  — THE methodology (load into project knowledge)
  SUB_AGENT_PLAYBOOK.md         — Phase 0-6 procedure for multi-domain builds

orchestration/
  GAME_DIRECTOR_AGENT.md        — Role doc for Layer 0 (creative direction agent)
  ORCHESTRATOR_AGENT.md         — Role doc for Layer 1 (technical orchestrator)
  FOREMAN_PLAYBOOK.md           — Instructions for foreman agent (Layer 2)
  run-stack.sh                  — 4-layer automated launcher

scripts/
  clamp-scope.sh                — Scope enforcement (run after EVERY worker)
  run-gates.sh                  — Validation pipeline (compile + test + lint + contract)
  checkpoint-state.sh           — Save orchestration state (conversation + files + ledger)
  restore-checkpoint.sh         — Restore from checkpoint when a wave goes wrong
  launch-lane.sh                — Create isolated work lane (worktree + branch)
  hook-contract-integrity.sh    — Pre-commit: reject contract modifications
  hook-agent-guard.sh           — Pre-commit: prevent orchestrator editing source
  hook-session-freshness.sh     — Warn if STATE.md not read recently
  install-hooks.sh              — Wire all hooks into .git/hooks/
```

## New project setup (Day 1)

1. Create your repo and add engine dependencies
2. Copy this entire kit into the repo root
3. Load `docs/AI_ORCHESTRATION_PLAYBOOK.md` into Claude project knowledge
4. Run `bash scripts/install-hooks.sh`
5. Create and freeze the type contract:
   ```bash
   # Write src/shared/mod.rs (or equivalent) with all cross-domain types
   shasum -a 256 src/shared/mod.rs > .contract.sha256
   git commit -m "chore: freeze shared type contract"
   ```
6. Write MANIFEST.md (phase, domains, decisions)
7. Write docs/spec.md (with QUANTITIES — "80 weapons" not "lots")
8. Edit `scripts/run-gates.sh` to set your build/test/lint commands:
   ```bash
   BUILD_CHECK_CMD="cargo check"   # or: npm run build, tsc --noEmit, etc.
   TEST_CMD="cargo test"           # or: npm test, pytest, etc.
   LINT_CMD="cargo clippy"         # or: eslint, ruff, etc.
   CONTRACT_FILE="src/shared/mod.rs"  # or: src/shared/types.ts, etc.
   ```
9. First dispatch:
   ```bash
   claude -p "Create the game window, camera, and player sprite with WASD movement" \
     --dangerously-skip-permissions
   ```

## Daily workflow

```bash
# Option A: Direct dispatch (Tier S — single fix)
claude -p "TASK: [exact spec]. SCOPE: [files]. COMMIT: [message]" --dangerously-skip-permissions

# Option B: Foreman dispatch (Tier M — multiple improvements)
claude -p "Read orchestration/FOREMAN_PLAYBOOK.md. Execute it." --dangerously-skip-permissions

# Option C: Full stack (Tier C — campaign)
bash orchestration/run-stack.sh "your creative direction here"
```

After every worker: `bash scripts/clamp-scope.sh src/domain/`
After every wave: `bash scripts/run-gates.sh`

## Campaign dispatch (walk-away autonomous)

Write a manifest (see AI_ORCHESTRATION_PLAYBOOK.md Section 13 — Audit template).
Then dispatch:

```bash
claude -p "Read docs/MANIFEST.md. Execute all waves. DO NOT stop between waves." \
  --dangerously-skip-permissions
```

## Session recovery

```
Continuing [project]. Read before acting:
- STATE.md, MANIFEST.md, docs/spec.md, src/shared/mod.rs
Recent: [1 sentence]. Task: [what to do now].
```
