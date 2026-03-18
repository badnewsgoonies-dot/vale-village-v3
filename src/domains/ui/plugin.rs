//! Bevy plugin wiring — window, camera, and system registration.

use std::path::Path;

use bevy::prelude::*;
use bevy::window::WindowResolution;

use crate::domains::battle_engine::{self, Battle, EnemyUnitData, PlayerUnitData};
use crate::domains::data_loader::{self, GameData};
use crate::domains::djinn::DjinnSlots;
use crate::domains::equipment::{self, EquipmentLoadout};
use crate::shared::{DjinnId, EncounterId, EquipmentId, EquipmentSlot, UnitDef, UnitId};

use super::animation;
use super::battle_scene;
use super::hud;
use super::planning;

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
        // Load game data before Bevy starts
        let data_dir = Path::new("data/full");
        let game_data = match data_loader::load_game_data(data_dir) {
            Ok(data) => data,
            Err(errors) => {
                for e in &errors {
                    eprintln!("Load error: {e:?}");
                }
                std::process::exit(1);
            }
        };

        // Build a demo battle from house-01
        let battle = build_demo_battle(&game_data, "house-01");

        app.add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Vale Village v3".into(),
                resolution: WindowResolution::new(1280.0, 720.0),
                ..default()
            }),
            ..default()
        }))
        .insert_resource(ClearColor(Color::srgb(
            0x1a as f32 / 255.0,
            0x1a as f32 / 255.0,
            0x2e as f32 / 255.0,
        )))
        .insert_resource(BattleRes(battle))
        .insert_resource(GameDataRes(game_data))
        .insert_resource(animation::EventQueue::default())
        .add_systems(
            Startup,
            (
                battle_scene::setup_battle_scene,
                hud::setup_hud,
                planning::init_planning,
                planning::setup_planning_panel,
            ),
        )
        .add_systems(
            Update,
            (
                planning::update_planning_ui,
                planning::handle_planning_clicks,
                battle_scene::sync_battle_scene_overlay,
                animation::check_for_new_events,
                animation::play_event_queue,
                animation::animate_floating_text,
            ),
        );
    }
}

/// Build a PlayerUnitData with optional equipment.
fn build_player_unit(
    unit_def: &UnitDef,
    weapon_id: &str,
    armor_id: &str,
    game_data: &GameData,
) -> PlayerUnitData {
    let mut loadout = EquipmentLoadout::default();

    // Try to equip weapon
    let weapon_eid = EquipmentId(weapon_id.to_string());
    if let Some(weapon_def) = game_data.equipment.get(&weapon_eid) {
        let _ = equipment::equip(
            &mut loadout,
            EquipmentSlot::Weapon,
            weapon_eid,
            weapon_def,
            unit_def.element,
        );
    }

    // Try to equip armor
    let armor_eid = EquipmentId(armor_id.to_string());
    if let Some(armor_def) = game_data.equipment.get(&armor_eid) {
        let _ = equipment::equip(
            &mut loadout,
            EquipmentSlot::Armor,
            armor_eid,
            armor_def,
            unit_def.element,
        );
    }

    let eq_effects = equipment::compute_equipment_effects(&loadout, &game_data.equipment);

    PlayerUnitData {
        id: unit_def.id.0.clone(),
        base_stats: unit_def.base_stats,
        equipment: loadout,
        djinn_slots: DjinnSlots::new(),
        mana_contribution: unit_def.mana_contribution,
        equipment_effects: eq_effects,
    }
}

fn build_team_djinn_slots(ids: &[&str], game_data: &GameData) -> DjinnSlots {
    let mut team_slots = DjinnSlots::new();
    for &djinn_id_str in ids {
        let djinn_id = DjinnId(djinn_id_str.to_string());
        if game_data.djinn.contains_key(&djinn_id) {
            team_slots.add(djinn_id);
        }
    }
    team_slots
}

/// Create a Battle from game data for the given encounter.
fn build_demo_battle(game_data: &GameData, encounter_id: &str) -> Battle {
    let adept = game_data
        .units
        .get(&UnitId("adept".to_string()))
        .expect("game data must contain unit 'adept'");
    let blaze = game_data
        .units
        .get(&UnitId("blaze".to_string()))
        .expect("game data must contain unit 'blaze'");

    let adept_data = build_player_unit(adept, "bronze-sword", "leather-vest", game_data);
    let blaze_data = build_player_unit(blaze, "wooden-axe", "leather-vest", game_data);
    let team_djinn_slots = build_team_djinn_slots(&["flint", "forge"], game_data);

    let enc_id = EncounterId(encounter_id.to_string());
    let encounter = game_data
        .encounters
        .get(&enc_id)
        .unwrap_or_else(|| panic!("encounter '{}' not found", encounter_id));

    let enemy_data: Vec<EnemyUnitData> = encounter
        .enemies
        .iter()
        .flat_map(|ee| {
            let enemy_def = game_data
                .enemies
                .get(&ee.enemy_id)
                .unwrap_or_else(|| panic!("enemy '{}' not found", ee.enemy_id.0));
            (0..ee.count).map(move |_| EnemyUnitData {
                enemy_def: enemy_def.clone(),
            })
        })
        .collect();

    let mut battle = battle_engine::new_battle(
        vec![adept_data, blaze_data],
        enemy_data,
        game_data.config.clone(),
        game_data.abilities.clone(),
        game_data.djinn.clone(),
    );
    battle_engine::set_team_djinn_slots(&mut battle, team_djinn_slots);
    battle
}
