# Vale Village v3 — Full Quality Audit

**Scope:** 61 commits (e029875..08b1a82), +6,550 LOC, 10 new domains
**Date:** March 22, 2026
**Auditors:** Claude Opus (orchestrator) + Claude Haiku (3 workers)

## Summary

| Category | Score | Notes |
|----------|-------|-------|
| Compilation | ✅ A | Clean, 2 pre-existing warnings |
| Tests | ✅ A | 359 pass, 0 fail, 128 new tests |
| Contract compliance | ✅ A- | All new domains import from shared. Menu has 7 local types (appropriate — UI display types) |
| Sprite coverage | ✅ A | 444 sprites, 0 missing from manifest. Idle/attack/hit complete for all 134 enemies |
| new_unchecked usage | ✅ A | Zero new_unchecked in any new domain |
| TODO/FIXME | ✅ A | Zero in production code |
| Cross-domain imports | ⚠ B | battle_engine imports 6 domains — justified (orchestration role) |
| Error handling | ⚠ C+ | 102 unwrap() calls, 62 in battle_engine alone |
| Save system | ⚠ C | No atomic writes, no migration, no integrity check |
| game_loop quality | ⚠ C+ | Gold duplication, inventory outside GameState, hardcoded boss data |
| Contract hygiene | ⚠ B- | Near-duplicate types, raw (f32,f32) tuples, string timestamp |
| Sprite consistency | ⚠ B | Mixed sizes: idle=256×256, attack/hit=64×64. Functional but inconsistent |

## Mechanical Checks (all pass)

- **Compile:** Clean (2 unused import warnings, pre-existing)
- **Tests:** 359 pass, 0 fail, 0 ignored
- **Contract checksum:** Valid
- **CI:** GitHub Actions passing
- **new_unchecked in new code:** 0 (clean — prior B19 finding heeded)
- **TODO/FIXME/HACK:** 0 in production code
- **Dead code:** Suppressed by `#![allow(dead_code)]` in game_loop.rs

## Findings — Blockers (0)

None. Everything compiles and tests pass.

## Findings — Warnings (8)

### W1: Gold state duplication [game_loop.rs]
`state.gold` and `save_data.gold` are manually synced at ~6 sites. Drift risk — miss one sync point and gold desyncs between gameplay and save.
**Fix:** Single source of truth. Either GameState owns gold exclusively, or derive save gold from state gold.

### W2: Inventory lives outside GameState [game_loop.rs L125+]
`inventory: Vec<ItemId>` is a local variable in run_game_loop passed by &mut through 4+ function signatures. Not persisted in GameState, easy to lose across save/load.
**Fix:** Move inventory into GameState.

### W3: No atomic saves [save/mod.rs]
`fs::write` is not atomic. Crash mid-save corrupts the file. No temp-then-rename pattern.
**Fix:** Write to .tmp, then rename.

### W4: No save version migration [save/mod.rs]
Old saves are hard-rejected on version mismatch. Any schema change bricks existing saves.
**Fix:** Migration functions per version.

### W5: Hardcoded boss data [game_loop.rs L549-556]
Boss encounter IDs, quest flags, and map unlocks are magic numbers inline. Content coupled to loop logic.
**Fix:** Move to DungeonDef or data files.

### W6: OverworldState / OverworldSaveData near-duplicate [shared/mod.rs L879, L904]
Same 4 fields with inconsistent naming. SaveData should embed OverworldState.
**Fix:** `OverworldSaveData { state: OverworldState, save_timestamp: u64 }`

### W7: Raw (f32, f32) as position [shared/mod.rs, 6+ uses]
MapNode, NpcPlacement, DjinnDiscoveryPoint, RoomItem, OverworldState all use bare tuples.
**Fix:** Named `Position { x: f32, y: f32 }` struct.

