# Wave A: Bevy UI — Title, WorldMap, Town, Shop

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Replace the battle-only Bevy GUI with a full game loop: Title → WorldMap → Town → Shop → Battle → WorldMap.

**Architecture:** Introduce `AppState` as Bevy `States` enum. Each screen is a plugin that spawns entities on `OnEnter`, runs systems in `Update` gated by `run_if(in_state)`, and despawns on `OnExit`. The existing `game_state::GameState` becomes a Bevy `Resource`. All game logic calls into existing domain functions.

**Tech Stack:** Bevy 0.15, existing 21 domain crates, shared type contract.

**Spec:** `docs/superpowers/specs/2026-03-22-wave-a-bevy-ui-design.md`

---

### Task 1: AppState Enum + Game State Resource

**Files:**
- Create: `src/domains/ui/app_state.rs`
- Modify: `src/domains/ui/mod.rs`
- Modify: `src/domains/ui/plugin.rs`

- [ ] **Step 1: Create app_state.rs with the AppState enum**

```rust
// src/domains/ui/app_state.rs
use bevy::prelude::*;

#[derive(States, Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum AppState {
    #[default]
    Title,
    WorldMap,
    Town,
    Shop,
    InBattle,
    PostBattle,
}
```

- [ ] **Step 2: Create GameStateRes and SaveDataRes resource wrappers**

Add to `app_state.rs`:

```rust
use crate::game_state::GameState;
use crate::domains::save::SaveData;

#[derive(Resource)]
pub struct GameStateRes(pub GameState);

#[derive(Resource)]
pub struct SaveDataRes(pub SaveData);

/// Which town we're currently in.
#[derive(Resource)]
pub struct CurrentTown(pub crate::shared::TownId);

/// Which shop we're currently in.
#[derive(Resource)]
pub struct CurrentShop(pub crate::shared::ShopId);
```

- [ ] **Step 3: Register module in mod.rs**

Add `pub mod app_state;` to `src/domains/ui/mod.rs`.

- [ ] **Step 4: Refactor plugin.rs to use AppState**

Replace the flat Startup-based registration with state-aware registration. The `ValeVillagePlugin::build()` method should:
1. Load game data (keep existing native/WASM split)
2. Create `GameState::new_game()` and `SaveData` via `save::create_new_game()`
3. Initialize world map from `starter_data::starter_map_nodes()`
4. Register `init_state::<AppState>()` (starts at Title)
5. Insert resources: `GameStateRes`, `GameDataRes`, `SaveDataRes`
6. Register sub-plugins for each screen (added in later tasks)
7. Move battle systems to `run_if(in_state(AppState::InBattle))`

Key changes in plugin.rs:
- Remove the `build_demo_battle` call from startup (battle is created when entering InBattle)
- Remove `Startup` system registration for battle_scene/hud/planning
- Add `OnEnter(AppState::InBattle)` for battle setup (move existing systems there)
- Add `OnExit(AppState::InBattle)` for cleanup
- Gate all `Update` battle systems with `.run_if(in_state(AppState::InBattle))`

- [ ] **Step 5: Verify compilation**

Run: `cargo check`
Expected: compiles with warnings (unused states, missing screens)

- [ ] **Step 6: Commit**

```bash
git add src/domains/ui/app_state.rs src/domains/ui/mod.rs src/domains/ui/plugin.rs
git commit -m "feat(ui): add AppState enum and GameStateRes resource"
```

---

### Task 2: Shared UI Components (JrpgPanel, MenuButton, GoldDisplay)

**Files:**
- Create: `src/domains/ui/ui_helpers.rs`
- Modify: `src/domains/ui/mod.rs`

- [ ] **Step 1: Create ui_helpers.rs with styling constants and helper functions**

