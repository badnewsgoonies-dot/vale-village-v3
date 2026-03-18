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
  for d in "$DOMAIN_DIR"/*/; do
    if [ -d "$d" ]; then
      if ! grep -rq "$CONTRACT_IMPORT" "$d" --include="*.$FILE_EXT" 2>/dev/null; then
        echo "✗ HERMETIC: $d has no $CONTRACT_IMPORT import"
        FAIL=1
      fi
    fi
  done
  if [ "$FAIL" -eq 0 ]; then
    echo "✓ All domains import from $CONTRACT_IMPORT"
  fi
else
  echo "⚠ No domain directory at $DOMAIN_DIR"
fi

echo ""
if [ "$FAIL" -eq 0 ]; then
  echo "══════════════════════════════"
  echo "  ALL GATES PASSED"
  echo "══════════════════════════════"
else
  echo "══════════════════════════════"
  echo "  GATES FAILED — see above"
  echo "══════════════════════════════"
  exit 1
fi
