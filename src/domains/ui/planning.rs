//! Planning phase UI — action selection, target selection, mana tracking.
//! Wave 4: interactive planning with live mana updates.
#![allow(dead_code)]

use std::collections::HashSet;

use bevy::prelude::*;

use crate::domains::battle_engine::{
    check_battle_end, execute_round, get_planning_order, plan_action,
    plan_enemy_actions_with_ai, Battle, BattleEvent, BattleResult,
};
use crate::domains::ai::AiStrategy;
use crate::domains::data_loader::GameData;
use crate::domains::djinn;
use crate::shared::{
    AbilityId, BattleAction, BattlePhase, DjinnState, Side, TargetRef, UnitId,
};

use super::plugin::{BattleRes, GameDataRes};

// ── State machine ──────────────────────────────────────────────────

/// Current planning state — tracked as a Bevy resource.
#[derive(Resource, Default)]
pub struct PlanningState {
    /// Which player unit index we're currently selecting an action for.
    pub current_unit: usize,
    /// Planning order (indices into battle.player_units).
    pub order: Vec<usize>,
    /// How far through the order we are.
    pub order_pos: usize,
    /// Sub-state for action selection.
    pub mode: PlanningMode,
    /// Collected battle events from last execution (for Wave 5 animation).
    pub last_events: Vec<BattleEvent>,
    /// Battle result if finished.
    pub result: Option<BattleResult>,
}

#[derive(Default, Clone, PartialEq)]
pub enum PlanningMode {
    #[default]
    SelectAction,
    SelectAbility,
    SelectTarget {
        action_type: PendingAction,
    },
    Executing,
    RoundComplete,
    BattleOver,
}

#[derive(Clone, PartialEq)]
pub enum PendingAction {
    Attack,
    Ability(AbilityId),
}

// ── UI markers ─────────────────────────────────────────────────────

#[derive(Component)]
pub struct PlanningPanel;

#[derive(Component)]
pub struct ActionButton(pub ActionChoice);

#[derive(Clone)]
pub enum ActionChoice {
    Attack,
    OpenAbilityMenu,
    UseAbility(AbilityId, String), // id + display name
    ActivateDjinn(u8, String),     // slot index + display name
    Summon(Vec<u8>, u8),           // chosen slot indices + tier
    SelectTarget(TargetRef),
    NextRound,
}

#[derive(Component)]
pub struct PlanningText;

// ── Systems ────────────────────────────────────────────────────────

/// Initialize planning state at startup.
pub fn init_planning(mut commands: Commands, battle: Res<BattleRes>) {
    let order = get_planning_order(&battle.0);
    let current = order.first().copied().unwrap_or(0);
    commands.insert_resource(PlanningState {
        current_unit: current,
        order: order.clone(),
        order_pos: 0,
        mode: PlanningMode::SelectAction,
        last_events: Vec::new(),
        result: None,
    });
}

/// Build the planning UI panel (called on Startup).
pub fn setup_planning_panel(mut commands: Commands) {
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                bottom: Val::Px(130.0),
                left: Val::Px(0.0),
                width: Val::Percent(100.0),
                height: Val::Px(220.0),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::FlexStart,
                padding: UiRect::all(Val::Px(8.0)),
                row_gap: Val::Px(4.0),
                ..default()
            },
            BackgroundColor(Color::srgba(0.08, 0.08, 0.15, 0.85)),
            PlanningPanel,
        ));
}

