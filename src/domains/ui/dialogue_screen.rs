//! Dialogue screen UI.

use bevy::prelude::*;

use crate::domains::dialogue::{self, ConditionContext, DialogueRunner};
use crate::domains::equipment::{self, EquipmentLoadout};
use crate::domains::progression;
use crate::domains::save::{self, SavedDjinn, SavedEquipment};
use crate::shared::bounded_types::Gold;
use crate::shared::{
    DialogueNode, DialogueSideEffect, DialogueTree, DjinnId, GameScreen, ItemId, MapNodeId,
    NodeUnlockState, NpcId, QuestFlagId, QuestStage, TownId, UnitId,
};
use crate::starter_data::{starter_dialogue_trees, starter_towns};

use super::app_state::{AppState, CurrentNpc, CurrentTown, GameStateRes, SaveDataRes};
use super::plugin::{build_battle_from_encounter, BattleRes, GameDataRes};
use super::ui_helpers::{
    despawn_screen, spawn_button, spawn_panel, ButtonAction, MenuButton, ScreenEntity,
    BORDER_COLOR, PANEL_BG, TEXT_COLOR, TEXT_DIM,
};

#[derive(Resource)]
struct DialogueScreenState {
    tree: DialogueTree,
    runner: DialogueRunner,
}

#[derive(Component)]
struct DialogueSpeakerText;

#[derive(Component)]
struct DialogueBodyText;

#[derive(Component)]
struct DialogueResponsesRoot;

pub struct DialogueScreenPlugin;

impl Plugin for DialogueScreenPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(AppState::Dialogue), setup_dialogue_screen)
            .add_systems(
                Update,
                handle_dialogue_buttons.run_if(in_state(AppState::Dialogue)),
            )
            .add_systems(
                OnExit(AppState::Dialogue),
                (despawn_screen, cleanup_dialogue_state),
            );
    }
}

fn setup_dialogue_screen(
    mut commands: Commands,
    current_npc: Option<Res<CurrentNpc>>,
    game_state: Option<Res<GameStateRes>>,
    save_data: Option<Res<SaveDataRes>>,
    mut next_state: ResMut<NextState<AppState>>,
) {
    let Some(current_npc) = current_npc else {
        next_state.set(AppState::Town);
        return;
    };
    let (Some(game_state), Some(save_data)) = (game_state, save_data) else {
        restore_town_for_npc(&mut commands, current_npc.0);
        next_state.set(AppState::Town);
        return;
    };

    let Some(tree) = find_dialogue_tree(current_npc.0) else {
        restore_town_for_npc(&mut commands, current_npc.0);
        next_state.set(AppState::Town);
        return;
    };

    let runner = dialogue::start_dialogue(&tree);
    let node = dialogue::get_current_node(&runner, &tree);
    let context = UiDialogueContext {
        game_state: &game_state.0,
        save_data: &save_data.0,
    };
    let responses = dialogue::get_available_responses(&runner, &tree, &context);
    let speaker_label = speaker_name(node).to_string();
    let body_text = node.text.clone();
    let response_buttons: Vec<(usize, String)> = responses
        .into_iter()
        .map(|(response_index, response)| (response_index, response.text.clone()))
        .collect();
    commands.insert_resource(DialogueScreenState { tree, runner });

    commands.spawn((Camera2d, ScreenEntity));
    commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                padding: UiRect::all(Val::Px(32.0)),
                ..default()
            },
            BackgroundColor(Color::Srgba(PANEL_BG.to_srgba().with_alpha(1.0))),
            ScreenEntity,
        ))
        .with_children(|root| {
            spawn_panel(
                root,
                Node {
                    width: Val::Px(860.0),
                    min_height: Val::Px(520.0),
                    flex_direction: FlexDirection::Column,
                    row_gap: Val::Px(18.0),
                    justify_content: JustifyContent::FlexStart,
                    ..default()
                },
                |panel| {
                    panel.spawn((
                        Text::new(speaker_label.clone()),
                        TextFont {
                            font_size: 34.0,
                            ..default()
                        },
                        TextColor(TEXT_COLOR),
                        DialogueSpeakerText,
                    ));
                    panel
                        .spawn((
                            Node {
                                width: Val::Percent(100.0),
                                min_height: Val::Px(220.0),
                                padding: UiRect::all(Val::Px(20.0)),
                                border: UiRect::all(Val::Px(2.0)),
                                ..default()
                            },
                            BackgroundColor(PANEL_BG),
                            BorderColor(BORDER_COLOR),
                        ))
                        .with_children(|text_panel| {
                            text_panel.spawn((
                                Text::new(body_text.clone()),
                                TextFont {
                                    font_size: 28.0,
                                    ..default()
                                },
                                TextColor(TEXT_COLOR),
                                DialogueBodyText,
                            ));
                        });
                    panel.spawn((
                        Text::new("Choose a response."),
                        TextFont {
                            font_size: 18.0,
                            ..default()
                        },
                        TextColor(TEXT_DIM),
                    ));
                    panel
                        .spawn((
                            Node {
                                width: Val::Percent(100.0),
                                flex_direction: FlexDirection::Column,
                                row_gap: Val::Px(10.0),
                                ..default()
                            },
                            DialogueResponsesRoot,
                        ))
                        .with_children(|response_root| {
                            for (response_index, response_text) in &response_buttons {
                                spawn_button(
                                    response_root,
                                    response_text.clone(),
                                    ButtonAction::DialogueResponse(*response_index),
                                    22.0,
                                );
                            }
                        });
                },
            );
        });
}

