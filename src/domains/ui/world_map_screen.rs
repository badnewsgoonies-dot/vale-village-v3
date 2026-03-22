//! World map screen UI.

use bevy::prelude::*;

use crate::game_state::GameState;
use crate::shared::{GameScreen, MapNode, MapNodeId, MapNodeType, NodeUnlockState};

use super::app_state::{AppState, CurrentDungeon, CurrentTown, GameStateRes, SaveDataRes};
use super::plugin::{build_battle_from_encounter, BattleRes, GameDataRes};
use super::ui_helpers::{
    despawn_screen, spawn_panel, ButtonAction, ButtonBaseColor, MenuButton, ScreenEntity, BG_COLOR,
    BORDER_COLOR, GOLD_COLOR, HIGHLIGHT, TEXT_COLOR, TEXT_DIM,
};

#[derive(Component)]
struct WorldMapStatusText;

pub struct WorldMapPlugin;

impl Plugin for WorldMapPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(AppState::WorldMap), setup_world_map)
            .add_systems(
                Update,
                handle_world_map_buttons.run_if(in_state(AppState::WorldMap)),
            )
            .add_systems(OnExit(AppState::WorldMap), despawn_screen);
    }
}

fn setup_world_map(
    mut commands: Commands,
    game_state: Res<GameStateRes>,
    save_data: Res<SaveDataRes>,
) {
    commands.spawn((Camera2d, ScreenEntity));

    let Some(world_map) = game_state.0.world_map.as_ref() else {
        return;
    };

    commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                ..default()
            },
            BackgroundColor(BG_COLOR),
            ScreenEntity,
        ))
        .with_children(|root| {
            root.spawn((
                Node {
                    width: Val::Percent(100.0),
                    flex_grow: 1.0,
                    position_type: PositionType::Relative,
                    ..default()
                },
                BackgroundColor(BG_COLOR),
            ))
            .with_children(|map_layer| {
                for node in &world_map.nodes {
                    let unlock_state = world_map
                        .unlock_states
                        .get(&node.id)
                        .copied()
                        .unwrap_or(NodeUnlockState::Locked);

                    if unlock_state == NodeUnlockState::Locked {
                        continue;
                    }

                    let (left, top) = node_screen_position(node);
                    let (fill_color, border_color) = node_colors(unlock_state);
                    let is_interactive = matches!(
                        unlock_state,
                        NodeUnlockState::Unlocked | NodeUnlockState::Completed
                    );

                    map_layer
                        .spawn((Node {
                            position_type: PositionType::Absolute,
                            left: Val::Px(left),
                            top: Val::Px(top),
                            width: Val::Px(140.0),
                            flex_direction: FlexDirection::Column,
                            align_items: AlignItems::Center,
                            row_gap: Val::Px(8.0),
                            ..default()
                        },))
                        .with_children(|node_parent| {
                            if is_interactive {
                                node_parent
                                    .spawn((
                                        Button,
                                        Node {
                                            width: Val::Px(36.0),
                                            height: Val::Px(36.0),
                                            justify_content: JustifyContent::Center,
                                            align_items: AlignItems::Center,
                                            border: UiRect::all(Val::Px(3.0)),
                                            ..default()
                                        },
                                        BackgroundColor(fill_color),
                                        BorderColor(border_color),
                                        BorderRadius::all(Val::Px(999.0)),
                                        MenuButton {
                                            action: ButtonAction::TravelTo(node.id),
                                        },
                                        ButtonBaseColor(fill_color),
                                    ))
                                    .with_children(|button| {
                                        button.spawn((
                                            Text::new(node_marker_text(unlock_state)),
                                            TextFont {
                                                font_size: 14.0,
                                                ..default()
                                            },
                                            TextColor(
                                                if unlock_state == NodeUnlockState::Completed {
                                                    BG_COLOR
                                                } else {
                                                    TEXT_COLOR
                                                },
                                            ),
                                        ));
                                    });
                            } else {
                                node_parent.spawn((
                                    Node {
                                        width: Val::Px(36.0),
                                        height: Val::Px(36.0),
                                        border: UiRect::all(Val::Px(2.0)),
                                        ..default()
                                    },
                                    BackgroundColor(fill_color),
                                    BorderColor(border_color),
                                    BorderRadius::all(Val::Px(999.0)),
                                ));
                            }

                            node_parent.spawn((
                                Text::new(node.name.clone()),
                                TextFont {
                                    font_size: 18.0,
                                    ..default()
                                },
                                TextColor(TEXT_COLOR),
                            ));
                        });
                }
            });

            spawn_panel(
                root,
                Node {
                    width: Val::Percent(100.0),
                    min_height: Val::Px(96.0),
                    justify_content: JustifyContent::SpaceBetween,
                    align_items: AlignItems::Center,
                    flex_direction: FlexDirection::Row,
                    margin: UiRect::top(Val::Px(8.0)),
                    ..default()
                },
                |bar| {
                    bar.spawn((
                        Text::new(format!("Gold: {}", save_data.0.gold)),
                        TextFont {
                            font_size: 28.0,
                            ..default()
                        },
                        TextColor(GOLD_COLOR),
                    ));
                    bar.spawn((
                        Text::new("Choose a destination."),
                        TextFont {
                            font_size: 18.0,
                            ..default()
                        },
                        TextColor(TEXT_DIM),
                        WorldMapStatusText,
                    ));
                },
            );
        });
}

