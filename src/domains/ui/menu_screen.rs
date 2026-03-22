//! Read-only JRPG menu screens for party, equipment, djinn, items, psynergy,
//! status, and quest progress.

use std::collections::BTreeMap;

use bevy::prelude::*;

use crate::domains::djinn::{DjinnInstance, DjinnSlots};
use crate::domains::equipment::{self, EquipmentLoadout};
use crate::domains::menu as menu_domain;
use crate::domains::quest::QuestManager;
use crate::domains::save::{SaveData, SavedEquipment};
use crate::game_state::{self, GameState};
use crate::shared::{DjinnState, GameScreen, MenuScreen, QuestDef, ScreenTransition};

use super::app_state::{AppState, CurrentMenu, CurrentTown, GameStateRes, SaveDataRes};
use super::plugin::GameDataRes;
use super::ui_helpers::{
    despawn_screen, spawn_panel, ButtonBaseColor, ScreenEntity, BG_COLOR, BORDER_COLOR, GOLD_COLOR,
    HIGHLIGHT, PANEL_BG, TEXT_COLOR, TEXT_DIM,
};

const MENU_TABS: [MenuScreen; 7] = [
    MenuScreen::Party,
    MenuScreen::Equipment,
    MenuScreen::Djinn,
    MenuScreen::Items,
    MenuScreen::Psynergy,
    MenuScreen::Status,
    MenuScreen::QuestLog,
];

const TAB_BG: Color = PANEL_BG;
const BUTTON_PRESSED: Color = GOLD_COLOR;

#[derive(Component)]
struct MenuContentText;

#[derive(Component, Clone, Copy)]
struct MenuTabButton(MenuScreen);

#[derive(Component)]
struct MenuBackButton;

pub struct MenuScreenPlugin;

impl Plugin for MenuScreenPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(AppState::Menu), setup_menu_screen)
            .add_systems(
                Update,
                (
                    handle_menu_buttons,
                    refresh_menu_content,
                    sync_tab_button_styles,
                    menu_button_hover_system,
                )
                    .run_if(in_state(AppState::Menu)),
            )
            .add_systems(
                OnExit(AppState::Menu),
                (despawn_screen, cleanup_menu_resources),
            );
    }
}

