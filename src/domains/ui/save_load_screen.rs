//! Save/load screen with native file persistence and WASM localStorage support.

use bevy::prelude::*;

use crate::domains::{save, world_map};
use crate::game_state::{self, GameState};
use crate::shared::bounded_types::Gold;
use crate::shared::{GameScreen, NodeUnlockState, ScreenTransition};
use crate::starter_data::starter_map_nodes;

use super::app_state::{AppState, CurrentMenu, CurrentTown, GameStateRes, SaveDataRes};
use super::ui_helpers::{
    despawn_screen, spawn_panel, ButtonBaseColor, ScreenEntity, BG_COLOR, BORDER_COLOR, GOLD_COLOR,
    HIGHLIGHT, PANEL_BG, TEXT_COLOR, TEXT_DIM,
};

#[cfg(target_arch = "wasm32")]
use web_sys::Storage;

const BUTTON_BG: Color = PANEL_BG;
const BUTTON_PRESSED: Color = GOLD_COLOR;
const SAVE_PATH: &str = "saves/game.ron";

#[cfg(target_arch = "wasm32")]
const SAVE_KEY: &str = "vale-village-save";

#[derive(Component)]
struct SaveLoadStatusText;

#[derive(Component)]
struct SaveLoadInfoText;

#[derive(Component, Clone, Copy)]
struct SaveLoadButton(SaveLoadAction);

#[derive(Clone, Copy)]
enum SaveLoadAction {
    Save,
    Load,
    Back,
}

pub struct SaveLoadPlugin;

impl Plugin for SaveLoadPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(AppState::SaveLoad), setup_save_load_screen)
            .add_systems(
                Update,
                (handle_save_load_buttons, save_load_button_hover_system)
                    .run_if(in_state(AppState::SaveLoad)),
            )
            .add_systems(OnExit(AppState::SaveLoad), despawn_screen);
    }
}

fn setup_save_load_screen(mut commands: Commands) {
    commands.spawn((Camera2d, ScreenEntity));

    let (status_text, info_text) = match load_saved_data_from_storage() {
        Ok(Some(save_data)) => (String::new(), build_save_info(Some(&save_data))),
        Ok(None) => (
            String::new(),
            "No saved game found yet.\nUse Save to write the current party state.".to_string(),
        ),
        Err(error) => (
            format!("Could not read save data: {error}"),
            "Save preview unavailable.".to_string(),
        ),
    };

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
                    width: Val::Px(620.0),
                    flex_direction: FlexDirection::Column,
                    row_gap: Val::Px(14.0),
                    ..default()
                },
                |panel| {
                    panel.spawn((
                        Text::new("SAVE / LOAD"),
                        TextFont {
                            font_size: 46.0,
                            ..default()
                        },
                        TextColor(BORDER_COLOR),
                    ));
                    panel.spawn((
                        Text::new(
                            "Persist the current run or restore the most recent saved state.",
                        ),
                        TextFont {
                            font_size: 18.0,
                            ..default()
                        },
                        TextColor(TEXT_DIM),
                    ));

                    spawn_panel(
                        panel,
                        Node {
                            width: Val::Percent(100.0),
                            min_height: Val::Px(150.0),
                            ..default()
                        },
                        |info_panel| {
                            info_panel.spawn((
                                Text::new(info_text),
                                TextFont {
                                    font_size: 20.0,
                                    ..default()
                                },
                                TextColor(TEXT_COLOR),
                                Node {
                                    width: Val::Percent(100.0),
                                    ..default()
                                },
                                SaveLoadInfoText,
                            ));
                        },
                    );

                    spawn_save_load_button(panel, "Save", SaveLoadAction::Save);
                    spawn_save_load_button(panel, "Load", SaveLoadAction::Load);
                    spawn_save_load_button(panel, "Back", SaveLoadAction::Back);

                    panel.spawn((
                        Text::new(status_text),
                        TextFont {
                            font_size: 18.0,
                            ..default()
                        },
                        TextColor(TEXT_DIM),
                        SaveLoadStatusText,
                    ));
                },
            );
        });
}

