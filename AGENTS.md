# AGENTS.md

Read these before acting:
1. STATE.md
2. MANIFEST.md
3. docs/spec.md
4. docs/MANUAL.md
5. src/shared/mod.rs

Core rules:
- Trust fresh code, tests, and runtime over conversation.
- Follow the wave cadence: Feature → Gate → Document → Harden → Graduate.
- Keep scope mechanical. Use `bash scripts/clamp-scope.sh <allowed-path>`.
- Do not edit `src/shared/mod.rs` unless explicitly routed to contract work.
- Update state artifacts only after verified changes.