/// Rebuild planning buttons based on current state.
pub fn update_planning_ui(
    mut commands: Commands,
    panel_q: Query<Entity, With<PlanningPanel>>,
    battle: Res<BattleRes>,
    game_data: Res<GameDataRes>,
    state: Res<PlanningState>,
) {
    let Ok(panel_entity) = panel_q.get_single() else {
        return;
    };

    // Clear existing children
    commands.entity(panel_entity).despawn_descendants();

    let battle = &battle.0;

    commands.entity(panel_entity).with_children(|panel| {
        match &state.mode {
            PlanningMode::SelectAction => {
                let unit_idx = state.current_unit;
                let unit = &battle.player_units[unit_idx];
                let unit_name = &unit.unit.id;
                let available_abilities = current_player_ability_ids(battle, &game_data.0, unit_idx);
                let ability_preview = current_player_ability_names(
                    battle,
                    &game_data.0,
                    unit_idx,
                );
                let djinn_summary = current_djinn_summary(&game_data.0, battle, unit_idx);
                let djinn_choices = current_good_djinn_choices(&game_data.0, battle, unit_idx);
                let summon_choices = current_summon_choices(&game_data.0, battle, unit_idx);

                // Header
                panel.spawn((
                    Text::new(format!("{} — Choose action:", unit_name)),
                    TextFont { font_size: 16.0, ..default() },
                    TextColor(Color::WHITE),
                    PlanningText,
                ));

                // Button row
                panel.spawn(Node {
                    flex_direction: FlexDirection::Row,
                    column_gap: Val::Px(12.0),
                    ..default()
                }).with_children(|row| {
                    // ATTACK button
                    spawn_action_button(
                        row,
                        "ATTACK (0 mana)",
                        ActionChoice::Attack,
                        Color::srgb(0.267, 0.667, 0.267),
                    );

                    // ABILITY button (if current kit has abilities)
                    if !available_abilities.is_empty() {
                        spawn_action_button(
                            row,
                            "ABILITY",
                            ActionChoice::OpenAbilityMenu,
                            Color::srgb(0.267, 0.533, 0.8),
                        );
                    }
                });

                panel.spawn((
                    Text::new(format!("Djinn: {}", djinn_summary)),
                    TextFont { font_size: 12.0, ..default() },
                    TextColor(Color::srgb(0.8, 0.8, 0.6)),
                ));

                if !ability_preview.is_empty() {
                    panel.spawn((
                        Text::new(format!("Current kit: {}", ability_preview.join(", "))),
                        TextFont { font_size: 11.0, ..default() },
                        TextColor(Color::srgb(0.6, 0.6, 0.8)),
                    ));
                }

                if !djinn_choices.is_empty() {
                    panel.spawn((
                        Text::new("Djinn menu:"),
                        TextFont { font_size: 12.0, ..default() },
                        TextColor(Color::srgb(0.8, 0.8, 0.267)),
                    ));

                    panel
                        .spawn(Node {
                            flex_direction: FlexDirection::Row,
                            column_gap: Val::Px(8.0),
                            flex_wrap: FlexWrap::Wrap,
                            ..default()
                        })
                        .with_children(|row| {
                            for (slot_idx, name) in djinn_choices {
                                spawn_action_button(
                                    row,
                                    &format!("ACTIVATE {}", name),
                                    ActionChoice::ActivateDjinn(slot_idx, name),
                                    Color::srgb(0.8, 0.533, 0.267),
                                );
                            }
                        });
                }

                if !summon_choices.is_empty() {
                    panel.spawn((
                        Text::new("Summons execute before all other actions:"),
                        TextFont { font_size: 12.0, ..default() },
                        TextColor(Color::srgb(0.8, 0.667, 0.267)),
                    ));

                    panel
                        .spawn(Node {
                            flex_direction: FlexDirection::Row,
                            column_gap: Val::Px(8.0),
                            ..default()
                        })
                        .with_children(|row| {
                            for (djinn_indices, tier) in summon_choices {
                                spawn_action_button(
                                    row,
                                    &format!("SUMMON T{}", tier),
                                    ActionChoice::Summon(djinn_indices, tier),
                                    Color::srgb(0.8, 0.267, 0.533),
                                );
                            }
                        });
                }

                // Queue status
                let planned = state.order_pos;
                let total = state.order.len();
                panel.spawn((
                    Text::new(format!("Queue: {}/{} planned", planned, total)),
                    TextFont { font_size: 12.0, ..default() },
                    TextColor(Color::srgb(0.5, 0.5, 0.5)),
                ));
            }

            PlanningMode::SelectAbility => {
                let unit_idx = state.current_unit;
                let unit = &battle.player_units[unit_idx];
                let available_abilities = current_player_ability_ids(battle, &game_data.0, unit_idx);

                panel.spawn((
                    Text::new(format!("{} — Choose ability:", unit.unit.id)),
                    TextFont { font_size: 16.0, ..default() },
                    TextColor(Color::WHITE),
                    PlanningText,
                ));

                panel.spawn(Node {
                    flex_direction: FlexDirection::Row,
                    column_gap: Val::Px(8.0),
                    flex_wrap: FlexWrap::Wrap,
                    ..default()
                }).with_children(|row| {
                    for aid in &available_abilities {
                        if let Some(ability) = game_data.0.abilities.get(aid) {
                            if ability.mana_cost <= battle.mana_pool.current_mana {
                                let label = format!("{} ({})", ability.name, ability.mana_cost);
                                spawn_action_button(
                                    row,
                                    &label,
                                    ActionChoice::UseAbility(aid.clone(), ability.name.clone()),
                                    Color::srgb(0.267, 0.533, 0.8),
                                );
                            }
                        }
                    }
                });
            }

            PlanningMode::SelectTarget { action_type } => {
                let label = match action_type {
                    PendingAction::Attack => "Select target for ATTACK:",
                    PendingAction::Ability(_) => "Select target:",
                };

                panel.spawn((
                    Text::new(label.to_string()),
                    TextFont { font_size: 16.0, ..default() },
                    TextColor(Color::WHITE),
                    PlanningText,
                ));

                panel.spawn(Node {
                    flex_direction: FlexDirection::Row,
                    column_gap: Val::Px(8.0),
                    ..default()
                }).with_children(|row| {
                    // Show alive enemies as targets
                    for (i, enemy) in battle.enemies.iter().enumerate() {
                        if enemy.unit.is_alive {
                            let target = TargetRef { side: Side::Enemy, index: i as u8 };
                            let label = format!("{} ({}/{})",
                                enemy.unit.id, enemy.unit.current_hp, enemy.unit.stats.hp);
                            spawn_action_button(
                                row,
                                &label,
                                ActionChoice::SelectTarget(target),
                                Color::srgb(0.8, 0.267, 0.267),
                            );
                        }
                    }
                });
            }

            PlanningMode::Executing => {
                panel.spawn((
                    Text::new("Executing round..."),
                    TextFont { font_size: 18.0, ..default() },
                    TextColor(Color::srgb(0.8, 0.667, 0.267)),
                    PlanningText,
                ));
            }

            PlanningMode::RoundComplete => {
                panel.spawn((
                    Text::new("Round complete!"),
                    TextFont { font_size: 18.0, ..default() },
                    TextColor(Color::srgb(0.267, 0.8, 0.267)),
                    PlanningText,
                ));

                spawn_action_button(
                    panel,
                    "Next Round",
                    ActionChoice::NextRound,
                    Color::srgb(0.8, 0.667, 0.267),
                );
            }

            PlanningMode::BattleOver => {
                let msg = match &state.result {
                    Some(BattleResult::Victory { xp, gold }) =>
                        format!("VICTORY! +{} XP, +{} gold", xp, gold),
                    Some(BattleResult::Defeat) => "DEFEAT".to_string(),
                    None => "Battle ended".to_string(),
                };
                let color = match &state.result {
                    Some(BattleResult::Victory { .. }) => Color::srgb(0.267, 0.8, 0.267),
                    _ => Color::srgb(0.8, 0.267, 0.267),
                };
                panel.spawn((
                    Text::new(msg),
                    TextFont { font_size: 22.0, ..default() },
                    TextColor(color),
                    PlanningText,
                ));
            }
        }
    });
}

