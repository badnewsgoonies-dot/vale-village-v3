//! Battle scene — spawns camera and placeholder unit/enemy sprites.
//! Wave 1: colored rectangles only. Sprites loaded in Wave 2.
#![allow(dead_code)]

use bevy::prelude::*;

/// Marker for player unit entities.
#[derive(Component)]
pub struct PlayerUnit {
    pub index: u8,
}

/// Marker for enemy unit entities.
#[derive(Component)]
pub struct EnemyUnit {
    pub index: u8,
}

/// Spawn camera and placeholder battle scene.
/// Player units: blue rectangles on the left.
/// Enemy units: red rectangles on the right.
pub fn setup_battle_scene(mut commands: Commands) {
    // 2D camera
    commands.spawn(Camera2d);

    // Player units — blue, left side
    let player_count: u8 = 4;
    let player_color = Color::srgb(0.27, 0.53, 0.80); // #4488cc
    for i in 0..player_count {
        let y = (i as f32 - (player_count as f32 - 1.0) / 2.0) * 80.0;
        commands.spawn((
            Sprite::from_color(player_color, Vec2::new(48.0, 64.0)),
            Transform::from_xyz(-200.0, y, 0.0),
            PlayerUnit { index: i },
        ));
    }

    // Enemy units — red, right side
    let enemy_count: u8 = 3;
    let enemy_color = Color::srgb(0.80, 0.27, 0.27); // #cc4444
    for i in 0..enemy_count {
        let y = (i as f32 - (enemy_count as f32 - 1.0) / 2.0) * 80.0;
        commands.spawn((
            Sprite::from_color(enemy_color, Vec2::new(48.0, 64.0)),
            Transform::from_xyz(200.0, y, 0.0),
            EnemyUnit { index: i },
        ));
    }
}
