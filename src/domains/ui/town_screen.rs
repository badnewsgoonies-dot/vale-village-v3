//! Town screen UI.

use bevy::prelude::*;

use crate::domains::save::SavedDjinn;
use crate::domains::town;
use crate::shared::DjinnState;
use crate::starter_data::starter_towns;

use super::app_state::{AppState, CurrentShop, CurrentTown, GameStateRes, SaveDataRes};
use super::ui_helpers::{
    despawn_screen, spawn_button, spawn_panel, ButtonAction, BG_COLOR, GOLD_COLOR, ScreenEntity,
    TEXT_COLOR, TEXT_DIM, MenuButton,
};

#[derive(Component)]
struct TownStatusText;

pub struct TownScreenPlugin;

impl Plugin for TownScreenPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(AppState::Town), setup_town_screen)
            .add_systems(
                Update,
                handle_town_buttons.run_if(in_state(AppState::Town)),
            )
            .add_systems(OnExit(AppState::Town), (despawn_screen, cleanup_town_state));
    }
}

fn setup_town_screen(
    mut commands: Commands,
    current_town: Res<CurrentTown>,
    game_state: Res<GameStateRes>,
    save_data: Res<SaveDataRes>,
) {
    commands.spawn((Camera2d, ScreenEntity));

    let towns = starter_towns();
    let Some(town_def) = towns.into_iter().find(|town| town.id == current_town.0) else {
        return;
    };

    let town_state = town::load_town(&town_def);
    let available_djinn = town_def
        .djinn_points
        .iter()
        .find_map(|point| {
            town::check_djinn_discovery(
                &town_def,
                &town_state,
                point.position,
                &game_state.0.quest_state,
            )
        })
        .filter(|djinn_id| {
            !save_data
                .0
                .team_djinn
                .iter()
                .any(|saved| saved.djinn_id == *djinn_id)
        });
    let shop_id = town::get_shops(&town_def).first().copied();

    commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            BackgroundColor(BG_COLOR),
            ScreenEntity,
        ))
        .with_children(|root| {
            spawn_panel(
                root,
                Node {
                    width: Val::Px(560.0),
                    flex_direction: FlexDirection::Column,
                    row_gap: Val::Px(12.0),
                    ..default()
                },
                |panel| {
                    panel.spawn((
                        Text::new(town_def.name.clone()),
                        TextFont {
                            font_size: 42.0,
                            ..default()
                        },
                        TextColor(TEXT_COLOR),
                    ));
                    panel.spawn((
                        Text::new(format!("Gold: {}", save_data.0.gold)),
                        TextFont {
                            font_size: 22.0,
                            ..default()
                        },
                        TextColor(GOLD_COLOR),
                    ));
                    panel.spawn((
                        Text::new("Talk to townsfolk, visit the shop, or head back to the map."),
                        TextFont {
                            font_size: 18.0,
                            ..default()
                        },
                        TextColor(TEXT_DIM),
                    ));

                    for npc in town::get_npcs(&town_def) {
                        spawn_button(
                            panel,
                            format!("Talk to NPC {}", npc.npc_id.0),
                            ButtonAction::TalkToNpc(npc.npc_id),
                            22.0,
                        );
                    }

                    if let Some(shop_id) = shop_id {
                        spawn_button(panel, "Visit Shop", ButtonAction::EnterShop(shop_id), 24.0);
                    }

                    if let Some(djinn_id) = available_djinn {
                        spawn_button(
                            panel,
                            format!("Collect {}", djinn_id.0),
                            ButtonAction::CollectDjinn(djinn_id),
                            22.0,
                        );
                    }

                    spawn_button(panel, "Leave Town", ButtonAction::LeaveToMap, 24.0);
                    panel.spawn((
                        Text::new(""),
                        TextFont {
                            font_size: 18.0,
                            ..default()
                        },
                        TextColor(TEXT_DIM),
                        TownStatusText,
                    ));
                },
            );
        });
}

fn handle_town_buttons(
    mut commands: Commands,
    mut next_state: ResMut<NextState<AppState>>,
    mut save_data: ResMut<SaveDataRes>,
    button_q: Query<(&Interaction, &MenuButton), Changed<Interaction>>,
    mut status_q: Query<&mut Text, With<TownStatusText>>,
) {
    for (interaction, button) in &button_q {
        if *interaction != Interaction::Pressed {
            continue;
        }

        match &button.action {
            ButtonAction::TalkToNpc(npc_id) => {
                set_town_status(&mut status_q, &format!("NPC {} says hello.", npc_id.0));
            }
            ButtonAction::EnterShop(shop_id) => {
                commands.insert_resource(CurrentShop(*shop_id));
                next_state.set(AppState::Shop);
            }
            ButtonAction::CollectDjinn(djinn_id) => {
                if save_data
                    .0
                    .team_djinn
                    .iter()
                    .any(|saved| saved.djinn_id == *djinn_id)
                {
                    set_town_status(&mut status_q, "That djinn is already with the party.");
                } else {
                    save_data.0.team_djinn.push(SavedDjinn {
                        djinn_id: djinn_id.clone(),
                        state: DjinnState::Good,
                    });
                    set_town_status(&mut status_q, &format!("{} joined the party.", djinn_id.0));
                }
            }
            ButtonAction::LeaveToMap => {
                next_state.set(AppState::WorldMap);
            }
            _ => {}
        }
    }
}

fn cleanup_town_state(mut commands: Commands) {
    commands.remove_resource::<CurrentTown>();
}

fn set_town_status(status_q: &mut Query<&mut Text, With<TownStatusText>>, message: &str) {
    if let Ok(mut text) = status_q.get_single_mut() {
        text.0 = message.to_string();
    }
}
