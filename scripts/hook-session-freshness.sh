#!/bin/bash
# SessionStart hook: STATE.md ancestry check
# Confirms the recorded verified commit is still on the current branch.
# The recorded commit does not need to equal HEAD; it only needs to remain an ancestor.
# Non-blocking (exit 0) — informational only.

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$REPO_ROOT"

resolve_state_file() {
  if [[ -f STATE.md ]]; then
    printf '%s\n' "STATE.md"
  elif [[ -f .memory/STATE.md ]]; then
    printf '%s\n' ".memory/STATE.md"
  else
    return 1
  fi
}

extract_state_commit() {
  local state_file="$1"
  grep -oP '\*\*(?:Verified Commit|HEAD):\*\*\s*\K[a-f0-9]+' "$state_file" 2>/dev/null || true
}

# Run once per session — use repo-specific lock
REPO_HASH=$(echo "$REPO_ROOT" | shasum -a 256 | cut -c1-8)
LOCK="/tmp/.project-freshness-${REPO_HASH}"
if [[ -f "$LOCK" ]]; then
  exit 0
fi
touch "$LOCK"

# Skip if no STATE.md
STATE_FILE="$(resolve_state_file || true)"
if [[ -z "$STATE_FILE" ]]; then
  exit 0
fi

state_head="$(extract_state_commit "$STATE_FILE")"
actual_head=$(git rev-parse --short HEAD 2>/dev/null || echo "")

if [[ -z "$state_head" || -z "$actual_head" ]]; then
  exit 0
fi

# A recorded verified commit remains valid as long as it is an ancestor of HEAD.
if git merge-base --is-ancestor "$state_head" HEAD 2>/dev/null; then
  exit 0
fi

echo "════════════════════════════════════════" >&2
echo "  STATE.md ANCESTRY CHECK" >&2
echo "════════════════════════════════════════" >&2
echo "  STATE VERIFIED COMMIT: ${state_head}" >&2
echo "  Actual HEAD: ${actual_head}" >&2
echo "  The recorded state commit is not an ancestor of the current checkout." >&2
echo "" >&2
echo "  Update STATE.md if you want the recorded baseline to follow this branch." >&2

# Run full claim verification if available (subsumes the old test-count check)
if [[ -f scripts/verify-state-claims.sh ]]; then
  CLAIM_OUTPUT=$(bash scripts/verify-state-claims.sh 2>&1 || true)
  CLAIM_FAILURES=$(echo "$CLAIM_OUTPUT" | grep -c '✗' || true)
  if [[ "$CLAIM_FAILURES" -gt 0 ]]; then
    echo "" >&2
    echo "  CLAIM VERIFICATION ($CLAIM_FAILURES failures):" >&2
    echo "$CLAIM_OUTPUT" | grep '✗' >&2
  fi
else
  # Fallback: check test count specifically when claim verification is unavailable.
  state_tests=$(grep -oP 'Gate 3.*?:\s*\K\d+(?=\s+headless)' "$STATE_FILE" 2>/dev/null || echo "")
  if [[ -n "$state_tests" && -f tests/headless.rs ]]; then
    actual_tests=$(grep -c '#\[test\]' tests/headless.rs 2>/dev/null || echo "?")
    if [[ "$state_tests" != "$actual_tests" ]]; then
      echo "  ⚠ Test count: STATE says ${state_tests}, actual is ${actual_tests}" >&2
    fi
  fi
fi

echo "════════════════════════════════════════" >&2
exit 0