```rust
// src/domains/ui/ui_helpers.rs
use bevy::prelude::*;

// JRPG color palette
pub const BG_COLOR: Color = Color::srgb(0x0a as f32 / 255.0, 0x0a as f32 / 255.0, 0x1a as f32 / 255.0);
pub const PANEL_BG: Color = Color::srgba(0x1a as f32 / 255.0, 0x1a as f32 / 255.0, 0x2e as f32 / 255.0, 0.95);
pub const BORDER_COLOR: Color = Color::srgb(0xcc as f32 / 255.0, 0x88 as f32 / 255.0, 0x44 as f32 / 255.0);
pub const TEXT_COLOR: Color = Color::srgb(0xe8 as f32 / 255.0, 0xd5 as f32 / 255.0, 0xb7 as f32 / 255.0);
pub const TEXT_DIM: Color = Color::srgb(0x88 as f32 / 255.0, 0x88 as f32 / 255.0, 0x88 as f32 / 255.0);
pub const HIGHLIGHT: Color = Color::srgb(0xff as f32 / 255.0, 0xcc as f32 / 255.0, 0x66 as f32 / 255.0);
pub const GOLD_COLOR: Color = Color::srgb(0xff as f32 / 255.0, 0xd7 as f32 / 255.0, 0x00 as f32 / 255.0);

/// Marker for screen-owned entities. Despawn all entities with this on state exit.
#[derive(Component)]
pub struct ScreenEntity;

/// Marker for interactive buttons.
#[derive(Component)]
pub struct MenuButton {
    pub action: ButtonAction,
}

/// What a button does when clicked.
#[derive(Debug, Clone)]
pub enum ButtonAction {
    NewGame,
    Continue,
    TravelTo(crate::shared::MapNodeId),
    EnterShop(crate::shared::ShopId),
    TalkToNpc(crate::shared::NpcId),
    CollectDjinn(crate::shared::DjinnId),
    BuyItem(crate::shared::EquipmentId),
    LeaveToMap,
    LeaveToTown,
}

/// Spawn a JRPG-style bordered panel as a UI node. Returns the entity ID.
pub fn spawn_panel(commands: &mut Commands, width: Val, height: Val, position: Option<Node>) -> Entity {
    let node = position.unwrap_or(Node {
        width,
        height,
        flex_direction: FlexDirection::Column,
        align_items: AlignItems::Center,
        justify_content: JustifyContent::Center,
        padding: UiRect::all(Val::Px(16.0)),
        border: UiRect::all(Val::Px(2.0)),
        ..default()
    });

    commands.spawn((
        node,
        BackgroundColor(PANEL_BG),
        BorderColor(BORDER_COLOR),
        ScreenEntity,
    )).id()
}

/// Spawn a text button inside a parent entity.
pub fn spawn_button(commands: &mut Commands, parent: Entity, text: &str, action: ButtonAction, font_size: f32) -> Entity {
    let button_id = commands.spawn((
        Button,
        Node {
            padding: UiRect::axes(Val::Px(24.0), Val::Px(8.0)),
            margin: UiRect::all(Val::Px(4.0)),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            ..default()
        },
        BackgroundColor(Color::NONE),
        BorderColor(BORDER_COLOR),
        MenuButton { action },
        ScreenEntity,
    )).with_children(|parent_builder| {
        parent_builder.spawn((
            Text::new(text),
            TextFont { font_size, ..default() },
            TextColor(TEXT_COLOR),
        ));
    }).id();

    commands.entity(parent).add_child(button_id);
    button_id
}

/// Global button hover/click system. Run in Update for all states.
pub fn button_hover_system(
    mut query: Query<(&Interaction, &mut BackgroundColor), (Changed<Interaction>, With<MenuButton>)>,
) {
    for (interaction, mut bg) in &mut query {
        *bg = match interaction {
            Interaction::Hovered => BackgroundColor(Color::srgba(1.0, 1.0, 1.0, 0.1)),
            Interaction::Pressed => BackgroundColor(Color::srgba(1.0, 1.0, 1.0, 0.2)),
            Interaction::None => BackgroundColor(Color::NONE),
        };
    }
}

/// Despawn all entities with ScreenEntity marker.
pub fn despawn_screen(mut commands: Commands, query: Query<Entity, With<ScreenEntity>>) {
    for entity in &query {
        commands.entity(entity).despawn_recursive();
    }
}
```

- [ ] **Step 2: Register in mod.rs**

Add `pub mod ui_helpers;` to `src/domains/ui/mod.rs`.

- [ ] **Step 3: Register button_hover_system globally in plugin.rs**

In `ValeVillagePlugin::build()`, add:
```rust
.add_systems(Update, ui_helpers::button_hover_system)
```

- [ ] **Step 4: Verify compilation**

Run: `cargo check`

- [ ] **Step 5: Commit**

```bash
git add src/domains/ui/ui_helpers.rs src/domains/ui/mod.rs src/domains/ui/plugin.rs
git commit -m "feat(ui): add shared JRPG UI components"
```

---

### Task 3: Title Screen

**Files:**
- Create: `src/domains/ui/title_screen.rs`
- Modify: `src/domains/ui/mod.rs`
- Modify: `src/domains/ui/plugin.rs`

- [ ] **Step 1: Create title_screen.rs**

