//! Dungeon screen UI.

use bevy::prelude::*;

use crate::domains::dungeon::{self, DungeonState};
use crate::domains::save;
use crate::shared::{
    Direction, DungeonDef, DungeonId, EncounterDef, EncounterId, ItemId, QuestFlagId, QuestStage,
    RoomDef, RoomId, RoomType, UnitId,
};
use crate::starter_data::starter_dungeons;

use super::app_state::{AppState, CurrentDungeon, CurrentPuzzle, GameStateRes, SaveDataRes};
use super::plugin::{build_battle_from_encounter, BattleRes, GameDataRes};
use super::ui_helpers::{
    despawn_screen, spawn_button, spawn_panel, ButtonAction, MenuButton, ScreenEntity, BG_COLOR,
    GOLD_COLOR, TEXT_COLOR, TEXT_DIM,
};

#[derive(Resource)]
struct DungeonScreenState {
    dungeon: DungeonDef,
    state: DungeonState,
}

#[derive(Resource)]
struct DungeonStatusMessage(String);

#[derive(Component)]
struct DungeonContentColumn;

pub struct DungeonScreenPlugin;

impl Plugin for DungeonScreenPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(AppState::Dungeon), setup_dungeon_screen)
            .add_systems(
                Update,
                (handle_dungeon_buttons, refresh_dungeon_screen)
                    .chain()
                    .run_if(in_state(AppState::Dungeon)),
            )
            .add_systems(
                OnExit(AppState::Dungeon),
                (despawn_screen, cleanup_dungeon_state),
            );
    }
}

fn setup_dungeon_screen(
    mut commands: Commands,
    current_dungeon: Option<Res<CurrentDungeon>>,
    mut game_state: ResMut<GameStateRes>,
    mut next_state: ResMut<NextState<AppState>>,
) {
    let Some(current_dungeon) = current_dungeon else {
        next_state.set(AppState::WorldMap);
        return;
    };

    let Some(dungeon_def) = starter_dungeons()
        .into_iter()
        .find(|dungeon| dungeon.id == current_dungeon.0)
    else {
        next_state.set(AppState::WorldMap);
        return;
    };

    let dungeon_state = if let Some(existing_state) = game_state.0.dungeon_state.clone() {
        existing_state
    } else {
        let mut state = dungeon::enter_dungeon(&dungeon_def);
        for (dungeon_id, room_id, item_index) in &game_state.0.dungeon_collected_items {
            if *dungeon_id == current_dungeon.0 {
                state.collected_items.insert((*room_id, *item_index));
            }
        }
        state
    };
    game_state.0.dungeon_state = Some(dungeon_state.clone());

    commands.insert_resource(DungeonScreenState {
        dungeon: dungeon_def.clone(),
        state: dungeon_state,
    });
    commands.insert_resource(DungeonStatusMessage(format!(
        "Entered {}.",
        dungeon_def.name
    )));
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
                    width: Val::Px(920.0),
                    min_height: Val::Px(560.0),
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
                        DungeonContentColumn,
                    ));
                },
            );
        });
}

