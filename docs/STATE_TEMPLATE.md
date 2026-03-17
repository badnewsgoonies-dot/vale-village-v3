# State Management Templates

> These files live at the repo root. They are the orchestrator's brain on disk.
> When a session dies, these files are how the next session recovers.

---

## STATE.md

> Current truth. Updated at the end of every session and every wave.

```markdown
# [Project] — Current State

**Phase:** [current phase and wave number]
**HEAD:** [git short hash]
**Date:** [ISO date]

## Spine Status: [NOT STARTED / IN PROGRESS / COMPLETE]
[One sentence describing what the current build can do end-to-end]

## Domains

| Domain | LOC | Tests | Wired? | Verified |
|--------|-----|-------|--------|----------|
| [name] | [n] | [n] | [YES/NO] | [Observed/Inferred/Assumed] + detail |

## Gate Status
- [ ] Contract checksum: [OK / FAIL]
- [ ] Compile: [OK / FAIL]
- [ ] Tests: [n passed, n failed]
- [ ] Connectivity: [OK / hermetic domains listed]

## P0 Debt (blocks shipping)
- [ ] [description] — [evidence level]

## P1 Debt (blocks next milestone)
- [ ] [description]

## P2 Debt (nice to fix)
- [ ] [description]

## Verified Claims
- [Observed] [claim] — [source_ref]
- [Assumed] [claim] — NEEDS VERIFICATION before [decision] depends on it

## Open Questions
- [question] — blocks [what]
```

---

## MANIFEST.md

> The orchestrator's plan. Less volatile than STATE.md. Updated when the plan changes, not every wave.

```markdown
# [Project] — Build Manifest

## Current Phase: [Phase N — description]

## Architecture
- Language: [Rust / TypeScript / Go / etc.]
- Framework: [Bevy / Next.js / Axum / etc.]
- Data format: [RON / JSON / YAML / etc.]
- Build method: [AI-orchestrated with playbook]

## Domain List

| Domain | Path | Owner | Status |
|--------|------|-------|--------|
| shared (contract) | src/shared/ | orchestrator | frozen |
| [domain] | src/domains/[name]/ | worker | [not started/in progress/complete] |

## Key Constants (from spec.md — duplicated here for quick reference)
- [CONSTANT = VALUE]
- [CONSTANT = VALUE]

## Wave Plan
- Wave 1: [scope] — [status]
- Wave 2: [scope] — [status]
- Wave N: [scope] — [status]

## Key Decisions
- [Decision]: [rationale] — [evidence level]

## Blockers
- [blocker]: [what it blocks] — [owner]
```

---

## status/dispatch-state.yaml

> Process table for active work lanes. Read by orchestrator to track parallel work.

```yaml
campaign_id: [project-name-phase]
baseline_branch: master
baseline_commit: [short hash]
phase: [current phase name]

active_lanes:
- lane_id: [domain-name]
  status: pending # pending | in-progress | gated | merged | failed
  goal: "[one sentence]"
  owned_paths:
  - src/domains/[name]/
  worker_model: claude-sonnet-4.6
  next_action: "[what happens next]"
  validation:
    compile: [pass/fail/pending]
    tests: [pass/fail/pending]
  report_path: status/workers/[name].md

- lane_id: [another-domain]
  status: in-progress
  goal: "[one sentence]"
  owned_paths:
  - src/domains/[name]/
  worker_model: gpt-5.4
  next_action: "Clamp scope, run gates"
  validation:
    compile: pending
    tests: pending
  report_path: status/workers/[name].md
```

---

## status/workers/[domain].md

> Written by the worker at completion. Read by integrator and orchestrator.

```markdown
# Worker Report: [DOMAIN]

## Files created/modified
- [file path] ([lines] lines) — [what it does]

## What was implemented
- [Feature 1]
- [Feature 2]

## Quantitative targets
- [Target]: [expected] → [actual] — [HIT/MISS]

## Shared type imports used
- [Type1, Type2, Enum1]

## Validation results
- Compile: [pass/fail]
- Tests: [n passed, n failed, n skipped]

## Assumptions made
- [Assumption] — [evidence level]

## Known risks / open items for integration
- [Risk 1]
- [Risk 2]
```

---

## status/integration.md

> Written during integration phase. Documents what was wired and what remains.

```markdown
# Integration Report

**Date:** [ISO date]
**Integrator:** [model/session]

## What was wired
- [Domain A] → [Domain B] via [mechanism]

## Contract amendments
- [Amendment 1]: [rationale]
- Checksum updated: [old hash] → [new hash]

## What remains unwired
- [Domain C] is hermetic — needs [specific wiring]

## New debt discovered
- [Debt item] — [evidence level]

## Final gate status
- Compile: [pass/fail]
- Tests: [n passed]
- Connectivity: [pass/fail]
- Contract checksum: [pass/fail]
```

---

## Quick Setup

```bash
# Create all state files from templates
mkdir -p status/workers
touch MANIFEST.md STATE.md status/dispatch-state.yaml
echo "# Integration pending" > status/integration.md

# Create scripts
mkdir -p scripts
# (copy clamp-scope.sh and run-gates.sh from playbook)

# Freeze contract
shasum -a 256 src/shared/mod.rs > .contract.sha256
git add -A && git commit -m "chore: initial project scaffold"
```
