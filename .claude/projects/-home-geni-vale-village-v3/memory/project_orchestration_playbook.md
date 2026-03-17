---
name: ai-orchestration-playbook
description: How this project is built - wave-based AI agent dispatch with mechanical scope enforcement, evidence-typed claims, and zero handwritten code
type: project
---

All code is written by AI agents. The orchestrator (Opus 4.6, 1M context) dispatches workers, never writes code directly.

**Why:** 739 commits, 64K LOC, 295M tokens across 98 sessions proved this methodology. Manual dispatch ships 67%, foreman dispatch ships 100%. Prompt-only scope control: 0/20. Mechanical clamp: 20/20.

**How to apply:**

**Role:** I am the orchestrator (Layer 1). I read codebase, write specs on disk, dispatch workers. I do NOT write implementation code directly.

**Wave cadence (never skip):** Feature → Gate → Document → Harden → Graduate

**Critical rules:**
1. Freeze type contract (src/shared/mod.rs) before any workers launch. Checksum it.
2. Commit after every wave BEFORE dispatching next worker
3. Run scope clamp after EVERY worker (revert out-of-scope edits mechanically)
4. Stagger parallel launches ~3s. 2-3 simultaneous stable, 5+ crashes
5. Prefer editing existing files (~90% success) over new files (~50%)
6. Wave-based dispatch, never one-shot for large builds
7. "DO NOT stop between waves" line sustains multi-wave campaigns

**Trust order (highest to lowest):**
1. Fresh code, tests, runtime output
2. [Observed] artifacts with source_refs
3. Current STATE.md
4. Project docs, contracts, specs
5. Research findings with certainty labels
6. Conversation history (LOWEST)

**Evidence levels for persisted claims:**
- [Observed] — directly verified, can freeze into gates
- [Inferred] — logically derived, cannot freeze
- [Assumed] — must verify before critical decisions depend on it

**Worker spec requires:** TASK, CONTEXT (file:line), WHAT TO DO (exact values), SCOPE (file list), DO NOT, COMMIT message

**Stop conditions:** Contract drift, clamp breaks fix, false green, abstraction reflex, delegation compression, ghost progress, cadence break

**Session protocol:** Start by reading playbook + STATE.md. State tier (S/M/C) before acting. End by updating STATE.md + writing triggered artifacts.

**Model notes:** gpt-4.1 fabricates verification. Sonnet 4.6 is resourceful coding worker. GPT-5.4 reliable scoped output.

**Orchestration kit (ai-orchestration-kit.zip) is available at repo root with:**
- `scripts/clamp-scope.sh` — multi-prefix scope enforcement with post-clamp verification
- `scripts/run-gates.sh` — 8-gate validation (contract, build, test, lint, connectivity, STATE freshness, artifact refs, claim verification)
- `scripts/checkpoint-state.sh` — composite checkpoint (session fork + filesystem + ledger)
- `scripts/restore-checkpoint.sh` — verified checkpoint restore with hash validation
- `scripts/launch-lane.sh` — isolated worktree lane launcher for Codex workers
- `scripts/install-hooks.sh` — pre-commit (contract integrity), pre-push (full gates), post-checkout (STATE freshness)
- `scripts/hook-contract-integrity.sh` — PostToolUse hook blocking contract edits
- `scripts/hook-agent-guard.sh` — PreToolUse hook auditing agent dispatches without specs on disk
- `scripts/hook-session-freshness.sh` — SessionStart hook checking STATE.md drift
- `orchestration/run-stack.sh` — 4-layer automated launcher (Director → Orchestrator → Foreman → Workers)
- `orchestration/ORCHESTRATOR_AGENT.md` — role doc for Layer 1
- `orchestration/FOREMAN_PLAYBOOK.md` — foreman dispatch procedure
- `orchestration/GAME_DIRECTOR_AGENT.md` — role doc for Layer 0
- `docs/SUB_AGENT_PLAYBOOK.md` — full Phase 0-6 procedure with reality gates, graduation, Decision Fields

**Sub-agent playbook adds to the kernel:**
- Decision Fields (6 elements: preferred/why/tempting alt/consequence/drift cue/recovery) on every non-obvious instruction
- Reality gates: EntryPoint, First-60-Seconds, Asset Reachability, Content Reachability, Event Connectivity, Save/Load Round-Trip
- Per-wave player traces tagged [Observed]/[Inferred]/[Assumed]
- Value audit rule: any value that zeros a player loop needs a graduation test
- Anti-thrash rule: errors are diagnostic data, not moral events