fn refresh_dungeon_screen(
    mut commands: Commands,
    current_dungeon: Res<CurrentDungeon>,
    dungeon_screen: Res<DungeonScreenState>,
    save_data: Res<SaveDataRes>,
    status: Res<DungeonStatusMessage>,
    content_q: Query<Entity, With<DungeonContentColumn>>,
) {
    let Ok(content_column) = content_q.get_single() else {
        return;
    };
    let room = dungeon::get_current_room(&dungeon_screen.state, &dungeon_screen.dungeon);
    let exits = available_exits(&dungeon_screen, &save_data.0);
    let visible_items = collectable_items(&dungeon_screen, room);
    let encounter_lines = encounter_lines(room);
    let puzzle_summary = puzzle_summary(room);

    commands.entity(content_column).despawn_descendants();
    commands.entity(content_column).with_children(|column| {
        column.spawn((
            Text::new(format!(
                "{}  |  {}",
                room_title(&dungeon_screen.dungeon, room),
                room_type_label(room.room_type)
            )),
            TextFont {
                font_size: 34.0,
                ..default()
            },
            TextColor(TEXT_COLOR),
        ));
        column.spawn((
            Text::new(room_description(&dungeon_screen.dungeon, room)),
            TextFont {
                font_size: 22.0,
                ..default()
            },
            TextColor(TEXT_DIM),
        ));
        column.spawn((
            Text::new(format!(
                "Visited rooms: {}  |  Current dungeon: {}",
                dungeon_screen.state.visited_rooms.len(),
                current_dungeon.0 .0
            )),
            TextFont {
                font_size: 18.0,
                ..default()
            },
            TextColor(TEXT_DIM),
        ));
        column.spawn((
            Text::new(status.0.clone()),
            TextFont {
                font_size: 18.0,
                ..default()
            },
            TextColor(GOLD_COLOR),
        ));

        if !encounter_lines.is_empty() {
            column.spawn((
                Text::new("Encounters"),
                TextFont {
                    font_size: 22.0,
                    ..default()
                },
                TextColor(TEXT_COLOR),
            ));
            for line in encounter_lines {
                column.spawn((
                    Text::new(line),
                    TextFont {
                        font_size: 18.0,
                        ..default()
                    },
                    TextColor(TEXT_DIM),
                ));
            }
        }

        if let Some(puzzle_summary) = puzzle_summary {
            column.spawn((
                Text::new(format!("Puzzle: {puzzle_summary}")),
                TextFont {
                    font_size: 18.0,
                    ..default()
                },
                TextColor(TEXT_DIM),
            ));
            for puzzle_index in 0..room.puzzles.len() {
                spawn_button(
                    column,
                    format!("Inspect Puzzle {}", puzzle_index + 1),
                    ButtonAction::OpenPuzzle(puzzle_index),
                    22.0,
                );
            }
        }

        if !visible_items.is_empty() {
            column.spawn((
                Text::new("Items"),
                TextFont {
                    font_size: 22.0,
                    ..default()
                },
                TextColor(TEXT_COLOR),
            ));
            for (item_index, item_label) in visible_items {
                spawn_button(
                    column,
                    format!("Collect {item_label}"),
                    ButtonAction::CollectRoomItem(item_index),
                    22.0,
                );
            }
        }

        column.spawn((
            Text::new("Exits"),
            TextFont {
                font_size: 22.0,
                ..default()
            },
            TextColor(TEXT_COLOR),
        ));
        for (label, room_id) in exits {
            spawn_button(
                column,
                format!("{label} to Room {}", room_id.0),
                ButtonAction::DungeonExit(room_id),
                22.0,
            );
        }

        if dungeon::is_boss_room(&dungeon_screen.dungeon, room.id) {
            spawn_button(column, "Challenge Boss", ButtonAction::FightBoss, 24.0);
        }

        spawn_button(column, "Leave Dungeon", ButtonAction::LeaveToMap, 24.0);
    });
}

