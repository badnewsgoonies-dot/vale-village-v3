//! Battle scene — spawns camera and unit/enemy sprites from battle state.
//! Wave 2: data-driven with element colors and name labels.
#![allow(dead_code)]

use bevy::prelude::*;

use crate::shared::{Element, EnemyId, UnitId};

use super::plugin::{BattleRes, GameDataRes};

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

/// Marker for unit name label.
#[derive(Component)]
pub struct UnitLabel;

/// Map element to its display color.
/// Venus=#8B7355, Mars=#CC4444, Mercury=#4488CC, Jupiter=#CCCC44
fn element_color(element: Element) -> Color {
    match element {
        Element::Venus => Color::srgb(0x8B as f32 / 255.0, 0x73 as f32 / 255.0, 0x55 as f32 / 255.0),
        Element::Mars => Color::srgb(0xCC as f32 / 255.0, 0x44 as f32 / 255.0, 0x44 as f32 / 255.0),
        Element::Mercury => Color::srgb(0x44 as f32 / 255.0, 0x88 as f32 / 255.0, 0xCC as f32 / 255.0),
        Element::Jupiter => Color::srgb(0xCC as f32 / 255.0, 0xCC as f32 / 255.0, 0x44 as f32 / 255.0),
    }
}

/// Look up element for a unit by checking UnitDef then EnemyDef in game data.
fn lookup_element(id: &str, game_data: &GameDataRes) -> Element {
    if let Some(unit_def) = game_data.0.units.get(&UnitId(id.to_string())) {
        return unit_def.element;
    }
    if let Some(enemy_def) = game_data.0.enemies.get(&EnemyId(id.to_string())) {
        return enemy_def.element;
    }
    Element::Venus // fallback
}

/// Look up display name for a unit by checking UnitDef then EnemyDef.
fn lookup_name(id: &str, game_data: &GameDataRes) -> String {
    if let Some(unit_def) = game_data.0.units.get(&UnitId(id.to_string())) {
        return unit_def.name.clone();
    }
    if let Some(enemy_def) = game_data.0.enemies.get(&EnemyId(id.to_string())) {
        return enemy_def.name.clone();
    }
    id.to_string()
}

/// Spawn camera and battle scene from actual Battle state.
pub fn setup_battle_scene(
    mut commands: Commands,
    battle: Res<BattleRes>,
    game_data: Res<GameDataRes>,
) {
    let battle = &battle.0;

    // 2D camera
    commands.spawn(Camera2d);

    // Player units — element-colored, left side
    let player_count = battle.player_units.len();
    for (i, unit) in battle.player_units.iter().enumerate() {
        let y = (i as f32 - (player_count as f32 - 1.0) / 2.0) * 80.0;
        let element = lookup_element(&unit.unit.id, &game_data);
        let color = element_color(element);
        let name = lookup_name(&unit.unit.id, &game_data);

        // Unit sprite
        commands.spawn((
            Sprite::from_color(color, Vec2::new(48.0, 64.0)),
            Transform::from_xyz(-200.0, y, 0.0),
            PlayerUnit { index: i as u8 },
        ));

        // Name label below unit
        commands.spawn((
            Text2d::new(name),
            TextFont {
                font_size: 14.0,
                ..default()
            },
            TextColor(Color::WHITE),
            Transform::from_xyz(-200.0, y - 44.0, 1.0),
            UnitLabel,
        ));

        // HP text above unit
        let hp_text = format!("{}/{}", unit.unit.current_hp, unit.unit.stats.hp);
        commands.spawn((
            Text2d::new(hp_text),
            TextFont {
                font_size: 11.0,
                ..default()
            },
            TextColor(Color::srgb(0.27, 0.80, 0.27)),
            Transform::from_xyz(-200.0, y + 40.0, 1.0),
        ));
    }

    // Enemy units — element-colored, right side
    let enemy_count = battle.enemies.len();
    for (i, unit) in battle.enemies.iter().enumerate() {
        let y = (i as f32 - (enemy_count as f32 - 1.0) / 2.0) * 80.0;
        let element = lookup_element(&unit.unit.id, &game_data);
        let color = element_color(element);
        let name = lookup_name(&unit.unit.id, &game_data);

        // Unit sprite
        commands.spawn((
            Sprite::from_color(color, Vec2::new(48.0, 64.0)),
            Transform::from_xyz(200.0, y, 0.0),
            EnemyUnit { index: i as u8 },
        ));

        // Name label below enemy
        commands.spawn((
            Text2d::new(name),
            TextFont {
                font_size: 14.0,
                ..default()
            },
            TextColor(Color::WHITE),
            Transform::from_xyz(200.0, y - 44.0, 1.0),
            UnitLabel,
        ));
    }

    // Mana pool indicator (top center)
    let mana_text = format!(
        "Mana: {}/{}",
        battle.mana_pool.current_mana, battle.mana_pool.max_mana
    );
    commands.spawn((
        Text2d::new(mana_text),
        TextFont {
            font_size: 18.0,
            ..default()
        },
        TextColor(Color::srgb(0.27, 0.53, 0.80)),
        Transform::from_xyz(0.0, 320.0, 1.0),
    ));

    // Round indicator
    let round_text = format!("Round {}", battle.round);
    commands.spawn((
        Text2d::new(round_text),
        TextFont {
            font_size: 16.0,
            ..default()
        },
        TextColor(Color::srgb(0.6, 0.6, 0.6)),
        Transform::from_xyz(0.0, 295.0, 1.0),
    ));

    // Phase indicator
    let phase_text = format!("Phase: {:?}", battle.phase);
    commands.spawn((
        Text2d::new(phase_text),
        TextFont {
            font_size: 14.0,
            ..default()
        },
        TextColor(Color::srgb(0.5, 0.5, 0.5)),
        Transform::from_xyz(0.0, 275.0, 1.0),
    ));
}
