# Vale Village v3 — Current State

**Phase:** Wave 2 complete + audit fixes applied
**Verified Commit:** 4c7d1b7
**Date:** 2026-03-22

## Spine Status: EXTENDED + AUDITED
CLI battle path (--cli) functional. Adventure mode (--adventure) integrates 10
beyond-battle domains. Post-hoc audit completed: 4 high-severity findings fixed,
3 medium-severity fixed, zero warnings.

## Stats
- Commits: 224
- LOC: 22,366
- Tests: 360 (all pass)
- Domains: 23
- Sprites: 444 (134 enemies x 3 poses + 23 djinn + 8 effects + 11 portraits)
- CI: GitHub Actions with rust-cache
- Compiler warnings: 0

## Domains

| Domain | LOC | Tests | Status |
|--------|-----|-------|--------|
| shared | 909+ | 0 | Contract frozen, checksum verified |
| ai | 679 | 14 | [Observed] |
| battle_engine | 4292 | 52 | [Observed] Core combat loop |
| cli_runner | 1624 | 6 | [Observed] |
| combat | 745 | 24 | [Observed] |
| damage_mods | 283 | 14 | [Observed] |
| data_loader | 548 | 12 | [Observed] |
| dialogue | 444 | 12 | [Observed] Wired to game_loop |
| djinn | 838 | 29 | [Observed] |
| dungeon | 429 | 14 | [Observed] Wired to game_loop |
| encounter | 221 | 13 | [Observed] |
| equipment | 435 | 15 | [Observed] |
| menu | 416 | 12 | [Observed] Wired to game_loop |
| progression | 424 | 17 | [Observed] |
| puzzle | 393 | 12 | [Observed] Push-block ignores direction (P2) |
| quest | 331 | 10 | [Observed] claim_rewards guarded |
| save | 731 | 15 | [Observed] SaveDataExtension |
| screens | 327 | 13 | [Observed] |
| shop | 268 | 10 | [Observed] Uses Gold bounded type |
| sprite_loader | 291 | 0 | [Observed] Manifest-driven |
| status | 1048 | 31 | [Observed] |
| town | 263 | 12 | [Observed] requires_puzzle guarded |
| ui (hub) | 10 | 0 | battle_scene, planning, animation, hud |
| world_map | 320 | 17 | [Observed] |

## Type System
- Bounded types: 14 wired, 0 new_unchecked, 434 validated new()
- Unchecked audit: 228 scanned, 87 upgraded

## Visual Pipeline
- build.rs + sprite_loader + battle_scene wired end-to-end
- sprite_gen.py: Imagen 3 + Gemini eval operational

## Debt
P0: None
P1: puzzle direction, menu djinn state, save extension round-trip test
P2: sprite REDO regen, overworld visual, sprite_loader tests
