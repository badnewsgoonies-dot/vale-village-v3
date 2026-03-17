//! Bevy plugin wiring — window, camera, and system registration.

use bevy::prelude::*;
use bevy::window::WindowResolution;

use super::battle_scene;

/// Top-level plugin for the Vale Village visual layer.
pub struct ValeVillagePlugin;

impl Plugin for ValeVillagePlugin {
    fn build(&self, app: &mut App) {
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
        .add_systems(Startup, battle_scene::setup_battle_scene);
    }
}