/// Handle button clicks.
pub fn handle_planning_clicks(
    interaction_q: Query<(&Interaction, &ActionButton), Changed<Interaction>>,
    mut battle: ResMut<BattleRes>,
    mut state: ResMut<PlanningState>,
) {
    for (interaction, button) in interaction_q.iter() {
        if *interaction != Interaction::Pressed {
            continue;
        }

        match &button.0 {
            ActionChoice::Attack => {
                // Go to target selection for attack
                state.mode = PlanningMode::SelectTarget {
                    action_type: PendingAction::Attack,
                };
            }

            ActionChoice::OpenAbilityMenu => {
                state.mode = PlanningMode::SelectAbility;
            }

            ActionChoice::UseAbility(id, _name) => {
                // Specific ability selected → target selection
                state.mode = PlanningMode::SelectTarget {
                    action_type: PendingAction::Ability(id.clone()),
                };
            }

            ActionChoice::ActivateDjinn(djinn_index, _name) => {
                let unit_ref = TargetRef {
                    side: Side::Player,
                    index: state.current_unit as u8,
                };

                if plan_action(
                    &mut battle.0,
                    unit_ref,
                    BattleAction::ActivateDjinn {
                        djinn_index: *djinn_index,
                    },
                )
                .is_ok()
                {
                    advance_after_success(&mut battle, &mut state);
                }
            }

            ActionChoice::Summon(djinn_indices, _tier) => {
                let unit_ref = TargetRef {
                    side: Side::Player,
                    index: state.current_unit as u8,
                };

                if plan_action(
                    &mut battle.0,
                    unit_ref,
                    BattleAction::Summon {
                        djinn_indices: djinn_indices.clone(),
                    },
                )
                .is_ok()
                {
                    advance_after_success(&mut battle, &mut state);
                }
            }

            ActionChoice::SelectTarget(target) => {
                let unit_idx = state.current_unit;
                let unit_ref = TargetRef {
                    side: Side::Player,
                    index: unit_idx as u8,
                };

                let action = match &state.mode {
                    PlanningMode::SelectTarget { action_type } => match action_type {
                        PendingAction::Attack => BattleAction::Attack { target: *target },
                        PendingAction::Ability(aid) => BattleAction::UseAbility {
                            ability_id: aid.clone(),
                            targets: vec![*target],
                        },
                    },
                    _ => return,
                };

                if plan_action(&mut battle.0, unit_ref, action).is_ok() {
                    advance_after_success(&mut battle, &mut state);
                }
            }

            ActionChoice::NextRound => {
                // Start new planning round
                battle.0.phase = BattlePhase::Planning;
                battle.0.planned_actions.clear();
                // Reset mana for new round
                battle.0.mana_pool.current_mana = battle.0.mana_pool.max_mana;
                battle.0.mana_pool.projected_mana = 0;

                let order = get_planning_order(&battle.0);
                state.order = order.clone();
                state.order_pos = 0;
                state.current_unit = order.first().copied().unwrap_or(0);
                state.mode = PlanningMode::SelectAction;
                state.last_events.clear();
            }
        }
    }
}

