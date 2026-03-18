//! Battle scene — spawns camera and unit/enemy sprites from battle state.
//! Wave 2: data-driven with element colors and name labels.
#![allow(dead_code)]

use bevy::prelude::*;
use bevy::window::PrimaryWindow;

use crate::domains::battle_engine::Battle;
use crate::domains::data_loader::GameData;
use crate::domains::djinn::{self, DjinnInstance};
use crate::shared::{BattlePhase, DjinnState, Element, EnemyId, UnitId};

use super::planning;
use super::plugin::{BattleRes, GameDataRes};

const PLAYER_X: f32 = -200.0;
const ENEMY_X: f32 = 200.0;
const UNIT_SPACING_Y: f32 = 80.0;
const PLAYER_SPRITE_SIZE: Vec2 = Vec2::new(48.0, 64.0);
const MARKER_ROW_X_OFFSET: f32 = 42.0;
const MARKER_ROW_Y_OFFSET: f32 = 18.0;

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

/// Screen-space overlay root for player djinn markers and summon shortcuts.
#[derive(Component)]
pub struct BattleSceneOverlayRoot;

/// Marker row metadata for one player's djinn display.
#[derive(Component)]
pub struct PlayerDjinnMarkerRow {
    pub unit_index: usize,
}

/// One rendered djinn slot marker beside a player sprite.
#[derive(Component)]
pub struct PlayerDjinnMarker {
    pub unit_index: usize,
    pub slot_index: u8,
}

