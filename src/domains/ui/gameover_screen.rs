//! Game over and victory screens.

use bevy::prelude::*;

use crate::shared::GameScreen;

use super::app_state::{AppState, GameStateRes};
use super::ui_helpers::{
    despawn_screen, ButtonBaseColor, ScreenEntity, BG_COLOR, BORDER_COLOR, GOLD_COLOR, HIGHLIGHT,
    PANEL_BG, TEXT_COLOR, TEXT_DIM,
};

const BUTTON_BG: Color = PANEL_BG;
const BUTTON_PRESSED: Color = GOLD_COLOR;

#[derive(Component)]
struct ReturnToTitleButton;

pub struct GameOverPlugin;

impl Plugin for GameOverPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(AppState::GameOver), setup_game_over_screen)
            .add_systems(OnEnter(AppState::Victory), setup_victory_screen)
            .add_systems(
                Update,
                (handle_result_screen_buttons, result_button_hover_system)
                    .run_if(in_state(AppState::GameOver)),
            )
            .add_systems(
                Update,
                (handle_result_screen_buttons, result_button_hover_system)
                    .run_if(in_state(AppState::Victory)),
            )
            .add_systems(OnExit(AppState::GameOver), despawn_screen)
            .add_systems(OnExit(AppState::Victory), despawn_screen);
    }
}

fn setup_game_over_screen(mut commands: Commands) {
    spawn_result_screen(
        &mut commands,
        "DEFEAT",
        "The party was overwhelmed. Return to title to begin again.",
    );
}

fn setup_victory_screen(mut commands: Commands) {
    spawn_result_screen(
        &mut commands,
        "VICTORY - All encounters completed!",
        "The campaign route is clear. Return to title when ready.",
    );
}

fn spawn_result_screen(commands: &mut Commands, title: &str, subtitle: &str) {
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
            root.spawn((
                Node {
                    width: Val::Px(720.0),
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::Center,
                    row_gap: Val::Px(18.0),
                    border: UiRect::all(Val::Px(2.0)),
                    padding: UiRect::all(Val::Px(24.0)),
                    ..default()
                },
                BackgroundColor(PANEL_BG),
                BorderColor(BORDER_COLOR),
            ))
            .with_children(|panel| {
                panel.spawn((
                    Text::new(title),
                    TextFont {
                        font_size: 58.0,
                        ..default()
                    },
                    TextColor(BORDER_COLOR),
                ));
                panel.spawn((
                    Text::new(subtitle),
                    TextFont {
                        font_size: 20.0,
                        ..default()
                    },
                    TextColor(TEXT_DIM),
                ));
                panel
                    .spawn((
                        Button,
                        Node {
                            width: Val::Px(280.0),
                            min_height: Val::Px(56.0),
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            padding: UiRect::axes(Val::Px(18.0), Val::Px(12.0)),
                            border: UiRect::all(Val::Px(2.0)),
                            margin: UiRect::top(Val::Px(12.0)),
                            ..default()
                        },
                        BackgroundColor(BUTTON_BG),
                        BorderColor(BORDER_COLOR),
                        BorderRadius::all(Val::Px(6.0)),
                        ButtonBaseColor(BUTTON_BG),
                        ReturnToTitleButton,
                    ))
                    .with_children(|button| {
                        button.spawn((
                            Text::new("Return to Title"),
                            TextFont {
                                font_size: 24.0,
                                ..default()
                            },
                            TextColor(TEXT_COLOR),
                        ));
                    });
            });
        });
}

fn handle_result_screen_buttons(
    mut next_state: ResMut<NextState<AppState>>,
    mut game_state: Option<ResMut<GameStateRes>>,
    button_q: Query<&Interaction, (Changed<Interaction>, With<ReturnToTitleButton>)>,
) {
    for interaction in &button_q {
        if *interaction != Interaction::Pressed {
            continue;
        }

        if let Some(game_state) = game_state.as_mut() {
            game_state.0.screen = GameScreen::Title;
        }
        next_state.set(AppState::Title);
    }
}

fn result_button_hover_system(
    mut button_q: Query<
        (&Interaction, &ButtonBaseColor, &mut BackgroundColor),
        (Changed<Interaction>, With<ReturnToTitleButton>),
    >,
) {
    for (interaction, base_color, mut background) in &mut button_q {
        background.0 = match interaction {
            Interaction::Pressed => BUTTON_PRESSED,
            Interaction::Hovered => HIGHLIGHT,
            Interaction::None => base_color.0,
        };
    }
}