// ── Helpers ────────────────────────────────────────────────────────

fn advance_after_success(battle: &mut BattleRes, state: &mut PlanningState) {
    state.order_pos += 1;
    if state.order_pos >= state.order.len() {
        plan_enemy_actions_with_ai(&mut battle.0, AiStrategy::Aggressive);
        let events = execute_round(&mut battle.0);
        state.last_events = events;

        if let Some(result) = check_battle_end(&battle.0) {
            state.result = Some(result);
            state.mode = PlanningMode::BattleOver;
        } else {
            state.mode = PlanningMode::RoundComplete;
        }
    } else {
        state.current_unit = state.order[state.order_pos];
        state.mode = PlanningMode::SelectAction;
    }
}

fn current_player_ability_ids(
    battle: &Battle,
    game_data: &GameData,
    unit_idx: usize,
) -> Vec<AbilityId> {
    let Some(unit) = battle.player_units.get(unit_idx) else {
        return Vec::new();
    };
    let Some(unit_def) = game_data.units.get(&UnitId(unit.unit.id.clone())) else {
        return Vec::new();
    };

    let mut seen: HashSet<String> = HashSet::new();
    let mut ability_ids = Vec::new();
    let mut push_unique = |ability_id: &AbilityId| {
        if battle.ability_defs.contains_key(ability_id) && seen.insert(ability_id.0.clone()) {
            ability_ids.push(ability_id.clone());
        }
    };

    for progression in &unit_def.abilities {
        push_unique(&progression.ability_id);
    }

    for equipment_id in [
        unit.equipment.weapon.as_ref(),
        unit.equipment.helm.as_ref(),
        unit.equipment.armor.as_ref(),
        unit.equipment.boots.as_ref(),
        unit.equipment.accessory.as_ref(),
    ]
    .into_iter()
    .flatten()
    {
        if let Some(equipment_def) = game_data.equipment.get(equipment_id) {
            if let Some(unlocks_ability) = &equipment_def.unlocks_ability {
                push_unique(unlocks_ability);
            }
        }
    }

    for inst in &unit.djinn_slots.slots {
        if let Some(djinn_def) = game_data.djinn.get(&inst.djinn_id) {
            let compatibility = djinn::determine_compatibility(djinn_def.element, unit_def.element);
            for ability_id in djinn::get_granted_abilities(djinn_def, compatibility, inst.state) {
                push_unique(&ability_id);
            }
        }
    }

    ability_ids
}

