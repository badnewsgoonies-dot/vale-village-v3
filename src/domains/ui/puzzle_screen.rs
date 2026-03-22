//! Puzzle screen UI.

use bevy::prelude::*;

use crate::domains::puzzle::{self, PuzzleInput, PuzzleInstance, PuzzleResult};
use crate::domains::save;
use crate::shared::{DialogueSideEffect, Direction, DjinnId, Element, GameScreen, PuzzleDef, PuzzleType};

use super::app_state::{AppState, CurrentDungeon, CurrentPuzzle, GameStateRes, SaveDataRes};
use super::plugin::GameDataRes;
use super::ui_helpers::{
    despawn_screen, spawn_button, spawn_panel, ButtonAction, GOLD_COLOR, MenuButton, ScreenEntity,
    TEXT_COLOR, TEXT_DIM, BG_COLOR,
};

#[derive(Resource)]
struct PuzzleScreenState {
    instance: PuzzleInstance,
    status: String,
}

#[derive(Component)]
struct PuzzleContentColumn;

pub struct PuzzleScreenPlugin;

impl Plugin for PuzzleScreenPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(AppState::Puzzle), setup_puzzle_screen)
            .add_systems(
                Update,
                (handle_puzzle_buttons, refresh_puzzle_screen)
                    .chain()
                    .run_if(in_state(AppState::Puzzle)),
            )
            .add_systems(OnExit(AppState::Puzzle), (despawn_screen, cleanup_puzzle_state));
    }
}

fn setup_puzzle_screen(
    mut commands: Commands,
    current_puzzle: Option<Res<CurrentPuzzle>>,
    mut next_state: ResMut<NextState<AppState>>,
) {
    let Some(current_puzzle) = current_puzzle else {
        next_state.set(AppState::WorldMap);
        return;
    };

    commands.insert_resource(PuzzleScreenState {
        instance: PuzzleInstance::new(current_puzzle.0.clone()),
        status: puzzle_description(&current_puzzle.0),
    });
    commands.spawn((Camera2d, ScreenEntity));
    commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                padding: UiRect::all(Val::Px(28.0)),
                ..default()
            },
            BackgroundColor(BG_COLOR),
            ScreenEntity,
        ))
        .with_children(|root| {
            spawn_panel(
                root,
                Node {
                    width: Val::Px(840.0),
                    min_height: Val::Px(520.0),
                    flex_direction: FlexDirection::Column,
                    row_gap: Val::Px(18.0),
                    ..default()
                },
                |panel| {
                    panel.spawn((
                        Node {
                            width: Val::Percent(100.0),
                            flex_direction: FlexDirection::Column,
                            row_gap: Val::Px(14.0),
                            ..default()
                        },
                        PuzzleContentColumn,
                    ));
                },
            );
        });
}

fn refresh_puzzle_screen(
    mut commands: Commands,
    puzzle_state: Res<PuzzleScreenState>,
    content_q: Query<Entity, With<PuzzleContentColumn>>,
    save_data: Res<SaveDataRes>,
) {
    let Ok(content_column) = content_q.get_single() else {
        return;
    };
    let puzzle_def = &puzzle_state.instance.def;
    let has_required_djinn = match &puzzle_def.puzzle_type {
        PuzzleType::DjinnPuzzle(djinn_id) => save_data
            .0
            .team_djinn
            .iter()
            .any(|saved_djinn| saved_djinn.djinn_id == *djinn_id),
        _ => false,
    };

    commands.entity(content_column).despawn_descendants();
    commands.entity(content_column).with_children(|column| {
        column.spawn((
            Text::new(format!("{} Puzzle", puzzle_type_name(&puzzle_def.puzzle_type))),
            TextFont {
                font_size: 34.0,
                ..default()
            },
            TextColor(TEXT_COLOR),
        ));
        column.spawn((
            Text::new(puzzle_description(puzzle_def)),
            TextFont {
                font_size: 22.0,
                ..default()
            },
            TextColor(TEXT_DIM),
        ));
        column.spawn((
            Text::new(puzzle_state.status.clone()),
            TextFont {
                font_size: 20.0,
                ..default()
            },
            TextColor(GOLD_COLOR),
        ));

        match &puzzle_def.puzzle_type {
            PuzzleType::PushBlock => {
                for (label, direction) in directional_buttons() {
                    spawn_button(column, label, ButtonAction::PuzzlePush(direction), 22.0);
                }
            }
            PuzzleType::ElementPillar(element) => {
                column.spawn((
                    Text::new(format!("Required element: {element:?}")),
                    TextFont {
                        font_size: 20.0,
                        ..default()
                    },
                    TextColor(TEXT_DIM),
                ));
                spawn_button(column, "Activate", ButtonAction::PuzzleActivate, 24.0);
            }
            PuzzleType::DjinnPuzzle(djinn_id) => {
                column.spawn((
                    Text::new(format!(
                        "Required djinn: {} ({})",
                        djinn_id.0,
                        if has_required_djinn { "ready" } else { "missing" }
                    )),
                    TextFont {
                        font_size: 20.0,
                        ..default()
                    },
                    TextColor(TEXT_DIM),
                ));
                spawn_button(column, "Check Djinn", ButtonAction::PuzzleVerifyDjinn, 24.0);
            }
            PuzzleType::SwitchSequence => {
                for switch_index in 0..3 {
                    spawn_button(
                        column,
                        format!("Switch {}", switch_index + 1),
                        ButtonAction::PuzzleSwitch(switch_index),
                        22.0,
                    );
                }
            }
            PuzzleType::IceSlide => {
                for (label, direction) in directional_buttons() {
                    spawn_button(column, label, ButtonAction::PuzzleSlide(direction), 22.0);
                }
            }
        }

        spawn_button(column, "Return to Dungeon", ButtonAction::ReturnToDungeon, 24.0);
    });
}