```rust
// src/domains/ui/title_screen.rs
use bevy::prelude::*;
use super::app_state::AppState;
use super::ui_helpers::*;

pub struct TitleScreenPlugin;

impl Plugin for TitleScreenPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(AppState::Title), setup_title)
           .add_systems(Update, handle_title_buttons.run_if(in_state(AppState::Title)))
           .add_systems(OnExit(AppState::Title), despawn_screen);
    }
}

fn setup_title(mut commands: Commands) {
    // Camera
    commands.spawn((Camera2d, ScreenEntity));

    // Center panel
    let panel = commands.spawn((
        Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            ..default()
        },
        BackgroundColor(BG_COLOR),
        ScreenEntity,
    )).id();

    // Title text
    let title = commands.spawn((
        Text::new("VALE VILLAGE"),
        TextFont { font_size: 64.0, ..default() },
        TextColor(BORDER_COLOR),
        Node { margin: UiRect::bottom(Val::Px(48.0)), ..default() },
        ScreenEntity,
    )).id();
    commands.entity(panel).add_child(title);

    // Buttons
    spawn_button(&mut commands, panel, "New Game", ButtonAction::NewGame, 28.0);
    spawn_button(&mut commands, panel, "Continue", ButtonAction::Continue, 28.0);
}

fn handle_title_buttons(
    query: Query<(&Interaction, &MenuButton), Changed<Interaction>>,
    mut next_state: ResMut<NextState<AppState>>,
    mut game_state: ResMut<super::app_state::GameStateRes>,
) {
    for (interaction, button) in &query {
        if *interaction != Interaction::Pressed { continue; }
        match &button.action {
            ButtonAction::NewGame => {
                // Initialize world map
                let nodes = crate::starter_data::starter_map_nodes();
                let map = crate::domains::world_map::load_map(nodes);
                game_state.0.world_map = Some(map);
                game_state.0.screen = crate::shared::GameScreen::WorldMap;
                next_state.set(AppState::WorldMap);
            }
            ButtonAction::Continue => {
                // TODO: load from save
                // For now, same as new game
                let nodes = crate::starter_data::starter_map_nodes();
                let map = crate::domains::world_map::load_map(nodes);
                game_state.0.world_map = Some(map);
                next_state.set(AppState::WorldMap);
            }
            _ => {}
        }
    }
}
```

- [ ] **Step 2: Register in mod.rs and plugin.rs**

mod.rs: add `pub mod title_screen;`
plugin.rs: add `.add_plugins(title_screen::TitleScreenPlugin)` in build()

- [ ] **Step 3: Verify compilation and test manually**

Run: `cargo check`
Then: `cargo run -- --gui` (should show title screen)

- [ ] **Step 4: Commit**

```bash
git add src/domains/ui/title_screen.rs src/domains/ui/mod.rs src/domains/ui/plugin.rs
git commit -m "feat(ui): title screen with New Game / Continue"
```

---

### Task 4: World Map Screen

**Files:**
- Create: `src/domains/ui/world_map_screen.rs`
- Modify: `src/domains/ui/mod.rs`
- Modify: `src/domains/ui/plugin.rs`

- [ ] **Step 1: Create world_map_screen.rs**

The world map renders MapNode positions as clickable circles connected by lines. Uses Bevy UI nodes positioned absolutely based on MapNode.position (f32, f32) scaled to screen space.

Key systems:
- `setup_world_map` — spawns camera, background, nodes as buttons at scaled positions, connection lines
- `handle_map_clicks` — click a node → check if unlocked → travel (3% encounter check) → transition to Town/Dungeon/Battle
- `despawn_screen` on exit

Node rendering: each MapNode gets a circular UI element (rounded border-radius node) at `(position.0 * scale_x, position.1 * scale_y)`. Color based on `NodeUnlockState`: Locked=hidden, Visible=dim gray, Unlocked=gold border, Completed=green.

Connection lines: spawn thin rectangular UI nodes rotated to connect adjacent node positions. (Or use Gizmos for debug lines initially, replace with sprites later.)

Bottom bar: gold display, "Menu" placeholder text.

Travel logic: when a node is clicked, check `world_map::can_travel()`. If accessible, increment `steps_since_encounter`, check for random encounter (3% per step from starter_data overworld encounters). If encounter → insert `BattleRes` and transition to `InBattle`. If no encounter → transition to `Town` or placeholder for Dungeon.

- [ ] **Step 2: Register in mod.rs and plugin.rs**

- [ ] **Step 3: Verify — New Game from title should show the world map with nodes**

Run: `cargo run -- --gui`

- [ ] **Step 4: Commit**

```bash
git add src/domains/ui/world_map_screen.rs src/domains/ui/mod.rs src/domains/ui/plugin.rs
git commit -m "feat(ui): world map with node graph and travel"
```

---

### Task 5: Town Screen

**Files:**
- Create: `src/domains/ui/town_screen.rs`
- Modify: `src/domains/ui/mod.rs`
- Modify: `src/domains/ui/plugin.rs`

- [ ] **Step 1: Create town_screen.rs**

Panel-based layout. Reads `CurrentTown` resource to find the `TownDef` in `GameDataRes`.