fn handle_world_map_buttons(
    mut commands: Commands,
    mut next_state: ResMut<NextState<AppState>>,
    mut game_state: ResMut<GameStateRes>,
    save_data: Res<SaveDataRes>,
    game_data: Res<GameDataRes>,
    button_q: Query<(&Interaction, &MenuButton), Changed<Interaction>>,
    mut status_q: Query<&mut Text, With<WorldMapStatusText>>,
) {
    for (interaction, button) in &button_q {
        if *interaction != Interaction::Pressed {
            continue;
        }

        let ButtonAction::TravelTo(node_id) = button.action.clone() else {
            continue;
        };

        let (is_travel_step, destination) = {
            let Some(world_map) = game_state.0.world_map.as_ref() else {
                continue;
            };

            if !can_select_node(&game_state.0, world_map, node_id) {
                set_world_map_status(&mut status_q, "That route is still locked.");
                continue;
            }

            (
                current_map_anchor(&game_state.0, world_map) != Some(node_id),
                crate::domains::world_map::get_node_type(world_map, node_id).copied(),
            )
        };

        if is_travel_step {
            game_state.0.steps_since_encounter =
                game_state.0.steps_since_encounter.saturating_add(1);

            let encounter_table = crate::starter_data::overworld_encounter_table();
            let encounter_chance = encounter_table.base_rate.get();

            if encounter_roll_succeeds(encounter_chance, game_state.0.steps_since_encounter) {
                if let Some(encounter) = crate::domains::encounter::select_encounter(
                    &encounter_table,
                    game_state.0.steps_since_encounter,
                )
                .cloned()
                {
                    let Some(battle) =
                        build_battle_from_encounter(&game_data.0, &save_data.0, &encounter)
                    else {
                        set_world_map_status(
                            &mut status_q,
                            "Encounter setup failed. Destination unchanged.",
                        );
                        continue;
                    };

                    game_state.0.steps_since_encounter = 0;
                    game_state.0.active_encounter = Some(encounter);
                    commands.insert_resource(BattleRes(battle));
                    next_state.set(AppState::InBattle);
                    continue;
                }
            }
        }

        match destination {
            Some(MapNodeType::Town(town_id)) => {
                game_state.0.screen = GameScreen::Town(town_id);
                commands.insert_resource(CurrentTown(town_id));
                next_state.set(AppState::Town);
            }
            Some(MapNodeType::Dungeon(dungeon_id)) => {
                game_state.0.screen = GameScreen::Dungeon(dungeon_id);
                commands.insert_resource(CurrentDungeon(dungeon_id));
                next_state.set(AppState::Dungeon);
            }
            Some(MapNodeType::Landmark) => {
                set_world_map_status(&mut status_q, "Landmarks are not enterable yet.");
            }
            Some(MapNodeType::Hidden) | None => {
                set_world_map_status(&mut status_q, "This destination is unavailable.");
            }
        }
    }
}

fn node_screen_position(node: &MapNode) -> (f32, f32) {
    if node.position.0 <= 12.0 && node.position.1 <= 8.0 {
        (
            node.position.0 * 100.0 + 640.0,
            node.position.1 * 100.0 + 360.0,
        )
    } else {
        (node.position.0, node.position.1)
    }
}

fn node_colors(unlock_state: NodeUnlockState) -> (Color, Color) {
    match unlock_state {
        NodeUnlockState::Locked => (Color::NONE, Color::NONE),
        NodeUnlockState::Visible => (TEXT_DIM, TEXT_DIM),
        NodeUnlockState::Unlocked => (BG_COLOR, BORDER_COLOR),
        NodeUnlockState::Completed => (Color::srgb(0.2, 0.65, 0.35), HIGHLIGHT),
    }
}

fn node_marker_text(unlock_state: NodeUnlockState) -> &'static str {
    match unlock_state {
        NodeUnlockState::Completed => "X",
        _ => "•",
    }
}

fn can_select_node(
    game_state: &GameState,
    world_map: &crate::domains::world_map::WorldMap,
    target: MapNodeId,
) -> bool {
    if let Some(from_node) = current_map_anchor(game_state, world_map) {
        from_node == target || crate::domains::world_map::can_travel(world_map, from_node, target)
    } else {
        crate::domains::world_map::get_accessible_nodes(world_map)
            .into_iter()
            .any(|node| node.id == target)
    }
}

fn current_map_anchor(
    game_state: &GameState,
    world_map: &crate::domains::world_map::WorldMap,
) -> Option<MapNodeId> {
    match game_state.screen {
        GameScreen::Town(town_id) => world_map
            .nodes
            .iter()
            .find(|node| matches!(node.node_type, MapNodeType::Town(id) if id == town_id))
            .map(|node| node.id),
        _ => None,
    }
}

fn set_world_map_status(status_q: &mut Query<&mut Text, With<WorldMapStatusText>>, message: &str) {
    if let Ok(mut text) = status_q.get_single_mut() {
        text.0 = message.to_string();
    }
}

fn encounter_roll_succeeds(chance_percent: u8, steps_since_encounter: u16) -> bool {
    let threshold = u32::from(chance_percent);

    getrandom::u32()
        .map(|roll| roll % 100 < threshold)
        .unwrap_or_else(|_| {
            (u32::from(steps_since_encounter)
                .wrapping_mul(73)
                .wrapping_add(19)
                % 100)
                < threshold
        })
}