fn handle_dungeon_buttons(
    mut commands: Commands,
    mut next_state: ResMut<NextState<AppState>>,
    current_dungeon: Res<CurrentDungeon>,
    mut dungeon_screen: ResMut<DungeonScreenState>,
    mut game_state: ResMut<GameStateRes>,
    save_data: Res<SaveDataRes>,
    game_data: Res<GameDataRes>,
    mut status: ResMut<DungeonStatusMessage>,
    button_q: Query<(&Interaction, &MenuButton), Changed<Interaction>>,
) {
    for (interaction, button) in &button_q {
        if *interaction != Interaction::Pressed {
            continue;
        }

        match &button.action {
            ButtonAction::DungeonExit(room_id) => {
                let dungeon_def = dungeon_screen.dungeon.clone();
                match dungeon::move_to_room(&mut dungeon_screen.state, &dungeon_def, *room_id) {
                    Ok(()) => {
                        status.0 = format!("Moved to room {}.", room_id.0);
                        game_state.0.dungeon_state = Some(dungeon_screen.state.clone());
                    }
                    Err(error) => {
                        status.0 = error.to_string();
                    }
                }
            }
            ButtonAction::CollectRoomItem(item_index) => {
                let room =
                    dungeon::get_current_room(&dungeon_screen.state, &dungeon_screen.dungeon);
                let room_id = room.id;
                let dungeon_def = dungeon_screen.dungeon.clone();
                match dungeon::collect_item(&mut dungeon_screen.state, &dungeon_def, *item_index) {
                    Some(item_id) => {
                        game_state.0.dungeon_collected_items.insert((
                            current_dungeon.0,
                            room_id,
                            *item_index,
                        ));
                        game_state.0.dungeon_state = Some(dungeon_screen.state.clone());
                        status.0 = format!("Collected {}.", item_id.0);
                    }
                    None => {
                        status.0 = "That item is no longer here.".to_string();
                    }
                }
            }
            ButtonAction::FightBoss => {
                let Some(encounter) = boss_encounter_for_dungeon(&game_data.0, current_dungeon.0)
                else {
                    status.0 = "No boss encounter is configured for this room.".to_string();
                    continue;
                };
                let Some(battle) =
                    build_battle_from_encounter(&game_data.0, &save_data.0, &encounter)
                else {
                    status.0 = "Boss encounter setup failed.".to_string();
                    continue;
                };

                game_state.0.active_encounter = Some(encounter.clone());
                game_state.0.screen = crate::shared::GameScreen::Battle;
                game_state.0.dungeon_state = Some(dungeon_screen.state.clone());
                commands.insert_resource(BattleRes(battle));
                next_state.set(AppState::InBattle);
                return;
            }
            ButtonAction::OpenPuzzle(puzzle_index) => {
                let room =
                    dungeon::get_current_room(&dungeon_screen.state, &dungeon_screen.dungeon);
                let Some(puzzle) = room.puzzles.get(*puzzle_index).cloned() else {
                    status.0 = "That puzzle is not available.".to_string();
                    continue;
                };
                game_state.0.dungeon_state = Some(dungeon_screen.state.clone());
                commands.insert_resource(CurrentPuzzle(puzzle));
                next_state.set(AppState::Puzzle);
                return;
            }
            ButtonAction::LeaveToMap => {
                game_state.0.screen = crate::shared::GameScreen::WorldMap;
                next_state.set(AppState::WorldMap);
                return;
            }
            _ => {}
        }
    }
}

fn cleanup_dungeon_state(mut commands: Commands) {
    commands.remove_resource::<CurrentDungeon>();
    commands.remove_resource::<DungeonScreenState>();
    commands.remove_resource::<DungeonStatusMessage>();
}

fn available_exits(
    dungeon_screen: &DungeonScreenState,
    save_data: &save::SaveData,
) -> Vec<(String, RoomId)> {
    let context = DungeonUiContext {
        game_state: None,
        save_data,
    };
    dungeon::get_available_exits(&dungeon_screen.state, &dungeon_screen.dungeon, &context)
        .into_iter()
        .map(|(direction, room_id)| (direction_label(direction).to_string(), room_id))
        .collect()
}

fn collectable_items(dungeon_screen: &DungeonScreenState, room: &RoomDef) -> Vec<(usize, String)> {
    room.items
        .iter()
        .enumerate()
        .filter(|(item_index, item)| {
            item.visible
                && !dungeon_screen
                    .state
                    .collected_items
                    .contains(&(room.id, *item_index))
        })
        .map(|(item_index, item)| (item_index, item.item_id.0.clone()))
        .collect()
}