fn current_player_ability_names(
    battle: &Battle,
    game_data: &GameData,
    unit_idx: usize,
) -> Vec<String> {
    current_player_ability_ids(battle, game_data, unit_idx)
        .into_iter()
        .map(|ability_id| {
            battle
                .ability_defs
                .get(&ability_id)
                .map(|ability| ability.name.clone())
                .unwrap_or(ability_id.0)
        })
        .collect()
}

fn current_djinn_summary(game_data: &GameData, battle: &Battle, unit_idx: usize) -> String {
    let Some(unit) = battle.player_units.get(unit_idx) else {
        return "none".to_string();
    };
    if unit.djinn_slots.slots.is_empty() {
        return "none".to_string();
    }

    unit.djinn_slots
        .slots
        .iter()
        .map(|inst| {
            let name = game_data
                .djinn
                .get(&inst.djinn_id)
                .map(|djinn| djinn.name.clone())
                .unwrap_or_else(|| inst.djinn_id.0.clone());
            match inst.state {
                DjinnState::Good => format!("{}[Good]", name),
                DjinnState::Recovery if inst.recovery_turns_remaining > 0 => {
                    format!("{}[Recovery:{}]", name, inst.recovery_turns_remaining)
                }
                DjinnState::Recovery => format!("{}[Recovery]", name),
            }
        })
        .collect::<Vec<_>>()
        .join(", ")
}

fn current_good_djinn_choices(
    game_data: &GameData,
    battle: &Battle,
    unit_idx: usize,
) -> Vec<(u8, String)> {
    let Some(unit) = battle.player_units.get(unit_idx) else {
        return Vec::new();
    };

    unit.djinn_slots
        .slots
        .iter()
        .enumerate()
        .filter(|(_, inst)| inst.state == DjinnState::Good)
        .map(|(idx, inst)| {
            let name = game_data
                .djinn
                .get(&inst.djinn_id)
                .map(|djinn| djinn.name.clone())
                .unwrap_or_else(|| inst.djinn_id.0.clone());
            (idx as u8, name)
        })
        .collect()
}

fn current_summon_choices(
    game_data: &GameData,
    battle: &Battle,
    unit_idx: usize,
) -> Vec<(Vec<u8>, u8)> {
    if battle.player_units.get(unit_idx).is_none() {
        return Vec::new();
    }

    let good_slots = current_good_djinn_choices(game_data, battle, unit_idx);
    djinn::get_available_summons(good_slots.len())
        .into_iter()
        .map(|tier| {
            let indices = good_slots
                .iter()
                .take(tier.required_good as usize)
                .map(|(slot_idx, _)| *slot_idx)
                .collect::<Vec<_>>();
            (indices, tier.tier)
        })
        .collect()
}

