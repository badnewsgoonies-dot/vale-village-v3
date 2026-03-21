# Vale Village v3 — Current State

**Phase:** Wave 2 complete, integration done — adventure mode playable
**Verified Commit:** b824125
**Date:** 2026-03-21

## Spine Status: EXTENDED
The CLI battle path (--cli mode) remains functional. A new --adventure mode integrates
10 beyond-battle domains into a text-based game loop: world map navigation, town
exploration, NPC dialogue with branching and side effects, shop buy/sell, dungeon
room-by-room traversal with item pickup and boss rooms.

## Stats
- Commits: 101
- LOC: 21,095
- Tests: 357
- Domains: 24 (14 original + 10 new)

## Domains

| Domain | LOC | Tests | Wired? | Status |
|--------|-----|-------|--------|--------|
| shared | 909 | 0 | YES | Contract frozen (Wave 1 extension) |
| ai | 679 | 14 | YES | [Observed] |
| battle_engine | 4292 | 49 | YES | [Observed] |
| cli_runner | 1624 | 6 | YES | [Observed] |
| combat | 745 | 23 | YES | [Observed] |
| damage_mods | 283 | 14 | YES | [Observed] |
| data_loader | 548 | 12 | YES | [Observed] |
| dialogue | 444 | 12 | YES | [Observed] Wave 2, wired to game_loop |
| djinn | 838 | 27 | YES | [Observed] |
| dungeon | 405 | 14 | YES | [Observed] Wave 2, wired to game_loop |
| encounter | 221 | 13 | YES | [Observed] Wave 2, domain logic only |
| equipment | 435 | 15 | YES | [Observed] |
| menu | 1 | 0 | NO | Stub |
| progression | 424 | 17 | YES | [Observed] |
| puzzle | 393 | 12 | YES | [Observed] Wave 2, domain logic only |
| quest | 326 | 10 | YES | [Observed] Wave 2, wired to game_loop |
| save | 728 | 15 | YES | [Observed] Extended with SaveDataExtension |
| screens | 327 | 13 | YES | [Observed] Wave 2, wired to game_state |
| shop | 268 | 10 | YES | [Observed] Wave 2, wired to game_loop |
| sprite_loader | 291 | 0 | partial | Bevy sprite loader |
| status | 1048 | 31 | YES | [Observed] |
| town | 233 | 11 | YES | [Observed] Wave 2, wired to game_loop |
| ui | 10 | 0 | YES | Bevy UI plugin |
| world_map | 320 | 17 | YES | [Observed] Wave 2, wired to game_loop |

## Gate Status
- [x] Contract checksum: OK
- [x] Compile: cargo check OK
- [x] Test compile: cargo check --tests OK
- [ ] Test execution: cargo test OOMs (Bevy in 4GB)
- [x] Connectivity: 22/24 domains import shared (menu + sprite_loader are stubs)

## Integration Layer
- game_state.rs (181 LOC): GameState composite, save extension roundtrip
- game_loop.rs (870 LOC): CLI adventure mode — world map, town, shop, dialogue, dungeon

## Pending
- [ ] RON data loader for world data (currently hardcoded in game_loop.rs)
- [ ] Encounter domain wiring (random encounters in dungeons)
- [ ] Puzzle domain wiring (puzzle rooms in dungeons)
- [ ] Menu screens (party, equipment, djinn, items)
- [ ] Battle integration from dungeon encounters
- [ ] Save extension persistence in --adventure mode
- [ ] Bevy GUI for adventure mode

## Wave 2 Costs
10 premium (Sonnet workers via Copilot) + orchestrator integration time
- Batch A: 3 premium, 1,097 LOC, 4 fixes
- Batch B: 3 premium, 881 LOC, 3 fixes
- Batch C: 3 premium, 958 LOC, 0 fixes
- Batch D: 1 premium, 167 LOC, 0 fixes