fn handle_save_load_buttons(
    mut commands: Commands,
    mut next_state: ResMut<NextState<AppState>>,
    mut game_state: ResMut<GameStateRes>,
    mut save_data: ResMut<SaveDataRes>,
    button_q: Query<(&Interaction, &SaveLoadButton), Changed<Interaction>>,
    mut info_q: Query<&mut Text, With<SaveLoadInfoText>>,
    mut status_q: Query<&mut Text, With<SaveLoadStatusText>>,
) {
    for (interaction, button) in &button_q {
        if *interaction != Interaction::Pressed {
            continue;
        }

        match button.0 {
            SaveLoadAction::Save => {
                sync_save_snapshot(&mut save_data.0, &game_state.0);
                match save_to_storage(&save_data.0) {
                    Ok(()) => {
                        set_save_load_status(&mut status_q, "Saved!");
                        set_save_load_info(&mut info_q, &build_save_info(Some(&save_data.0)));
                    }
                    Err(error) => {
                        set_save_load_status(&mut status_q, &format!("Save failed: {error}"));
                    }
                }
            }
            SaveLoadAction::Load => match load_saved_data_from_storage() {
                Ok(Some(loaded_save)) => {
                    save_data.0 = loaded_save;
                    game_state.0 = restore_game_state_from_save(&save_data.0);
                    commands.remove_resource::<CurrentTown>();
                    commands.remove_resource::<CurrentMenu>();
                    next_state.set(AppState::WorldMap);
                }
                Ok(None) => {
                    set_save_load_status(&mut status_q, "No saved game found.");
                }
                Err(error) => {
                    set_save_load_status(&mut status_q, &format!("Load failed: {error}"));
                }
            },
            SaveLoadAction::Back => {
                let previous_screen = game_state.0.screen_stack.stack.last().cloned();
                game_state::apply_transition(&mut game_state.0, ScreenTransition::ReturnToPrevious);
                restore_previous_app_state(
                    &mut commands,
                    &mut next_state,
                    &mut game_state.0,
                    previous_screen,
                );
            }
        }
    }
}