fn handle_puzzle_buttons(
    mut commands: Commands,
    mut next_state: ResMut<NextState<AppState>>,
    mut puzzle_state: ResMut<PuzzleScreenState>,
    current_puzzle: Res<CurrentPuzzle>,
    mut game_state: ResMut<GameStateRes>,
    mut save_data: ResMut<SaveDataRes>,
    game_data: Res<GameDataRes>,
    button_q: Query<(&Interaction, &MenuButton), Changed<Interaction>>,
) {
    for (interaction, button) in &button_q {
        if *interaction != Interaction::Pressed {
            continue;
        }

        let maybe_input = match &button.action {
            ButtonAction::PuzzlePush(direction) => Some(PuzzleInput::PushDirection(*direction)),
            ButtonAction::PuzzleActivate => match current_puzzle.0.puzzle_type.clone() {
                PuzzleType::ElementPillar(element) => Some(PuzzleInput::ActivateElement(element)),
                _ => None,
            },
            ButtonAction::PuzzleVerifyDjinn => match current_puzzle.0.puzzle_type.clone() {
                PuzzleType::DjinnPuzzle(djinn_id) => Some(PuzzleInput::UseDjinn(djinn_id)),
                _ => None,
            },
            ButtonAction::PuzzleSwitch(index) => Some(PuzzleInput::ToggleSwitch(*index)),
            ButtonAction::PuzzleSlide(direction) => Some(PuzzleInput::SlideDirection(*direction)),
            ButtonAction::ReturnToDungeon => {
                restore_dungeon(&mut commands, &game_state.0);
                next_state.set(AppState::Dungeon);
                return;
            }
            _ => None,
        };

        let Some(input) = maybe_input else {
            continue;
        };

        let context = PuzzleUiContext {
            save_data: &save_data.0,
            game_data: &game_data.0,
        };
        if !puzzle::can_attempt(&puzzle_state.instance.def, &context) {
            puzzle_state.status = "Try again: the party does not meet the puzzle requirement.".to_string();
            continue;
        }

        match puzzle::attempt_solve(&mut puzzle_state.instance, input) {
            PuzzleResult::Solved(reward) => {
                if let Some(effect) = reward {
                    apply_puzzle_reward(&mut game_state.0, &mut save_data.0, &game_data.0, &effect);
                }
                restore_dungeon(&mut commands, &game_state.0);
                next_state.set(AppState::Dungeon);
                return;
            }
            PuzzleResult::Progress => {
                puzzle_state.status = "Progress made. Keep going.".to_string();
            }
            PuzzleResult::Failed => {
                puzzle_state.status = "Try again or return to the dungeon room.".to_string();
            }
            PuzzleResult::AlreadySolved => {
                restore_dungeon(&mut commands, &game_state.0);
                next_state.set(AppState::Dungeon);
                return;
            }
        }
    }
}

fn cleanup_puzzle_state(mut commands: Commands) {
    commands.remove_resource::<CurrentPuzzle>();
    commands.remove_resource::<PuzzleScreenState>();
}