### W8: Sprite size inconsistency
Idle sprites: 256×256. Attack/hit sprites: 64×64. Unit portraits: 48×64. Bevy's `custom_size` handles display scaling but the asset pipeline should standardize.
**Fix:** Either downscale idle to 64×64 or upscale attack/hit to 256×256. Pick one target size.

## Findings — Notes (5)

### N1: 102 unwrap() calls in production code
62 in battle_engine (justified — internal state should never be None after setup). 40 across other modules. Consider `expect("reason")` for the non-obvious cases.

### N2: `#![allow(dead_code)]` in game_loop.rs
Blanket suppression hides real cleanup debt. Remove and fix individual warnings.

### N3: save_timestamp is a raw String
Should be `u64` (epoch seconds) or a bounded newtype.

### N4: Menu local types (7) are appropriate
MenuState, UnitSummary, EquipmentSummary, DjinnMenuState, DjinnSummary, ItemSummary, QuestLogEntry — these are UI display types, correctly scoped to the domain.

### N5: ItemCount type imported but unused in save inventory
Save stores `Vec<EquipmentId>` with no quantities. Consumables lost.

## Test Coverage

| Domain | Tests | LOC | Ratio |
|--------|-------|-----|-------|
| battle_engine | 52 | 4,292 | 1:83 |
| status | 31 | 1,048 | 1:34 |
| djinn | 29 | 838 | 1:29 |
| combat | 24 | 745 | 1:31 |
| world_map | 17 | 320 | 1:19 |
| progression | 17 | 424 | 1:25 |
| save | 15 | 728 | 1:49 |
| equipment | 15 | 435 | 1:29 |
| dungeon | 14 | 405 | 1:29 |
| damage_mods | 14 | 283 | 1:20 |
| ai | 14 | 679 | 1:49 |
| screens | 13 | 327 | 1:25 |
| encounter | 13 | 221 | 1:17 |
| puzzle | 12 | 393 | 1:33 |
| menu | 12 | 416 | 1:35 |
| dialogue | 12 | 444 | 1:37 |
| data_loader | 12 | 548 | 1:46 |
| town | 11 | 233 | 1:21 |
| shop | 10 | 268 | 1:27 |
| quest | 10 | 326 | 1:33 |
| cli_runner | 6 | 1,624 | 1:271 |
| ui | 0 | 2,880 | ∞ |
| sprite_loader | 0 | 291 | ∞ |

**ui (2,880 LOC, 0 tests)** and **cli_runner (1,624 LOC, 6 tests)** are the coverage gaps. UI is hard to unit test (Bevy ECS), cli_runner is the REPL runner.

## What's Good

1. **Every new domain has tests.** 10 new domains, all with 10+ tests. No empty test modules.
2. **Zero new_unchecked.** The B19 finding about bypass constructors was heeded — new code uses bounded types correctly.
3. **Sprite pipeline is complete.** 134 enemies × 3 poses = 402 sprites. All manifest entries covered. Plus effects, djinn, portraits.
4. **Animation system works.** Hit stop, sprite swapping, knockback, afterimages — real combat juice.
5. **Domain specs exist on disk.** 9 new domain specs in docs/domains/. Workers had written contracts.
6. **Contract grew from 460 to 909 LOC** — 97 types, covering the full game. New types (QuestState, DungeonDef, DialogueTree, etc.) are properly placed in shared.
7. **game_loop.rs is functional** despite its issues. 1,007 LOC that wires 10+ domains into a working game loop with world map, towns, shops, dungeons, dialogue, battles, and saving.

## Verdict

The implementation is solid. 359 tests pass, everything compiles, sprites are complete, domains are well-structured. The warnings are real (save system needs hardening, gold duplication is a bug waiting to happen, sprite sizes should standardize) but none are blocking. The code was clearly built by workers following specs — consistent style, proper shared imports, bounded types used correctly.

**Priority fixes:** W1 (gold duplication) and W3 (atomic saves) should be next. W2 (inventory in GameState) prevents data loss. The rest are P2 cleanup.