Setup spawns:
- Town name + description header
- NPC list (from `town::get_npcs()`) — each as a button with `ButtonAction::TalkToNpc`
- Shop button (from `town::get_shops()`) — `ButtonAction::EnterShop`
- Djinn discovery (from `town::check_djinn_discovery()`) — if available, collect button
- "Leave" button → `ButtonAction::LeaveToMap`

Button handler:
- `TalkToNpc` → placeholder text popup "NPC says hello!" (real dialogue in Wave B)
- `EnterShop` → insert `CurrentShop` resource, transition to `AppState::Shop`
- `CollectDjinn` → call `town::discover_djinn()`, update save data
- `LeaveToMap` → transition to `AppState::WorldMap`

- [ ] **Step 2: Register in mod.rs and plugin.rs**

- [ ] **Step 3: Verify — travel to Vale Village from world map shows town screen**

- [ ] **Step 4: Commit**

```bash
git add src/domains/ui/town_screen.rs src/domains/ui/mod.rs src/domains/ui/plugin.rs
git commit -m "feat(ui): town screen with NPCs, shop, djinn"
```

---

### Task 6: Shop Screen

**Files:**
- Create: `src/domains/ui/shop_screen.rs`
- Modify: `src/domains/ui/mod.rs`
- Modify: `src/domains/ui/plugin.rs`

- [ ] **Step 1: Create shop_screen.rs**

Two-column panel. Reads `CurrentShop` to find `ShopDef` in `GameDataRes`.

Left column: shop inventory. For each `ShopEntry`:
- Item name (from equipment defs), price, stock remaining
- Click → `ButtonAction::BuyItem`
- Grayed out if gold < price or stock == 0

Right column:
- "Gold: {amount}" display
- Player inventory list (from SaveDataRes)

Buy handler:
- Call `shop::can_buy()` first
- If ok, call `shop::execute_buy()` → deduct gold, decrement stock, add to inventory
- Update UI reactively

"Leave" button → remove `CurrentShop` resource, transition to `AppState::Town`

- [ ] **Step 2: Register**

- [ ] **Step 3: Verify — enter shop from town, buy an item, gold decrements**

- [ ] **Step 4: Commit**

```bash
git add src/domains/ui/shop_screen.rs src/domains/ui/mod.rs src/domains/ui/plugin.rs
git commit -m "feat(ui): shop screen with buy functionality"
```

---

### Task 7: Battle Integration + Return Flow

**Files:**
- Modify: `src/domains/ui/plugin.rs`
- Modify: `src/domains/ui/app_state.rs` (add PostBattle → WorldMap transition)

- [ ] **Step 1: Wire PostBattle → WorldMap return**

After battle victory:
- Apply XP, gold, rewards to `SaveDataRes` and `GameStateRes`
- Mark encounter completed in `GameStateRes`
- Complete the node on the world map
- Transition back to `AppState::WorldMap`

After defeat:
- Transition to `AppState::Title` (or GameOver in Wave C)

- [ ] **Step 2: Wire WorldMap travel → Battle entry**

When travel triggers a random encounter:
- Build battle from encounter def using `cli_runner::build_campaign_battle` or equivalent
- Insert `BattleRes`
- Transition to `AppState::InBattle`

When entering a dungeon node with boss encounter (Wave B placeholder):
- For now, trigger a battle directly

- [ ] **Step 3: Full loop test**

Verify: Title → New Game → WorldMap → click Vale Village → Town → Shop → buy item → Leave → Leave → click another node → random encounter → battle → victory → back to WorldMap.

- [ ] **Step 4: Commit**

```bash
git add src/domains/ui/plugin.rs src/domains/ui/app_state.rs
git commit -m "feat(ui): complete game loop — Title→Map→Town→Shop→Battle→Map"
```

---

### Task 8: WASM Build + Verify

**Files:**
- Modify: `scripts/build-wasm.sh` (if needed)
- Modify: `web/index.html` (if needed)

- [ ] **Step 1: Build WASM**

Run: `bash scripts/build-wasm.sh`
Expected: 25MB+ binary, no errors

- [ ] **Step 2: Serve and test in browser**

Run: `python3 -m http.server 8080 -d ./web`
Test: full game loop works in browser

- [ ] **Step 3: Final commit**

```bash
git add -A
git commit -m "feat(ui): Wave A complete — full game loop in browser"
```

---

## Execution Notes

- **Existing battle code:** Don't rewrite. Move the existing Startup systems to OnEnter(InBattle) and gate Update systems with run_if.
- **Camera management:** Each screen spawns its own Camera2d tagged with ScreenEntity. On exit, it's despawned. The battle scene already spawns a camera — ensure no conflicts.
- **Font:** Use Bevy's default_font (included in features). No custom font needed for Wave A.
- **Testing:** `cargo check` + `cargo run -- --gui` after every task. WASM build at the end.
- **Domain logic:** Never reimplement. Always call into existing domain functions (world_map::, shop::, town::, etc.).
