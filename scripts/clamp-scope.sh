#!/usr/bin/env bash
# clamp-scope.sh — Revert all changes outside allowed path prefixes
#
# Usage:
#   bash scripts/clamp-scope.sh src/domains/combat/
#   bash scripts/clamp-scope.sh src/domains/combat/ src/domains/status/
#
# status/workers/ is ALWAYS preserved (worker completion reports).
# Reverted paths are printed to stderr for drift analysis.
set -euo pipefail

if [ $# -eq 0 ]; then
  echo "Usage: clamp-scope.sh <allowed-prefix> [additional-prefix...]" >&2
  exit 1
fi

ALLOWED=("$@" "status/workers/")
REVERT_LOG=$(mktemp)
trap 'rm -f "$REVERT_LOG"' EXIT

is_allowed() {
  local f="$1"
  for prefix in "${ALLOWED[@]}"; do
    [[ "$f" == "${prefix}"* ]] && return 0
  done
  return 1
}

# Revert tracked unstaged changes outside scope
git diff --name-only -z | while IFS= read -r -d '' f; do
  is_allowed "$f" && continue
  echo "$f" >> "$REVERT_LOG"
  git restore --worktree -- "$f"
done

# Revert tracked staged changes outside scope
git diff --name-only -z --cached | while IFS= read -r -d '' f; do
  is_allowed "$f" && continue
  echo "$f" >> "$REVERT_LOG"
  git restore --staged --worktree -- "$f"
done

# Remove untracked files outside scope
git ls-files --others --exclude-standard -z | while IFS= read -r -d '' f; do
  is_allowed "$f" && continue
  echo "$f" >> "$REVERT_LOG"
  rm -rf -- "$f"
done

# Report
COUNT=$(wc -l < "$REVERT_LOG" 2>/dev/null || echo 0)
COUNT=$(echo "$COUNT" | tr -d ' ')
if [ "$COUNT" -gt 0 ]; then
  echo "⚠ Reverted $COUNT out-of-scope path(s):" >&2
  while IFS= read -r r; do
    echo "  - $r" >&2
  done < "$REVERT_LOG"
else
  echo "✓ No out-of-scope changes found"
fi
echo "✓ Clamped to: ${ALLOWED[*]}"
