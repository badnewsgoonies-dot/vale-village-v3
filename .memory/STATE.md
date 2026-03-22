# Vale Village v3 — Current State

**Phase:** Wave 3 — combat polish + save system + PixelLab pipeline
**Verified Commit:** 00e80a0
**Date:** 2026-03-22

## Spine Status: ALL SCREENS FUNCTIONAL
CLI battle path (--cli) functional. Adventure mode (--adventure) integrates
all beyond-battle domains. All 11 GameScreen variants wired and operational.
Title screen has New/Load/Quit. Save/Load fully functional with round-trip
graduation tests.

## Stats
- Commits: 229
- LOC: 22,582
- Tests: 363 (all pass)
- Domains: 23
- Sprites: 402 enemy + 8 effects + 23 djinn + 11 portraits = 444
- CI: GitHub Actions with rust-cache
- Compiler warnings: 0
- Wave 2 audit findings: 5/5 resolved

## This Session (March 22)
- Combat juice: hit stop (50ms/80ms crit), variable event timing, knockback, afterimages
- Menu domain wired into game_loop
- Save roundtrip fix: dungeon collected items now persist (AUD-W2-001)
- SaveLoad screen: fully functional save/load from adventure mode
- Title screen: New Game / Load Game / Quit
- 3 save graduation tests (shop stock, dungeon items, full extension)
- PixelLab API integrated: pipeline script at tools/pixellab_pipeline.py
- First 2 enemies regenerated with PixelLab (dramatically higher quality)
- CLI handoff delivered for full 137-enemy regen (~8 hours, ~405 generations)
- B13 inoculation research committed (N=135, 5 models)

## Debt
P0: None
P1: PixelLab full enemy regen (CLI handoff delivered)
P1: puzzle push-block direction
P2: overworld visual rendering
P2: sprite_loader tests
P2: Psynergy + Status menu screens (stubbed)
