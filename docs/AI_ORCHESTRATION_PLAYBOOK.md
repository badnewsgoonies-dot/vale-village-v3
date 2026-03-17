# AI-Orchestrated Software Development Playbook (v2)

Use this as a procedural manual. Follow it in order. Do not skip steps.

**Mission:** Ship working software with zero handwritten code by enforcing (1) a frozen type contract before parallel work begins, (2) mechanical scope clamping on all workers, (3) wave-based dispatch with gates between waves, (4) typed evidence on all persistent claims, and (5) the wave cadence: **Feature → Gate → Document → Harden → Graduate.**

**When to use this:** Any project where AI agents write code — web apps, CLIs, games, APIs, libraries. Load into project knowledge or CLAUDE.md at project start.

**Companion documents:**

- `SPEC_TEMPLATE.md` — how to write the project spec (what to build)
- `WORKER_TEMPLATE.md` — worker dispatch templates
- `STATE_TEMPLATE.md` — state management templates
- This playbook defines how to build it and how to govern the agents that do.

-----

## 0-WARNING: The two failure classes that pass all gates

|Class       |Example                                       |Caught by             |Enforcement          |
|------------|----------------------------------------------|----------------------|---------------------|
|Structural  |Type mismatch, scope violation, missing import|Compiler, tests, lints|Mechanical (20/20)   |
|Experiential|Dead feature, unreachable path, wrong behavior|Nothing automated     |Judgment + graduation|

Your mechanical gates will pass. They always pass by Wave 3. **That feeling of "on track" is a false signal.** The countermeasure is the Harden phase (§6.4) and the stop conditions (§10).

Evidence: 15K-LOC module, 120 tests, all gates green for 9 waves, 6 user-journey breaks undetected until final audit.

-----

## 0-TRUST ORDER (read this before anything else)

When sources conflict, use this precedence. This is the single most important rule in the playbook.

1. **Fresh code, tests, runtime output** — what the files actually say right now
1. **[Observed] artifacts** with concrete source_refs
1. **Current STATE.md** snapshot
1. **Project docs, contracts, specs** on disk
1. **Research findings** with certainty labels
1. **Conversation history** — the LOWEST trust tier

**Conversation is scaffolding, not substrate.** An agent that trusts its own prior conversation over the current code will propagate stale claims as truth.

Evidence: Blackout Test — 7 task packets, stateless workers, quarantined conversations. Blind integrator gets only repo + diffs. Build lands. 5 network faults all recovered. Contamination radius: 0.

-----

# PHASE 0 — SETUP

## 0.1 Your role

You are the orchestrator, not the implementer. Agents write all code. Your job is to define what gets built, draw boundaries, validate results, and make decisions when agents surface ambiguity.

Treat every instruction below as a constraint, not a suggestion. If a constraint is not mechanically enforced, assume it will be violated.

## 0.2 Install the dispatch stack

Two independent dispatch paths minimum. Any single tool's auth or quota fails mid-session.

```
Claude Code CLI:
  Install: npm install -g @anthropic-ai/claude-code
  Auth:    ANTHROPIC_API_KEY env var, or claude auth login
  Run:     claude -p "prompt" --allowedTools "Read,Grep,Glob,Edit,Write,Bash"
  Alt:     claude -p "prompt" --dangerously-skip-permissions
  Note:    Supports mid-run injection via interactive mode

Codex CLI:
  Install: npm install -g @openai/codex
  Auth:    ~/.codex/auth.json (ChatGPT session token — NOT API key)
  Run:     timeout 180 codex exec --dangerously-bypass-approvals-and-sandbox --skip-git-repo-check -C /path "prompt"
  Note:    --skip-git-repo-check needed in some container environments

Copilot CLI:
  Install: npm install -g @github/copilot
  Auth:    export COPILOT_GITHUB_TOKEN="github_pat_..." (fine-grained PAT ONLY)
  Run:     timeout 120 copilot -p "prompt" --model claude-sonnet-4.6 --allow-all-tools
  Note:    Classic PATs (ghp_) rejected. GITHUB_TOKEN env var rejected.
           Only COPILOT_GITHUB_TOKEN with fine-grained PAT works.
```

**Rules:**

- Without `--allow-all-tools` (Copilot) or equivalent, some models spend the entire timeout reading files instead of executing.
- Without `timeout`, workers hang indefinitely.
- Processes that "crash" often complete in background. Always check output files before assuming failure.

