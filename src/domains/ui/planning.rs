//! Planning phase UI — action selection, target selection, mana tracking.
//! Wave 4: interactive planning with live mana updates.
#![allow(dead_code)]

use bevy::prelude::*;

use crate::domains::battle_engine::{
    check_battle_end, execute_round, get_planning_order, plan_action,
    plan_enemy_actions_with_ai, BattleEvent, BattleResult,
};
use crate::domains::ai::AiStrategy;
use crate::shared::{
    AbilityId, BattleAction, BattlePhase, Side, TargetRef,
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
    Ability(AbilityId, String), // id + display name
    SelectTarget(TargetRef),
    Execute,
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
                height: Val::Px(160.0),
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
                    spawn_action_button(row, "ATTACK (0 mana)", ActionChoice::Attack,
                        Color::srgb(0.267, 0.667, 0.267));

                    // ABILITY button (if unit has abilities and mana > 0)
                    if !unit.ability_ids.is_empty() {
                        spawn_action_button(row, "ABILITY", ActionChoice::Ability(
                            AbilityId(String::new()), String::new()),
                            Color::srgb(0.267, 0.533, 0.8));
                    }
                });

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
                    for aid in &unit.ability_ids {
                        if let Some(ability) = game_data.0.abilities.get(aid) {
                            if ability.mana_cost as u8 <= battle.mana_pool.current_mana {
                                let label = format!("{} ({})", ability.name, ability.mana_cost);
                                spawn_action_button(
                                    row,
                                    &label,
                                    ActionChoice::Ability(aid.clone(), ability.name.clone()),
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

            ActionChoice::Ability(id, _name) => {
                if id.0.is_empty() {
                    // "ABILITY" meta-button → show ability list
                    state.mode = PlanningMode::SelectAbility;
                } else {
                    // Specific ability selected → target selection
                    state.mode = PlanningMode::SelectTarget {
                        action_type: PendingAction::Ability(id.clone()),
                    };
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
                    // Advance to next unit
                    state.order_pos += 1;
                    if state.order_pos >= state.order.len() {
                        // All planned → execute
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
            }

            ActionChoice::Execute => {
                // Shouldn't reach here in current flow, but handle gracefully
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
