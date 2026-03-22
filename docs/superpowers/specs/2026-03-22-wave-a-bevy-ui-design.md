# Wave A: Full Bevy UI — Title, WorldMap, Town, Shop

## Goal
Replace the text-mode game loop with visual Bevy screens so the game is fully playable in the browser via WASM. Wave A covers the core flow: Title → WorldMap → Town → Shop → Battle → back to WorldMap.

## Architecture

### AppState Migration
Introduce `AppState` as a Bevy `States` enum replacing the flat battle-only setup:

```
AppState: Title | WorldMap | Town | Shop | PreBattle | InBattle | PostBattle
```

Each state has:
- `OnEnter` system: spawns screen entities
- `Update` systems: run_if(in_state) for interaction
- `OnExit` system: despawns all screen entities (marker component per screen)

The existing battle flow (PreBattle → InBattle → PostBattle) stays exactly as-is. We add states around it.

### Resources
- `GameStateRes(GameState)` — full game state (screen stack, gold, quests, dungeon progress). Uses the existing `game_state::GameState` struct.
- `GameDataRes(GameData)` — all content definitions (already exists)
- `SaveDataRes(SaveData)` — save file data (already exists)
- `SelectedTown(TownId)` — which town we're in
- `SelectedShop(ShopId)` — which shop we're in

### Shared UI Components
- **JrpgPanel** — reusable bordered container. Node with border color #cc8844, background #1a1a2e.
- **MenuButton** — clickable text with hover highlight (#cc8844 → #ffcc66 on hover).
- **GoldDisplay** — "Gold: 150" in accent color.
- All screens use dark navy background (#0a0a1a) with gold (#cc8844) accent. Pixel art aesthetic.

### Data Flow
All screens read `GameStateRes` and `GameDataRes`. Mutations go through existing domain functions (world_map::unlock_node, shop::execute_buy, etc.) then update `GameStateRes`. State transitions use `NextState<AppState>`.

## Screen Designs

### 1. Title Screen
- Centered panel on dark background
- "VALE VILLAGE" title in large text
- Three stacked buttons: New Game, Continue, Quit
- Continue grayed out if no save exists
- New Game → creates fresh GameState + SaveData, transitions to WorldMap
- Continue → loads save, transitions to WorldMap

### 2. World Map
- Dark background with positioned nodes from MapNode data
- Nodes rendered as circles at defined positions, connected by lines
- Node visual states: Locked (hidden), Visible (dim), Unlocked (bright/clickable), Completed (solid/checkmark)
- Click unlocked node → travel. Travel triggers 3% random encounter check.
- If encounter → transition to battle. Otherwise → enter Town or Dungeon.
- Bottom bar: gold display, party info
- Dungeon nodes: placeholder "Coming in Wave B" for now

### 3. Town
- Panel layout. Town name at top with description.
- Vertical list of options:
  - NPCs listed by name (click → placeholder dialogue for now, real dialogue in Wave B)
  - Shop button → transitions to Shop state
  - Djinn discovery point if conditions met → collect button
  - Leave → returns to WorldMap
- Uses TownDef data from GameDataRes

### 4. Shop
- Two-column bordered panel
- Left: shop inventory (item name, price, stock)
- Right: player gold + inventory
- Click item to buy → calls shop::execute_buy(), updates gold display
- 0-stock items grayed out
- Leave button → returns to Town
- Uses ShopDef + shop domain logic

## File Structure
```
src/domains/ui/
  mod.rs              — add new module declarations
  plugin.rs           — refactor to use AppState, register all screen plugins
  app_state.rs        — NEW: AppState enum
  ui_helpers.rs       — NEW: JrpgPanel, MenuButton, GoldDisplay components
  title_screen.rs     — NEW: TitleScreenPlugin
  world_map_screen.rs — NEW: WorldMapPlugin
  town_screen.rs      — NEW: TownScreenPlugin
  shop_screen.rs      — NEW: ShopScreenPlugin
  battle_scene.rs     — existing (minor refactor for AppState)
  planning.rs         — existing (unchanged)
  hud.rs              — existing (unchanged)
  animation.rs        — existing (unchanged)
```

## Dependencies
- All 21 logic domains already exist and are tested
- GameState, GameData, SaveData types already defined in shared/
- starter_data.rs already provides all content
- No new crate dependencies needed

## Success Criteria
- WASM build loads Title screen
- New Game → WorldMap with 10 nodes
- Click Vale Village → Town screen with NPCs, shop access
- Buy items in shop, gold updates
- Leave town → back to WorldMap
- Travel to encounter node → battle triggers
- Win battle → back to WorldMap with progression
- Full loop: Title → Map → Town → Shop → Map → Battle → Map