## 0.3 Quota fallback chain

When premium models hit rate limits:

1. Switch CLI tool (different quota pool — Codex vs Copilot vs Claude Code)
1. Drop to capable non-premium model (gpt-5-mini via Copilot)
1. Last resort: gpt-4.1 — **fabricates verification** (claims files exist that don't). Mechanical verification mandatory.

## 0.4 Model selection

|Role               |Best choice                           |Critical notes                                                     |
|-------------------|--------------------------------------|-------------------------------------------------------------------|
|Orchestrator       |Opus-class (1M context)               |Sustains 20+ wave campaigns                                        |
|Coding worker      |Sonnet-class or GPT-5.4               |Resourceful, finds tool paths                                      |
|Cheap/scoped tasks |Mini-class models                     |Evidence tags still work on simple claims                          |
|**Avoid for trust**|**Models that fabricate verification**|**Currently: gpt-4.1 — asserts source_refs exist without checking**|

**Rule:** The model is the first-order throughput variable. Same architecture, 9.8× output gap between best and worst worker models (measured). Always pick the strongest available for implementation work.

## 0.5 Create the workflow filesystem

```
project/
├── docs/
│   ├── spec.md                    # Full spec — see SPEC_TEMPLATE.md
│   └── domains/                   # Per-domain specs (when >5 domains)
├── status/
│   ├── workers/                   # Worker completion reports
│   └── dispatch-state.yaml        # Process table for active lanes
├── scripts/
│   ├── clamp-scope.sh             # Mechanical scope enforcement
│   └── run-gates.sh               # Validation pipeline
├── src/
│   └── shared/                    # THE CONTRACT — frozen, checksummed
├── MANIFEST.md                    # Phase, domain list, key decisions
├── STATE.md                       # Current truth — debts, gates, uncertainties
└── .contract.sha256               # Contract checksum
```

-----

# PHASE 1 — WRITE THE SPEC

> This section was missing from v1. It's the most common failure point — agents can't build what isn't specified.

## 1.1 The spec is the product

The spec is not documentation. It is the input that determines whether agents produce correct output. A vague spec produces vague code. A spec with wrong numbers produces code with wrong numbers.

**Rule:** If a value matters, it must appear as a literal number in the spec. Agents do not infer values from context — they default to whatever their training suggests.

Evidence: `crit_multiplier` unspecified → 8/10 workers defaulted to 1.5x. Specified as 2.0 → 10/10 correct.

## 1.2 What every spec must contain

|Required                      |Example                                       |What happens without it  |
|------------------------------|----------------------------------------------|-------------------------|
|**Quantities**                |"80 weapons" not "lots of weapons"            |Worker produces 8        |
|**Constants with values**     |`crit_multiplier = 2.0`                       |Worker invents a value   |
|**Formulas with all terms**   |`dmg = basePower + ATK - (DEF × 0.5), floor 1`|Worker guesses formula   |
|**Enumerative tables**        |Stat curves, item lists, status effects       |Worker invents entries   |
|**"Does NOT handle" sections**|"This domain does NOT handle persistence"     |Worker builds persistence|
|**Definition of done**        |"cargo test passes, no skipped tests"         |Worker self-declares done|

## 1.3 Spec structure (see SPEC_TEMPLATE.md for full version)

```markdown
# [Project] — Specification

## 1. One-Sentence Pitch
## 2. Core Pillars (3 max)
## 3. System Rules (with formulas, constants, enumerations)
## 4. Domain Boundaries (what each module owns and does NOT own)
## 5. Content Quantities (exact counts)
## 6. Data Schema (types, enums, shapes)
## 7. What This Spec Does NOT Cover
```

**Rule:** Workers read the spec from disk. Never rely on summarized prompts passed through intermediaries. Hierarchies compress information. Numbers die first.

Evidence: 327-line objective through 3 delegation levels → 8 weapons against target of 80+.

-----

# PHASE 2 — FREEZE THE TYPE CONTRACT

## 2.1 Why this exists

No workers launch until the contract exists and is frozen. Without it, N workers invent N incompatible type systems.

Evidence: 10 workers without contract → 6 incompatible interfaces. 50+ domain builds with contract → zero integration type errors.

## 2.2 What goes in the contract

The contract is the **shared type file** that every domain imports from:

- Every cross-domain entity type (User, Order, Event, etc.)
- All shared enums (Status, Role, Category, etc.)
- Shared event/message types
- Cross-module function signatures
- Strict primitive decisions — IDs are `string` or `number`, decide once

## 2.3 Freeze rules

1. **Freeze shapes, not values.** Types, enums, interfaces, event shapes are frozen. Thresholds, timings, balance numbers, and copy live in config/data files.
1. **Checksum and commit:**

```bash
shasum -a 256 src/shared/mod.rs > .contract.sha256
git commit -m "chore: freeze shared type contract"
```

1. **No worker edits the contract during parallel build.** Enforced mechanically via pre-commit hook or `disallowedTools`.

## 2.4 Contract amendment process

When the contract must change (it will):

1. Only during an integration phase (§8) or by orchestrator decision
1. Propose the change in a dedicated commit with rationale
1. Update the checksum: `shasum -a 256 src/shared/mod.rs > .contract.sha256`
1. Re-run ALL domain gates after amendment
1. Update MANIFEST.md with the amendment and reason

**Rule:** If a worker needs a contract change to complete its task, it reports the need in its worker report and stops. It does not modify the contract.

-----

# PHASE 3 — DRAW DOMAIN BOUNDARIES

## 3.1 Define domains and path prefixes

For each domain, define the only allowed path prefix:

```
src/domains/auth/
src/domains/api/
src/domains/billing/
src/domains/ui/
```

## 3.2 Boundary survivability test

A domain boundary is valid ONLY if:

- It can compile + pass local tests while all other domains remain unchanged
- Its fixes do not require edits outside its path prefix after clamping

**If clamping breaks the fix:** your boundary is wrong, or the task is integration work. Merge the domains or route to an integration worker (§8).

Two tightly coupled modules are one module. Draw the boundary where architectural independence holds.

## 3.3 Spec on disk for each domain

When domain count exceeds 5, write a per-domain spec in `docs/domains/[name].md` containing:

- What this domain owns
- What this domain does NOT own
- Required imports from the contract
- Deliverables and validation criteria

-----

# PHASE 4 — DISPATCH WORKERS

## 4.1 Briefing quality determines output quality

|Briefing style                                     |Ship rate|Notes         |
|---------------------------------------------------|---------|--------------|
|Exact values ("set alpha to 0.6 in file X line 40")|**100%** |Always prefer |
|Named actions ("add particle effect to tool swing")|**67%**  |Acceptable    |
|Vague goals ("make it feel better")                |**0%**   |Never dispatch|

**Rule:** If you cannot name the file and the value, the prompt is not ready. Do not dispatch.

## 4.2 What NOT to do matters more than what to do

|Format                                  |Output        |Ship?       |
|----------------------------------------|--------------|------------|
|Freeform ("make X better")              |446 lines     |Inconsistent|
|Formal spec                             |9 lines       |Barely      |
|**Decision Fields (do/don't/drift-cue)**|**1514 lines**|**✓**       |

**Rule:** Telling the agent what NOT to do and what drift looks like produces more output than telling it what TO do.

## 4.3 Worker spec template (see WORKER_TEMPLATE.md for variants)

Every field is required:

```markdown
TASK: [Exact description — specific values, file targets]

CONTEXT:
- [File A:line] has [system X] which currently does [behavior]

WHAT TO DO:
1. Read [file] to find [function/struct]
2. [Exact change with before/after values]

SCOPE: ONLY modify files under [path prefix].
DO NOT: [constraints — modify shared types, create frameworks, etc.]
DRIFT CUE: If you find yourself [building X], stop — that's scope creep.
COMMIT: git add -A && git commit -m '[type]: [description]'
DONE: [validation command passes, file exists, etc.]
```

## 4.4 Campaign dispatch template

For sustained multi-wave autonomous work:

```markdown
You have [tool] available for worker dispatch. You are the orchestrator.
Read [manifest file] first.

DO: Work in waves. Commit after each.
DO NOT: One giant edit. Stop between waves. Rewrite systems. Build frameworks.
DO NOT: Stop after one wave and ask if you should continue.
   Continue until exhausted or genuinely blocked.

Wave 1: [exact scope, exact files, commit message]
Wave 2: ...

Start now. Read the manifest, begin Wave 1.
```

**Rule:** "DO NOT stop between waves" sustains multi-wave campaigns. Without it, every agent completes Wave 1 and waits.

Evidence: 18-commit campaign completed autonomously with this line. Without it, agents stopped after every wave.

## 4.5 Dispatch discipline

- **Commit after every wave** BEFORE dispatching the next worker. Prevents scope wipe.
- **Clamp scope after every worker.** Not optional.
- **Stagger parallel launches ~3 seconds.** 2-3 simultaneous: stable. 5+: crashes.
- **Prefer editing existing files over creating new ones.** New file success ~50% first-try vs ~90% for edits. When new files are needed, specify module registration explicitly.
- **Wave-based dispatch, not one-shot.** One-shot for large builds is an explicit anti-pattern.

-----

# PHASE 5 — SCOPE ENFORCEMENT

## 5.1 The only method that works

Prompt-only scope control under compiler pressure: **0/20.**
Mechanical clamping after worker completes: **20/20.**

Evidence: 20 workers under compile errors. Every one edited files outside scope. Every prompt-based restriction was ignored. Mechanical revert: 20/20 clean.

**Rule:** Don't ask AI to follow scope rules. Let it edit anything. Then revert everything outside its allowed prefix.

## 5.2 Clamp script

```bash
#!/usr/bin/env bash
set -euo pipefail
ALLOW_PREFIX="${1:?Usage: clamp-scope.sh src/domains/combat/}"

# Revert tracked unstaged changes outside scope
git diff --name-only -z | while IFS= read -r -d '' f; do
  [[ "$f" == "${ALLOW_PREFIX}"* ]] && continue
  git restore --worktree -- "$f"
done

# Revert tracked staged changes outside scope
git diff --name-only -z --cached | while IFS= read -r -d '' f; do
  [[ "$f" == "${ALLOW_PREFIX}"* ]] && continue
  git restore --staged --worktree -- "$f"
done

# Remove untracked files outside scope
git ls-files --others --exclude-standard -z | while IFS= read -r -d '' f; do
  [[ "$f" == "${ALLOW_PREFIX}"* ]] && continue
  rm -rf -- "$f"
done

echo "Clamped to ${ALLOW_PREFIX}"
```

## 5.3 Additional mechanical enforcement

|Mechanism                                  |Reliability         |Use when                                            |
|-------------------------------------------|--------------------|----------------------------------------------------|
|`clamp-scope.sh` after every worker        |20/20               |Always                                              |
|`.claude/settings.json` + `disallowedTools`|Reliable            |Preventing contract edits                           |
|Pre-commit hook (`hook-contract-integrity`)|Reliable            |Blocking contract changes without override          |
|`hook-agent-guard`                         |Reliable            |Preventing orchestrator from editing source directly|
|CLAUDE.md instructions                     |**Suggestions only**|Low-stakes guidance (model can deprioritize)        |

-----

# PHASE 6 — WAVE CADENCE

Every wave follows this exact sequence. No skips.

```
Feature → Gate → Document → Harden → Graduate
```

## 6.1 Feature

Build or change the targeted surface. Workers handle bounded structural work. Workers should not create orchestration infrastructure instead of features.

## 6.2 Gate

Run mechanical checks: compile, test, lint, contract checksum, scope clamp.

```bash
#!/usr/bin/env bash
set -euo pipefail

echo "== Contract integrity =="
shasum -a 256 -c .contract.sha256

echo "== Compile =="
cargo check  # or: npm run build / tsc --noEmit / go build ./...

echo "== Tests =="
cargo test   # or: npm test / go test ./...

echo "== Connectivity (no hermetic domains) =="
FAIL=0
for d in src/domains/*/; do
  if ! grep -rq "shared" "$d" --include="*.rs"; then
    echo "FAIL: $d has no shared contract import"
    FAIL=1
  fi
done
[ "$FAIL" -eq 0 ] || { echo "Hermetic domains detected"; exit 1; }

echo "== All gates passed =="
```

**Rule:** Green means ready to examine, NOT ready to ship.

## 6.3 Document

Emit artifacts ONLY when one of these triggers fires:

1. Non-obvious decision made
1. Direct verification happened
1. Reusable principle emerged
1. Open debt appeared
1. Contradiction surfaced
1. Correction invalidated prior belief
1. Graduation test created

If nothing triggers, write nothing.

## 6.4 Harden

This is where experiential failures get caught. Inspect the actual result:

- **Reachable** end-to-end? Can a user/caller actually reach this code path?
- **Feedback visible?** Does the system show the user what happened?
- **Edge behavior sane?** Empty inputs, max values, concurrent access?
- **Diagnosable** when it fails? Error messages, logs, state?

If reachable but wrong, it is not finished.

**For "beautiful dead product" (stop condition #8):** Run this checklist for every surface. Write a P0 debt for each unreachable surface. Do not dispatch more workers until every P0 has a concrete path to reachability.

## 6.5 Graduate

For each `[Observed]` truth:

1. Name the invariant (e.g., "auto-attack always generates exactly 1 mana per hit")
1. Encode it as a test
1. Add the test to the gate suite
1. Track remaining ungraduated work as P0 / P1 / P2

**Rule:** Do not start the next wave until Document, Harden, and Graduate are complete.

-----

# PHASE 7 — EVIDENCE AND MEMORY

## 7.1 Evidence levels

Every claim an agent persists must carry one of:

|Level         |Meaning                                          |Can be frozen into gates?|
|--------------|-------------------------------------------------|-------------------------|
|**[Observed]**|Directly verified against code, tests, or runtime|Yes                      |
|**[Inferred]**|Logically derived but not directly verified      |No                       |
|**[Assumed]** |Stated without verification                      |No — must verify first   |

Evidence: Without evidence levels, untyped memory performs WORSE than no memory — 0% correct vs 83% with no memory at all. With evidence levels: 54% calibrated abstention on ambiguous decisions.

## 7.2 The poisoning defense

~200 characters of evidence metadata change agent behavior categorically:

|Condition                  |Result                       |Trials               |
|---------------------------|-----------------------------|---------------------|
|No metadata on claims      |100% adoption of false claims|50/50 across 5 models|
|Evidence tags + source_refs|96% rejection of false claims|24/25 across 5 models|

This is model-independent (works on Opus, Sonnet, GPT-5.4, Gemini 3 Pro, Haiku). It is NOT capability-dependent (Opus adopts false claims at 100% without metadata). The defense comes from the labels, not from model intelligence.

## 7.3 Artifact schema

```yaml
id: DEC-2026-03-10-001
type: decision | observation | debt | principle
evidence: Observed | Inferred | Assumed
domain: [domain name]
summary: "One sentence."
source_refs:
  - "file:repo@src/path/file.rs:10-40"
  - "commit:repo@abc1234"
  - "test:repo@cargo test -- test_name"
status: active | resolved | superseded
supersedes: []
```

**Rules:**

- One artifact per file
- Supersede old artifacts, don't silently mutate them
- `[Observed]` claims must have non-empty source_refs
- The full schema is optional. Evidence tags alone provide the poisoning defense. The schema adds reasoning quality and retrieval routing.

## 7.4 Evidence interaction with model capability

|                                  |Simple claims         |Complex ambiguity        |
|----------------------------------|----------------------|-------------------------|
|Frontier models (Sonnet, GPT-5.4) |Works                 |**13% → 54% calibration**|
|Cheap models (mini-class)         |**33% → 100% defense**|No effect                |
|Fabrication-prone models (gpt-4.1)|No effect             |No effect                |

## 7.5 What the labels actually do

The labels don't make AI "more careful." They change the information format so the AI activates a different reasoning pathway. The AI reads `[Assumed, no source refs]` and treats it like a human treats an unsourced rumor — with skepticism. Without the label, it treats everything as equally trustworthy and the most recent thing wins.

-----

# PHASE 8 — INTEGRATION

> This section was missing from v1. Integration is where the most expensive bugs live.

## 8.1 When to integrate

After all domain workers have completed their wave and gates are green. Integration is a separate phase, not something that happens inside domain work.

## 8.2 Start a fresh session

**Do not carry the full orchestration conversation forward.** ~95% of orchestrator cost at this point is re-reading conversation history.

The integration session ingests ONLY:

- `src/shared/` (the contract) + `.contract.sha256`
- `docs/spec.md` + `docs/domains/*.md`
- `status/workers/*.md` (worker reports)
- Current compiler/test errors (if any)

## 8.3 Integration worker scope

- **Allowed:** composition root, wiring files, domain index files, `src/` top-level
- **Forbidden:** rewriting domain internals unless compilation requires it
- **Responsibilities:** wire domains together, resolve remaining type mismatches via contract amendment (§2.4), ensure data flows are connected, run global gates

## 8.4 Integration gate

The global gate script (§6.2) plus:

```bash
echo "== Connectivity check =="
# Every domain must import from the shared contract
for d in src/domains/*/; do
  if ! grep -rq "shared" "$d" --include="*.rs" --include="*.ts"; then
    echo "FAIL: $d is hermetic (no shared contract import)"
    exit 1
  fi
done
```

For stronger verification: use AST parsing or require each domain to export an entry point that imports at least one shared type in a value position (not type-only, which gets tree-shaken).

## 8.5 Integration report

Write `status/integration.md` containing:

- What was wired
- What contract amendments were needed (if any)
- What remains unwired
- What was discovered during integration (new debt)

-----

# PHASE 9 — SESSION PROTOCOL

## 9.1 Start (every session)

1. Read this playbook
1. Read STATE.md
1. Verify contract: `shasum -a 256 -c .contract.sha256`
1. Mount the current objective
1. Pre-touch retrieval: `git log --oneline -15 -- <path>`, read active debt
1. State BEFORE acting: tier (S/M/C), surface being touched, current phase, any [Assumed] claims on the critical path

## 9.2 Tiering

|Tier |Scope                                             |Workers useful?|
|-----|--------------------------------------------------|---------------|
|**S**|Single-surface fix or bounded hotfix              |Rarely         |
|**M**|Module or subsystem, 1-3 domains                  |Yes            |
|**C**|Campaign, multiple domains, orchestration required|Required       |

Start at S if ambiguous. Escalate when touching shared contracts, persistence, trust boundaries, or multiple interacting surfaces.

## 9.3 End (every session)

1. Update STATE.md — phase, debts, decisions, gate status, uncertainties
1. Write triggered artifacts only
1. Commit state changes
1. Do NOT rely on conversation to preserve what was learned

## 9.4 Recovery prompt (when resuming after crash/reset)

```
Continuing [project]. Read before acting:
- STATE.md, MANIFEST.md, docs/spec.md, src/shared/[contract file]

Recent: [1 sentence]. Task: [what to do now].
State tier (S/M/C) and any [Assumed] claims on critical path before acting.
```

-----

# PHASE 10 — STATE MANAGEMENT

## 10.1 dispatch-state.yaml

```yaml
lanes:
  - id: auth-fix
    status: in-progress  # pending | in-progress | gated | merged | failed
    goal: "Fix token refresh race condition"
    owned_paths:
      - src/domains/auth/
    next_action: "Run cargo test after worker completes"
```

Every active work lane tracked on disk. Survives session death.

## 10.2 Three core transactions

|Transaction   |What it does                                      |When                |
|--------------|--------------------------------------------------|--------------------|
|**Checkpoint**|Preserves filesystem + ledger + conversation state|Before risky waves  |
|**Restore**   |Recovers from a named checkpoint                  |When wave goes wrong|
|**Launch**    |Creates isolated work lane (new worktree + branch)|Parallel domain work|

## 10.3 Mechanical state verification

- `verify-state-claims` — script that checks STATE.md claims against actual repo state
- `hook-contract-integrity` — pre-commit hook rejecting contract edits without override flag
- `hook-agent-guard` — prevents orchestrator from directly editing source files

-----

# STOP CONDITIONS

Cease work and reassess when any of these appear:

|# |Condition                                                            |What to do                                                                                                   |
|--|---------------------------------------------------------------------|-------------------------------------------------------------------------------------------------------------|
|1 |**Contract drift** — checksum fails                                  |Restore contract. Re-run all domain gates.                                                                   |
|2 |**Clamp breaks the fix** — out-of-scope edits were needed            |Boundary is wrong. Re-scope as integration or merge domains.                                                 |
|3 |**False green** — tests pass but contract unused/bypassed            |Wire imports. Add connectivity gate.                                                                         |
|4 |**Abstraction reflex** — redesigning architecture to avoid debugging |Stop. Debug the actual issue.                                                                                |
|5 |**Delegation compression** — asked for 80 items, got 8               |Worker read summary, not spec. Ensure disk spec with quantities.                                             |
|6 |**Self-model error** — agent claims it can't do things it can        |Add to prompt: "You have bash access. You can read and write files."                                         |
|7 |**Identity paradox** — one agent playing architect + worker          |Use separate sessions per role.                                                                              |
|8 |**Beautiful dead product** — gates green, surface unreachable        |Run harden checklist. Write P0 debt for each dead surface. No new workers until P0s have a reachability path.|
|9 |**Ghost progress** — nothing newly reachable after the wave          |The wave produced scaffolding, not product. Re-scope to a reachable deliverable.                             |
|10|**Cadence break** — documenting while coding, coding while diagnosing|Stop. Finish the current phase before starting the next.                                                     |

**Rule:** When a stop condition fires: stop. Don't push through. Diagnose. Restart the wave.

-----

# DISCOVERY TAXONOMY

When a worker finds something unexpected:

|Discovery                      |Action                                      |
|-------------------------------|--------------------------------------------|
|Reproducible bug (in scope)    |Fix it                                      |
|Fragile seam (missing coverage)|Write regression test FIRST, then fix       |
|Fidelity gap (cross-domain)    |Escalate to orchestrator — don't widen scope|
|Out of scope                   |Record as debt artifact. Don't touch.       |

-----

# AGENT FAILURE MODES

|Failure                              |Fix                                                                            |
|-------------------------------------|-------------------------------------------------------------------------------|
|Defaults to solo execution           |"You have [tool] available. Dispatch workers."                                 |
|Stops after one wave                 |"DO NOT stop between waves." Still stops: "continue."                          |
|Builds frameworks instead of features|"Implement deliverables only. Do not create orchestration infrastructure."     |
|Reads summaries, ignores files       |Specs on disk with quantities. "Read [path] before acting."                    |
|Edits frozen contract                |Mechanical clamp. `disallowedTools`. Pre-commit hook.                          |
|Claims it can't do things it can     |"You have bash access. You can read and write files."                          |
|New file creation fails (~50%)       |Prefer edits. Specify module registration in prompt.                           |
|Session dies mid-campaign            |Manifest on disk is the state. "Resume from Wave N."                           |
|Mid-run injection needed             |Choose a tool with injection points (Claude Code, not Copilot continuous mode).|

-----

# COMPLETION CRITERIA

You are done ONLY when:

- [ ] Contract checksum passes
- [ ] Global compile/typecheck passes
- [ ] Global test suite passes
- [ ] Connectivity gate passes (no hermetic domains)
- [ ] Each worker report exists in `status/workers/`
- [ ] Integration report exists in `status/integration.md`
- [ ] STATE.md reflects current truth with evidence levels
- [ ] No `[Assumed]` claims on the critical path
- [ ] Every P0 surface is reachable and operable (harden verified)
- [ ] MANIFEST.md updated with final status

-----

# MEASUREMENTS

|Metric                                  |Value               |Source                 |
|----------------------------------------|--------------------|-----------------------|
|Manual dispatch ship rate               |67% (10/15)         |Corpus observation     |
|Foreman dispatch ship rate              |100% (5/5)          |Corpus observation     |
|Scope: prompt-only                      |0/20                |Controlled experiment  |
|Scope: mechanical clamp                 |20/20               |Controlled experiment  |
|Evidence tags on ambiguous decisions    |13% → 54%           |n=5 × 5 models         |
|Poisoning defense (cross-model)         |0% → 96%            |50/50 → 24/25, 5 models|
|Poisoning defense (multi-repo)          |33% → 100%          |Rust async API codebase|
|Blackout fault recovery                 |5/5, contamination 0|Controlled experiment  |
|Briefing: Decision Fields vs formal spec|1514 lines vs 9     |Same task              |
|Exact-value ship rate                   |100%                |Corpus observation     |
|Vague-goal ship rate                    |0%                  |Corpus observation     |
|New file success rate                   |~50% (edits ~90%)   |Corpus observation     |
|Parallel workers stable                 |2-3 (5+ crashes)    |Operational observation|
|Model output gap (same task)            |9.8×                |Controlled comparison  |

-----

*Derived from 739 commits, 64K LOC, 295M tokens, 98 agent sessions, 172 controlled trials across 5 frontier models. Zero handwritten lines of code.*
