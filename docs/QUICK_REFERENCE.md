# Quick Reference Card

Print this. Tape it to your monitor. Check it before every dispatch.

---

## The Loop

```
FREEZE CONTRACT → DISPATCH WAVE → CLAMP SCOPE → GATE → HARDEN → GRADUATE → NEXT WAVE
```

## Before Every Dispatch

- [ ] Can I name the exact file and value? (If no → not ready)
- [ ] Is the spec on disk with quantities? (If no → write it first)
- [ ] Is the contract frozen and checksummed? (If no → freeze it)
- [ ] Is scope defined as a path prefix? (If no → define it)

## After Every Worker

```bash
bash scripts/clamp-scope.sh src/domains/[name]/    # 1. Clamp
shasum -a 256 -c .contract.sha256                   # 2. Contract
cargo test                                          # 3. Gate
git add -A && git commit -m '[type]: [desc]'        # 4. Commit
```

## Trust Order (top = highest)

1. Fresh code + tests + runtime
2. [Observed] artifacts with source_refs
3. STATE.md
4. Specs on disk
5. Research findings
6. Conversation ← **LOWEST**

## Evidence Tags

- `[Observed]` — verified against code/tests/runtime → CAN freeze into gates
- `[Inferred]` — logically derived → CANNOT freeze
- `[Assumed]` — unverified → MUST verify before critical decisions

## Wave Cadence (no skips)

**Feature** → **Gate** → **Document** → **Harden** → **Graduate**

## Stop Immediately When

- Contract checksum fails
- Scope clamp breaks the fix (boundary is wrong)
- Tests pass but feature is unreachable (false green)
- Agent builds frameworks instead of features
- Asked for 80 items, got 8 (delegation compression)

## Key Numbers

| What | Value |
|------|-------|
| Prompt-only scope enforcement | 0/20 |
| Mechanical scope enforcement | 20/20 |
| Poisoning defense with evidence tags | 96% (24/25) |
| Poisoning without evidence tags | 0% (50/50 adopted) |
| Exact-value prompts ship rate | 100% |
| Vague-goal prompts ship rate | 0% |
| New file creation success | ~50% |
| Editing existing files success | ~90% |
| Parallel workers stable | 2-3 max |

## Dispatch Commands

```bash
# Claude Code
claude -p "prompt" --allowedTools "Read,Grep,Glob,Edit,Write,Bash"

# Codex
timeout 180 codex exec --dangerously-bypass-approvals-and-sandbox --skip-git-repo-check "prompt"

# Copilot
COPILOT_GITHUB_TOKEN="github_pat_..." timeout 120 copilot -p "prompt" --model claude-sonnet-4.6 --allow-all-tools
```

## Recovery After Session Death

```
Read STATE.md → Read MANIFEST.md → Read spec.md → Read contract →
State tier (S/M/C) → State [Assumed] claims → Resume from last wave
```
