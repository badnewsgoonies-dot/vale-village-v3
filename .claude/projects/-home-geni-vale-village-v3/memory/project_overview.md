---
name: vale-village-v3-overview
description: Core project context - Golden Sun-inspired deterministic tactical RPG in Rust/Bevy, pre-build phase with locked design and extracted v2 data
type: project
---

Vale Village v3 is a Golden Sun-inspired deterministic tactical RPG. Fresh Rust/Bevy build.

**Status:** Pre-Build (Design Locked). All combat mechanics locked. Data extracted and validated. Ready for vertical slice.

**Why:** Rebuilding from a v2 TypeScript codebase (183 commits of balance work). The v2 data (241 abilities, 137 enemies, 109 equipment, 23 djinn, 55 encounters, 11 units) is canonical and must be migrated, not recreated.

**How to apply:** All design decisions are in DESIGN_LOCK.md - treat as contract. SYSTEMS_FOUNDATION.md defines the 19 mechanical systems. DATA_MANIFEST.md has the complete content inventory and proposed RON file structure. COMPLETE_DATA.md has every data table. SESSION_NOTES.md has deep mechanical explanations.

**Key architecture decisions:**
- Engine: Bevy (Rust), data-driven via RON files
- Build method: AI-orchestrated (Codex workers + Claude orchestrator)
- Content is separate from systems - adding content = adding data, not code
- 7-phase implementation order: Schema+Loader → Core Combat → Status → Advanced Damage → Life/Death+Equipment → Djinn → Content Polish
