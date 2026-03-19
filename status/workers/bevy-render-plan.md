# Minimum Bevy Render Pipeline Plan

## Goal

Ship the smallest Bevy 0.15 render slice that makes the current combat loop in `src/domains/ui/plugin.rs` and `src/domains/ui/planning.rs` visible, clickable, and diagnosable:

- spawn a 2D battle scene
- render player/enemy combatants plus djinn affordances
- render HUD and planning UI
- keep visuals synchronized with `BattleRes`, `PlanningState`, and `EventQueue`

This plan assumes the existing game flow stays resource-driven:

- `BattleRes` remains the authoritative battle state
- `GameDataRes` remains the authoritative metadata source
- `PlanningState` remains the UI state machine
- `animation::EventQueue` remains the playback bridge

## Minimum app wiring

Use one render plugin that keeps startup and frame systems grouped by concern:

```rust
app.add_plugins(DefaultPlugins.set(WindowPlugin {
    primary_window: Some(Window {
        title: "Vale Village v3".into(),
        resolution: WindowResolution::new(1280.0, 720.0),
        ..default()
    }),
    ..default()
}))
.insert_resource(ClearColor(Color::srgb(0.10, 0.10, 0.18)))
.init_resource::<RenderAssets>()
.add_systems(
    Startup,
    (
        setup_camera,
        load_render_assets,
        setup_battle_scene,
        setup_hud,
        init_planning,
        setup_planning_panel,
    ),
)
.add_systems(
    Update,
    (
        sync_combatant_sprites,
        sync_scene_overlay,
        update_hud,
        update_planning_ui,
        handle_planning_clicks,
        animate_floating_text,
    ),
);
```

Bevy 0.15 specifics to keep:

- register schedules with `add_systems(Startup, ...)` and `add_systems(Update, ...)`
- use `Camera2d` for the main 2D camera
- use UI `Node`, `Text`, `Button`, `BackgroundColor`, `Interaction`, and `Visibility`
- store handles in a `Resource`; load via `AssetServer::load`
- use `Query<..., Changed<T>>` for cheap sync systems where possible

## Required resources

### `RenderAssets`

Central render asset registry:

- `background: Handle<Image>`
- `player_frame: Handle<Image>`
- `enemy_frame: Handle<Image>`
- `highlight_frame: Handle<Image>`
- `djinn_good_icon: Handle<Image>`
- `djinn_recovery_icon: Handle<Image>`
- `font_ui: Handle<Font>`
- `font_numbers: Handle<Font>`

Optional if sprite sheets are used:

- `combatant_layout: Handle<TextureAtlasLayout>`
- `fx_layout: Handle<TextureAtlasLayout>`

### Existing gameplay resources already sufficient

- `BattleRes`
- `GameDataRes`
- `PlanningState`
- `EventQueue`

No extra simulation resource is needed for the minimum slice. Rendering should stay read-only against battle state except for click handlers that already mutate planning.

## Required components

### Scene structure

- `BattleSceneRoot`: top-level world-space root for the combat scene
- `HudRoot`: top-level UI root for HUD
- `PlanningPanel`: already present; keep as the action panel root

### Combatant presentation

- `CombatantSprite { side: Side, index: usize }`
- `CombatantNameplate { side: Side, index: usize }`
- `CombatantHpBar { side: Side, index: usize }`
- `ActingUnitHighlight { side: Side, index: usize }`

These let `sync_combatant_sprites` and `sync_scene_overlay` map ECS entities back to `BattleRes`.

### Djinn presentation

- `DjinnButton { slot_index: u8 }`
- `DjinnStateIcon { slot_index: u8 }`
- `DjinnLabel { slot_index: u8 }`

The current planning flow expects scene-side djinn interaction, so these components should sit near the active player unit and be rebuilt or toggled from team djinn state.

### Animation / feedback

- `FloatingText { timer: Timer, velocity: Vec3 }`
- `PendingDespawnOnTimer(Timer)`

These are enough for damage numbers, mana deltas, KO markers, and djinn state cues from `EventQueue`.

## Required systems

### Startup systems

#### `setup_camera`

- spawn `(Camera2d, Transform::default())`
- optionally add a marker like `MainCamera`

#### `load_render_assets`

- `Res<AssetServer>` loads all image/font handles into `RenderAssets`
- do not block startup on load; rely on Bevy handles becoming ready

#### `setup_battle_scene`

- spawn background sprite
- spawn combatant sprite entities for every player and enemy in `BattleRes`
- spawn overlay children per combatant:
  - name text
  - HP bar/background
  - highlight frame
  - djinn button row for player side