/// Map element to its display color.
/// Venus=#8B7355, Mars=#CC4444, Mercury=#4488CC, Jupiter=#CCCC44
fn element_color(element: Element) -> Color {
    match element {
        Element::Venus => Color::srgb(
            0x8B as f32 / 255.0,
            0x73 as f32 / 255.0,
            0x55 as f32 / 255.0,
        ),
        Element::Mars => Color::srgb(
            0xCC as f32 / 255.0,
            0x44 as f32 / 255.0,
            0x44 as f32 / 255.0,
        ),
        Element::Mercury => Color::srgb(
            0x44 as f32 / 255.0,
            0x88 as f32 / 255.0,
            0xCC as f32 / 255.0,
        ),
        Element::Jupiter => Color::srgb(
            0xCC as f32 / 255.0,
            0xCC as f32 / 255.0,
            0x44 as f32 / 255.0,
        ),
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

fn lookup_name_in_data(id: &str, game_data: &GameData) -> String {
    if let Some(unit_def) = game_data.units.get(&UnitId(id.to_string())) {
        return unit_def.name.clone();
    }
    if let Some(enemy_def) = game_data.enemies.get(&EnemyId(id.to_string())) {
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

    // Screen-space overlay for djinn markers and summon shortcuts.
    commands.spawn((
        Node {
            position_type: PositionType::Absolute,
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            ..default()
        },
        BattleSceneOverlayRoot,
    ));

    // Player units — element-colored, left side
    let player_count = battle.player_units.len();
    for (i, unit) in battle.player_units.iter().enumerate() {
        let y = unit_row_world_y(i, player_count);
        let element = lookup_element(&unit.unit.id, &game_data);
        let color = element_color(element);
        let name = lookup_name(&unit.unit.id, &game_data);

        // Unit sprite
        commands.spawn((
            Sprite::from_color(color, PLAYER_SPRITE_SIZE),
            Transform::from_xyz(PLAYER_X, y, 0.0),
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
            Transform::from_xyz(PLAYER_X, y - 44.0, 1.0),
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
            Transform::from_xyz(PLAYER_X, y + 40.0, 1.0),
        ));
    }

    // Enemy units — element-colored, right side
    let enemy_count = battle.enemies.len();
    for (i, unit) in battle.enemies.iter().enumerate() {
        let y = unit_row_world_y(i, enemy_count);
        let element = lookup_element(&unit.unit.id, &game_data);
        let color = element_color(element);
        let name = lookup_name(&unit.unit.id, &game_data);

        // Unit sprite
        commands.spawn((
            Sprite::from_color(color, PLAYER_SPRITE_SIZE),
            Transform::from_xyz(ENEMY_X, y, 0.0),
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
            Transform::from_xyz(ENEMY_X, y - 44.0, 1.0),
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

/// Keep the battle-scene djinn markers and summon controls in sync with planning state.
pub fn sync_battle_scene_overlay(
    mut commands: Commands,
    overlay_q: Query<(Entity, Option<&Children>), With<BattleSceneOverlayRoot>>,
    window_q: Query<&Window, With<PrimaryWindow>>,
    battle: Res<BattleRes>,
    game_data: Res<GameDataRes>,
    state: Res<planning::PlanningState>,
) {
    let Ok((overlay_root, children)) = overlay_q.get_single() else {
        return;
    };

    let overlay_already_built = children.is_some_and(|children| !children.is_empty());
    if !battle.is_changed() && !state.is_changed() && overlay_already_built {
        return;
    }

    let Ok(window) = window_q.get_single() else {
        return;
    };

    commands.entity(overlay_root).despawn_descendants();

    let battle = &battle.0;
    let highlighted_unit = highlighted_unit_index(battle, &state);
    let interactive_unit = interactive_unit_index(battle, &state);

    commands.entity(overlay_root).with_children(|overlay| {
        for unit_idx in 0..battle.player_units.len() {
            spawn_player_djinn_row(
                overlay,
                window,
                battle,
                &game_data.0,
                unit_idx,
                highlighted_unit == Some(unit_idx),
                interactive_unit == Some(unit_idx),
            );
        }
    });
}

fn spawn_player_djinn_row(
    parent: &mut ChildBuilder,
    window: &Window,
    battle: &Battle,
    game_data: &GameData,
    unit_idx: usize,
    is_highlighted: bool,
    is_interactive: bool,
) {
    let Some(unit) = battle.player_units.get(unit_idx) else {
        return;
    };
    if unit.djinn_slots.slots.is_empty() {
        return;
    }

    let unit_name = lookup_name_in_data(&unit.unit.id, game_data);
    let world_y = unit_row_world_y(unit_idx, battle.player_units.len());
    let left = window.width() * 0.5 + PLAYER_X + PLAYER_SPRITE_SIZE.x * 0.5 + MARKER_ROW_X_OFFSET;
    let top = window.height() * 0.5 - world_y - MARKER_ROW_Y_OFFSET;
    let summon_choices = if is_interactive {
        current_summon_choices(game_data, battle, unit_idx)
    } else {
        Vec::new()
    };

    parent
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                left: Val::Px(left),
                top: Val::Px(top),
                flex_direction: FlexDirection::Row,
                align_items: AlignItems::Center,
                column_gap: Val::Px(8.0),
                padding: UiRect::axes(Val::Px(10.0), Val::Px(6.0)),
                ..default()
            },
            BackgroundColor(if is_highlighted {
                Color::srgba(0.16, 0.18, 0.25, 0.92)
            } else {
                Color::srgba(0.08, 0.10, 0.16, 0.72)
            }),
            BorderRadius::all(Val::Px(8.0)),
            PlayerDjinnMarkerRow {
                unit_index: unit_idx,
            },
        ))
        .with_children(|row| {
            row.spawn((
                Text::new(unit_name),
                TextFont {
                    font_size: 12.0,
                    ..default()
                },
                TextColor(if is_highlighted {
                    Color::srgb(0.95, 0.88, 0.45)
                } else {
                    Color::srgb(0.7, 0.7, 0.78)
                }),
            ));

            if is_highlighted {
                row.spawn((
                    Node {
                        width: Val::Px(4.0),
                        height: Val::Px(24.0),
                        ..default()
                    },
                    BackgroundColor(Color::srgb(0.9, 0.78, 0.35)),
                    BorderRadius::all(Val::Px(999.0)),
                ));
            }

            for (slot_idx, inst) in unit.djinn_slots.slots.iter().enumerate() {
                let marker_label = djinn_marker_label(game_data, inst);
                if is_interactive && inst.state == DjinnState::Good {
                    spawn_scene_action_button(
                        row,
                        &marker_label,
                        planning::ActionChoice::ActivateDjinn(
                            slot_idx as u8,
                            djinn_display_name(game_data, inst),
                        ),
                        djinn_marker_background(inst.state, true),
                        Some(PlayerDjinnMarker {
                            unit_index: unit_idx,
                            slot_index: slot_idx as u8,
                        }),
                    );
                } else {
                    spawn_djinn_marker_tag(
                        row,
                        &marker_label,
                        unit_idx,
                        slot_idx as u8,
                        djinn_marker_background(inst.state, false),
                    );
                }
            }

            if !summon_choices.is_empty() {
                row.spawn((
                    Text::new("SUMMON"),
                    TextFont {
                        font_size: 11.0,
                        ..default()
                    },
                    TextColor(Color::srgb(0.85, 0.74, 0.45)),
                ));

                for (djinn_indices, tier) in summon_choices {
                    spawn_scene_action_button(
                        row,
                        &format!("T{}", tier),
                        planning::ActionChoice::Summon(djinn_indices, tier),
                        Color::srgb(0.64, 0.22, 0.42),
                        None,
                    );
                }
            }
        });
}

fn spawn_scene_action_button(
    parent: &mut ChildBuilder,
    label: &str,
    choice: planning::ActionChoice,
    color: Color,
    marker: Option<PlayerDjinnMarker>,
) {
    let mut entity = parent.spawn((
        Button,
        Node {
            padding: UiRect::axes(Val::Px(12.0), Val::Px(6.0)),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            ..default()
        },
        BackgroundColor(color),
        BorderRadius::all(Val::Px(999.0)),
        planning::ActionButton(choice),
    ));

    if let Some(marker) = marker {
        entity.insert(marker);
    }

    entity.with_children(|button| {
        button.spawn((
            Text::new(label.to_string()),
            TextFont {
                font_size: 12.0,
                ..default()
            },
            TextColor(Color::WHITE),
        ));
    });
}

fn spawn_djinn_marker_tag(
    parent: &mut ChildBuilder,
    label: &str,
    unit_index: usize,
    slot_index: u8,
    color: Color,
) {
    parent
        .spawn((
            Node {
                padding: UiRect::axes(Val::Px(12.0), Val::Px(6.0)),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            BackgroundColor(color),
            BorderRadius::all(Val::Px(999.0)),
            PlayerDjinnMarker {
                unit_index,
                slot_index,
            },
        ))
        .with_children(|marker| {
            marker.spawn((
                Text::new(label.to_string()),
                TextFont {
                    font_size: 12.0,
                    ..default()
                },
                TextColor(Color::WHITE),
            ));
        });
}

fn djinn_marker_background(state: DjinnState, is_interactive: bool) -> Color {
    match (state, is_interactive) {
        (DjinnState::Good, true) => Color::srgb(0.24, 0.56, 0.36),
        (DjinnState::Good, false) => Color::srgba(0.18, 0.32, 0.24, 0.88),
        (DjinnState::Recovery, _) => Color::srgba(0.30, 0.30, 0.36, 0.88),
    }
}

fn djinn_marker_label(game_data: &GameData, inst: &DjinnInstance) -> String {
    format!(
        "{} {}",
        djinn_display_name(game_data, inst),
        djinn_state_label(inst)
    )
}

fn djinn_display_name(game_data: &GameData, inst: &DjinnInstance) -> String {
    game_data
        .djinn
        .get(&inst.djinn_id)
        .map(|djinn| djinn.name.clone())
        .unwrap_or_else(|| inst.djinn_id.0.clone())
}

fn djinn_state_label(inst: &DjinnInstance) -> String {
    match inst.state {
        DjinnState::Good => "GOOD".to_string(),
        DjinnState::Recovery if inst.recovery_turns_remaining > 0 => {
            format!("REC:{}", inst.recovery_turns_remaining)
        }
        DjinnState::Recovery => "REC".to_string(),
    }
}

fn highlighted_unit_index(battle: &Battle, state: &planning::PlanningState) -> Option<usize> {
    if battle.phase != BattlePhase::Planning {
        return None;
    }

    match state.mode {
        planning::PlanningMode::SelectAction
        | planning::PlanningMode::SelectAbility
        | planning::PlanningMode::SelectTarget { .. } => battle
            .player_units
            .get(state.current_unit)
            .map(|_| state.current_unit),
        planning::PlanningMode::Executing
        | planning::PlanningMode::RoundComplete
        | planning::PlanningMode::BattleOver => None,
    }
}

fn interactive_unit_index(battle: &Battle, state: &planning::PlanningState) -> Option<usize> {
    if battle.phase != BattlePhase::Planning {
        return None;
    }

    if !matches!(state.mode, planning::PlanningMode::SelectAction) {
        return None;
    }

    battle
        .player_units
        .get(state.current_unit)
        .map(|_| state.current_unit)
}

fn unit_row_world_y(index: usize, total: usize) -> f32 {
    (index as f32 - (total as f32 - 1.0) / 2.0) * UNIT_SPACING_Y
}

fn current_good_djinn_choices(
    game_data: &GameData,
    battle: &Battle,
    unit_idx: usize,
) -> Vec<(u8, String)> {
    let Some(unit) = battle.player_units.get(unit_idx) else {
        return Vec::new();
    };

    unit.djinn_slots
        .slots
        .iter()
        .enumerate()
        .filter(|(_, inst)| inst.state == DjinnState::Good)
        .map(|(idx, inst)| {
            let name = game_data
                .djinn
                .get(&inst.djinn_id)
                .map(|djinn| djinn.name.clone())
                .unwrap_or_else(|| inst.djinn_id.0.clone());
            (idx as u8, name)
        })
        .collect()
}

fn current_summon_choices(
    game_data: &GameData,
    battle: &Battle,
    unit_idx: usize,
) -> Vec<(Vec<u8>, u8)> {
    if battle.player_units.get(unit_idx).is_none() {
        return Vec::new();
    }

    let good_slots = current_good_djinn_choices(game_data, battle, unit_idx);
    djinn::get_available_summons(good_slots.len())
        .into_iter()
        .map(|tier| {
            let indices = good_slots
                .iter()
                .take(tier.required_good as usize)
                .map(|(slot_idx, _)| *slot_idx)
                .collect::<Vec<_>>();
            (indices, tier.tier)
        })
        .collect()
}