fn spawn_action_button(
    parent: &mut ChildBuilder,
    label: &str,
    choice: ActionChoice,
    color: Color,
) {
    parent
        .spawn((
            Button,
            Node {
                padding: UiRect::axes(Val::Px(16.0), Val::Px(8.0)),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            BackgroundColor(color),
            BorderRadius::all(Val::Px(4.0)),
            ActionButton(choice),
        ))
        .with_children(|btn| {
            btn.spawn((
                Text::new(label.to_string()),
                TextFont {
                    font_size: 14.0,
                    ..default()
                },
                TextColor(Color::WHITE),
            ));
        });
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use super::*;
    use crate::domains::battle_engine::{new_battle, EnemyUnitData, PlayerUnitData};
    use crate::domains::data_loader;
    use crate::domains::djinn::DjinnSlots;
    use crate::domains::equipment::{self, EquipmentLoadout};
    use crate::shared::{DjinnId, EncounterId};

    fn load_full_game_data() -> GameData {
        match data_loader::load_game_data(Path::new("data/full")) {
            Ok(data) => data,
            Err(errors) => panic!("expected full game data to load: {errors:?}"),
        }
    }

    fn build_player_unit(unit_id: &str, djinn_id: &str, game_data: &GameData) -> PlayerUnitData {
        let unit_def = game_data
            .units
            .get(&UnitId(unit_id.to_string()))
            .expect("test unit should exist");
        let mut djinn_slots = DjinnSlots::new();
        if !djinn_id.is_empty() {
            djinn_slots.add(DjinnId(djinn_id.to_string()));
        }

        let equipment = EquipmentLoadout::default();
        let equipment_effects = equipment::compute_equipment_effects(&equipment, &game_data.equipment);

        PlayerUnitData {
            id: unit_def.id.0.clone(),
            base_stats: unit_def.base_stats,
            equipment,
            djinn_slots,
            mana_contribution: unit_def.mana_contribution,
            equipment_effects,
        }
    }

    fn build_test_battle(game_data: &GameData) -> Battle {
        let player = build_player_unit("adept", "flint", game_data);
        let encounter = game_data
            .encounters
            .get(&EncounterId("house-01".to_string()))
            .expect("house-01 encounter should exist");
        let enemy_def = game_data
            .enemies
            .get(&encounter.enemies[0].enemy_id)
            .expect("encounter enemy should exist")
            .clone();

        new_battle(
            vec![player],
            vec![EnemyUnitData { enemy_def }],
            game_data.config.clone(),
            game_data.abilities.clone(),
            game_data.djinn.clone(),
        )
    }

    #[test]
    fn current_player_ability_ids_include_good_state_djinn_abilities() {
        let game_data = load_full_game_data();
        let battle = build_test_battle(&game_data);

        let ability_ids = current_player_ability_ids(&battle, &game_data, 0);

        assert!(ability_ids.iter().any(|ability_id| ability_id.0 == "flint-stone-fist"));
        assert!(ability_ids.iter().any(|ability_id| ability_id.0 == "flint-granite-guard"));
    }

    #[test]
    fn current_player_ability_ids_remove_same_element_djinn_abilities_in_recovery() {
        let game_data = load_full_game_data();
        let mut battle = build_test_battle(&game_data);
        battle.player_units[0].djinn_slots.slots[0].state = DjinnState::Recovery;
        battle.player_units[0].djinn_slots.slots[0].recovery_turns_remaining = 1;

        let ability_ids = current_player_ability_ids(&battle, &game_data, 0);

        assert!(!ability_ids.iter().any(|ability_id| ability_id.0 == "flint-stone-fist"));
        assert!(!ability_ids.iter().any(|ability_id| ability_id.0 == "flint-granite-guard"));
    }

    #[test]
    fn current_player_ability_ids_use_counter_recovery_djinn_abilities() {
        let game_data = load_full_game_data();
        let player = build_player_unit("ranger", "flint", &game_data);
        let encounter = game_data
            .encounters
            .get(&EncounterId("house-01".to_string()))
            .expect("house-01 encounter should exist");
        let enemy_def = game_data
            .enemies
            .get(&encounter.enemies[0].enemy_id)
            .expect("encounter enemy should exist")
            .clone();
        let mut battle = new_battle(
            vec![player],
            vec![EnemyUnitData { enemy_def }],
            game_data.config.clone(),
            game_data.abilities.clone(),
            game_data.djinn.clone(),
        );
        battle.player_units[0].djinn_slots.slots[0].state = DjinnState::Recovery;
        battle.player_units[0].djinn_slots.slots[0].recovery_turns_remaining = 1;

        let ability_ids = current_player_ability_ids(&battle, &game_data, 0);

        assert!(ability_ids.iter().any(|ability_id| ability_id.0 == "flint-lava-stone"));
        assert!(ability_ids.iter().any(|ability_id| ability_id.0 == "flint-magma-shield"));
    }
}
