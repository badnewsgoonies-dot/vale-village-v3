//! Bevy plugin wiring — window, camera, and system registration.

use std::path::Path;

use bevy::prelude::*;
use bevy::window::WindowResolution;

use crate::domains::battle_engine::{self, Battle, BattleResult, EnemyUnitData, PlayerUnitData};
use crate::domains::data_loader::{self, GameData};
use crate::domains::djinn::{DjinnInstance, DjinnSlots};
use crate::domains::equipment::{self, EquipmentLoadout};
use crate::domains::progression;
use crate::domains::save::{SaveData, SavedEquipment, SavedUnit};
use crate::domains::sprite_loader::SpriteLoaderPlugin;
use crate::shared::bounded_types::{Gold, Xp};
use crate::shared::{DjinnState, EncounterDef, GameScreen};

use super::app_state::AppState;
use super::dialogue_screen;
use super::dungeon_screen;
use super::puzzle_screen;
use super::screenshot;
use super::shop_screen;
use super::title_screen;
use super::town_screen;
use super::ui_helpers;
use super::world_map_screen;
use super::{animation, battle_scene, hud, planning};

/// Bevy resource wrapping the loaded game data.
#[derive(Resource)]
pub struct GameDataRes(pub GameData);

/// Bevy resource wrapping the active battle state.
#[derive(Resource)]
pub struct BattleRes(pub Battle);

/// Top-level plugin for the Vale Village visual layer.
pub struct ValeVillagePlugin;

impl Plugin for ValeVillagePlugin {
    fn build(&self, app: &mut App) {
        // Load game data — filesystem on native, embedded on WASM
        #[cfg(not(target_arch = "wasm32"))]
        let game_data = {
            let data_dir = Path::new("data/full");
            match data_loader::load_game_data(data_dir) {
                Ok(data) => data,
                Err(errors) => {
                    for e in &errors {
                        eprintln!("Load error: {e:?}");
                    }
                    std::process::exit(1);
                }
            }
        };
        #[cfg(target_arch = "wasm32")]
        let game_data =
            data_loader::load_game_data_embedded().expect("embedded game data must parse");

        app.add_plugins(
            DefaultPlugins
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: "Vale Village v3".into(),
                        resolution: WindowResolution::new(1280.0, 720.0),
                        #[cfg(target_arch = "wasm32")]
                        canvas: Some("#vale-village-canvas".into()),
                        prevent_default_event_handling: true,
                        ..default()
                    }),
                    ..default()
                })
                .set(bevy::asset::AssetPlugin {
                    meta_check: bevy::asset::AssetMetaCheck::Never,
                    ..default()
                })
                .set(bevy::render::texture::ImagePlugin::default_nearest()),
        )
        .add_plugins(SpriteLoaderPlugin)
        .add_plugins(screenshot::ScreenshotPlugin)
        .insert_resource(ClearColor(ui_helpers::BG_COLOR))
        .insert_resource(GameDataRes(game_data))
        .init_state::<AppState>()
        .add_plugins(title_screen::TitleScreenPlugin)
        .add_plugins(world_map_screen::WorldMapPlugin)
        .add_plugins(town_screen::TownScreenPlugin)
        .add_plugins(dialogue_screen::DialogueScreenPlugin)
        .add_plugins(dungeon_screen::DungeonScreenPlugin)
        .add_plugins(puzzle_screen::PuzzleScreenPlugin)
        .add_plugins(shop_screen::ShopScreenPlugin)
        .add_systems(
            OnEnter(AppState::InBattle),
            (
                setup_battle_runtime,
                battle_scene::setup_battle_scene,
                hud::setup_hud,
                planning::init_planning,
                planning::setup_planning_panel,
            ),
        )
        .add_systems(
            Update,
            (
                planning::handle_planning_clicks,
                animation::check_for_new_events,
                animation::play_event_queue,
                animation::revert_sprite_swaps,
                animation::animate_knockbacks,
                animation::animate_afterimages,
                animation::animate_projectiles,
                animation::animate_impacts,
                animation::animate_screen_shake,
                animation::animate_screen_flash,
                animation::animate_floating_text,
                battle_scene::sync_battle_scene_overlay,
                planning::update_planning_ui,
                transition_to_post_battle,
            )
                .run_if(in_state(AppState::InBattle)),
        )
        .add_systems(OnExit(AppState::InBattle), cleanup_battle_screen)
        .add_systems(OnEnter(AppState::PostBattle), resolve_post_battle)
        .add_systems(Update, ui_helpers::button_hover_system);
    }
}

pub(crate) fn build_battle_from_encounter(
    game_data: &GameData,
    save_data: &SaveData,
    encounter: &EncounterDef,
) -> Option<Battle> {
    let player_data: Vec<PlayerUnitData> = save_data
        .player_party
        .iter()
        .map(|saved_unit| build_saved_player_unit(saved_unit, game_data))
        .collect::<Option<_>>()?;

    let enemy_data: Vec<EnemyUnitData> = encounter
        .enemies
        .iter()
        .flat_map(|encounter_enemy| {
            let enemy_def = game_data.enemies.get(&encounter_enemy.enemy_id)?.clone();
            Some((0..encounter_enemy.count).map(move |_| EnemyUnitData {
                enemy_def: enemy_def.clone(),
            }))
        })
        .flatten()
        .collect();

    if player_data.is_empty() || enemy_data.is_empty() {
        return None;
    }

    let mut battle = battle_engine::new_battle(
        player_data,
        enemy_data,
        game_data.config.clone(),
        game_data.abilities.clone(),
        game_data.djinn.clone(),
    );
    battle_engine::set_team_djinn_slots(&mut battle, build_saved_team_djinn_slots(save_data));
    sync_saved_hp_to_battle(&mut battle, save_data);

    Some(battle)
}

