#!/usr/bin/env bash
# run-gates.sh — Validation pipeline. Run after every wave.
# Customize the BUILD_CMD, TEST_CMD, and LINT_CMD for your project.
set -euo pipefail

# ── Configure these for your project ──────────────────────────
CONTRACT_FILE="${CONTRACT_FILE:-src/shared/mod.rs}"
CHECKSUM_FILE="${CHECKSUM_FILE:-.contract.sha256}"
DOMAIN_DIR="${DOMAIN_DIR:-src/domains}"
CONTRACT_IMPORT="${CONTRACT_IMPORT:-shared}"
FILE_EXT="${FILE_EXT:-rs}"

# Build/test commands — override via environment
BUILD_CMD="${BUILD_CMD:-cargo check}"
TEST_CMD="${TEST_CMD:-cargo test}"
LINT_CMD="${LINT_CMD:-}"  # Optional: cargo clippy, eslint, etc.
# ──────────────────────────────────────────────────────────────

FAIL=0
STATE_STALE=0
ARTIFACT_WARN=0
CLAIM_WARN=0

resolve_state_file() {
  if [ -f "STATE.md" ]; then
    printf '%s\n' "STATE.md"
  elif [ -f ".memory/STATE.md" ]; then
    printf '%s\n' ".memory/STATE.md"
  else
    return 1
  fi
}

extract_state_commit() {
  local state_file="$1"
  grep -oP '\*\*(?:Verified Commit|HEAD):\*\*\s*\K[a-f0-9]+' "$state_file" 2>/dev/null || true
}

echo "== Contract integrity =="
if [ -f "$CHECKSUM_FILE" ]; then
  if shasum -a 256 -c "$CHECKSUM_FILE"; then
    echo "✓ Contract checksum OK"
  else
    echo "✗ CONTRACT DRIFT — checksum failed"
    FAIL=1
  fi
else
  echo "⚠ No checksum file found at $CHECKSUM_FILE"
fi

echo ""
echo "== Build =="
if $BUILD_CMD; then
  echo "✓ Build passed"
else
  echo "✗ Build FAILED"
  FAIL=1
fi

echo ""
echo "== Tests =="
if $TEST_CMD; then
  echo "✓ Tests passed"
else
  echo "✗ Tests FAILED"
  FAIL=1
fi

if [ -n "$LINT_CMD" ]; then
  echo ""
  echo "== Lint =="
  if $LINT_CMD; then
    echo "✓ Lint passed"
  else
    echo "⚠ Lint warnings (non-blocking)"
  fi
fi

echo ""
echo "== Connectivity (no hermetic domains) =="
if [ -d "$DOMAIN_DIR" ]; then
  CONNECTIVITY_FAIL=0
  for d in "$DOMAIN_DIR"/*/; do
    if [ -d "$d" ]; then
      if ! grep -rq "$CONTRACT_IMPORT" "$d" --include="*.$FILE_EXT" 2>/dev/null; then
        echo "✗ HERMETIC: $d has no $CONTRACT_IMPORT import"
        CONNECTIVITY_FAIL=1
      fi
    fi
  done
  if [ "$CONNECTIVITY_FAIL" -eq 0 ]; then
    echo "✓ All domains import from $CONTRACT_IMPORT"
  else
    FAIL=1
  fi
else
  echo "⚠ No domain directory at $DOMAIN_DIR"
fi

echo ""
echo "== STATE.md baseline =="
STATE_FILE="$(resolve_state_file || true)"
if [ -n "$STATE_FILE" ]; then
  CURRENT_HEAD=$(git rev-parse --short HEAD)
  STATE_HEAD="$(extract_state_commit "$STATE_FILE")"

  if [ -z "$STATE_HEAD" ]; then
    echo "⚠ STATE.md has no Verified Commit reference"
    STATE_STALE=1
  elif git merge-base --is-ancestor "$STATE_HEAD" HEAD 2>/dev/null; then
    COMMITS_AHEAD=$(git rev-list --count "${STATE_HEAD}..HEAD" 2>/dev/null || echo "?")
    if [ "$COMMITS_AHEAD" = "0" ]; then
      echo "✓ STATE.md baseline matches current HEAD"
    else
      echo "✓ STATE.md baseline commit $STATE_HEAD is an ancestor of current HEAD ($CURRENT_HEAD, +$COMMITS_AHEAD commit(s))"
    fi
  else
    echo "⚠ STATE.md Verified Commit ($STATE_HEAD) is not an ancestor of current HEAD ($CURRENT_HEAD)"
    STATE_STALE=1
  fi
else
  echo "⚠ STATE.md not found"
  STATE_STALE=1
fi

echo ""
echo "== Artifact source refs =="
ARTIFACT_WARN_FILE=$(mktemp)
trap 'rm -f "$ARTIFACT_WARN_FILE"' EXIT
echo "0" > "$ARTIFACT_WARN_FILE"
for artifact in .memory/*.yaml; do
  [ -f "$artifact" ] || continue
  grep 'file:' "$artifact" 2>/dev/null | while IFS= read -r line; do
    filepath=$(echo "$line" | sed -n 's/.*@\([^:"]*\).*/\1/p')
    if [ -n "$filepath" ] && [ ! -f "$filepath" ]; then
      echo "⚠ $(basename "$artifact"): '$filepath' not found"
      echo "1" > "$ARTIFACT_WARN_FILE"
    fi
  done || true
done
ARTIFACT_WARN=$(cat "$ARTIFACT_WARN_FILE")
if [ "$ARTIFACT_WARN" -eq 0 ]; then
  echo "✓ All artifact file refs resolve"
fi

echo ""
echo "== STATE.md claim verification =="
if [ -f "scripts/verify-state-claims.sh" ]; then
  CLAIM_OUTPUT=$(bash scripts/verify-state-claims.sh 2>&1 || true)
  CLAIM_FAILURES=$(echo "$CLAIM_OUTPUT" | grep -c '✗' || true)
  if [ "$CLAIM_FAILURES" -gt 0 ]; then
    echo "$CLAIM_OUTPUT" | grep -E '(✗|✓|⚠|Verified:)' | head -20
    echo "⚠ $CLAIM_FAILURES claim(s) failed verification (warning only)"
    CLAIM_WARN=1
  else
    echo "✓ All STATE.md claims verified against code"
  fi
else
  echo "⚠ verify-state-claims.sh not found"
  CLAIM_WARN=1
fi

echo ""
if [ "$FAIL" -eq 0 ]; then
  echo "══════════════════════════════"
  echo "  ALL GATES PASSED"
  if [ "$STATE_STALE" -eq 1 ] || [ "$ARTIFACT_WARN" -eq 1 ] || [ "$CLAIM_WARN" -eq 1 ]; then
    echo "  (warnings present — review before shipping)"
  fi
  echo "══════════════════════════════"
else
  echo "══════════════════════════════"
  echo "  GATES FAILED — see above"
  echo "══════════════════════════════"
  exit 1
fi