#### `setup_hud`

- bottom HUD container
- per-unit stat readouts for HP, mana contribution, crit counter
- round summary area

#### `setup_planning_panel`

Already exists and uses Bevy 0.15 UI `Node` + `Text`. Keep that surface and avoid a parallel planning implementation.

### Update systems

#### `sync_combatant_sprites`

Reads `BattleRes` and updates:

- `Visibility` for KO units
- sprite tint or grayscale for disabled / dead units
- combatant positions if formation changes
- HP bar width from `current_hp / max_hp`

This should run only when battle state changes enough to matter. `Changed<BattleRes>` is not available on the inner resource contents, so the practical minimum is a regular `Res<BattleRes>` read each frame.

#### `sync_scene_overlay`

Reads `PlanningState`, `BattleRes`, and `GameDataRes` to update:

- acting-unit highlight for `state.current_unit`
- djinn labels and icon state from `battle.team_djinn_slots`
- summon/activation affordances for the active player
- round-complete / executing overlay text

This is the scene-side counterpart to `update_planning_ui`.

#### `update_hud`

Either keep `hud::refresh_hud` inside `update_planning_ui` as it is now, or split it into its own Bevy system so the HUD updates even when the planning panel does not rebuild. For the minimum pipeline, the split is cleaner:

- HP and max HP
- current/max/projected mana
- crit count
- battle result banner

#### `update_planning_ui`

Already exists and is the right minimum surface:

- rebuilds action buttons from `PlanningMode`
- exposes ability choices and target selection
- surfaces djinn summary text
- surfaces playback progress and round summaries

The render plan should keep this resource-driven rebuild instead of introducing a retained widget tree.

#### `handle_planning_clicks`

Already exists and should remain the only planning mutator:

- `Interaction::Pressed` on buttons
- translate button markers into `BattleAction`
- advance `PlanningState`

For scene-side djinn buttons, use the same pattern with `Interaction` and `ActionChoice::ActivateDjinn` / `ActionChoice::Summon`.

#### `animate_floating_text`

Consumes entities tagged with `FloatingText`:

- tick `Timer`
- move upward using `Transform`
- fade `TextColor` or `Sprite.color`
- despawn when timer completes

This is enough to visualize `BattleEvent` output without a more complex timeline system.

## Asset minimum

Minimum asset list for a readable prototype:

- 1 battle background image
- 2 player portrait or body sprites
- 2 enemy sprites
- 1 highlight frame image
- 2 djinn state icons: good, recovery
- 1 solid white pixel or bar texture for HP fill
- 1 UI font
- 1 numeric or HUD font if a second style is desired

Fallback path if art is incomplete:

- use colored `Sprite` quads for combatants and bars
- use Bevy UI text for all labels
- use tint-only state changes for djinn and active-unit feedback

That keeps the render slice shippable before final art exists.

## Data flow mapping to current UI code

From `plugin.rs`, the current order is already close to the needed minimum:

1. Load `GameData`
2. Build `Battle`
3. Insert `BattleRes`, `GameDataRes`, and `EventQueue`
4. Spawn scene, HUD, and planning panel at startup
5. Run planning, scene sync, and animation systems every frame

From `planning.rs`, the render layer must support these visible states:

- `SelectAction`
- `SelectAbility`
- `SelectTarget`
- `Executing`
- `RoundComplete`
- `BattleOver`

The minimum render pipeline succeeds if every one of those modes has a visible scene/HUD response and at least one clickable path forward.

## Implementation order

1. Keep `plugin.rs` as the single place that wires the render pipeline.
2. Stand up `RenderAssets` plus `setup_camera`.
3. Ensure `setup_battle_scene` spawns stable entities with marker components.
4. Ensure `sync_scene_overlay` reflects current acting unit and djinn states.
5. Keep `planning.rs` as the authoritative panel interaction layer.
6. Add floating-text playback once scene state is readable.

## Gate for this plan

The narrow verification target is the document itself plus the current file references:

- `status/workers/bevy-render-plan.md` exists and names Bevy 0.15 APIs
- the plan matches the current startup/update wiring in `src/domains/ui/plugin.rs`
- the plan matches the current planning state machine in `src/domains/ui/planning.rs`

## Scope note

The task requested `status/bevy-render-plan.md`, but the repo clamp for `docs/ src/` preserves `status/workers/` rather than `status/`. This draft therefore lives at `status/workers/bevy-render-plan.md` to stay inside enforced worker scope.