fn apply_puzzle_reward(
    game_state: &mut crate::game_state::GameState,
    save_data: &mut save::SaveData,
    game_data: &crate::domains::data_loader::GameData,
    reward: &DialogueSideEffect,
) {
    match reward {
        DialogueSideEffect::GiveGold(amount) => {
            let next_gold = save_data.gold.saturating_add(amount.get());
            save_data.gold = next_gold;
            game_state.gold = crate::shared::bounded_types::Gold::new(next_gold);
        }
        DialogueSideEffect::GiveItem(item_id, count) => {
            for _ in 0..count.get() {
                save_data
                    .inventory
                    .push(crate::shared::EquipmentId(item_id.0.clone()));
            }
        }
        DialogueSideEffect::AddDjinnToParty(djinn_id) => {
            if save_data
                .team_djinn
                .iter()
                .all(|saved_djinn| saved_djinn.djinn_id != *djinn_id)
            {
                save_data.team_djinn.push(crate::domains::save::SavedDjinn {
                    djinn_id: djinn_id.clone(),
                    state: crate::shared::DjinnState::Good,
                });
            }
        }
        DialogueSideEffect::Heal => {
            for saved_unit in &mut save_data.player_party {
                if let Some(unit_def) = game_data.units.get(&saved_unit.unit_id) {
                    let base_stats = crate::domains::progression::calculate_stats_at_level(
                        &unit_def.base_stats,
                        &unit_def.growth_rates,
                        saved_unit.level,
                    );
                    saved_unit.current_hp = base_stats.hp.get();
                }
            }
        }
        DialogueSideEffect::SetQuestStage(flag, stage) => {
            game_state.quest_state.advance(*flag, *stage);
            save_data
                .extension
                .get_or_insert_with(save::create_default_extension)
                .quest_state
                .insert(*flag, *stage);
        }
        DialogueSideEffect::UnlockMapNode(node_id) => {
            if let Some(world_map) = game_state.world_map.as_mut() {
                world_map
                    .unlock_states
                    .insert(*node_id, crate::shared::NodeUnlockState::Unlocked);
            }
            save_data
                .extension
                .get_or_insert_with(save::create_default_extension)
                .map_unlock_state
                .insert(*node_id, crate::shared::NodeUnlockState::Unlocked);
        }
        _ => {}
    }
}

fn restore_dungeon(commands: &mut Commands, game_state: &crate::game_state::GameState) {
    if let GameScreen::Dungeon(dungeon_id) = game_state.screen {
        commands.insert_resource(CurrentDungeon(dungeon_id));
    }
}

fn puzzle_type_name(puzzle_type: &PuzzleType) -> &'static str {
    match puzzle_type {
        PuzzleType::PushBlock => "Push Block",
        PuzzleType::ElementPillar(_) => "Element Pillar",
        PuzzleType::DjinnPuzzle(_) => "Djinn",
        PuzzleType::SwitchSequence => "Switch Sequence",
        PuzzleType::IceSlide => "Ice Slide",
    }
}

fn puzzle_description(puzzle: &PuzzleDef) -> String {
    match &puzzle.puzzle_type {
        PuzzleType::PushBlock => "Push the block in the correct three-step sequence.".to_string(),
        PuzzleType::ElementPillar(element) => {
            format!("Channel the matching element to activate the {:?} pillar.", element)
        }
        PuzzleType::DjinnPuzzle(djinn_id) => {
            format!("Present {} to satisfy the djinn puzzle.", djinn_id.0)
        }
        PuzzleType::SwitchSequence => "Toggle the switches in ascending order.".to_string(),
        PuzzleType::IceSlide => "Slide across the ice in the correct path.".to_string(),
    }
}

fn directional_buttons() -> [(String, Direction); 4] {
    [
        ("North".to_string(), Direction::Up),
        ("South".to_string(), Direction::Down),
        ("West".to_string(), Direction::Left),
        ("East".to_string(), Direction::Right),
    ]
}

struct PuzzleUiContext<'a> {
    save_data: &'a save::SaveData,
    game_data: &'a crate::domains::data_loader::GameData,
}

impl puzzle::PuzzleContext for PuzzleUiContext<'_> {
    fn has_djinn(&self, id: &DjinnId) -> bool {
        self.save_data
            .team_djinn
            .iter()
            .any(|saved_djinn| &saved_djinn.djinn_id == id)
    }

    fn has_element_ability(&self, element: Element) -> bool {
        self.save_data.team_djinn.iter().any(|saved_djinn| {
            self.game_data
                .djinn
                .get(&saved_djinn.djinn_id)
                .map(|djinn_def| djinn_def.element == element)
                .unwrap_or(false)
        })
    }
}