fn handle_dialogue_buttons(
    mut commands: Commands,
    mut next_state: ResMut<NextState<AppState>>,
    current_npc: Res<CurrentNpc>,
    mut dialogue_state: ResMut<DialogueScreenState>,
    mut game_state: ResMut<GameStateRes>,
    mut save_data: ResMut<SaveDataRes>,
    game_data: Res<GameDataRes>,
    mut speaker_q: Query<&mut Text, With<DialogueSpeakerText>>,
    mut body_q: Query<&mut Text, With<DialogueBodyText>>,
    responses_q: Query<Entity, With<DialogueResponsesRoot>>,
    button_q: Query<(&Interaction, &MenuButton), Changed<Interaction>>,
) {
    for (interaction, button) in &button_q {
        if *interaction != Interaction::Pressed {
            continue;
        }

        let ButtonAction::DialogueResponse(response_index) = &button.action else {
            continue;
        };
        let response_index = *response_index;

        let tree = dialogue_state.tree.clone();
        let (next_node, side_effects) =
            dialogue::choose_response(&mut dialogue_state.runner, &tree, response_index);

        let started_battle = apply_side_effects(
            &mut commands,
            &mut next_state,
            &mut game_state.0,
            &mut save_data.0,
            &game_data.0,
            &side_effects,
        );

        if started_battle {
            return;
        }

        if next_node.is_none() {
            restore_town_for_npc(&mut commands, current_npc.0);
            next_state.set(AppState::Town);
            return;
        }

        update_dialogue_texts(
            &dialogue_state.tree,
            &dialogue_state.runner,
            &mut commands,
            &game_state.0,
            &save_data.0,
            &mut speaker_q,
            &mut body_q,
            &responses_q,
        );
    }
}

fn cleanup_dialogue_state(mut commands: Commands) {
    commands.remove_resource::<CurrentNpc>();
    commands.remove_resource::<DialogueScreenState>();
}

fn update_dialogue_texts(
    tree: &DialogueTree,
    runner: &DialogueRunner,
    commands: &mut Commands,
    game_state: &crate::game_state::GameState,
    save_data: &save::SaveData,
    speaker_q: &mut Query<&mut Text, With<DialogueSpeakerText>>,
    body_q: &mut Query<&mut Text, With<DialogueBodyText>>,
    responses_q: &Query<Entity, With<DialogueResponsesRoot>>,
) {
    let node = dialogue::get_current_node(runner, tree);

    if let Ok(mut speaker_text) = speaker_q.get_single_mut() {
        speaker_text.0 = speaker_name(node).to_string();
    }

    if let Ok(mut body_text) = body_q.get_single_mut() {
        body_text.0 = node.text.clone();
    }

    if let Ok(response_root) = responses_q.get_single() {
        commands.entity(response_root).despawn_descendants();
        let context = UiDialogueContext {
            game_state,
            save_data,
        };
        let responses = dialogue::get_available_responses(runner, tree, &context);
        commands.entity(response_root).with_children(|parent| {
            for (response_index, response) in responses {
                spawn_button(
                    parent,
                    response.text.clone(),
                    ButtonAction::DialogueResponse(response_index),
                    22.0,
                );
            }
        });
    }
}

fn apply_side_effects(
    commands: &mut Commands,
    next_state: &mut ResMut<NextState<AppState>>,
    game_state: &mut crate::game_state::GameState,
    save_data: &mut save::SaveData,
    game_data: &crate::domains::data_loader::GameData,
    side_effects: &[DialogueSideEffect],
) -> bool {
    let mut started_battle = false;

    for effect in side_effects {
        match effect {
            DialogueSideEffect::GiveGold(amount) => {
                let next_gold = save_data.gold.saturating_add(amount.get());
                save_data.gold = next_gold;
                game_state.gold = Gold::new(next_gold);
            }
            DialogueSideEffect::UnlockMapNode(node_id) => {
                unlock_map_node(game_state, save_data, *node_id);
            }
            DialogueSideEffect::Heal => {
                heal_party(save_data, game_data);
            }
            DialogueSideEffect::SetQuestStage(flag, stage) => {
                game_state.quest_state.advance(*flag, *stage);
                save_data
                    .extension
                    .get_or_insert_with(save::create_default_extension)
                    .quest_state
                    .insert(*flag, *stage);
            }
            DialogueSideEffect::StartBattle(encounter) => {
                let Some(battle) = build_battle_from_encounter(game_data, save_data, encounter)
                else {
                    continue;
                };
                game_state.active_encounter = Some(encounter.clone());
                game_state.screen = GameScreen::Battle;
                commands.insert_resource(BattleRes(battle));
                next_state.set(AppState::InBattle);
                started_battle = true;
            }
            DialogueSideEffect::AddDjinnToParty(djinn_id) => {
                if save_data
                    .team_djinn
                    .iter()
                    .all(|saved_djinn| saved_djinn.djinn_id != *djinn_id)
                {
                    save_data.team_djinn.push(SavedDjinn {
                        djinn_id: djinn_id.clone(),
                        state: crate::shared::DjinnState::Good,
                    });
                }
            }
            _ => {}
        }
    }

    started_battle
}

