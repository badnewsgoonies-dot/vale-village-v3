# Domain: screens
## Scope: src/domains/screens/
## Imports from contract
GameScreen, MenuScreen, ScreenTransition, ScreenStack, TownId, DungeonId, ShopId, NpcId, EncounterDef

## Deliverables
1. ScreenManager struct holding current GameScreen + ScreenStack
2. fn push_screen(stack, screen) — validates max depth 8
3. fn pop_screen(stack) -> Option<GameScreen> — returns to previous
4. fn apply_transition(manager, ScreenTransition) -> GameScreen — processes all 12 transition variants
5. fn current_screen(manager) -> &GameScreen
6. Tests: push/pop roundtrip, max depth enforcement, all 12 transition types

## Does NOT handle
Rendering, input, Bevy systems. Pure state machine logic only.

## Quantitative targets
- 12 ScreenTransition variants handled
- ScreenStack max depth = 8
- ≥8 tests
