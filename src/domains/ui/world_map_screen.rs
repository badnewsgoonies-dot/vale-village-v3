//! World map screen UI.

use bevy::prelude::*;

use crate::game_state::GameState;
use crate::shared::{GameScreen, MapNode, MapNodeId, MapNodeType, NodeUnlockState};

use super::app_state::{AppState, CurrentTown, GameStateRes, SaveDataRes};
use super::ui_helpers::{
    despawn_screen, spawn_panel, ButtonAction, ButtonBaseColor, BORDER_COLOR, BG_COLOR,
    GOLD_COLOR, HIGHLIGHT, MenuButton, ScreenEntity, TEXT_COLOR, TEXT_DIM,
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
                        .spawn((
                            Node {
                                position_type: PositionType::Absolute,
                                left: Val::Px(left),
                                top: Val::Px(top),
                                width: Val::Px(140.0),
                                flex_direction: FlexDirection::Column,
                                align_items: AlignItems::Center,
                                row_gap: Val::Px(8.0),
                                ..default()
                            },
                        ))
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
                                            TextColor(if unlock_state == NodeUnlockState::Completed {
                                                BG_COLOR
                                            } else {
                                                TEXT_COLOR
                                            }),
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

        let Some(world_map) = game_state.0.world_map.as_ref() else {
            continue;
        };

        if !can_select_node(&game_state.0, world_map, node_id) {
            set_world_map_status(&mut status_q, "That route is still locked.");
            continue;
        }

        match crate::domains::world_map::get_node_type(world_map, node_id).copied() {
            Some(MapNodeType::Town(town_id)) => {
                game_state.0.screen = GameScreen::Town(town_id);
                commands.insert_resource(CurrentTown(town_id));
                next_state.set(AppState::Town);
            }
            Some(MapNodeType::Dungeon(_)) => {
                set_world_map_status(&mut status_q, "Dungeon travel is a placeholder for now.");
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

fn set_world_map_status(
    status_q: &mut Query<&mut Text, With<WorldMapStatusText>>,
    message: &str,
) {
    if let Ok(mut text) = status_q.get_single_mut() {
        text.0 = message.to_string();
    }
}
