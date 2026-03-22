//! Title screen UI.

use bevy::prelude::*;

use crate::domains::save;
use crate::domains::world_map;
use crate::game_state::GameState;
use crate::shared::{bounded_types::Gold, GameScreen};
use crate::starter_data::starter_map_nodes;

use super::app_state::{AppState, GameStateRes, SaveDataRes};
use super::ui_helpers::{
    despawn_screen, spawn_button, spawn_panel, ButtonAction, MenuButton, ScreenEntity, BG_COLOR,
    BORDER_COLOR,
};

pub struct TitleScreenPlugin;

impl Plugin for TitleScreenPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(AppState::Title), setup_title_screen)
            .add_systems(
                Update,
                handle_title_buttons.run_if(in_state(AppState::Title)),
            )
            .add_systems(OnExit(AppState::Title), despawn_screen);
    }
}

fn setup_title_screen(mut commands: Commands) {
    commands.spawn((Camera2d, ScreenEntity));

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
                    width: Val::Px(460.0),
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::Center,
                    row_gap: Val::Px(18.0),
                    ..default()
                },
                |panel| {
                    panel.spawn((
                        Text::new("VALE VILLAGE"),
                        TextFont {
                            font_size: 64.0,
                            ..default()
                        },
                        TextColor(BORDER_COLOR),
                        Node {
                            margin: UiRect::bottom(Val::Px(24.0)),
                            ..default()
                        },
                    ));
                    spawn_button(panel, "New Game", ButtonAction::NewGame, 28.0);
                    spawn_button(panel, "Continue", ButtonAction::Continue, 28.0);
                },
            );
        });
}

fn handle_title_buttons(
    mut commands: Commands,
    mut next_state: ResMut<NextState<AppState>>,
    button_q: Query<(&Interaction, &MenuButton), Changed<Interaction>>,
) {
    for (interaction, button) in &button_q {
        if *interaction != Interaction::Pressed {
            continue;
        }

        match &button.action {
            ButtonAction::NewGame | ButtonAction::Continue => {
                let save_data = save::create_new_game();
                let mut game_state = GameState::new_game();
                game_state.world_map = Some(world_map::load_map(starter_map_nodes()));
                game_state.screen = GameScreen::WorldMap;
                game_state.gold = Gold::new(save_data.gold);
                commands.insert_resource(GameStateRes(game_state));
                commands.insert_resource(SaveDataRes(save_data));
                next_state.set(AppState::WorldMap);
            }
            _ => {}
        }
    }
}
