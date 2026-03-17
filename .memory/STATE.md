# Vale Village v3 — Current State

**Phase:** 0 — Setup (orchestration infrastructure installed, type contract not yet frozen)
**HEAD:** 112d6c7
**Date:** 2026-03-17

## Current Status

- Orchestration kit extracted and hooks installed
- Workflow filesystem created (status/, scripts/, src/shared/, .memory/)
- run-gates.sh configured for Rust/Bevy (cargo check/test/clippy)
- No Cargo.toml yet — Bevy project not initialized
- No type contract yet — must freeze before any workers launch

## Next Actions

1. Initialize Rust/Bevy project (Cargo.toml + main.rs)
2. Write type contract (src/shared/mod.rs) with all cross-domain types from DESIGN_LOCK.md + SYSTEMS_FOUNDATION.md
3. Freeze contract (checksum + commit)
4. Write domain specs to docs/domains/
5. Begin Phase 1 dispatch

## Open Debts

None yet.

## Uncertainties

- [Assumed] Bevy version — need to decide exact version and plugin strategy
- [Assumed] RON vs JSON for data files — design docs say RON, confirm