fn save_load_button_hover_system(
    mut button_q: Query<
        (&Interaction, &ButtonBaseColor, &mut BackgroundColor),
        (Changed<Interaction>, With<SaveLoadButton>),
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

fn spawn_save_load_button(parent: &mut ChildBuilder, label: &str, action: SaveLoadAction) {
    parent
        .spawn((
            Button,
            Node {
                width: Val::Percent(100.0),
                min_height: Val::Px(56.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                padding: UiRect::axes(Val::Px(18.0), Val::Px(12.0)),
                border: UiRect::all(Val::Px(2.0)),
                ..default()
            },
            BackgroundColor(BUTTON_BG),
            BorderColor(BORDER_COLOR),
            BorderRadius::all(Val::Px(6.0)),
            ButtonBaseColor(BUTTON_BG),
            SaveLoadButton(action),
        ))
        .with_children(|button| {
            button.spawn((
                Text::new(label),
                TextFont {
                    font_size: 24.0,
                    ..default()
                },
                TextColor(TEXT_COLOR),
            ));
        });
}

fn set_save_load_status(status_q: &mut Query<&mut Text, With<SaveLoadStatusText>>, message: &str) {
    if let Ok(mut text) = status_q.get_single_mut() {
        text.0 = message.to_string();
    }
}

fn set_save_load_info(info_q: &mut Query<&mut Text, With<SaveLoadInfoText>>, message: &str) {
    if let Ok(mut text) = info_q.get_single_mut() {
        text.0 = message.to_string();
    }
}

fn build_save_info(save_data: Option<&save::SaveData>) -> String {
    let Some(save_data) = save_data else {
        return "No saved game found.".to_string();
    };

    let party_level = save_data
        .player_party
        .iter()
        .map(|unit| unit.level)
        .max()
        .unwrap_or(1);

    format!(
        "Stored save\nGold: {}\nLevel: {}\nEncounters completed: {}",
        save_data.gold,
        party_level,
        save_data.completed_encounters.len()
    )
}

fn sync_save_snapshot(save_data: &mut save::SaveData, game_state: &GameState) {
    save_data.gold = game_state.gold.get();
    save_data.extension = Some(game_state.to_save_extension());
}

fn restore_game_state_from_save(save_data: &save::SaveData) -> GameState {
    let mut game_state = if let Some(extension) = &save_data.extension {
        GameState::from_save(extension)
    } else {
        GameState::new_game()
    };

    let mut restored_world_map = world_map::load_map(starter_map_nodes());
    if let Some(extension) = &save_data.extension {
        for (node_id, unlock_state) in &extension.map_unlock_state {
            match unlock_state {
                NodeUnlockState::Visible => {
                    world_map::unlock_node(&mut restored_world_map, *node_id);
                }
                NodeUnlockState::Unlocked => {
                    world_map::unlock_node(&mut restored_world_map, *node_id);
                    world_map::unlock_node(&mut restored_world_map, *node_id);
                }
                NodeUnlockState::Completed => {
                    world_map::unlock_node(&mut restored_world_map, *node_id);
                    world_map::unlock_node(&mut restored_world_map, *node_id);
                    world_map::complete_node(&mut restored_world_map, *node_id);
                }
                NodeUnlockState::Locked => {}
            }
        }
    }

    game_state.world_map = Some(restored_world_map);
    game_state.gold = Gold::new(save_data.gold);
    game_state.screen = GameScreen::WorldMap;
    game_state
}

fn restore_previous_app_state(
    commands: &mut Commands,
    next_state: &mut ResMut<NextState<AppState>>,
    game_state: &mut GameState,
    previous_screen: Option<GameScreen>,
) {
    match previous_screen {
        Some(GameScreen::Title) => {
            game_state.screen = GameScreen::Title;
            next_state.set(AppState::Title);
        }
        Some(GameScreen::Town(town_id)) => {
            game_state.screen = GameScreen::Town(town_id);
            commands.insert_resource(CurrentTown(town_id));
            next_state.set(AppState::Town);
        }
        Some(GameScreen::Menu(menu_screen)) => {
            game_state.screen = GameScreen::Menu(menu_screen);
            commands.insert_resource(CurrentMenu(menu_screen));
            next_state.set(AppState::Menu);
        }
        _ => {
            game_state.screen = GameScreen::WorldMap;
            next_state.set(AppState::WorldMap);
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn save_to_storage(save_data: &save::SaveData) -> Result<(), String> {
    std::fs::create_dir_all("saves").map_err(|error| error.to_string())?;
    save::save_game(save_data, std::path::Path::new(SAVE_PATH)).map_err(|error| error.to_string())
}

#[cfg(target_arch = "wasm32")]
fn save_to_storage(save_data: &save::SaveData) -> Result<(), String> {
    let json = serde_json::to_string(save_data).map_err(|error| error.to_string())?;
    browser_storage()?
        .set_item(SAVE_KEY, &json)
        .map_err(|error| format!("{error:?}"))
}

#[cfg(not(target_arch = "wasm32"))]
fn load_saved_data_from_storage() -> Result<Option<save::SaveData>, String> {
    let path = std::path::Path::new(SAVE_PATH);
    if !path.exists() {
        return Ok(None);
    }

    save::load_game(path)
        .map(Some)
        .map_err(|error| error.to_string())
}

#[cfg(target_arch = "wasm32")]
fn load_saved_data_from_storage() -> Result<Option<save::SaveData>, String> {
    let Some(raw_value) = browser_storage()?
        .get_item(SAVE_KEY)
        .map_err(|error| format!("{error:?}"))?
    else {
        return Ok(None);
    };

    serde_json::from_str(&raw_value)
        .map(Some)
        .map_err(|error| error.to_string())
}

#[cfg(target_arch = "wasm32")]
fn browser_storage() -> Result<Storage, String> {
    let window = web_sys::window().ok_or_else(|| "window unavailable".to_string())?;
    let storage = window
        .local_storage()
        .map_err(|error| format!("{error:?}"))?
        .ok_or_else(|| "localStorage unavailable".to_string())?;
    Ok(storage)
}
