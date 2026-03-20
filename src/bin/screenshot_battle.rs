//! Headless screenshot binary — launches the battle scene, captures after
//! a short frame delay, saves to `screenshot_battle.png`, then exits.

use bevy::app::AppExit;
use bevy::prelude::*;
use bevy::window::WindowResolution;

use std::path::Path;

use vale_village::domains::battle_engine::{self, EnemyUnitData, PlayerUnitData};
use vale_village::domains::data_loader;
use vale_village::domains::djinn::DjinnSlots;
use vale_village::domains::equipment::{self, EquipmentLoadout};
use vale_village::domains::sprite_loader::SpriteLoaderPlugin;
use vale_village::domains::ui::{animation, battle_scene, hud, planning, screenshot as ss};
use vale_village::domains::ui::plugin::{BattleRes, GameDataRes};
use vale_village::shared::{DjinnId, EncounterId, EquipmentId, EquipmentSlot, UnitDef, UnitId};

const OUTPUT_PATH: &str = "screenshot_battle.png";
const FRAMES_TO_WAIT: u32 = 8;

fn main() {
    let data_dir = Path::new("data/full");
    let game_data = data_loader::load_game_data(data_dir).expect("failed to load game data");

    let battle = build_demo_battle(&game_data, "house-01");

    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Vale Village — Screenshot".into(),
                resolution: WindowResolution::new(1280.0, 720.0),
                ..default()
            }),
            ..default()
        }))
        .add_plugins(SpriteLoaderPlugin)
        .add_plugins(ss::ScreenshotPlugin)
        .insert_resource(ClearColor(Color::srgb(
            0x1a as f32 / 255.0,
            0x1a as f32 / 255.0,
            0x2e as f32 / 255.0,
        )))
        .insert_resource(BattleRes(battle))
        .insert_resource(GameDataRes(game_data))
        .insert_resource(animation::EventQueue::default())
        .insert_resource(FrameCounter(0))
        .add_systems(
            Startup,
            (
                battle_scene::setup_battle_scene
                    .after(vale_village::domains::sprite_loader::load_sprites),
                hud::setup_hud,
                planning::init_planning,
                planning::setup_planning_panel,
            ),
        )
        .add_systems(Update, delayed_screenshot)
        .run();
}

#[derive(Resource)]
struct FrameCounter(u32);

fn delayed_screenshot(
    mut commands: Commands,
    mut counter: ResMut<FrameCounter>,
    request: Option<Res<ss::ScreenshotRequest>>,
    mut exit: EventWriter<AppExit>,
) {
    counter.0 += 1;

    // Wait for render stabilisation
    if counter.0 == FRAMES_TO_WAIT {
        commands.insert_resource(ss::ScreenshotRequest {
            output_path: OUTPUT_PATH.to_string(),
            frames_to_wait: 0,
        });
        return;
    }

    // Give the screenshot system a couple frames to fire, then exit
    if counter.0 > FRAMES_TO_WAIT + 3 {
        if request.is_none() {
            println!("Screenshot saved to {OUTPUT_PATH}");
            exit.send(AppExit::Success);
        }
    }

    // Safety valve — don't run forever
    if counter.0 > FRAMES_TO_WAIT + 30 {
        eprintln!("Screenshot timed out — exiting anyway");
        exit.send(AppExit::Success);
    }
}

fn build_player_unit(
    unit_def: &UnitDef,
    weapon_id: &str,
    armor_id: &str,
    game_data: &data_loader::GameData,
) -> PlayerUnitData {
    let mut loadout = EquipmentLoadout::default();

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
        mana_contribution: unit_def.mana_contribution.get(),
        equipment_effects: eq_effects,
    }
}

fn build_demo_battle(
    game_data: &data_loader::GameData,
    encounter_id: &str,
) -> vale_village::domains::battle_engine::Battle {
    let adept = game_data
        .units
        .get(&UnitId("adept".to_string()))
        .expect("unit 'adept' not found");
    let blaze = game_data
        .units
        .get(&UnitId("blaze".to_string()))
        .expect("unit 'blaze' not found");

    let adept_data = build_player_unit(adept, "bronze-sword", "leather-vest", game_data);
    let blaze_data = build_player_unit(blaze, "wooden-axe", "leather-vest", game_data);

    let mut team_djinn = DjinnSlots::new();
    for id in &["flint", "forge"] {
        let djinn_id = DjinnId(id.to_string());
        if game_data.djinn.contains_key(&djinn_id) {
            team_djinn.add(djinn_id);
        }
    }

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
    battle_engine::set_team_djinn_slots(&mut battle, team_djinn);
    battle
}