fn build_saved_player_unit(saved_unit: &SavedUnit, game_data: &GameData) -> Option<PlayerUnitData> {
    let unit_def = game_data.units.get(&saved_unit.unit_id)?;
    let equipment = saved_equipment_loadout(&saved_unit.equipment);
    let equipment_effects = equipment::compute_equipment_effects(&equipment, &game_data.equipment);

    Some(PlayerUnitData {
        id: unit_def.id.0.clone(),
        base_stats: progression::calculate_stats_at_level(
            &unit_def.base_stats,
            &unit_def.growth_rates,
            saved_unit.level,
        ),
        equipment,
        djinn_slots: DjinnSlots::new(),
        mana_contribution: unit_def.mana_contribution.get(),
        equipment_effects,
    })
}

fn saved_equipment_loadout(saved_equipment: &SavedEquipment) -> EquipmentLoadout {
    EquipmentLoadout {
        weapon: saved_equipment.weapon.clone(),
        helm: saved_equipment.helm.clone(),
        armor: saved_equipment.armor.clone(),
        boots: saved_equipment.boots.clone(),
        accessory: saved_equipment.accessory.clone(),
    }
}

fn build_saved_team_djinn_slots(save_data: &SaveData) -> DjinnSlots {
    let mut team_djinn_slots = DjinnSlots::new();

    for (activation_order, saved_djinn) in save_data.team_djinn.iter().take(3).enumerate() {
        team_djinn_slots.slots.push(DjinnInstance {
            djinn_id: saved_djinn.djinn_id.clone(),
            state: saved_djinn.state,
            recovery_turns_remaining: if saved_djinn.state == DjinnState::Recovery {
                1
            } else {
                0
            },
            activation_order: activation_order as u32,
        });
    }

    team_djinn_slots.next_activation_order = team_djinn_slots.slots.len() as u32;
    team_djinn_slots
}

fn sync_saved_hp_to_battle(battle: &mut Battle, save_data: &SaveData) {
    for (battle_unit, saved_unit) in battle.player_units.iter_mut().zip(&save_data.player_party) {
        let max_hp = battle_unit.unit.stats.hp.get();
        let current_hp = saved_unit.current_hp.min(max_hp);
        battle_unit.unit.current_hp = current_hp;
        battle_unit.unit.is_alive = current_hp > 0;
    }
}

fn setup_battle_runtime(mut commands: Commands) {
    commands.insert_resource(animation::EventQueue::default());
    commands.insert_resource(animation::HitStop::default());
}

fn transition_to_post_battle(
    planning_state: Option<Res<planning::PlanningState>>,
    mut next_state: ResMut<NextState<AppState>>,
) {
    let Some(planning_state) = planning_state else {
        return;
    };

    if planning_state.mode == planning::PlanningMode::BattleOver {
        next_state.set(AppState::PostBattle);
    }
}

fn cleanup_battle_screen(
    mut commands: Commands,
    camera_q: Query<Entity, With<Camera2d>>,
    root_q: Query<
        Entity,
        Or<(
            With<battle_scene::BattleSceneOverlayRoot>,
            With<hud::HudRoot>,
            With<planning::PlanningPanel>,
        )>,
    >,
    sprite_q: Query<Entity, With<Sprite>>,
    text_q: Query<Entity, With<Text2d>>,
) {
    for entity in &camera_q {
        commands.entity(entity).despawn_recursive();
    }

    for entity in &root_q {
        commands.entity(entity).despawn_recursive();
    }

    for entity in &sprite_q {
        commands.entity(entity).despawn_recursive();
    }

    for entity in &text_q {
        commands.entity(entity).despawn_recursive();
    }
}

fn resolve_post_battle(
    mut commands: Commands,
    mut next_state: ResMut<NextState<AppState>>,
    mut game_state: ResMut<super::app_state::GameStateRes>,
    mut save_data: ResMut<super::app_state::SaveDataRes>,
    planning_state: Option<Res<planning::PlanningState>>,
) {
    let result = planning_state.and_then(|state| state.result.clone());

    match result {
        Some(BattleResult::Victory { xp, gold }) => {
            let total_xp = Xp::new(save_data.0.xp.saturating_add(xp));
            let total_gold = Gold::new(save_data.0.gold.saturating_add(gold));

            save_data.0.xp = total_xp.get();
            save_data.0.gold = total_gold.get();
            game_state.0.gold = total_gold;
            game_state.0.steps_since_encounter = 0;

            if let Some(encounter) = game_state.0.active_encounter.take() {
                if !save_data.0.completed_encounters.contains(&encounter.id) {
                    save_data.0.completed_encounters.push(encounter.id);
                }
            }

            next_state.set(AppState::WorldMap);
        }
        Some(BattleResult::Defeat) => {
            game_state.0.active_encounter = None;
            game_state.0.steps_since_encounter = 0;
            game_state.0.screen = GameScreen::Title;
            next_state.set(AppState::Title);
        }
        None => {
            game_state.0.active_encounter = None;
            next_state.set(AppState::WorldMap);
        }
    }

    commands.remove_resource::<BattleRes>();
    commands.remove_resource::<planning::PlanningState>();
    commands.remove_resource::<animation::EventQueue>();
    commands.remove_resource::<animation::HitStop>();
}