fn setup_menu_screen(
    mut commands: Commands,
    game_state: Res<GameStateRes>,
    save_data: Res<SaveDataRes>,
    game_data: Res<GameDataRes>,
) {
    let initial_menu = match game_state.0.screen {
        GameScreen::Menu(screen) => screen,
        _ => MenuScreen::Party,
    };
    let initial_content =
        build_menu_content(initial_menu, &game_state.0, &save_data.0, &game_data.0);

    commands.insert_resource(CurrentMenu(initial_menu));
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
                    width: Val::Px(1120.0),
                    height: Val::Px(660.0),
                    flex_direction: FlexDirection::Column,
                    row_gap: Val::Px(14.0),
                    ..default()
                },
                |panel| {
                    panel.spawn((
                        Text::new("FIELD MENU"),
                        TextFont {
                            font_size: 48.0,
                            ..default()
                        },
                        TextColor(BORDER_COLOR),
                    ));
                    panel.spawn((
                        Text::new("Read-only party records for Wave C."),
                        TextFont {
                            font_size: 18.0,
                            ..default()
                        },
                        TextColor(TEXT_DIM),
                    ));

                    panel
                        .spawn(Node {
                            width: Val::Percent(100.0),
                            flex_wrap: FlexWrap::Wrap,
                            column_gap: Val::Px(10.0),
                            row_gap: Val::Px(10.0),
                            margin: UiRect::top(Val::Px(6.0)),
                            ..default()
                        })
                        .with_children(|tabs| {
                            for screen in MENU_TABS {
                                let color = tab_color(screen, initial_menu);
                                tabs.spawn((
                                    Button,
                                    Node {
                                        width: Val::Px(142.0),
                                        min_height: Val::Px(46.0),
                                        justify_content: JustifyContent::Center,
                                        align_items: AlignItems::Center,
                                        padding: UiRect::axes(Val::Px(14.0), Val::Px(10.0)),
                                        border: UiRect::all(Val::Px(2.0)),
                                        ..default()
                                    },
                                    BackgroundColor(color),
                                    BorderColor(BORDER_COLOR),
                                    BorderRadius::all(Val::Px(6.0)),
                                    ButtonBaseColor(color),
                                    MenuTabButton(screen),
                                ))
                                .with_children(|button| {
                                    button.spawn((
                                        Text::new(menu_label(screen)),
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
                        panel,
                        Node {
                            width: Val::Percent(100.0),
                            flex_grow: 1.0,
                            min_height: Val::Px(420.0),
                            ..default()
                        },
                        |content_panel| {
                            content_panel.spawn((
                                Text::new(initial_content),
                                TextFont {
                                    font_size: 20.0,
                                    ..default()
                                },
                                TextColor(TEXT_COLOR),
                                Node {
                                    width: Val::Percent(100.0),
                                    ..default()
                                },
                                MenuContentText,
                            ));
                        },
                    );

                    panel
                        .spawn(Node {
                            width: Val::Percent(100.0),
                            justify_content: JustifyContent::SpaceBetween,
                            align_items: AlignItems::Center,
                            margin: UiRect::top(Val::Px(4.0)),
                            ..default()
                        })
                        .with_children(|footer| {
                            footer.spawn((
                                Text::new("Tabs switch pages. No equipment or item actions are available yet."),
                                TextFont {
                                    font_size: 18.0,
                                    ..default()
                                },
                                TextColor(TEXT_DIM),
                                Node {
                                    width: Val::Percent(70.0),
                                    ..default()
                                },
                            ));
                            footer
                                .spawn((
                                    Button,
                                    Node {
                                        width: Val::Px(220.0),
                                        min_height: Val::Px(52.0),
                                        justify_content: JustifyContent::Center,
                                        align_items: AlignItems::Center,
                                        padding: UiRect::axes(Val::Px(18.0), Val::Px(12.0)),
                                        border: UiRect::all(Val::Px(2.0)),
                                        ..default()
                                    },
                                    BackgroundColor(TAB_BG),
                                    BorderColor(BORDER_COLOR),
                                    BorderRadius::all(Val::Px(6.0)),
                                    ButtonBaseColor(TAB_BG),
                                    MenuBackButton,
                                ))
                                .with_children(|button| {
                                    button.spawn((
                                        Text::new("Back"),
                                        TextFont {
                                            font_size: 24.0,
                                            ..default()
                                        },
                                        TextColor(TEXT_COLOR),
                                    ));
                                });
                        });
                },
            );
        });
}

fn handle_menu_buttons(
    mut commands: Commands,
    mut next_state: ResMut<NextState<AppState>>,
    mut game_state: ResMut<GameStateRes>,
    mut current_menu: ResMut<CurrentMenu>,
    button_q: Query<
        (
            &Interaction,
            Option<&MenuTabButton>,
            Option<&MenuBackButton>,
        ),
        (Changed<Interaction>, With<Button>),
    >,
) {
    for (interaction, tab_button, back_button) in &button_q {
        if *interaction != Interaction::Pressed {
            continue;
        }

        if let Some(tab_button) = tab_button {
            current_menu.0 = tab_button.0;
            continue;
        }

        if back_button.is_none() {
            continue;
        }

        let previous_screen = game_state.0.screen_stack.stack.last().cloned();
        game_state::apply_transition(&mut game_state.0, ScreenTransition::ReturnToPrevious);

        match previous_screen {
            Some(GameScreen::Town(town_id)) => {
                game_state.0.screen = GameScreen::Town(town_id);
                commands.insert_resource(CurrentTown(town_id));
                next_state.set(AppState::Town);
            }
            _ => {
                game_state.0.screen = GameScreen::WorldMap;
                next_state.set(AppState::WorldMap);
            }
        }
    }
}

fn refresh_menu_content(
    current_menu: Res<CurrentMenu>,
    game_state: Res<GameStateRes>,
    save_data: Res<SaveDataRes>,
    game_data: Res<GameDataRes>,
    mut content_q: Query<&mut Text, With<MenuContentText>>,
) {
    if !current_menu.is_changed() {
        return;
    }

    let Ok(mut text) = content_q.get_single_mut() else {
        return;
    };

    text.0 = build_menu_content(current_menu.0, &game_state.0, &save_data.0, &game_data.0);
}

fn sync_tab_button_styles(
    current_menu: Res<CurrentMenu>,
    mut button_q: Query<(&MenuTabButton, &mut BackgroundColor, &mut ButtonBaseColor), With<Button>>,
) {
    if !current_menu.is_changed() {
        return;
    }

    for (button, mut background, mut base_color) in &mut button_q {
        let color = tab_color(button.0, current_menu.0);
        background.0 = color;
        base_color.0 = color;
    }
}

fn menu_button_hover_system(
    mut button_q: Query<
        (
            &Interaction,
            &ButtonBaseColor,
            &mut BackgroundColor,
            Option<&MenuTabButton>,
            Option<&MenuBackButton>,
        ),
        (Changed<Interaction>, With<Button>),
    >,
) {
    for (interaction, base_color, mut background, tab_button, back_button) in &mut button_q {
        if tab_button.is_none() && back_button.is_none() {
            continue;
        }

        background.0 = match interaction {
            Interaction::Pressed => BUTTON_PRESSED,
            Interaction::Hovered => HIGHLIGHT,
            Interaction::None => base_color.0,
        };
    }
}

fn cleanup_menu_resources(mut commands: Commands) {
    commands.remove_resource::<CurrentMenu>();
}

fn build_menu_content(
    screen: MenuScreen,
    game_state: &GameState,
    save_data: &SaveData,
    game_data: &crate::domains::data_loader::GameData,
) -> String {
    let lines = match screen {
        MenuScreen::Party => build_party_lines(save_data, game_data),
        MenuScreen::Equipment => build_equipment_lines(save_data, game_data),
        MenuScreen::Djinn => build_djinn_lines(save_data, game_data),
        MenuScreen::Items => build_item_lines(save_data, game_data),
        MenuScreen::Psynergy => build_psynergy_lines(save_data, game_data),
        MenuScreen::Status => build_status_lines(game_state, save_data),
        MenuScreen::QuestLog => build_quest_log_lines(game_state),
    };

    lines.join("\n")
}

fn build_party_lines(
    save_data: &SaveData,
    game_data: &crate::domains::data_loader::GameData,
) -> Vec<String> {
    let units = save_data
        .player_party
        .iter()
        .map(|saved_unit| {
            let unit_def = game_data.units.get(&saved_unit.unit_id);
            menu_domain::UnitSummary {
                name: unit_def
                    .map(|def| def.name.clone())
                    .unwrap_or_else(|| saved_unit.unit_id.0.clone()),
                level: saved_unit.level,
                hp_current: saved_unit.current_hp,
                hp_max: unit_def
                    .map(|def| def.base_stats.hp.get())
                    .unwrap_or(saved_unit.current_hp),
                element: unit_def
                    .map(|def| format!("{:?}", def.element))
                    .unwrap_or_else(|| "Unknown".to_string()),
            }
        })
        .collect::<Vec<_>>();

    menu_domain::show_party(&units)
}

fn build_equipment_lines(
    save_data: &SaveData,
    game_data: &crate::domains::data_loader::GameData,
) -> Vec<String> {
    let mut lines = vec!["=== EQUIPMENT ===".to_string()];

    if save_data.player_party.is_empty() {
        lines.push("  (no party members)".to_string());
        return lines;
    }

    for saved_unit in &save_data.player_party {
        let unit_def = game_data.units.get(&saved_unit.unit_id);
        let unit_name = unit_def
            .map(|def| def.name.as_str())
            .unwrap_or(saved_unit.unit_id.0.as_str());
        let loadout = saved_equipment_to_loadout(&saved_unit.equipment);
        let effects = equipment::compute_equipment_effects(&loadout, &game_data.equipment);

        lines.push(String::new());
        lines.push(format!("  {} (Lv.{})", unit_name, saved_unit.level));
        lines.push(format!(
            "    Weapon: {}",
            equipment_name(saved_unit.equipment.weapon.as_ref(), game_data)
        ));
        lines.push(format!(
            "    Armor:  {}",
            equipment_name(saved_unit.equipment.armor.as_ref(), game_data)
        ));
        lines.push(format!(
            "    Bonus:  HP {:+}  ATK {:+}  DEF {:+}  MAG {:+}  SPD {:+}",
            effects.total_stat_bonus.hp.get(),
            effects.total_stat_bonus.atk.get(),
            effects.total_stat_bonus.def.get(),
            effects.total_stat_bonus.mag.get(),
            effects.total_stat_bonus.spd.get(),
        ));

        if !effects.unlocked_abilities.is_empty() {
            let ability_names = effects
                .unlocked_abilities
                .iter()
                .map(|ability_id| {
                    game_data
                        .abilities
                        .get(ability_id)
                        .map(|ability| ability.name.clone())
                        .unwrap_or_else(|| ability_id.0.clone())
                })
                .collect::<Vec<_>>()
                .join(", ");
            lines.push(format!("    Unlocks: {}", ability_names));
        }
    }

    lines
}

fn build_djinn_lines(
    save_data: &SaveData,
    game_data: &crate::domains::data_loader::GameData,
) -> Vec<String> {
    let team_slots = build_team_djinn_slots(save_data);
    let mut lines = vec![format!(
        "Shared slots: {} equipped, {} good",
        team_slots.slots.len(),
        team_slots.good_count()
    )];
    lines.extend(menu_domain::show_djinn(
        &save_data
            .team_djinn
            .iter()
            .map(|saved_djinn| {
                let djinn_def = game_data.djinn.get(&saved_djinn.djinn_id);
                menu_domain::DjinnSummary {
                    name: djinn_def
                        .map(|def| def.name.clone())
                        .unwrap_or_else(|| saved_djinn.djinn_id.0.clone()),
                    element: djinn_def
                        .map(|def| format!("{:?}", def.element))
                        .unwrap_or_else(|| "Unknown".to_string()),
                    state: match saved_djinn.state {
                        DjinnState::Good => menu_domain::DjinnMenuState::Set,
                        DjinnState::Recovery => menu_domain::DjinnMenuState::Recovery,
                    },
                    tier: djinn_def.map(|def| def.tier.get()).unwrap_or(1),
                }
            })
            .collect::<Vec<_>>(),
    ));
    lines
}

fn build_item_lines(
    save_data: &SaveData,
    game_data: &crate::domains::data_loader::GameData,
) -> Vec<String> {
    let mut counts: BTreeMap<String, (u8, String)> = BTreeMap::new();

    for equipment_id in &save_data.inventory {
        let (name, description) = game_data
            .equipment
            .get(equipment_id)
            .map(|def| (def.name.clone(), format!("{:?} {:?}", def.slot, def.tier)))
            .unwrap_or_else(|| (equipment_id.0.clone(), "Unknown".to_string()));
        let entry = counts.entry(name).or_insert((0, description));
        entry.0 = entry.0.saturating_add(1);
    }

    let items = counts
        .into_iter()
        .map(|(name, (count, description))| menu_domain::ItemSummary {
            name,
            count,
            description,
        })
        .collect::<Vec<_>>();

    menu_domain::show_items(&items)
}

fn build_psynergy_lines(
    save_data: &SaveData,
    game_data: &crate::domains::data_loader::GameData,
) -> Vec<String> {
    let mut lines = vec!["=== PSYNERGY ===".to_string()];

    for saved_unit in &save_data.player_party {
        let Some(unit_def) = game_data.units.get(&saved_unit.unit_id) else {
            lines.push(format!("  {} (data missing)", saved_unit.unit_id.0));
            continue;
        };

        lines.push(String::new());
        lines.push(format!("  {} (Lv.{})", unit_def.name, saved_unit.level));

        let mut any_shown = false;
        for progression in &unit_def.abilities {
            if progression.level.get() > saved_unit.level {
                continue;
            }

            let ability_name = game_data
                .abilities
                .get(&progression.ability_id)
                .map(|ability| {
                    format!(
                        "    {:<18} {:>2} PP  {:>3} pow",
                        ability.name,
                        ability.mana_cost.get(),
                        ability.base_power.get(),
                    )
                })
                .unwrap_or_else(|| format!("    {}", progression.ability_id.0));
            lines.push(ability_name);
            any_shown = true;
        }

        if !any_shown {
            lines.push("    (no learned abilities)".to_string());
        }
    }

    if save_data.player_party.is_empty() {
        lines.push("  (no party members)".to_string());
    }

    lines
}

fn build_status_lines(game_state: &GameState, save_data: &SaveData) -> Vec<String> {
    let hours = game_state.play_time_seconds / 3600;
    let minutes = (game_state.play_time_seconds % 3600) / 60;
    let seconds = game_state.play_time_seconds % 60;

    vec![
        "=== STATUS ===".to_string(),
        format!("  Gold:       {}", save_data.gold),
        format!("  XP:         {}", save_data.xp),
        format!("  Play time:  {hours}h {minutes:02}m {seconds:02}s"),
        format!("  Party size: {}", save_data.player_party.len()),
        format!("  Djinn:      {}", save_data.team_djinn.len()),
        format!("  Inventory:  {}", save_data.inventory.len()),
    ]
}

fn build_quest_log_lines(game_state: &GameState) -> Vec<String> {
    let mut quest_defs = game_state
        .quest_state
        .flags
        .iter()
        .map(|(quest_id, stage)| QuestDef {
            id: *quest_id,
            name: format!("Quest #{}", quest_id.0),
            description: format!("Current stage: {:?}", stage),
            stages: vec![],
            rewards: vec![],
        })
        .collect::<Vec<_>>();
    quest_defs.sort_by_key(|def| def.id.0);

    let mut manager = QuestManager::new(quest_defs);
    manager.state = game_state.quest_state.clone();

    let mut lines = vec!["=== QUEST LOG ===".to_string()];
    let active_quests = manager.active_quests();

    if active_quests.is_empty() {
        lines.push("  (no active quests)".to_string());
        return lines;
    }

    for quest in active_quests {
        lines.push(format!("  {} - {:?}", quest.name, manager.stage(quest.id)));
    }

    lines
}

fn build_team_djinn_slots(save_data: &SaveData) -> DjinnSlots {
    let mut slots = DjinnSlots::new();

    for (index, saved_djinn) in save_data.team_djinn.iter().enumerate() {
        slots.slots.push(DjinnInstance {
            djinn_id: saved_djinn.djinn_id.clone(),
            state: saved_djinn.state,
            recovery_turns_remaining: if saved_djinn.state == DjinnState::Recovery {
                1
            } else {
                0
            },
            activation_order: index as u32,
        });
    }

    slots.next_activation_order = slots.slots.len() as u32;
    slots
}

fn saved_equipment_to_loadout(saved_equipment: &SavedEquipment) -> EquipmentLoadout {
    EquipmentLoadout {
        weapon: saved_equipment.weapon.clone(),
        helm: saved_equipment.helm.clone(),
        armor: saved_equipment.armor.clone(),
        boots: saved_equipment.boots.clone(),
        accessory: saved_equipment.accessory.clone(),
    }
}

fn equipment_name(
    equipment_id: Option<&crate::shared::EquipmentId>,
    game_data: &crate::domains::data_loader::GameData,
) -> String {
    equipment_id
        .and_then(|id| game_data.equipment.get(id))
        .map(|def| def.name.clone())
        .unwrap_or_else(|| "(none)".to_string())
}

fn tab_color(tab: MenuScreen, current: MenuScreen) -> Color {
    if tab == current {
        HIGHLIGHT
    } else {
        TAB_BG
    }
}

fn menu_label(screen: MenuScreen) -> &'static str {
    match screen {
        MenuScreen::Party => "Party",
        MenuScreen::Equipment => "Equipment",
        MenuScreen::Djinn => "Djinn",
        MenuScreen::Items => "Items",
        MenuScreen::Psynergy => "Psynergy",
        MenuScreen::Status => "Status",
        MenuScreen::QuestLog => "Quest Log",
    }
}
