# Wave 2 Audit Report — Vale Village v3
**Date:** 2026-03-21
**Auditor:** Claude (Opus 4.6, primary chat)
**Scope:** All Wave 2 work: 10 new domains, game_state, game_loop, battle integration
**Commit range:** 36dcd97..31cc165 (13 commits)

## Structural Gates
- [x] `cargo check` — PASS (warnings only, no errors)
- [x] `cargo check --tests` — PASS (warnings only, no errors)
- [x] Contract checksum — OK
- [ ] `cargo test` — OOMs (Bevy in 4GB, same as Hearthfield)

## Findings Summary

| ID | Severity | Domain | Finding | Status |
|----|----------|--------|---------|--------|
| AUD-W2-001 | INFO | game_state | 3 TODOs: shop_state not serialized, collected_items not serialized, position hardcoded (0,0) | Open — save roundtrip loses shop stock and dungeon progress |
| AUD-W2-002 | INFO | game_loop | SnapshotCtx has 3 TODOs: has_djinn always false, party_contains always true, has_element_ability always false | Open — dialogue/puzzle conditions partially evaluate |
| AUD-W2-003 | WARNING | game_loop | 1,232 lines mixing content data (starter_*) with integration logic (run_*). Growing toward monolith | Open — extract starter data to RON or data module |
| AUD-W2-004 | INFO | encounter | Domain exists (221 LOC, 13 tests) but not imported by any integration file. Overworld random encounters not wired | Open — wired for dungeon encounters via dungeon domain only |
| AUD-W2-005 | INFO | menu | Domain written (416 LOC, 12 tests) but not wired into game_loop | Open — show_* functions ready, need game_loop Menu handler |
| AUD-W2-006 | OK | dungeon→dialogue | Cross-domain import for ConditionContext. By design — dungeon needs condition evaluation for gated exits | Accepted |
| AUD-W2-007 | OK | all | Zero bare bounded-type constructor violations across all new domains | Clean |
| AUD-W2-008 | OK | all | Zero bare .0 field access violations on bounded types | Clean |
| AUD-W2-009 | OK | screens | All 12 ScreenTransition variants handled in apply_transition | Clean |
| AUD-W2-010 | OK | screens | All 11 GameScreen variants handled in game_loop main match | Clean |
| AUD-W2-011 | OK | quest | Monotonic advance enforced in contract (QuestState::advance checks to > current) | Clean |
| AUD-W2-012 | OK | dungeon | One-time item collection enforced via HashSet | Clean |
| AUD-W2-013 | OK | all | Boss encounter IDs (house-16, house-17) verified present in data/full/encounters.ron | Clean |
| AUD-W2-014 | OK | dialogue | Production expect at line 61 is logic invariant (current_node must exist in tree). Acceptable — only reachable if tree is malformed | Accepted |
| AUD-W2-015 | OK | game_loop | Production expect at line 587 (world_map.as_ref().expect). Only called from WorldMap screen state. Acceptable | Accepted |

## Production Unwrap/Expect Assessment
- **dialogue:61** — `expect("current_node must exist in tree")`. Only fails if DialogueTree data is internally inconsistent. Not reachable through normal game state transitions. **Acceptable.**
- **game_loop:587** — `state.world_map.as_ref().expect("world map not loaded")`. Only called when screen is WorldMap, and world_map is set during game_loop init before any screen transition. **Acceptable.**
- All other unwraps are in test code. **Clean.**

## Test Coverage
| Domain | Tests | Assessment |
|--------|-------|------------|
| screens | 13 | Good |
| dialogue | 12 | Good |
| quest | 10 | Good |
| shop | 10 | Good |
| encounter | 13 | Good (but domain unused) |
| puzzle | 12 | Good |
| world_map | 17 | Excellent |
| town | 11 | Good |
| dungeon | 14 | Good |
| save | 15 | Good |
| menu | 12 | Good (but not wired) |
| game_state | 4 | Adequate |
| game_loop | 5 | Adequate — covers data loading, dialogue traversal, dungeon traversal, shop flow, world map setup |
| **Total new** | **148** | |

## Save Roundtrip Gaps (AUD-W2-001 detail)
**What's saved:** quest_state, map_unlock_state, overworld location, play_time, screen, dungeon current_room, visited_rooms
**What's lost on save/load:**
- Shop stock changes (Unlimited items are fine, Limited stock resets)
- Dungeon collected_items (items respawn on reload)
- Player position within dungeon/town (resets to default)
- Inventory (lives in save_data.inventory, not game_state — separate system)

**Risk:** Low for current scope (2 dungeons, 2 shops). Becomes P1 when content expands.

## Architecture Assessment
**What works well:**
- Contract frozen at 909 lines, checksum verified, never touched by workers
- All 10 domains compile independently with shared contract imports
- Bounded types caught every hallucinated struct field and wrong constructor during worker dispatch
- Worker fix loop trend: 4 fixes → 3 → 0 → 0 (prompt learning curve)
- game_state.rs is clean separation of composite state from game loop logic

**What needs attention:**
- game_loop.rs is an integration module that also holds content data. The 6 `starter_*` functions (map nodes, towns, shops, dialogues, dungeons, shop defs) total ~450 LOC of hardcoded data that should move to RON files
- encounter domain is architecturally orphaned — its should_encounter/select_encounter functions aren't called. Dungeon encounters go through dungeon::trigger_encounter directly
- menu domain is complete but not wired into the GameScreen::Menu handler

## Recommendations (priority order)
1. **P1:** Wire menu domain into game_loop Menu handler (enables party/equipment/quest viewing)
2. **P1:** Extract starter_* data to RON files in data/world/ (game_loop.rs drops from 1,232 to ~750 LOC)
3. **P2:** Implement save extension for shop_stock and collected_items
4. **P2:** Wire encounter domain for overworld random encounters on world map travel
5. **P2:** Implement ConditionContext properly (has_djinn, party_contains, has_element_ability)
6. **P3:** Consider splitting game_loop.rs run_dungeon (currently 150+ lines) into a separate integration module

## Cost Accounting
| Item | Premium | LOC |
|------|---------|-----|
| Wave 2 Batch A (screens, dialogue, quest) | 3 | 1,097 |
| Wave 2 Batch B (shop, encounter, puzzle) | 3 | 881 |
| Wave 2 Batch C (world_map, town, dungeon) | 3 | 958 |
| Wave 2 Batch D (save extension) | 1 | 167 |
| Wave 3 menu worker | 1 | 416 |
| Orchestrator integration (game_state, game_loop, battle wiring, Kolima Forest) | 0 | ~1,700 |
| **Total** | **11** | **~5,219** |

0.47 premium per domain. 0.002 premium per LOC. 148 new tests at 0.074 premium per test.