fn unlock_map_node(
    game_state: &mut crate::game_state::GameState,
    save_data: &mut save::SaveData,
    node_id: MapNodeId,
) {
    if let Some(world_map) = game_state.world_map.as_mut() {
        world_map
            .unlock_states
            .insert(node_id, NodeUnlockState::Unlocked);
    }

    save_data
        .extension
        .get_or_insert_with(save::create_default_extension)
        .map_unlock_state
        .insert(node_id, NodeUnlockState::Unlocked);
}

fn heal_party(save_data: &mut save::SaveData, game_data: &crate::domains::data_loader::GameData) {
    for saved_unit in &mut save_data.player_party {
        let Some(unit_def) = game_data.units.get(&saved_unit.unit_id) else {
            continue;
        };
        let base_stats = progression::calculate_stats_at_level(
            &unit_def.base_stats,
            &unit_def.growth_rates,
            saved_unit.level,
        );
        let equipment = saved_equipment_loadout(&saved_unit.equipment);
        let equipment_effects =
            equipment::compute_equipment_effects(&equipment, &game_data.equipment);
        let max_hp = (i32::from(base_stats.hp.get())
            + i32::from(equipment_effects.total_stat_bonus.hp.get()))
        .clamp(1, i32::from(u16::MAX)) as u16;
        saved_unit.current_hp = max_hp;
    }
}

fn saved_equipment_loadout(saved_equipment: &SavedEquipment) -> EquipmentLoadout {
    EquipmentLoadout {
        weapon: saved_equipment.weapon.clone(),
        helm: saved_equipment.helm.clone(),
        armor: saved_equipment.armor.clone(),
        boots: saved_equipment.boots.clone(),
        accessory: saved_equipment.accessory.clone(),
    }
}

fn find_dialogue_tree(npc_id: NpcId) -> Option<DialogueTree> {
    let tree_id = starter_towns()
        .into_iter()
        .flat_map(|town| town.npcs.into_iter())
        .find(|npc| npc.npc_id == npc_id)
        .map(|npc| npc.dialogue_tree)?;

    starter_dialogue_trees()
        .into_iter()
        .find(|tree| tree.id == tree_id)
}

fn restore_town_for_npc(commands: &mut Commands, npc_id: NpcId) {
    if let Some(town_id) = find_town_for_npc(npc_id) {
        commands.insert_resource(CurrentTown(town_id));
    }
}

fn find_town_for_npc(npc_id: NpcId) -> Option<TownId> {
    starter_towns()
        .into_iter()
        .find(|town| town.npcs.iter().any(|npc| npc.npc_id == npc_id))
        .map(|town| town.id)
}

fn speaker_name(node: &DialogueNode) -> &'static str {
    match node.speaker {
        Some(NpcId(0)) => "Vale Elder",
        Some(NpcId(1)) => "Vale Villager",
        Some(NpcId(2)) => "Imil Healer",
        Some(NpcId(3)) => "Imil Scholar",
        Some(NpcId(4)) => "Kalay Lord",
        Some(NpcId(5)) => "Kalay Blacksmith",
        Some(NpcId(6)) => "Tolbi Mayor",
        Some(NpcId(7)) => "Tolbi Innkeeper",
        Some(NpcId(8)) => "Tolbi Sage",
        Some(_) => "Traveler",
        None => "Narrator",
    }
}

struct UiDialogueContext<'a> {
    game_state: &'a crate::game_state::GameState,
    save_data: &'a save::SaveData,
}

impl ConditionContext for UiDialogueContext<'_> {
    fn has_item(&self, item: &ItemId) -> bool {
        self.save_data
            .inventory
            .iter()
            .any(|equipment_id| equipment_id.0 == item.0)
    }

    fn has_djinn(&self, djinn: &DjinnId) -> bool {
        self.save_data
            .team_djinn
            .iter()
            .any(|saved_djinn| &saved_djinn.djinn_id == djinn)
    }

    fn quest_at_stage(&self, flag: &QuestFlagId, stage: QuestStage) -> bool {
        self.game_state.quest_state.at_least(*flag, stage)
    }

    fn gold_at_least(&self, amount: Gold) -> bool {
        self.save_data.gold >= amount.get()
    }

    fn party_contains(&self, unit: &UnitId) -> bool {
        self.save_data
            .player_party
            .iter()
            .any(|saved_unit| &saved_unit.unit_id == unit)
    }
}
