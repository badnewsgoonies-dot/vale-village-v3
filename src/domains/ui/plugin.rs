//! Bevy plugin wiring — window, camera, and system registration.

use std::path::Path;

use bevy::prelude::*;
use bevy::window::WindowResolution;

use crate::domains::battle_engine::Battle;
use crate::domains::data_loader::{self, GameData};
use crate::domains::sprite_loader::SpriteLoaderPlugin;

use super::app_state::AppState;
use super::screenshot;
use super::shop_screen;
use super::town_screen;
use super::title_screen;
use super::ui_helpers;
use super::world_map_screen;

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

        app.add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Vale Village v3".into(),
                resolution: WindowResolution::new(1280.0, 720.0),
                ..default()
            }),
            ..default()
        }))
        .add_plugins(SpriteLoaderPlugin)
        .add_plugins(screenshot::ScreenshotPlugin)
        .insert_resource(ClearColor(ui_helpers::BG_COLOR))
        .insert_resource(GameDataRes(game_data))
        .init_state::<AppState>()
        .add_plugins(title_screen::TitleScreenPlugin)
        .add_plugins(world_map_screen::WorldMapPlugin)
        .add_plugins(town_screen::TownScreenPlugin)
        .add_plugins(shop_screen::ShopScreenPlugin)
        .add_systems(Update, ui_helpers::button_hover_system);
    }
}