fn encounter_lines(room: &RoomDef) -> Vec<String> {
    room.encounters
        .iter()
        .map(|slot| {
            let max_triggers = slot
                .max_triggers
                .map(|count| count.to_string())
                .unwrap_or_else(|| "∞".to_string());
            format!(
                "{}  |  weight {}  |  max {}",
                slot.encounter.name, slot.weight, max_triggers
            )
        })
        .collect()
}

fn puzzle_summary(room: &RoomDef) -> Option<String> {
    room.puzzles
        .first()
        .map(|puzzle| match &puzzle.puzzle_type {
            crate::shared::PuzzleType::PushBlock => "Push Block".to_string(),
            crate::shared::PuzzleType::ElementPillar(element) => {
                format!("Element Pillar ({element:?})")
            }
            crate::shared::PuzzleType::DjinnPuzzle(djinn_id) => {
                format!("Djinn Puzzle ({})", djinn_id.0)
            }
            crate::shared::PuzzleType::SwitchSequence => "Switch Sequence".to_string(),
            crate::shared::PuzzleType::IceSlide => "Ice Slide".to_string(),
        })
}

fn room_title(dungeon: &DungeonDef, room: &RoomDef) -> String {
    format!("{} - Room {}", dungeon.name, room.id.0)
}

fn room_type_label(room_type: RoomType) -> &'static str {
    match room_type {
        RoomType::Normal => "Normal",
        RoomType::Puzzle => "Puzzle",
        RoomType::MiniBoss | RoomType::Boss => "Boss",
        RoomType::Treasure => "Treasure",
        RoomType::Safe => "Safe",
    }
}

fn room_description(dungeon: &DungeonDef, room: &RoomDef) -> String {
    let exits = room.exits.len();
    let items = room.items.iter().filter(|item| item.visible).count();
    let puzzles = room.puzzles.len();
    let encounters = room.encounters.len();
    format!(
        "{} contains {} exit(s), {} visible item(s), {} puzzle(s), and {} encounter slot(s).",
        room_title(dungeon, room),
        exits,
        items,
        puzzles,
        encounters
    )
}

fn direction_label(direction: Direction) -> &'static str {
    match direction {
        Direction::Up => "N",
        Direction::Down => "S",
        Direction::Left => "W",
        Direction::Right => "E",
    }
}

fn boss_encounter_for_dungeon(
    game_data: &crate::domains::data_loader::GameData,
    dungeon_id: DungeonId,
) -> Option<EncounterDef> {
    let encounter_id = match dungeon_id {
        DungeonId(0) => "house-16",
        DungeonId(1) => "house-17",
        DungeonId(2) => "house-18",
        DungeonId(3) => "house-19",
        DungeonId(4) => "house-20",
        _ => return None,
    };

    game_data
        .encounters
        .get(&EncounterId(encounter_id.to_string()))
        .cloned()
}

struct DungeonUiContext<'a> {
    game_state: Option<&'a crate::game_state::GameState>,
    save_data: &'a save::SaveData,
}

impl dungeon::ConditionContext for DungeonUiContext<'_> {
    fn has_item(&self, item: &ItemId) -> bool {
        self.save_data
            .inventory
            .iter()
            .any(|equipment_id| equipment_id.0 == item.0)
    }

    fn has_djinn(&self, djinn: &crate::shared::DjinnId) -> bool {
        self.save_data
            .team_djinn
            .iter()
            .any(|saved_djinn| &saved_djinn.djinn_id == djinn)
    }

    fn quest_at_stage(&self, flag: &QuestFlagId, stage: QuestStage) -> bool {
        self.game_state
            .map(|game_state| game_state.quest_state.at_least(*flag, stage))
            .unwrap_or(false)
    }

    fn gold_at_least(&self, amount: crate::shared::bounded_types::Gold) -> bool {
        self.save_data.gold >= amount.get()
    }

    fn party_contains(&self, unit: &UnitId) -> bool {
        self.save_data
            .player_party
            .iter()
            .any(|saved_unit| &saved_unit.unit_id == unit)
    }
}
