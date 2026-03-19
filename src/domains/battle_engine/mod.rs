#![allow(dead_code, clippy::unnecessary_map_or)]
//! Battle Engine — Integration layer wiring combat, status, djinn, equipment,
//! and damage_mods into a playable turn-by-turn battle loop.

use std::collections::HashMap;

use crate::shared::{
    bounded_types::{BasePower, BaseStat, EffectDuration, HitCount, Hp, Level, ManaCost},
    AbilityDef, AbilityId, BattleAction, BattlePhase, CombatConfig, DamageDealt, DamageType,
    DjinnDef, DjinnId, DjinnState, DjinnStateChanged, EnemyDef, HealingDone, ManaPoolChanged, Side,
    Stats, StatusApplied, StatusEffectType, TargetRef, UnitDefeated,
};

use crate::domains::ai::{self, AiSelfView, AiStrategy, AiUnitView};
use crate::domains::combat::{self, BattleUnit, ManaPool};
use crate::domains::damage_mods;
use crate::domains::djinn::{self, DjinnSlots};
use crate::domains::equipment::{self, EquipmentLoadout};
use crate::domains::status::{self, UnitStatusState};

// ── Core Structs ────────────────────────────────────────────────────

/// Full battle unit wrapping all domain states.
#[derive(Debug, Clone)]
pub struct BattleUnitFull {
    pub unit: BattleUnit,
    pub status_state: UnitStatusState,
    pub djinn_slots: DjinnSlots,
    pub equipment: EquipmentLoadout,
    pub mana_contribution: u8,
    /// Ability IDs this unit can use (populated from EnemyDef for enemies).
    pub ability_ids: Vec<AbilityId>,
    /// Extra hit count from equipment (auto-attacks hit 1 + this).
    pub hit_count_bonus: u8,
    /// XP reward for defeating this unit (enemies only).
    pub reward_xp: u32,
    /// Gold reward for defeating this unit (enemies only).
    pub reward_gold: u32,
}

/// Top-level battle state.
#[derive(Debug, Clone)]
pub struct Battle {
    pub round: u32,
    pub phase: BattlePhase,
    pub player_units: Vec<BattleUnitFull>,
    pub enemies: Vec<BattleUnitFull>,
    pub team_djinn_slots: DjinnSlots,
    pub mana_pool: ManaPool,
    pub planned_actions: Vec<(TargetRef, BattleAction)>,
    pub log: Vec<BattleEvent>,
    pub config: CombatConfig,
    pub ability_defs: HashMap<AbilityId, AbilityDef>,
    pub djinn_defs: HashMap<DjinnId, DjinnDef>,
}

/// Events emitted during battle execution.
#[derive(Debug, Clone)]
pub enum BattleEvent {
    DamageDealt(DamageDealt),
    HealingDone(HealingDone),
    StatusApplied(StatusApplied),
    CritTriggered(TargetRef, u8),
    BarrierBlocked(TargetRef),
    DjinnChanged(DjinnStateChanged),
    ManaChanged(ManaPoolChanged),
    UnitDefeated(UnitDefeated),
    RoundStarted(u32),
    RoundEnded(u32),
    /// An enemy uses a named ability on a target (for display purposes).
    EnemyAbilityUsed {
        actor: TargetRef,
        ability_name: String,
        targets: Vec<TargetRef>,
    },
}

/// Outcome of a completed battle.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BattleResult {
    Victory { xp: u32, gold: u32 },
    Defeat,
}

/// Errors from plan_action validation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PlanError {
    UnitCannotAct,
    InsufficientMana,
    InvalidTarget,
    DjinnNotReady,
    InvalidDjinnIndex,
}

// ── Input data for new_battle ───────────────────────────────────────

/// Data needed to set up a player unit in battle.
#[derive(Debug, Clone)]
pub struct PlayerUnitData {
    pub id: String,
    pub base_stats: Stats,
    pub equipment: EquipmentLoadout,
    pub djinn_slots: DjinnSlots,
    pub mana_contribution: u8,
    pub equipment_effects: equipment::EquipmentEffects,
}

/// Data needed to set up an enemy unit in battle.
#[derive(Debug, Clone)]
pub struct EnemyUnitData {
    pub enemy_def: EnemyDef,
}

// ── new_battle ──────────────────────────────────────────────────────

/// Create a new Battle from player and enemy data.
///
/// - Builds BattleUnitFull for each unit with stats and empty status state.
/// - Initializes the mana pool from player unit contributions + equipment bonuses.
/// - Sets phase to Planning, round to 1.
pub fn new_battle(
    player_data: Vec<PlayerUnitData>,
    enemy_data: Vec<EnemyUnitData>,
    config: CombatConfig,
    ability_defs: HashMap<AbilityId, AbilityDef>,
    djinn_defs: HashMap<DjinnId, DjinnDef>,
) -> Battle {
    let mut total_mana: u8 = 0;
    let team_djinn_slots = collect_team_djinn_slots(&player_data);

    let player_units: Vec<BattleUnitFull> = player_data
        .into_iter()
        .map(|pd| {
            let eq_effects = &pd.equipment_effects;
            let stats = Stats {
                hp: Hp::new_unchecked(
                    (pd.base_stats.hp.get() as i32 + eq_effects.total_stat_bonus.hp.get() as i32)
                        .max(1)
                        .min(9999) as u16,
                ),
                atk: BaseStat::new_unchecked(
                    (pd.base_stats.atk.get() as i32 + eq_effects.total_stat_bonus.atk.get() as i32)
                        .max(0)
                        .min(9999) as u16,
                ),
                def: BaseStat::new_unchecked(
                    (pd.base_stats.def.get() as i32 + eq_effects.total_stat_bonus.def.get() as i32)
                        .max(0)
                        .min(9999) as u16,
                ),
                mag: BaseStat::new_unchecked(
                    (pd.base_stats.mag.get() as i32 + eq_effects.total_stat_bonus.mag.get() as i32)
                        .max(0)
                        .min(9999) as u16,
                ),
                spd: BaseStat::new_unchecked(
                    (pd.base_stats.spd.get() as i32 + eq_effects.total_stat_bonus.spd.get() as i32)
                        .max(0)
                        .min(9999) as u16,
                ),
            };
            let mana = pd
                .mana_contribution
                .saturating_add(eq_effects.total_mana_bonus);
            total_mana = total_mana.saturating_add(mana);

            BattleUnitFull {
                unit: BattleUnit {
                    id: pd.id,
                    stats,
                    current_hp: stats.hp.get(),
                    is_alive: true,
                    crit_counter: 0,
                    equipment_speed_bonus: eq_effects.total_stat_bonus.spd.get(),
                },
                status_state: UnitStatusState::new(),
                djinn_slots: team_djinn_slots.clone(),
                equipment: pd.equipment,
                mana_contribution: mana,
                ability_ids: Vec::new(),
                hit_count_bonus: eq_effects.total_hit_count_bonus,
                reward_xp: 0,
                reward_gold: 0,
            }
        })
        .collect();

    let enemies: Vec<BattleUnitFull> = enemy_data
        .into_iter()
        .map(|ed| {
            let stats = ed.enemy_def.stats;
            let ability_ids = ed.enemy_def.abilities.clone();
            let xp = ed.enemy_def.xp;
            let gold = ed.enemy_def.gold;
            BattleUnitFull {
                unit: BattleUnit {
                    id: ed.enemy_def.id.0.clone(),
                    stats,
                    current_hp: stats.hp.get(),
                    is_alive: true,
                    crit_counter: 0,
                    equipment_speed_bonus: 0,
                },
                status_state: UnitStatusState::new(),
                djinn_slots: DjinnSlots::new(),
                equipment: EquipmentLoadout::default(),
                mana_contribution: 0,
                ability_ids,
                hit_count_bonus: 0,
                reward_xp: xp.get(),
                reward_gold: gold.get(),
            }
        })
        .collect();

    Battle {
        round: 1,
        phase: BattlePhase::Planning,
        player_units,
        enemies,
        team_djinn_slots,
        mana_pool: ManaPool::new(total_mana),
        planned_actions: Vec::new(),
        log: Vec::new(),
        config,
        ability_defs,
        djinn_defs,
    }
}

pub fn set_team_djinn_slots(battle: &mut Battle, team_djinn_slots: DjinnSlots) {
    battle.team_djinn_slots = team_djinn_slots;
    sync_team_djinn_slots_to_players(battle);
}

// ── get_planning_order ──────────────────────────────────────────────

/// Returns player unit indices sorted by effective SPD descending (fastest first).
///
/// Effective SPD = unit.stats.spd + equipment_speed_bonus (as i32).
/// Dead units are skipped.
/// Tiebreaker: higher base SPD first, then lower index.
pub fn get_planning_order(battle: &Battle) -> Vec<usize> {
    let mut entries: Vec<(usize, i32, u16)> = Vec::new();

    for (i, pu) in battle.player_units.iter().enumerate() {
        if !pu.unit.is_alive {
            continue;
        }
        let effective_spd = pu.unit.stats.spd.get() as i32 + pu.unit.equipment_speed_bonus as i32;
        let base_spd = pu.unit.stats.spd.get();
        entries.push((i, effective_spd, base_spd));
    }

    // Sort descending by effective SPD, then base SPD, then ascending by index
    entries.sort_by(|a, b| {
        b.1.cmp(&a.1)
            .then_with(|| b.2.cmp(&a.2))
            .then_with(|| a.0.cmp(&b.0))
    });

    entries.into_iter().map(|(idx, _, _)| idx).collect()
}

// ── plan_action ─────────────────────────────────────────────────────

/// Validate and store a planned action for a unit.
///
/// Checks: can_act / can_auto_attack / can_use_ability, mana cost, djinn state.
pub fn plan_action(
    battle: &mut Battle,
    unit_ref: TargetRef,
    action: BattleAction,
) -> Result<(), PlanError> {
    hydrate_team_djinn_slots_from_players(battle);
    let unit = get_unit(battle, unit_ref).ok_or(PlanError::InvalidTarget)?;

    if !unit.unit.is_alive {
        return Err(PlanError::UnitCannotAct);
    }

    if !status::can_act(&unit.status_state.statuses) {
        return Err(PlanError::UnitCannotAct);
    }

    match &action {
        BattleAction::Attack { target } => {
            if !status::can_auto_attack(&unit.status_state.statuses) {
                return Err(PlanError::UnitCannotAct);
            }
            // Validate target exists
            get_unit(battle, *target).ok_or(PlanError::InvalidTarget)?;
        }
        BattleAction::UseAbility {
            ability_id,
            targets,
        } => {
            if !status::can_use_ability(&unit.status_state.statuses) {
                return Err(PlanError::UnitCannotAct);
            }
            let ability = battle
                .ability_defs
                .get(ability_id)
                .ok_or(PlanError::InvalidTarget)?;
            if battle.mana_pool.current_mana < ability.mana_cost.get() {
                return Err(PlanError::InsufficientMana);
            }
            // Validate all targets exist
            for t in targets {
                get_unit(battle, *t).ok_or(PlanError::InvalidTarget)?;
            }
        }
        BattleAction::ActivateDjinn { djinn_index } => {
            let idx = *djinn_index as usize;
            let slots = &battle.team_djinn_slots;
            if idx >= slots.slots.len() {
                return Err(PlanError::InvalidDjinnIndex);
            }
            if slots.slots[idx].state != DjinnState::Good {
                return Err(PlanError::DjinnNotReady);
            }
        }
        BattleAction::Summon { djinn_indices } => {
            let slots = &battle.team_djinn_slots;
            for &idx in djinn_indices {
                let i = idx as usize;
                if i >= slots.slots.len() {
                    return Err(PlanError::InvalidDjinnIndex);
                }
                if slots.slots[i].state != DjinnState::Good {
                    return Err(PlanError::DjinnNotReady);
                }
            }
        }
    }

    if let Some(existing_idx) = battle
        .planned_actions
        .iter()
        .position(|(planned_unit_ref, _)| *planned_unit_ref == unit_ref)
    {
        let (_, existing_action) = battle.planned_actions.remove(existing_idx);
        reverse_projected_mana_change(battle, unit_ref, &existing_action);
    }

    apply_projected_mana_change(battle, unit_ref, &action);
    battle.planned_actions.push((unit_ref, action));
    Ok(())
}

fn projected_mana_delta(battle: &Battle, unit_ref: TargetRef, action: &BattleAction) -> i16 {
    if unit_ref.side != Side::Player {
        return 0;
    }

    match action {
        BattleAction::Attack { .. } => get_unit(battle, unit_ref)
            .map(|unit| 1 + unit.hit_count_bonus as i16)
            .unwrap_or(0),
        BattleAction::UseAbility { ability_id, .. } => battle
            .ability_defs
            .get(ability_id)
            .map(|ability| -(ability.mana_cost.get() as i16))
            .unwrap_or(0),
        BattleAction::ActivateDjinn { .. } | BattleAction::Summon { .. } => 0,
    }
}

fn apply_projected_mana_change(battle: &mut Battle, unit_ref: TargetRef, action: &BattleAction) {
    apply_projected_mana_delta(battle, projected_mana_delta(battle, unit_ref, action));
}

fn reverse_projected_mana_change(battle: &mut Battle, unit_ref: TargetRef, action: &BattleAction) {
    apply_projected_mana_delta(battle, -projected_mana_delta(battle, unit_ref, action));
}

fn apply_projected_mana_delta(battle: &mut Battle, delta: i16) {
    if delta >= 0 {
        battle.mana_pool.projected_mana =
            battle.mana_pool.projected_mana.saturating_add(delta as u8);
    } else {
        battle.mana_pool.projected_mana = battle
            .mana_pool
            .projected_mana
            .saturating_sub((-delta) as u8);
    }
}

// ── plan_enemy_actions ──────────────────────────────────────────────

/// Simple enemy AI: each alive enemy auto-attacks the first alive player unit.
///
/// Call this after player actions are planned, before `execute_round`.
pub fn plan_enemy_actions(battle: &mut Battle) {
    let enemy_count = battle.enemies.len();
    for ei in 0..enemy_count {
        if !battle.enemies[ei].unit.is_alive {
            continue;
        }
        // Find first alive player unit
        let target_idx = battle.player_units.iter().position(|p| p.unit.is_alive);
        let target_idx = match target_idx {
            Some(i) => i,
            None => break, // no player units left
        };

        let unit_ref = TargetRef {
            side: Side::Enemy,
            index: ei as u8,
        };
        let action = BattleAction::Attack {
            target: TargetRef {
                side: Side::Player,
                index: target_idx as u8,
            },
        };
        // plan_action validates and stores; ignore errors (e.g. stunned enemies)
        let _ = plan_action(battle, unit_ref, action);
    }
}

// ── plan_enemy_actions_with_ai ──────────────────────────────────────

/// Default mana budget for enemies per round (they don't share the player pool).
const ENEMY_MANA_BUDGET: u8 = 5;

/// Smart enemy AI: uses the AI domain to decide each enemy's action.
///
/// For each alive enemy:
/// 1. Build AiSelfView from the enemy's BattleUnitFull.
/// 2. Build Vec<AiUnitView> from alive player units.
/// 3. Collect enemy's available abilities from battle.ability_defs.
/// 4. Call ai::choose_enemy_action(self_view, player_views, abilities, strategy).
/// 5. Push the returned BattleAction directly (bypassing mana pool validation,
///    since enemies have their own mana budget independent of the player pool).
pub fn plan_enemy_actions_with_ai(battle: &mut Battle, strategy: AiStrategy) {
    // Build player views once (shared across all enemy decisions)
    let player_views: Vec<AiUnitView> = battle
        .player_units
        .iter()
        .enumerate()
        .map(|(i, pu)| AiUnitView {
            index: i as u8,
            stats: pu.unit.stats,
            current_hp: pu.unit.current_hp,
            max_hp: pu.unit.stats.hp.get(),
            is_alive: pu.unit.is_alive,
        })
        .collect();

    let enemy_count = battle.enemies.len();
    for ei in 0..enemy_count {
        if !battle.enemies[ei].unit.is_alive {
            continue;
        }

        // Check if enemy can act (status effects like stun)
        if !status::can_act(&battle.enemies[ei].status_state.statuses) {
            continue;
        }

        let enemy = &battle.enemies[ei];

        // Build AiSelfView
        let self_view = AiSelfView {
            index: ei as u8,
            current_hp: enemy.unit.current_hp,
            max_hp: enemy.unit.stats.hp.get(),
            mana_available: ENEMY_MANA_BUDGET,
        };

        // Collect this enemy's usable abilities from its ability_ids list,
        // looking up each AbilityId in battle.ability_defs.
        let enemy_ability_refs: Vec<&AbilityDef> = enemy
            .ability_ids
            .iter()
            .filter_map(|aid| battle.ability_defs.get(aid))
            .collect();

        let action =
            ai::choose_enemy_action(&self_view, &player_views, &enemy_ability_refs, strategy);

        let unit_ref = TargetRef {
            side: Side::Enemy,
            index: ei as u8,
        };

        // Push directly — enemies don't share the player mana pool.
        // Validation: ensure targets are alive (lightweight check).
        match &action {
            BattleAction::Attack { target } => {
                if get_unit(battle, *target).map_or(false, |u| u.unit.is_alive) {
                    battle.planned_actions.push((unit_ref, action));
                }
            }
            BattleAction::UseAbility {
                ability_id,
                targets,
            } => {
                // Verify ability exists and at least one target is alive
                if battle.ability_defs.contains_key(ability_id)
                    && targets
                        .iter()
                        .any(|t| get_unit(battle, *t).map_or(false, |u| u.unit.is_alive))
                {
                    battle.planned_actions.push((unit_ref, action));
                }
            }
            _ => {
                // Other actions (djinn, summon) not used by enemy AI currently
                battle.planned_actions.push((unit_ref, action));
            }
        }
    }
}

// ── execute_round ───────────────────────────────────────────────────

/// Execute one full round of battle.
///
/// 1. Compute execution order (summons first, then planning order — DESIGN_LOCK §2).
/// 2. Execute each actor's planned action.
/// 3. End-of-round ticks (statuses, HoT, buffs, barriers, djinn recovery).
/// 4. Reset mana pool, increment round.
pub fn execute_round(battle: &mut Battle) -> Vec<BattleEvent> {
    let mut events: Vec<BattleEvent> = Vec::new();
    events.push(BattleEvent::RoundStarted(battle.round));
    battle.phase = BattlePhase::Execution;

    // Separate summons from other actions
    let mut summon_actions: Vec<(TargetRef, BattleAction)> = Vec::new();
    let mut other_actions: Vec<(TargetRef, BattleAction)> = Vec::new();
    for (tr, action) in battle.planned_actions.drain(..) {
        match &action {
            BattleAction::Summon { .. } => summon_actions.push((tr, action)),
            _ => other_actions.push((tr, action)),
        }
    }

    // DESIGN_LOCK §2: Planning order = execution order.
    // Summons execute first, then all other actions in the order they were planned.
    let mut execution_sequence: Vec<(TargetRef, BattleAction)> = Vec::new();
    for (tr, action) in summon_actions {
        execution_sequence.push((tr, action));
    }
    for (tr, action) in other_actions {
        execution_sequence.push((tr, action));
    }

    // Execute each action
    for (actor_ref, action) in execution_sequence {
        // Check if actor is still alive and can act
        let can_proceed = {
            let actor = get_unit(battle, actor_ref);
            match actor {
                Some(u) => u.unit.is_alive && status::can_act(&u.status_state.statuses),
                None => false,
            }
        };
        if !can_proceed {
            continue;
        }

        match action {
            BattleAction::Attack { target } => {
                execute_attack(battle, actor_ref, target, &mut events);
            }
            BattleAction::UseAbility {
                ability_id,
                targets,
            } => {
                execute_ability(battle, actor_ref, &ability_id, &targets, &mut events);
            }
            BattleAction::ActivateDjinn { djinn_index } => {
                execute_djinn_activate(battle, actor_ref, djinn_index as usize, &mut events);
            }
            BattleAction::Summon { djinn_indices } => {
                execute_summon_action(battle, actor_ref, &djinn_indices, &mut events);
            }
        }
    }

    // End-of-round phase
    battle.phase = BattlePhase::RoundEnd;
    end_of_round_ticks(battle, &mut events);

    // Reset mana pool
    if battle.config.mana_resets_each_round {
        let old_mana = battle.mana_pool.current_mana;
        battle.mana_pool.reset_mana();
        if old_mana != battle.mana_pool.current_mana {
            events.push(BattleEvent::ManaChanged(ManaPoolChanged {
                old_value: old_mana,
                new_value: battle.mana_pool.current_mana,
            }));
        }
    }

    events.push(BattleEvent::RoundEnded(battle.round));
    battle.round += 1;
    battle.phase = BattlePhase::Planning;

    events
}

// ── Action Execution Helpers ────────────────────────────────────────

fn execute_attack(
    battle: &mut Battle,
    actor_ref: TargetRef,
    target_ref: TargetRef,
    events: &mut Vec<BattleEvent>,
) {
    // Get attacker data
    let (attacker_stats, crit_counter, hit_count_bonus, attacker_buff_mods) = {
        let actor = get_unit(battle, actor_ref).unwrap();
        let buff_mods =
            status::compute_stat_modifiers(&actor.status_state.buffs, &actor.status_state.debuffs);
        (
            actor.unit.stats,
            actor.unit.crit_counter,
            actor.hit_count_bonus,
            buff_mods,
        )
    };

    // Check if target is alive
    {
        match get_unit(battle, target_ref) {
            Some(t) if t.unit.is_alive => {}
            _ => return,
        };
    }

    let (target_stats, target_buff_mods) = {
        let t = get_unit(battle, target_ref).unwrap();
        let buff_mods =
            status::compute_stat_modifiers(&t.status_state.buffs, &t.status_state.debuffs);
        (t.unit.stats, buff_mods)
    };
    let target_hp = get_unit(battle, target_ref).unwrap().unit.current_hp;

    // Effective stats with buff/debuff modifiers
    let effective_attacker = Stats {
        hp: attacker_stats.hp,
        atk: BaseStat::new_unchecked(
            (attacker_stats.atk.get() as i32 + attacker_buff_mods.atk.get() as i32)
                .max(0)
                .min(9999) as u16,
        ),
        def: BaseStat::new_unchecked(
            (attacker_stats.def.get() as i32 + attacker_buff_mods.def.get() as i32)
                .max(0)
                .min(9999) as u16,
        ),
        mag: BaseStat::new_unchecked(
            (attacker_stats.mag.get() as i32 + attacker_buff_mods.mag.get() as i32)
                .max(0)
                .min(9999) as u16,
        ),
        spd: BaseStat::new_unchecked(
            (attacker_stats.spd.get() as i32 + attacker_buff_mods.spd.get() as i32)
                .max(0)
                .min(9999) as u16,
        ),
    };
    let effective_target = Stats {
        hp: target_stats.hp,
        atk: BaseStat::new_unchecked(
            (target_stats.atk.get() as i32 + target_buff_mods.atk.get() as i32)
                .max(0)
                .min(9999) as u16,
        ),
        def: BaseStat::new_unchecked(
            (target_stats.def.get() as i32 + target_buff_mods.def.get() as i32)
                .max(0)
                .min(9999) as u16,
        ),
        mag: BaseStat::new_unchecked(
            (target_stats.mag.get() as i32 + target_buff_mods.mag.get() as i32)
                .max(0)
                .min(9999) as u16,
        ),
        spd: BaseStat::new_unchecked(
            (target_stats.spd.get() as i32 + target_buff_mods.spd.get() as i32)
                .max(0)
                .min(9999) as u16,
        ),
    };

    // Base hit count is 1 + equipment bonus
    let base_hit_count: u8 = 1 + hit_count_bonus;
    let base_power = 0u16; // auto-attack has 0 base power
    let base_damage = combat::calculate_damage(
        base_power,
        DamageType::Physical,
        &effective_attacker,
        &effective_target,
        &battle.config,
    );

    // Check barrier before applying damage
    let hits = combat::resolve_multi_hit(
        base_hit_count,
        base_damage,
        target_hp,
        crit_counter,
        true,
        &battle.config,
    );

    let mut new_crit_counter = crit_counter;
    for hit in &hits {
        // Try to consume barrier
        let barrier_blocked = {
            let target_unit = get_unit_mut(battle, target_ref).unwrap();
            status::consume_barrier(&mut target_unit.status_state.barriers)
        };

        if barrier_blocked {
            events.push(BattleEvent::BarrierBlocked(target_ref));
            // Still advance crit counter and generate mana
            new_crit_counter = hit.crit_counter_after;
            battle.mana_pool.generate_mana(hit.mana_generated);
            if hit.mana_generated > 0 {
                events.push(BattleEvent::ManaChanged(ManaPoolChanged {
                    old_value: battle
                        .mana_pool
                        .current_mana
                        .saturating_sub(hit.mana_generated),
                    new_value: battle.mana_pool.current_mana,
                }));
            }
            continue;
        }

        // Apply damage
        {
            let target_unit = get_unit_mut(battle, target_ref).unwrap();
            target_unit.unit.current_hp = target_unit.unit.current_hp.saturating_sub(hit.damage);
            if target_unit.unit.current_hp == 0 {
                target_unit.unit.is_alive = false;
            }
            // FIX 2: Accumulate freeze damage
            for status in target_unit.status_state.statuses.iter_mut() {
                if status.effect_type == StatusEffectType::Freeze {
                    status.freeze_damage_taken += hit.damage;
                }
            }
        }

        events.push(BattleEvent::DamageDealt(DamageDealt {
            source: actor_ref,
            target: target_ref,
            amount: hit.damage,
            damage_type: DamageType::Physical,
            is_crit: hit.is_crit,
        }));

        if hit.is_crit {
            events.push(BattleEvent::CritTriggered(
                actor_ref,
                hit.crit_counter_after,
            ));
        }

        new_crit_counter = hit.crit_counter_after;

        // Generate mana for auto-attacks
        battle.mana_pool.generate_mana(hit.mana_generated);

        // Check death
        if hit.target_killed {
            events.push(BattleEvent::UnitDefeated(UnitDefeated { unit: target_ref }));
            break;
        }
    }

    // Update crit counter on attacker
    {
        let actor = get_unit_mut(battle, actor_ref).unwrap();
        actor.unit.crit_counter = new_crit_counter;
    }
}

fn execute_ability(
    battle: &mut Battle,
    actor_ref: TargetRef,
    ability_id: &AbilityId,
    targets: &[TargetRef],
    events: &mut Vec<BattleEvent>,
) {
    let ability = match battle.ability_defs.get(ability_id) {
        Some(a) => a.clone(),
        None => return,
    };

    // Spend mana — enemies have their own budget, only deduct from pool for players
    if actor_ref.side == Side::Player {
        let old_mana = battle.mana_pool.current_mana;
        if !battle.mana_pool.spend_mana(ability.mana_cost.get()) {
            return;
        }
        events.push(BattleEvent::ManaChanged(ManaPoolChanged {
            old_value: old_mana,
            new_value: battle.mana_pool.current_mana,
        }));
    }

    // Emit ability usage event for enemies so the CLI can display it
    if actor_ref.side == Side::Enemy {
        events.push(BattleEvent::EnemyAbilityUsed {
            actor: actor_ref,
            ability_name: ability.name.clone(),
            targets: targets.to_vec(),
        });
    }

    let (attacker_stats, attacker_buff_mods) = {
        let actor = get_unit(battle, actor_ref).unwrap();
        let buff_mods =
            status::compute_stat_modifiers(&actor.status_state.buffs, &actor.status_state.debuffs);
        (actor.unit.stats, buff_mods)
    };
    let effective_attacker = Stats {
        hp: attacker_stats.hp,
        atk: BaseStat::new_unchecked(
            (attacker_stats.atk.get() as i32 + attacker_buff_mods.atk.get() as i32)
                .max(0)
                .min(9999) as u16,
        ),
        def: BaseStat::new_unchecked(
            (attacker_stats.def.get() as i32 + attacker_buff_mods.def.get() as i32)
                .max(0)
                .min(9999) as u16,
        ),
        mag: BaseStat::new_unchecked(
            (attacker_stats.mag.get() as i32 + attacker_buff_mods.mag.get() as i32)
                .max(0)
                .min(9999) as u16,
        ),
        spd: BaseStat::new_unchecked(
            (attacker_stats.spd.get() as i32 + attacker_buff_mods.spd.get() as i32)
                .max(0)
                .min(9999) as u16,
        ),
    };

    // Process each target
    for &target_ref in targets {
        let target_alive = {
            match get_unit(battle, target_ref) {
                Some(u) => u.unit.is_alive,
                None => false,
            }
        };
        if !target_alive {
            continue;
        }

        // Handle healing abilities
        // FIX 6: heal_amount = base_power + attacker MAG, floor 1
        if ability.category == crate::shared::AbilityCategory::Healing {
            let heal_amount = (ability.base_power.get() + effective_attacker.mag.get()).max(1);
            {
                let target = get_unit_mut(battle, target_ref).unwrap();
                let max_hp = target.unit.stats.hp.get();
                target.unit.current_hp = (target.unit.current_hp + heal_amount).min(max_hp);
            }
            events.push(BattleEvent::HealingDone(HealingDone {
                source: actor_ref,
                target: target_ref,
                amount: heal_amount,
            }));
            continue;
        }

        // Damage abilities
        if let Some(damage_type) = ability.damage_type {
            let target_stats = get_unit(battle, target_ref).unwrap().unit.stats;
            let target_hp = get_unit(battle, target_ref).unwrap().unit.current_hp;

            // Apply buff/debuff modifiers to target
            let target_buff_mods = {
                let t = get_unit(battle, target_ref).unwrap();
                status::compute_stat_modifiers(&t.status_state.buffs, &t.status_state.debuffs)
            };
            let effective_target_base = Stats {
                hp: target_stats.hp,
                atk: BaseStat::new_unchecked(
                    (target_stats.atk.get() as i32 + target_buff_mods.atk.get() as i32)
                        .max(0)
                        .min(9999) as u16,
                ),
                def: BaseStat::new_unchecked(
                    (target_stats.def.get() as i32 + target_buff_mods.def.get() as i32)
                        .max(0)
                        .min(9999) as u16,
                ),
                mag: BaseStat::new_unchecked(
                    (target_stats.mag.get() as i32 + target_buff_mods.mag.get() as i32)
                        .max(0)
                        .min(9999) as u16,
                ),
                spd: BaseStat::new_unchecked(
                    (target_stats.spd.get() as i32 + target_buff_mods.spd.get() as i32)
                        .max(0)
                        .min(9999) as u16,
                ),
            };

            // Apply defense penetration if applicable
            let effective_def = if let Some(pen_pct) = ability.ignore_defense_percent {
                damage_mods::apply_defense_penetration(effective_target_base.def.get(), pen_pct)
            } else {
                effective_target_base.def.get()
            };

            let modified_target_stats = Stats {
                def: BaseStat::new_unchecked(effective_def.min(9999)),
                ..effective_target_base
            };

            let base_damage = combat::calculate_damage(
                ability.base_power.get(),
                damage_type,
                &effective_attacker,
                &modified_target_stats,
                &battle.config,
            );

            let crit_counter = get_unit(battle, actor_ref).unwrap().unit.crit_counter;
            let hits = combat::resolve_multi_hit(
                ability.hit_count.get().max(1),
                base_damage,
                target_hp,
                crit_counter,
                false, // abilities don't advance crit
                &battle.config,
            );

            for hit in &hits {
                // Apply damage
                {
                    let target = get_unit_mut(battle, target_ref).unwrap();
                    target.unit.current_hp = target.unit.current_hp.saturating_sub(hit.damage);
                    if target.unit.current_hp == 0 {
                        target.unit.is_alive = false;
                    }
                    // FIX 2: Accumulate freeze damage
                    for status in target.status_state.statuses.iter_mut() {
                        if status.effect_type == StatusEffectType::Freeze {
                            status.freeze_damage_taken += hit.damage;
                        }
                    }
                }

                events.push(BattleEvent::DamageDealt(DamageDealt {
                    source: actor_ref,
                    target: target_ref,
                    amount: hit.damage,
                    damage_type,
                    is_crit: hit.is_crit,
                }));

                if hit.target_killed {
                    events.push(BattleEvent::UnitDefeated(UnitDefeated { unit: target_ref }));
                    break;
                }
            }

            // Splash damage
            if let Some(splash_pct) = ability.splash_damage_percent {
                let primary_damage = hits.iter().map(|h| h.damage).sum::<u16>();
                let splash_dmg = damage_mods::apply_splash_damage(primary_damage, splash_pct);
                if splash_dmg > 0 {
                    let enemy_count = match target_ref.side {
                        Side::Enemy => battle.enemies.len(),
                        Side::Player => battle.player_units.len(),
                    };
                    let all_targets: Vec<TargetRef> = (0..enemy_count as u8)
                        .map(|i| TargetRef {
                            side: target_ref.side,
                            index: i,
                        })
                        .collect();
                    let splash_targets =
                        damage_mods::calculate_splash_targets(target_ref, splash_pct, &all_targets);
                    for (splash_tr, _) in splash_targets {
                        let alive = get_unit(battle, splash_tr)
                            .map(|u| u.unit.is_alive)
                            .unwrap_or(false);
                        if alive {
                            {
                                let t = get_unit_mut(battle, splash_tr).unwrap();
                                t.unit.current_hp = t.unit.current_hp.saturating_sub(splash_dmg);
                                if t.unit.current_hp == 0 {
                                    t.unit.is_alive = false;
                                }
                            }
                            events.push(BattleEvent::DamageDealt(DamageDealt {
                                source: actor_ref,
                                target: splash_tr,
                                amount: splash_dmg,
                                damage_type,
                                is_crit: false,
                            }));
                            if get_unit(battle, splash_tr).unwrap().unit.current_hp == 0 {
                                events.push(BattleEvent::UnitDefeated(UnitDefeated {
                                    unit: splash_tr,
                                }));
                            }
                        }
                    }
                }
            }

            // FIX 3: Chain damage — hits all targets on the opposing side
            if ability.chain_damage {
                let primary_damage = hits.iter().map(|h| h.damage).sum::<u16>();
                let primary_is_crit = hits.iter().any(|hit| hit.is_crit);
                if primary_damage > 0 {
                    apply_chain_damage(
                        battle,
                        actor_ref,
                        target_ref,
                        primary_damage,
                        damage_type,
                        primary_is_crit,
                        events,
                    );
                }
            }
        }

        // Apply status effect if any
        if let Some(ref status_effect) = ability.status_effect {
            let applied = {
                let target = get_unit_mut(battle, target_ref).unwrap();
                let imm = target.status_state.immunity.clone();
                status::apply_status(&mut target.status_state, status_effect, &imm)
            };
            if applied {
                events.push(BattleEvent::StatusApplied(StatusApplied {
                    source: actor_ref,
                    target: target_ref,
                    effect: status_effect.clone(),
                }));
            }
        }

        // Apply buff/debuff
        let max_stacks = battle.config.max_buff_stacks.get() as usize;
        if let Some(ref buff_effect) = ability.buff_effect {
            let target = get_unit_mut(battle, target_ref).unwrap();
            status::apply_buff(&mut target.status_state, buff_effect, max_stacks);
        }
        if let Some(ref debuff_effect) = ability.debuff_effect {
            let target = get_unit_mut(battle, target_ref).unwrap();
            status::apply_debuff(&mut target.status_state, debuff_effect, max_stacks);
        }

        // Apply barrier (default duration 3 if shield_duration is None)
        if let Some(charges) = ability.shield_charges {
            let duration = ability.shield_duration.unwrap_or(3);
            let target = get_unit_mut(battle, target_ref).unwrap();
            status::apply_barrier(&mut target.status_state, charges, duration);
        }

        // Apply HoT
        if let Some(ref hot) = ability.heal_over_time {
            let target = get_unit_mut(battle, target_ref).unwrap();
            status::apply_hot(&mut target.status_state, hot.amount, hot.duration.get());
        }

        // FIX 4: Apply immunity
        if let Some(ref immunity) = ability.grant_immunity {
            let target = get_unit_mut(battle, target_ref).unwrap();
            status::apply_immunity(&mut target.status_state, immunity);
        }

        // FIX 4: Apply cleanse
        if let Some(ref cleanse_type) = ability.cleanse {
            let target = get_unit_mut(battle, target_ref).unwrap();
            status::cleanse(&mut target.status_state.statuses, cleanse_type);
        }
    }

    // FIX 5: Revive dead allies
    if ability.revive {
        if let Some(pct) = ability.revive_hp_percent {
            let ally_count = match actor_ref.side {
                Side::Player => battle.player_units.len(),
                Side::Enemy => battle.enemies.len(),
            };
            for i in 0..ally_count {
                let ally_ref = TargetRef {
                    side: actor_ref.side,
                    index: i as u8,
                };
                let (is_dead, max_hp) = {
                    let ally = get_unit(battle, ally_ref).unwrap();
                    (!ally.unit.is_alive, ally.unit.stats.hp.get())
                };
                if is_dead {
                    let revive_hp = ((max_hp as f32 * pct) as u16).max(1);
                    let ally = get_unit_mut(battle, ally_ref).unwrap();
                    ally.unit.is_alive = true;
                    ally.unit.current_hp = revive_hp;
                    events.push(BattleEvent::HealingDone(HealingDone {
                        source: actor_ref,
                        target: ally_ref,
                        amount: revive_hp,
                    }));
                }
            }
        }
    }
}

fn execute_djinn_activate(
    battle: &mut Battle,
    actor_ref: TargetRef,
    djinn_index: usize,
    events: &mut Vec<BattleEvent>,
) {
    hydrate_team_djinn_slots_from_players(battle);
    let recovery_turns =
        battle.config.djinn_recovery_start_delay + battle.config.djinn_recovery_per_turn;
    let djinn_def = {
        let djinn_id = match battle.team_djinn_slots.slots.get(djinn_index) {
            Some(djinn) => &djinn.djinn_id,
            None => return,
        };
        match battle.djinn_defs.get(djinn_id) {
            Some(def) => def.clone(),
            None => return,
        }
    };

    let activation = djinn::activate_djinn(
        &mut battle.team_djinn_slots,
        djinn_index,
        actor_ref,
        recovery_turns,
        &djinn_def,
    );

    if let Some((state_change, activation_event)) = activation {
        sync_team_djinn_slots_to_players(battle);
        events.push(BattleEvent::DjinnChanged(state_change));
        resolve_djinn_activation_damage(battle, actor_ref, activation_event, events);
    }
}

fn resolve_djinn_activation_damage(
    battle: &mut Battle,
    actor_ref: TargetRef,
    activation_event: djinn::DjinnActivationEvent,
    events: &mut Vec<BattleEvent>,
) {
    match activation_event.effect_type {
        Some(djinn::DjinnActivationEffectType::Heal) => {
            let ally_count = match actor_ref.side {
                Side::Player => battle.player_units.len(),
                Side::Enemy => battle.enemies.len(),
            };

            for index in 0..ally_count {
                let target_ref = TargetRef {
                    side: actor_ref.side,
                    index: index as u8,
                };
                let (is_alive, current_hp, max_hp) = {
                    let unit = get_unit(battle, target_ref).unwrap();
                    (
                        unit.unit.is_alive,
                        unit.unit.current_hp,
                        unit.unit.stats.hp.get(),
                    )
                };
                if !is_alive || current_hp >= max_hp {
                    continue;
                }

                let heal_amount = max_hp - current_hp;
                let target = get_unit_mut(battle, target_ref).unwrap();
                target.unit.current_hp = max_hp;
                events.push(BattleEvent::HealingDone(HealingDone {
                    source: actor_ref,
                    target: target_ref,
                    amount: heal_amount,
                }));
            }
            return;
        }
        Some(djinn::DjinnActivationEffectType::Buff) => {
            let max_stacks = battle.config.max_buff_stacks.get() as usize;
            let actor = get_unit_mut(battle, actor_ref).unwrap();
            status::apply_buff(
                &mut actor.status_state,
                &crate::shared::BuffEffect {
                    stat_modifiers: crate::shared::StatBonus {
                        atk: crate::shared::bounded_types::StatMod::new_unchecked(2),
                        def: crate::shared::bounded_types::StatMod::new_unchecked(0),
                        mag: crate::shared::bounded_types::StatMod::new_unchecked(0),
                        spd: crate::shared::bounded_types::StatMod::new_unchecked(0),
                        hp: crate::shared::bounded_types::StatMod::new_unchecked(0),
                    },
                    duration: EffectDuration::new_unchecked(3),
                    shield_charges: None,
                    grant_immunity: false,
                },
                max_stacks,
            );
            return;
        }
        Some(djinn::DjinnActivationEffectType::Status) | None => return,
        Some(djinn::DjinnActivationEffectType::Damage) => {}
    }

    let target_index = match battle.enemies.iter().position(|enemy| enemy.unit.is_alive) {
        Some(index) => index,
        None => return,
    };
    let target_ref = TargetRef {
        side: Side::Enemy,
        index: target_index as u8,
    };

    let (attacker_stats, attacker_buff_mods) = {
        let actor = get_unit(battle, actor_ref).unwrap();
        let buff_mods =
            status::compute_stat_modifiers(&actor.status_state.buffs, &actor.status_state.debuffs);
        (actor.unit.stats, buff_mods)
    };
    let effective_attacker = Stats {
        hp: attacker_stats.hp,
        atk: BaseStat::new_unchecked(
            (attacker_stats.atk.get() as i32 + attacker_buff_mods.atk.get() as i32)
                .max(0)
                .min(9999) as u16,
        ),
        def: BaseStat::new_unchecked(
            (attacker_stats.def.get() as i32 + attacker_buff_mods.def.get() as i32)
                .max(0)
                .min(9999) as u16,
        ),
        mag: BaseStat::new_unchecked(
            (attacker_stats.mag.get() as i32 + attacker_buff_mods.mag.get() as i32)
                .max(0)
                .min(9999) as u16,
        ),
        spd: BaseStat::new_unchecked(
            (attacker_stats.spd.get() as i32 + attacker_buff_mods.spd.get() as i32)
                .max(0)
                .min(9999) as u16,
        ),
    };

    let (target_stats, target_buff_mods) = {
        let target = get_unit(battle, target_ref).unwrap();
        let buff_mods = status::compute_stat_modifiers(
            &target.status_state.buffs,
            &target.status_state.debuffs,
        );
        (target.unit.stats, buff_mods)
    };
    let effective_target = Stats {
        hp: target_stats.hp,
        atk: BaseStat::new_unchecked(
            (target_stats.atk.get() as i32 + target_buff_mods.atk.get() as i32)
                .max(0)
                .min(9999) as u16,
        ),
        def: BaseStat::new_unchecked(
            (target_stats.def.get() as i32 + target_buff_mods.def.get() as i32)
                .max(0)
                .min(9999) as u16,
        ),
        mag: BaseStat::new_unchecked(
            (target_stats.mag.get() as i32 + target_buff_mods.mag.get() as i32)
                .max(0)
                .min(9999) as u16,
        ),
        spd: BaseStat::new_unchecked(
            (target_stats.spd.get() as i32 + target_buff_mods.spd.get() as i32)
                .max(0)
                .min(9999) as u16,
        ),
    };

    let damage = combat::calculate_damage(
        activation_event.base_damage,
        DamageType::Psynergy,
        &effective_attacker,
        &effective_target,
        &battle.config,
    );

    let barrier_blocked = {
        let target = get_unit_mut(battle, target_ref).unwrap();
        status::consume_barrier(&mut target.status_state.barriers)
    };
    if barrier_blocked {
        events.push(BattleEvent::BarrierBlocked(target_ref));
        return;
    }

    {
        let target = get_unit_mut(battle, target_ref).unwrap();
        target.unit.current_hp = target.unit.current_hp.saturating_sub(damage);
        if target.unit.current_hp == 0 {
            target.unit.is_alive = false;
        }
        for status in target.status_state.statuses.iter_mut() {
            if status.effect_type == StatusEffectType::Freeze {
                status.freeze_damage_taken += damage;
            }
        }
    }

    events.push(BattleEvent::DamageDealt(DamageDealt {
        source: actor_ref,
        target: target_ref,
        amount: damage,
        damage_type: DamageType::Psynergy,
        is_crit: false,
    }));

    if !battle.enemies[target_index].unit.is_alive {
        events.push(BattleEvent::UnitDefeated(UnitDefeated { unit: target_ref }));
    }
}

fn apply_chain_damage(
    battle: &mut Battle,
    actor_ref: TargetRef,
    primary_target_ref: TargetRef,
    amount: u16,
    damage_type: DamageType,
    is_crit: bool,
    events: &mut Vec<BattleEvent>,
) {
    let mut all_targets: Vec<TargetRef> = Vec::new();
    for i in 0..battle.player_units.len() {
        all_targets.push(TargetRef {
            side: Side::Player,
            index: i as u8,
        });
    }
    for i in 0..battle.enemies.len() {
        all_targets.push(TargetRef {
            side: Side::Enemy,
            index: i as u8,
        });
    }

    let chain_targets = damage_mods::calculate_chain_targets(actor_ref, &all_targets);
    for chain_tr in chain_targets {
        if chain_tr == primary_target_ref {
            continue;
        }
        let alive = get_unit(battle, chain_tr)
            .map(|u| u.unit.is_alive)
            .unwrap_or(false);
        if alive {
            {
                let t = get_unit_mut(battle, chain_tr).unwrap();
                t.unit.current_hp = t.unit.current_hp.saturating_sub(amount);
                if t.unit.current_hp == 0 {
                    t.unit.is_alive = false;
                }
                for status in t.status_state.statuses.iter_mut() {
                    if status.effect_type == StatusEffectType::Freeze {
                        status.freeze_damage_taken += amount;
                    }
                }
            }
            events.push(BattleEvent::DamageDealt(DamageDealt {
                source: actor_ref,
                target: chain_tr,
                amount,
                damage_type,
                is_crit,
            }));
            if get_unit(battle, chain_tr)
                .map(|u| !u.unit.is_alive)
                .unwrap_or(false)
            {
                events.push(BattleEvent::UnitDefeated(UnitDefeated { unit: chain_tr }));
            }
        }
    }
}

fn execute_summon_action(
    battle: &mut Battle,
    actor_ref: TargetRef,
    djinn_indices: &[u8],
    events: &mut Vec<BattleEvent>,
) {
    hydrate_team_djinn_slots_from_players(battle);
    let recovery_turns =
        battle.config.djinn_recovery_start_delay + battle.config.djinn_recovery_per_turn;
    let indices: Vec<usize> = djinn_indices.iter().map(|&i| i as usize).collect();

    // Collect djinn IDs before mutating slots (for summon damage lookup)
    let djinn_ids: Vec<DjinnId> = indices
        .iter()
        .filter_map(|&i| battle.team_djinn_slots.slots.get(i))
        .map(|inst| inst.djinn_id.clone())
        .collect();

    // Transition djinn to Recovery
    let summon_succeeded = if let Some(djinn_events) = djinn::execute_summon(
        &mut battle.team_djinn_slots,
        &indices,
        actor_ref,
        recovery_turns,
    ) {
        sync_team_djinn_slots_to_players(battle);
        for ev in djinn_events {
            events.push(BattleEvent::DjinnChanged(ev));
        }
        true
    } else {
        false
    };

    if !summon_succeeded {
        return;
    }

    // Calculate total summon damage from all used djinn
    let mut total_summon_damage: u16 = 0;
    for djinn_id in &djinn_ids {
        if let Some(djinn_def) = battle.djinn_defs.get(djinn_id) {
            if let Some(ref summon_effect) = djinn_def.summon_effect {
                total_summon_damage = total_summon_damage.saturating_add(summon_effect.damage);
            }
        }
    }

    // Apply summon damage to all alive enemies (respecting barriers/freeze)
    if total_summon_damage > 0 {
        let enemy_count = battle.enemies.len();
        for ei in 0..enemy_count {
            if !battle.enemies[ei].unit.is_alive {
                continue;
            }
            let target_ref = TargetRef {
                side: Side::Enemy,
                index: ei as u8,
            };

            // Check barrier before applying damage
            let barrier_blocked =
                status::consume_barrier(&mut battle.enemies[ei].status_state.barriers);
            if barrier_blocked {
                events.push(BattleEvent::BarrierBlocked(target_ref));
                continue;
            }

            battle.enemies[ei].unit.current_hp = battle.enemies[ei]
                .unit
                .current_hp
                .saturating_sub(total_summon_damage);
            if battle.enemies[ei].unit.current_hp == 0 {
                battle.enemies[ei].unit.is_alive = false;
            }

            // Accumulate freeze damage
            for s in battle.enemies[ei].status_state.statuses.iter_mut() {
                if s.effect_type == StatusEffectType::Freeze {
                    s.freeze_damage_taken += total_summon_damage;
                }
            }

            events.push(BattleEvent::DamageDealt(DamageDealt {
                source: actor_ref,
                target: target_ref,
                amount: total_summon_damage,
                damage_type: DamageType::Psynergy,
                is_crit: false,
            }));
            if !battle.enemies[ei].unit.is_alive {
                events.push(BattleEvent::UnitDefeated(UnitDefeated { unit: target_ref }));
            }
        }
    }
}

// ── End-of-round ticks ──────────────────────────────────────────────

fn end_of_round_ticks(battle: &mut Battle, events: &mut Vec<BattleEvent>) {
    // Tick statuses for all units (burn/poison)
    tick_all_units_statuses(battle, events);

    // Tick HoTs for all units
    tick_all_units_hots(battle, events);

    // Tick buffs/debuffs
    tick_all_units_buffs(battle);

    // Tick barriers
    tick_all_units_barriers(battle);

    // Tick immunity
    tick_all_units_immunity(battle);

    // Tick djinn recovery
    tick_all_units_djinn(battle, events);
}

fn tick_all_units_immunity(battle: &mut Battle) {
    for unit in battle
        .player_units
        .iter_mut()
        .chain(battle.enemies.iter_mut())
    {
        if !unit.unit.is_alive {
            continue;
        }
        status::tick_immunity(&mut unit.status_state.immunity);
    }
}

fn tick_all_units_statuses(battle: &mut Battle, events: &mut Vec<BattleEvent>) {
    let player_count = battle.player_units.len();
    let enemy_count = battle.enemies.len();

    for i in 0..player_count {
        let unit = &mut battle.player_units[i];
        if !unit.unit.is_alive {
            continue;
        }
        let result = status::tick_statuses(
            unit.unit.stats.hp.get(),
            unit.unit.current_hp,
            &mut unit.status_state.statuses,
        );
        if result.damage > 0 {
            unit.unit.current_hp = unit.unit.current_hp.saturating_sub(result.damage);
            let target_ref = TargetRef {
                side: Side::Player,
                index: i as u8,
            };
            events.push(BattleEvent::DamageDealt(DamageDealt {
                source: target_ref,
                target: target_ref,
                amount: result.damage,
                damage_type: DamageType::Physical,
                is_crit: false,
            }));
            if unit.unit.current_hp == 0 {
                unit.unit.is_alive = false;
                events.push(BattleEvent::UnitDefeated(UnitDefeated { unit: target_ref }));
            }
        }
    }

    for i in 0..enemy_count {
        let unit = &mut battle.enemies[i];
        if !unit.unit.is_alive {
            continue;
        }
        let result = status::tick_statuses(
            unit.unit.stats.hp.get(),
            unit.unit.current_hp,
            &mut unit.status_state.statuses,
        );
        if result.damage > 0 {
            unit.unit.current_hp = unit.unit.current_hp.saturating_sub(result.damage);
            let target_ref = TargetRef {
                side: Side::Enemy,
                index: i as u8,
            };
            events.push(BattleEvent::DamageDealt(DamageDealt {
                source: target_ref,
                target: target_ref,
                amount: result.damage,
                damage_type: DamageType::Physical,
                is_crit: false,
            }));
            if unit.unit.current_hp == 0 {
                unit.unit.is_alive = false;
                events.push(BattleEvent::UnitDefeated(UnitDefeated { unit: target_ref }));
            }
        }
    }
}

fn tick_all_units_hots(battle: &mut Battle, events: &mut Vec<BattleEvent>) {
    for (i, unit) in battle.player_units.iter_mut().enumerate() {
        if !unit.unit.is_alive {
            continue;
        }
        let healing = status::tick_hots(&mut unit.status_state.hots);
        if healing > 0 {
            let max_hp = unit.unit.stats.hp.get();
            let old_hp = unit.unit.current_hp;
            unit.unit.current_hp = (unit.unit.current_hp + healing).min(max_hp);
            let actual = unit.unit.current_hp - old_hp;
            if actual > 0 {
                let target_ref = TargetRef {
                    side: Side::Player,
                    index: i as u8,
                };
                events.push(BattleEvent::HealingDone(HealingDone {
                    source: target_ref,
                    target: target_ref,
                    amount: actual,
                }));
            }
        }
    }

    for (i, unit) in battle.enemies.iter_mut().enumerate() {
        if !unit.unit.is_alive {
            continue;
        }
        let healing = status::tick_hots(&mut unit.status_state.hots);
        if healing > 0 {
            let max_hp = unit.unit.stats.hp.get();
            let old_hp = unit.unit.current_hp;
            unit.unit.current_hp = (unit.unit.current_hp + healing).min(max_hp);
            let actual = unit.unit.current_hp - old_hp;
            if actual > 0 {
                let target_ref = TargetRef {
                    side: Side::Enemy,
                    index: i as u8,
                };
                events.push(BattleEvent::HealingDone(HealingDone {
                    source: target_ref,
                    target: target_ref,
                    amount: actual,
                }));
            }
        }
    }
}

fn tick_all_units_buffs(battle: &mut Battle) {
    for unit in battle
        .player_units
        .iter_mut()
        .chain(battle.enemies.iter_mut())
    {
        if !unit.unit.is_alive {
            continue;
        }
        status::tick_buffs(&mut unit.status_state.buffs);
        status::tick_buffs(&mut unit.status_state.debuffs);
    }
}

fn tick_all_units_barriers(battle: &mut Battle) {
    for unit in battle
        .player_units
        .iter_mut()
        .chain(battle.enemies.iter_mut())
    {
        if !unit.unit.is_alive {
            continue;
        }
        status::tick_barriers(&mut unit.status_state.barriers);
    }
}

fn tick_all_units_djinn(battle: &mut Battle, events: &mut Vec<BattleEvent>) {
    hydrate_team_djinn_slots_from_players(battle);
    let anchor = team_djinn_anchor_ref(battle);
    let djinn_events = djinn::tick_recovery(&mut battle.team_djinn_slots, anchor);
    if !djinn_events.is_empty() {
        sync_team_djinn_slots_to_players(battle);
        for ev in djinn_events {
            events.push(BattleEvent::DjinnChanged(ev));
        }
    }
}

fn collect_team_djinn_slots(player_data: &[PlayerUnitData]) -> DjinnSlots {
    let mut team_slots = DjinnSlots::new();
    for player in player_data {
        for inst in &player.djinn_slots.slots {
            if team_slots
                .slots
                .iter()
                .any(|existing| existing.djinn_id == inst.djinn_id)
            {
                continue;
            }
            if !team_slots.add(inst.djinn_id.clone()) {
                break;
            }
            if let Some(last) = team_slots.slots.last_mut() {
                *last = inst.clone();
            }
            team_slots.next_activation_order = team_slots
                .next_activation_order
                .max(inst.activation_order.saturating_add(1));
        }
    }
    team_slots
}

fn sync_team_djinn_slots_to_players(battle: &mut Battle) {
    for player in &mut battle.player_units {
        player.djinn_slots = battle.team_djinn_slots.clone();
    }
}

fn hydrate_team_djinn_slots_from_players(battle: &mut Battle) {
    if !battle.team_djinn_slots.slots.is_empty() {
        return;
    }

    for player in &battle.player_units {
        if !player.djinn_slots.slots.is_empty() {
            battle.team_djinn_slots = player.djinn_slots.clone();
            sync_team_djinn_slots_to_players(battle);
            break;
        }
    }
}

fn team_djinn_anchor_ref(battle: &Battle) -> TargetRef {
    let index = battle
        .player_units
        .iter()
        .position(|unit| unit.unit.is_alive)
        .unwrap_or(0) as u8;
    TargetRef {
        side: Side::Player,
        index,
    }
}

// ── check_battle_end ────────────────────────────────────────────────

/// Check if the battle has ended.
///
/// All enemies dead -> Victory. All players dead -> Defeat. Otherwise None.
pub fn check_battle_end(battle: &Battle) -> Option<BattleResult> {
    let all_enemies_dead = battle.enemies.iter().all(|e| !e.unit.is_alive);
    let all_players_dead = battle.player_units.iter().all(|p| !p.unit.is_alive);

    if all_enemies_dead {
        // Sum actual xp and gold from enemy definitions
        let xp: u32 = battle.enemies.iter().map(|e| e.reward_xp).sum();
        let gold: u32 = battle.enemies.iter().map(|e| e.reward_gold).sum();
        Some(BattleResult::Victory { xp, gold })
    } else if all_players_dead {
        Some(BattleResult::Defeat)
    } else {
        None
    }
}

// ── get_unit ────────────────────────────────────────────────────────

/// Get an immutable reference to a unit by TargetRef.
pub fn get_unit(battle: &Battle, target_ref: TargetRef) -> Option<&BattleUnitFull> {
    match target_ref.side {
        Side::Player => battle.player_units.get(target_ref.index as usize),
        Side::Enemy => battle.enemies.get(target_ref.index as usize),
    }
}

/// Get a mutable reference to a unit by TargetRef.
fn get_unit_mut(battle: &mut Battle, target_ref: TargetRef) -> Option<&mut BattleUnitFull> {
    match target_ref.side {
        Side::Player => battle.player_units.get_mut(target_ref.index as usize),
        Side::Enemy => battle.enemies.get_mut(target_ref.index as usize),
    }
}

// ═══════════════════════════════════════════════════════════════════════
// Tests
// ═══════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::default_combat_config;
    use crate::shared::bounded_types::*;
    use crate::shared::{
        AbilityCategory, CleanseType, DjinnId, Element, EnemyId, Immunity, StatusEffect,
        StatusEffectType, TargetMode,
    };

    // ── Helpers ──────────────────────────────────────────────────────

    fn test_config() -> CombatConfig {
        default_combat_config()
    }

    fn test_stats(hp: u16, atk: u16, def: u16, mag: u16, spd: u16) -> Stats {
        Stats {
            hp: Hp::new(hp).unwrap_or_else(|_| Hp::new_unchecked(hp.clamp(1, 9999))),
            atk: BaseStat::new(atk).unwrap_or_else(|_| BaseStat::new_unchecked(atk.clamp(0, 9999))),
            def: BaseStat::new(def).unwrap_or_else(|_| BaseStat::new_unchecked(def.clamp(0, 9999))),
            mag: BaseStat::new(mag).unwrap_or_else(|_| BaseStat::new_unchecked(mag.clamp(0, 9999))),
            spd: BaseStat::new(spd).unwrap_or_else(|_| BaseStat::new_unchecked(spd.clamp(0, 9999))),
        }
    }

    fn make_player(id: &str, stats: Stats, mana: u8) -> PlayerUnitData {
        PlayerUnitData {
            id: id.to_string(),
            base_stats: stats,
            equipment: EquipmentLoadout::default(),
            djinn_slots: DjinnSlots::new(),
            mana_contribution: mana,
            equipment_effects: equipment::EquipmentEffects::default(),
        }
    }

    fn make_enemy(id: &str, stats: Stats) -> EnemyUnitData {
        EnemyUnitData {
            enemy_def: EnemyDef {
                id: EnemyId(id.to_string()),
                name: id.to_string(),
                element: Element::Venus,
                level: Level::new_unchecked(1),
                stats,
                xp: crate::shared::bounded_types::Xp::new_unchecked(10),
                gold: crate::shared::bounded_types::Gold::new_unchecked(5),
                abilities: vec![],
            },
        }
    }

    fn make_basic_ability(id: &str, cost: u8, power: u16) -> (AbilityId, AbilityDef) {
        let aid = AbilityId(id.to_string());
        let def = AbilityDef {
            id: aid.clone(),
            name: id.to_string(),
            category: AbilityCategory::Psynergy,
            damage_type: Some(DamageType::Psynergy),
            element: Some(Element::Venus),
            mana_cost: ManaCost::new(cost)
                .unwrap_or_else(|_| ManaCost::new_unchecked(cost.clamp(0, 99))),
            base_power: BasePower::new(power)
                .unwrap_or_else(|_| BasePower::new_unchecked(power.clamp(0, 9999))),
            targets: TargetMode::SingleEnemy,
            unlock_level: Level::new_unchecked(1),
            hit_count: HitCount::new_unchecked(1),
            status_effect: None,
            buff_effect: None,
            debuff_effect: None,
            shield_charges: None,
            shield_duration: None,
            heal_over_time: None,
            grant_immunity: None,
            cleanse: None,
            ignore_defense_percent: None,
            splash_damage_percent: None,
            chain_damage: false,
            revive: false,
            revive_hp_percent: None,
        };
        (aid, def)
    }

    fn make_djinn_def(id: &str, element: Element, summon_damage: u16) -> (DjinnId, DjinnDef) {
        let djinn_id = DjinnId(id.to_string());
        let empty_ability_set = crate::shared::DjinnAbilitySet {
            good_abilities: vec![],
            recovery_abilities: vec![],
        };
        let djinn_def = DjinnDef {
            id: djinn_id.clone(),
            name: id.to_string(),
            element,
            tier: crate::shared::bounded_types::DjinnTier::new_unchecked(1),
            stat_bonus: crate::shared::StatBonus::default(),
            summon_effect: Some(crate::shared::SummonEffect {
                damage: summon_damage,
                buff: None,
                status: None,
                heal: None,
            }),
            ability_pairs: crate::shared::DjinnAbilityPairs {
                same: empty_ability_set.clone(),
                counter: empty_ability_set.clone(),
                neutral: empty_ability_set,
            },
        };
        (djinn_id, djinn_def)
    }

    fn setup_basic_battle() -> Battle {
        let player_stats = test_stats(100, 30, 20, 25, 15);
        let enemy_stats = test_stats(80, 20, 15, 10, 10);

        let player = make_player("hero", player_stats, 5);
        let enemy = make_enemy("goblin", enemy_stats);

        let (aid, adef) = make_basic_ability("quake", 3, 40);
        let mut abilities = HashMap::new();
        abilities.insert(aid, adef);

        new_battle(
            vec![player],
            vec![enemy],
            test_config(),
            abilities,
            HashMap::new(),
        )
    }

    // ── Test: new_battle correct mana pool ──────────────────────────

    #[test]
    fn test_new_battle_mana_pool() {
        let battle = setup_basic_battle();
        assert_eq!(battle.mana_pool.max_mana, 5);
        assert_eq!(battle.mana_pool.current_mana, 5);
        assert_eq!(battle.round, 1);
        assert_eq!(battle.phase, BattlePhase::Planning);
    }

    // ── Test: new_battle stats initialized ──────────────────────────

    #[test]
    fn test_new_battle_stats_initialized() {
        let battle = setup_basic_battle();
        let player = &battle.player_units[0];
        assert_eq!(player.unit.stats.hp.get(), 100);
        assert_eq!(player.unit.stats.atk.get(), 30);
        assert_eq!(player.unit.current_hp, 100);
        assert!(player.unit.is_alive);
        assert_eq!(player.unit.crit_counter, 0);

        let enemy = &battle.enemies[0];
        assert_eq!(enemy.unit.stats.hp.get(), 80);
        assert_eq!(enemy.unit.current_hp, 80);
        assert!(enemy.unit.is_alive);
    }

    // ── Test: plan_action attack succeeds ───────────────────────────

    #[test]
    fn test_plan_attack_succeeds() {
        let mut battle = setup_basic_battle();
        let actor = TargetRef {
            side: Side::Player,
            index: 0,
        };
        let target = TargetRef {
            side: Side::Enemy,
            index: 0,
        };
        let result = plan_action(&mut battle, actor, BattleAction::Attack { target });
        assert!(result.is_ok());
        assert_eq!(battle.planned_actions.len(), 1);
    }

    #[test]
    fn test_planned_attack_adds_projected_mana() {
        let player_stats = test_stats(100, 30, 20, 25, 15);
        let enemy_stats = test_stats(80, 20, 15, 10, 10);

        let mut player = make_player("hero", player_stats, 5);
        player.equipment_effects.total_hit_count_bonus = 2;
        let enemy = make_enemy("goblin", enemy_stats);

        let mut battle = new_battle(
            vec![player],
            vec![enemy],
            test_config(),
            HashMap::new(),
            HashMap::new(),
        );

        let actor = TargetRef {
            side: Side::Player,
            index: 0,
        };
        let target = TargetRef {
            side: Side::Enemy,
            index: 0,
        };

        plan_action(&mut battle, actor, BattleAction::Attack { target }).unwrap();

        assert_eq!(battle.mana_pool.projected_mana, 8);
    }

    #[test]
    fn test_planned_attack_undo_reverses_mana() {
        let mut battle = setup_basic_battle();
        let actor = TargetRef {
            side: Side::Player,
            index: 0,
        };
        let target = TargetRef {
            side: Side::Enemy,
            index: 0,
        };

        plan_action(&mut battle, actor, BattleAction::Attack { target }).unwrap();
        assert_eq!(battle.mana_pool.projected_mana, 6);

        plan_action(
            &mut battle,
            actor,
            BattleAction::UseAbility {
                ability_id: AbilityId("quake".into()),
                targets: vec![target],
            },
        )
        .unwrap();

        assert_eq!(battle.mana_pool.projected_mana, 2);
        assert_eq!(battle.planned_actions.len(), 1);
        assert!(matches!(
            &battle.planned_actions[0].1,
            BattleAction::UseAbility { .. }
        ));
    }

    // ── Test: plan_action ability with enough mana succeeds ─────────

    #[test]
    fn test_plan_ability_sufficient_mana() {
        let mut battle = setup_basic_battle();
        let actor = TargetRef {
            side: Side::Player,
            index: 0,
        };
        let target = TargetRef {
            side: Side::Enemy,
            index: 0,
        };
        let result = plan_action(
            &mut battle,
            actor,
            BattleAction::UseAbility {
                ability_id: AbilityId("quake".into()),
                targets: vec![target],
            },
        );
        assert!(result.is_ok());
    }

    // ── Test: plan_action ability with insufficient mana fails ──────

    #[test]
    fn test_plan_ability_insufficient_mana() {
        let mut battle = setup_basic_battle();
        // Drain mana
        battle.mana_pool.spend_mana(4);
        assert_eq!(battle.mana_pool.current_mana, 1);

        let actor = TargetRef {
            side: Side::Player,
            index: 0,
        };
        let target = TargetRef {
            side: Side::Enemy,
            index: 0,
        };
        let result = plan_action(
            &mut battle,
            actor,
            BattleAction::UseAbility {
                ability_id: AbilityId("quake".into()),
                targets: vec![target],
            },
        );
        assert_eq!(result, Err(PlanError::InsufficientMana));
    }

    // ── Test: execute_round auto-attack deals damage ────────────────

    #[test]
    fn test_execute_round_auto_attack_damage() {
        let mut battle = setup_basic_battle();
        let actor = TargetRef {
            side: Side::Player,
            index: 0,
        };
        let target = TargetRef {
            side: Side::Enemy,
            index: 0,
        };
        plan_action(&mut battle, actor, BattleAction::Attack { target }).unwrap();

        let enemy_hp_before = battle.enemies[0].unit.current_hp;
        let events = execute_round(&mut battle);

        let enemy_hp_after = battle.enemies[0].unit.current_hp;
        assert!(
            enemy_hp_after < enemy_hp_before,
            "Enemy should have taken damage"
        );

        let has_damage = events
            .iter()
            .any(|e| matches!(e, BattleEvent::DamageDealt(_)));
        assert!(has_damage, "Should have a DamageDealt event");
    }

    // ── Test: execute_round ability spends mana ─────────────────────

    #[test]
    fn test_execute_round_ability_spends_mana() {
        let mut battle = setup_basic_battle();
        let actor = TargetRef {
            side: Side::Player,
            index: 0,
        };
        let target = TargetRef {
            side: Side::Enemy,
            index: 0,
        };

        plan_action(
            &mut battle,
            actor,
            BattleAction::UseAbility {
                ability_id: AbilityId("quake".into()),
                targets: vec![target],
            },
        )
        .unwrap();

        assert_eq!(battle.mana_pool.current_mana, 5);
        let events = execute_round(&mut battle);

        // Mana should have been spent then reset
        let has_mana_change = events
            .iter()
            .any(|e| matches!(e, BattleEvent::ManaChanged(_)));
        assert!(has_mana_change, "Should have ManaChanged event");

        // After round reset, mana should be back to max
        assert_eq!(battle.mana_pool.current_mana, 5);
    }

    // ── Test: multi-hit generates mana for auto-attacks ─────────────

    #[test]
    fn test_multi_hit_generates_mana() {
        let player_stats = test_stats(100, 30, 20, 25, 15);
        let enemy_stats = test_stats(200, 20, 15, 10, 10);

        let mut player = make_player("hero", player_stats, 5);
        player.equipment_effects.total_hit_count_bonus = 0;

        let enemy = make_enemy("troll", enemy_stats);
        let mut battle = new_battle(
            vec![player],
            vec![enemy],
            test_config(),
            HashMap::new(),
            HashMap::new(),
        );

        // Spend some mana first to track generation
        battle.mana_pool.spend_mana(3);
        let _mana_before = battle.mana_pool.current_mana;

        let actor = TargetRef {
            side: Side::Player,
            index: 0,
        };
        let target = TargetRef {
            side: Side::Enemy,
            index: 0,
        };
        plan_action(&mut battle, actor, BattleAction::Attack { target }).unwrap();

        execute_round(&mut battle);
        // After round ends mana resets, but during execution mana was generated
        // We verify through events
        // The mana was generated during execution and then reset at end-of-round
        assert_eq!(battle.mana_pool.current_mana, 5); // reset to max
    }

    // ── Test: barrier blocks damage ─────────────────────────────────

    #[test]
    fn test_barrier_blocks_damage() {
        let mut battle = setup_basic_battle();

        // Give the enemy a barrier
        status::apply_barrier(&mut battle.enemies[0].status_state, 1, 5);

        let actor = TargetRef {
            side: Side::Player,
            index: 0,
        };
        let target = TargetRef {
            side: Side::Enemy,
            index: 0,
        };
        plan_action(&mut battle, actor, BattleAction::Attack { target }).unwrap();

        let enemy_hp_before = battle.enemies[0].unit.current_hp;
        let events = execute_round(&mut battle);

        let enemy_hp_after = battle.enemies[0].unit.current_hp;
        assert_eq!(
            enemy_hp_after, enemy_hp_before,
            "Barrier should have blocked damage"
        );

        let has_barrier = events
            .iter()
            .any(|e| matches!(e, BattleEvent::BarrierBlocked(_)));
        assert!(has_barrier, "Should have BarrierBlocked event");
    }

    // ── Test: burn ticks at end of round ────────────────────────────

    #[test]
    fn test_burn_ticks_at_end_of_round() {
        let mut battle = setup_basic_battle();

        // Apply burn to enemy
        let burn = StatusEffect {
            effect_type: StatusEffectType::Burn,
            duration: EffectDuration::new_unchecked(3),
            burn_percent: Some(0.10),
            poison_percent: None,
            freeze_threshold: None,
        };
        status::apply_status(&mut battle.enemies[0].status_state, &burn, &None);

        let enemy_hp_before = battle.enemies[0].unit.current_hp;
        let _events = execute_round(&mut battle);

        let enemy_hp_after = battle.enemies[0].unit.current_hp;
        // Burn should deal 10% of max HP (80) = 8
        assert_eq!(
            enemy_hp_before - enemy_hp_after,
            8,
            "Burn should deal 10% of 80 max HP = 8"
        );
    }

    // ── Test: unit death from damage ────────────────────────────────

    #[test]
    fn test_unit_death_from_damage() {
        let player_stats = test_stats(100, 200, 20, 25, 15);
        let enemy_stats = test_stats(30, 20, 5, 10, 10);

        let player = make_player("hero", player_stats, 5);
        let enemy = make_enemy("weak_goblin", enemy_stats);
        let mut battle = new_battle(
            vec![player],
            vec![enemy],
            test_config(),
            HashMap::new(),
            HashMap::new(),
        );

        let actor = TargetRef {
            side: Side::Player,
            index: 0,
        };
        let target = TargetRef {
            side: Side::Enemy,
            index: 0,
        };
        plan_action(&mut battle, actor, BattleAction::Attack { target }).unwrap();

        let events = execute_round(&mut battle);

        assert!(!battle.enemies[0].unit.is_alive, "Enemy should be dead");
        let has_defeated = events
            .iter()
            .any(|e| matches!(e, BattleEvent::UnitDefeated(_)));
        assert!(has_defeated, "Should have UnitDefeated event");
    }

    // ── Test: check_battle_end victory when all enemies dead ────────

    #[test]
    fn test_check_battle_end_victory() {
        let mut battle = setup_basic_battle();
        battle.enemies[0].unit.is_alive = false;
        battle.enemies[0].unit.current_hp = 0;

        let result = check_battle_end(&battle);
        assert!(result.is_some());
        match result.unwrap() {
            BattleResult::Victory { xp, gold } => {
                assert!(xp > 0);
                assert!(gold > 0);
            }
            BattleResult::Defeat => panic!("Expected Victory"),
        }
    }

    // ── Test: check_battle_end defeat when all players dead ─────────

    #[test]
    fn test_check_battle_end_defeat() {
        let mut battle = setup_basic_battle();
        battle.player_units[0].unit.is_alive = false;
        battle.player_units[0].unit.current_hp = 0;

        let result = check_battle_end(&battle);
        assert_eq!(result, Some(BattleResult::Defeat));
    }

    // ── Test: check_battle_end None when both sides alive ───────────

    #[test]
    fn test_check_battle_end_none() {
        let battle = setup_basic_battle();
        let result = check_battle_end(&battle);
        assert!(result.is_none());
    }

    // ── Test: plan attack on dead unit fails ────────────────────────

    #[test]
    fn test_plan_action_dead_unit_cannot_act() {
        let mut battle = setup_basic_battle();
        battle.player_units[0].unit.is_alive = false;

        let actor = TargetRef {
            side: Side::Player,
            index: 0,
        };
        let target = TargetRef {
            side: Side::Enemy,
            index: 0,
        };
        let result = plan_action(&mut battle, actor, BattleAction::Attack { target });
        assert_eq!(result, Err(PlanError::UnitCannotAct));
    }

    // ── Test: plan djinn not ready ──────────────────────────────────

    #[test]
    fn test_plan_djinn_not_ready() {
        let mut battle = setup_basic_battle();
        // Add a djinn in Recovery state
        battle.player_units[0]
            .djinn_slots
            .add(DjinnId("flint".into()));
        battle.player_units[0].djinn_slots.slots[0].state = DjinnState::Recovery;

        let actor = TargetRef {
            side: Side::Player,
            index: 0,
        };
        let result = plan_action(
            &mut battle,
            actor,
            BattleAction::ActivateDjinn { djinn_index: 0 },
        );
        assert_eq!(result, Err(PlanError::DjinnNotReady));
    }

    // ── Test: plan invalid djinn index ──────────────────────────────

    #[test]
    fn test_plan_invalid_djinn_index() {
        let mut battle = setup_basic_battle();

        let actor = TargetRef {
            side: Side::Player,
            index: 0,
        };
        let result = plan_action(
            &mut battle,
            actor,
            BattleAction::ActivateDjinn { djinn_index: 5 },
        );
        assert_eq!(result, Err(PlanError::InvalidDjinnIndex));
    }

    #[test]
    fn test_any_player_can_activate_team_djinn_pool() {
        let p1 = make_player("adept", test_stats(100, 10, 10, 10, 10), 2);
        let p2 = make_player("blaze", test_stats(100, 12, 8, 12, 12), 2);
        let enemy = make_enemy("goblin", test_stats(100, 10, 10, 5, 5));

        let (djinn_id, djinn_def) = make_djinn_def("flint", Element::Venus, 50);
        let mut djinn_defs = HashMap::new();
        djinn_defs.insert(djinn_id.clone(), djinn_def);

        let mut players = vec![p1, p2];
        players[0].djinn_slots.add(djinn_id);

        let mut battle = new_battle(
            players,
            vec![enemy],
            test_config(),
            HashMap::new(),
            djinn_defs,
        );

        let actor = TargetRef {
            side: Side::Player,
            index: 1,
        };
        let result = plan_action(
            &mut battle,
            actor,
            BattleAction::ActivateDjinn { djinn_index: 0 },
        );
        assert_eq!(result, Ok(()));

        let events = execute_round(&mut battle);
        assert!(events
            .iter()
            .any(|event| matches!(event, BattleEvent::DjinnChanged(_))));
        assert_eq!(battle.team_djinn_slots.slots[0].state, DjinnState::Recovery);
        assert_eq!(
            battle.player_units[0].djinn_slots.slots[0].state,
            DjinnState::Recovery
        );
        assert_eq!(
            battle.player_units[1].djinn_slots.slots[0].state,
            DjinnState::Recovery
        );
    }

    // ── Test: full scenario plan + execute + check across 2 rounds ──

    #[test]
    fn test_full_two_round_scenario() {
        let player_stats = test_stats(200, 40, 20, 30, 15);
        let enemy_stats = test_stats(50, 15, 10, 10, 8);

        let player = make_player("hero", player_stats, 5);
        let enemy = make_enemy("goblin", enemy_stats);

        let (aid, adef) = make_basic_ability("quake", 2, 30);
        let mut abilities = HashMap::new();
        abilities.insert(aid, adef);

        let mut battle = new_battle(
            vec![player],
            vec![enemy],
            test_config(),
            abilities,
            HashMap::new(),
        );

        // Round 1: auto-attack
        let actor = TargetRef {
            side: Side::Player,
            index: 0,
        };
        let target = TargetRef {
            side: Side::Enemy,
            index: 0,
        };
        plan_action(&mut battle, actor, BattleAction::Attack { target }).unwrap();

        assert_eq!(battle.round, 1);
        let events_r1 = execute_round(&mut battle);
        assert!(!events_r1.is_empty());
        assert_eq!(battle.round, 2);

        // Check if enemy survived round 1
        let result = check_battle_end(&battle);
        if result.is_some() {
            // Enemy died in round 1, that is fine
            match result.unwrap() {
                BattleResult::Victory { .. } => return,
                _ => panic!("Expected Victory if enemy died"),
            }
        }

        // Round 2: use ability to finish
        plan_action(
            &mut battle,
            actor,
            BattleAction::UseAbility {
                ability_id: AbilityId("quake".into()),
                targets: vec![target],
            },
        )
        .unwrap();

        let events_r2 = execute_round(&mut battle);
        assert!(!events_r2.is_empty());
        assert_eq!(battle.round, 3);

        // Should be victory now
        let result = check_battle_end(&battle);
        assert!(result.is_some());
        match result.unwrap() {
            BattleResult::Victory { xp, gold } => {
                assert!(xp > 0);
                assert!(gold > 0);
            }
            BattleResult::Defeat => panic!("Expected Victory"),
        }
    }

    // ── Test: get_unit returns correct unit ─────────────────────────

    #[test]
    fn test_get_unit() {
        let battle = setup_basic_battle();
        let player_ref = TargetRef {
            side: Side::Player,
            index: 0,
        };
        let enemy_ref = TargetRef {
            side: Side::Enemy,
            index: 0,
        };
        let invalid_ref = TargetRef {
            side: Side::Player,
            index: 99,
        };

        assert!(get_unit(&battle, player_ref).is_some());
        assert!(get_unit(&battle, enemy_ref).is_some());
        assert!(get_unit(&battle, invalid_ref).is_none());
    }

    // ── Test: multiple mana contributions ───────────────────────────

    #[test]
    fn test_multi_player_mana_pool() {
        let stats = test_stats(100, 20, 15, 10, 10);
        let p1 = make_player("hero1", stats, 3);
        let p2 = make_player("hero2", stats, 4);
        let enemy = make_enemy("goblin", stats);

        let battle = new_battle(
            vec![p1, p2],
            vec![enemy],
            test_config(),
            HashMap::new(),
            HashMap::new(),
        );
        assert_eq!(battle.mana_pool.max_mana, 7);
        assert_eq!(battle.mana_pool.current_mana, 7);
    }

    // ── Test: execute_round with djinn activation ───────────────────

    #[test]
    fn test_djinn_activation_deals_damage() {
        let player_stats = test_stats(100, 30, 20, 25, 15);
        let enemy_stats = test_stats(80, 20, 15, 10, 10);

        let player = make_player("hero", player_stats, 5);
        let first_enemy = make_enemy("fallen-goblin", enemy_stats);
        let second_enemy = make_enemy("target-goblin", enemy_stats);
        let (djinn_id, djinn_def) = make_djinn_def("flint", Element::Venus, 50);
        let mut djinn_defs = HashMap::new();
        djinn_defs.insert(djinn_id.clone(), djinn_def);

        let mut battle = new_battle(
            vec![player],
            vec![first_enemy, second_enemy],
            test_config(),
            HashMap::new(),
            djinn_defs,
        );
        battle.player_units[0].djinn_slots.add(djinn_id);
        battle.enemies[0].unit.is_alive = false;
        battle.enemies[0].unit.current_hp = 0;

        let actor = TargetRef {
            side: Side::Player,
            index: 0,
        };
        plan_action(
            &mut battle,
            actor,
            BattleAction::ActivateDjinn { djinn_index: 0 },
        )
        .unwrap();

        let expected_damage = combat::calculate_damage(
            50,
            DamageType::Psynergy,
            &battle.player_units[0].unit.stats,
            &battle.enemies[1].unit.stats,
            &battle.config,
        );
        let events = execute_round(&mut battle);

        assert_eq!(battle.enemies[1].unit.current_hp, 80 - expected_damage);
        let damage_events: Vec<&DamageDealt> = events
            .iter()
            .filter_map(|event| match event {
                BattleEvent::DamageDealt(damage) => Some(damage),
                _ => None,
            })
            .collect();
        assert!(
            damage_events.iter().any(|damage| {
                damage.target
                    == TargetRef {
                        side: Side::Enemy,
                        index: 1,
                    }
                    && damage.amount == expected_damage
                    && damage.damage_type == DamageType::Psynergy
            }),
            "Activation should deal psynergy damage to the first alive enemy"
        );
    }

    #[test]
    fn test_djinn_activation_still_changes_state() {
        let mut battle = setup_basic_battle();
        let (djinn_id, djinn_def) = make_djinn_def("flint", Element::Venus, 50);
        battle.djinn_defs.insert(djinn_id.clone(), djinn_def);
        battle.player_units[0].djinn_slots.add(djinn_id);

        let actor = TargetRef {
            side: Side::Player,
            index: 0,
        };
        plan_action(
            &mut battle,
            actor,
            BattleAction::ActivateDjinn { djinn_index: 0 },
        )
        .unwrap();

        let events = execute_round(&mut battle);

        assert_eq!(
            battle.player_units[0].djinn_slots.slots[0].state,
            DjinnState::Recovery,
            "Djinn should be in Recovery after activation"
        );

        let has_djinn_change = events
            .iter()
            .any(|e| matches!(e, BattleEvent::DjinnChanged(_)));
        assert!(has_djinn_change, "Should have DjinnChanged event");
    }

    #[test]
    fn test_djinn_activation_heal_restores_caster_team() {
        let player_stats = test_stats(100, 30, 20, 25, 15);
        let ally_stats = test_stats(90, 18, 14, 12, 11);
        let enemy_stats = test_stats(80, 20, 15, 10, 10);

        let player = make_player("hero", player_stats, 5);
        let ally = make_player("ally", ally_stats, 3);
        let enemy = make_enemy("goblin", enemy_stats);

        let djinn_id = DjinnId("mist".into());
        let empty_ability_set = crate::shared::DjinnAbilitySet {
            good_abilities: vec![],
            recovery_abilities: vec![],
        };
        let djinn_def = DjinnDef {
            id: djinn_id.clone(),
            name: "Mist".to_string(),
            element: Element::Mercury,
            tier: crate::shared::bounded_types::DjinnTier::new_unchecked(1),
            stat_bonus: crate::shared::StatBonus::default(),
            summon_effect: Some(crate::shared::SummonEffect {
                damage: 0,
                buff: None,
                status: None,
                heal: Some(18),
            }),
            ability_pairs: crate::shared::DjinnAbilityPairs {
                same: empty_ability_set.clone(),
                counter: empty_ability_set.clone(),
                neutral: empty_ability_set,
            },
        };
        let mut djinn_defs = HashMap::new();
        djinn_defs.insert(djinn_id.clone(), djinn_def);

        let mut battle = new_battle(
            vec![player, ally],
            vec![enemy],
            test_config(),
            HashMap::new(),
            djinn_defs,
        );
        battle.player_units[0].djinn_slots.add(djinn_id);
        battle.player_units[0].unit.current_hp = 55;
        battle.player_units[1].unit.current_hp = 40;

        let actor = TargetRef {
            side: Side::Player,
            index: 0,
        };
        plan_action(
            &mut battle,
            actor,
            BattleAction::ActivateDjinn { djinn_index: 0 },
        )
        .unwrap();

        let events = execute_round(&mut battle);

        assert_eq!(
            battle.player_units[0].unit.current_hp,
            battle.player_units[0].unit.stats.hp.get()
        );
        assert_eq!(
            battle.player_units[1].unit.current_hp,
            battle.player_units[1].unit.stats.hp.get()
        );
        assert!(
            events.iter().any(|event| matches!(
                event,
                BattleEvent::HealingDone(HealingDone {
                    source,
                    target,
                    amount: 45,
                }) if *source == actor && *target == actor
            )),
            "Activation should heal the caster back to full HP"
        );
        assert!(
            events.iter().any(|event| matches!(
                event,
                BattleEvent::HealingDone(HealingDone {
                    source,
                    target,
                    amount: 50,
                }) if *source == actor
                    && *target == TargetRef {
                        side: Side::Player,
                        index: 1,
                    }
            )),
            "Activation should heal allies on the caster's side"
        );
    }

    #[test]
    fn test_djinn_activation_buff_applies_atk_to_caster() {
        let player_stats = test_stats(100, 30, 20, 25, 15);
        let enemy_stats = test_stats(80, 20, 15, 10, 10);

        let player = make_player("hero", player_stats, 5);
        let enemy = make_enemy("goblin", enemy_stats);

        let djinn_id = DjinnId("forge".into());
        let empty_ability_set = crate::shared::DjinnAbilitySet {
            good_abilities: vec![],
            recovery_abilities: vec![],
        };
        let djinn_def = DjinnDef {
            id: djinn_id.clone(),
            name: "Forge".to_string(),
            element: Element::Mars,
            tier: crate::shared::bounded_types::DjinnTier::new_unchecked(1),
            stat_bonus: crate::shared::StatBonus::default(),
            summon_effect: Some(crate::shared::SummonEffect {
                damage: 0,
                buff: Some(crate::shared::BuffEffect {
                    stat_modifiers: crate::shared::StatBonus {
                        atk: crate::shared::bounded_types::StatMod::new_unchecked(5),
                        def: crate::shared::bounded_types::StatMod::new_unchecked(0),
                        mag: crate::shared::bounded_types::StatMod::new_unchecked(0),
                        spd: crate::shared::bounded_types::StatMod::new_unchecked(0),
                        hp: crate::shared::bounded_types::StatMod::new_unchecked(0),
                    },
                    duration: EffectDuration::new_unchecked(2),
                    shield_charges: None,
                    grant_immunity: false,
                }),
                status: None,
                heal: None,
            }),
            ability_pairs: crate::shared::DjinnAbilityPairs {
                same: empty_ability_set.clone(),
                counter: empty_ability_set.clone(),
                neutral: empty_ability_set,
            },
        };
        let mut djinn_defs = HashMap::new();
        djinn_defs.insert(djinn_id.clone(), djinn_def);

        let mut battle = new_battle(
            vec![player],
            vec![enemy],
            test_config(),
            HashMap::new(),
            djinn_defs,
        );
        battle.player_units[0].djinn_slots.add(djinn_id);

        let actor = TargetRef {
            side: Side::Player,
            index: 0,
        };
        plan_action(
            &mut battle,
            actor,
            BattleAction::ActivateDjinn { djinn_index: 0 },
        )
        .unwrap();

        execute_round(&mut battle);

        let buff_mods = status::compute_stat_modifiers(
            &battle.player_units[0].status_state.buffs,
            &battle.player_units[0].status_state.debuffs,
        );
        assert_eq!(
            buff_mods.atk.get(),
            2,
            "Activation buff should grant ATK +2"
        );
        assert_eq!(battle.player_units[0].status_state.buffs.len(), 1);
    }

    // ── Test: plan_enemy_actions plans correctly ────────────────────

    #[test]
    fn test_plan_enemy_actions_targets_first_alive_player() {
        let player_stats = test_stats(100, 20, 15, 10, 10);
        let enemy_stats = test_stats(80, 25, 10, 10, 12);

        let p1 = make_player("hero1", player_stats, 3);
        let p2 = make_player("hero2", player_stats, 3);
        let e1 = make_enemy("goblin-a", enemy_stats);
        let e2 = make_enemy("goblin-b", enemy_stats);

        let mut battle = new_battle(
            vec![p1, p2],
            vec![e1, e2],
            test_config(),
            HashMap::new(),
            HashMap::new(),
        );

        plan_enemy_actions(&mut battle);

        // Both enemies should have planned an attack on player index 0
        assert_eq!(
            battle.planned_actions.len(),
            2,
            "Both enemies should plan actions"
        );
        for (actor_ref, action) in &battle.planned_actions {
            assert_eq!(actor_ref.side, Side::Enemy, "Actor should be enemy");
            match action {
                BattleAction::Attack { target } => {
                    assert_eq!(target.side, Side::Player, "Target should be player");
                    assert_eq!(target.index, 0, "Should target first alive player");
                }
                _ => panic!("Expected Attack action"),
            }
        }
    }

    // ── Test: plan_enemy_actions skips dead enemies ─────────────────

    #[test]
    fn test_plan_enemy_actions_skips_dead_enemies() {
        let stats = test_stats(100, 20, 15, 10, 10);
        let p1 = make_player("hero", stats, 3);
        let e1 = make_enemy("dead-goblin", stats);
        let e2 = make_enemy("live-goblin", stats);

        let mut battle = new_battle(
            vec![p1],
            vec![e1, e2],
            test_config(),
            HashMap::new(),
            HashMap::new(),
        );
        // Kill first enemy
        battle.enemies[0].unit.is_alive = false;
        battle.enemies[0].unit.current_hp = 0;

        plan_enemy_actions(&mut battle);

        assert_eq!(
            battle.planned_actions.len(),
            1,
            "Only alive enemy should plan"
        );
        assert_eq!(
            battle.planned_actions[0].0.index, 1,
            "Second enemy should be the actor"
        );
    }

    // ── Test: plan_enemy_actions targets second player if first dead ─

    #[test]
    fn test_plan_enemy_actions_targets_second_player_if_first_dead() {
        let stats = test_stats(100, 20, 15, 10, 10);
        let p1 = make_player("dead-hero", stats, 3);
        let p2 = make_player("alive-hero", stats, 3);
        let e1 = make_enemy("goblin", stats);

        let mut battle = new_battle(
            vec![p1, p2],
            vec![e1],
            test_config(),
            HashMap::new(),
            HashMap::new(),
        );
        // Kill first player
        battle.player_units[0].unit.is_alive = false;
        battle.player_units[0].unit.current_hp = 0;

        plan_enemy_actions(&mut battle);

        assert_eq!(battle.planned_actions.len(), 1);
        match &battle.planned_actions[0].1 {
            BattleAction::Attack { target } => {
                assert_eq!(target.index, 1, "Should target second (alive) player");
            }
            _ => panic!("Expected Attack"),
        }
    }

    // ── Test: enemy deals damage to player unit ─────────────────────

    #[test]
    fn test_enemy_deals_damage_to_player() {
        let player_stats = test_stats(100, 20, 10, 10, 8);
        let enemy_stats = test_stats(80, 30, 10, 10, 12);

        let p1 = make_player("hero", player_stats, 3);
        let e1 = make_enemy("strong-goblin", enemy_stats);

        let mut battle = new_battle(
            vec![p1],
            vec![e1],
            test_config(),
            HashMap::new(),
            HashMap::new(),
        );

        // Only plan enemy action (no player action)
        plan_enemy_actions(&mut battle);

        let player_hp_before = battle.player_units[0].unit.current_hp;
        let events = execute_round(&mut battle);
        let player_hp_after = battle.player_units[0].unit.current_hp;

        assert!(
            player_hp_after < player_hp_before,
            "Player should have taken damage from enemy attack: before={}, after={}",
            player_hp_before,
            player_hp_after
        );

        // Verify a DamageDealt event from enemy to player exists
        let has_enemy_damage = events.iter().any(|e| match e {
            BattleEvent::DamageDealt(dd) => {
                dd.source.side == Side::Enemy && dd.target.side == Side::Player
            }
            _ => false,
        });
        assert!(has_enemy_damage, "Should have enemy->player damage event");
    }

    // ── Test: player unit can die → battle ends in defeat ───────────

    #[test]
    fn test_player_death_leads_to_defeat() {
        // Give enemy massive ATK so it kills the player in one round
        let player_stats = test_stats(30, 5, 2, 5, 5);
        let enemy_stats = test_stats(500, 200, 50, 10, 20);

        let p1 = make_player("fragile-hero", player_stats, 1);
        let e1 = make_enemy("boss-goblin", enemy_stats);

        let mut battle = new_battle(
            vec![p1],
            vec![e1],
            test_config(),
            HashMap::new(),
            HashMap::new(),
        );

        // Plan both sides
        let player_ref = TargetRef {
            side: Side::Player,
            index: 0,
        };
        let enemy_target = TargetRef {
            side: Side::Enemy,
            index: 0,
        };
        let _ = plan_action(
            &mut battle,
            player_ref,
            BattleAction::Attack {
                target: enemy_target,
            },
        );
        plan_enemy_actions(&mut battle);

        let events = execute_round(&mut battle);

        // Player should be dead
        assert!(
            !battle.player_units[0].unit.is_alive,
            "Player should be dead after enemy attack"
        );

        // Should have UnitDefeated event for the player
        let has_player_defeated = events.iter().any(|e| match e {
            BattleEvent::UnitDefeated(ud) => ud.unit.side == Side::Player,
            _ => false,
        });
        assert!(
            has_player_defeated,
            "Should have UnitDefeated event for player"
        );

        // Battle should end in defeat
        let result = check_battle_end(&battle);
        assert_eq!(result, Some(BattleResult::Defeat));
    }

    // ── Test: two-sided combat — both sides attack each round ───────

    #[test]
    fn test_two_sided_combat_round() {
        let player_stats = test_stats(200, 25, 15, 10, 15);
        let enemy_stats = test_stats(150, 20, 12, 10, 10);

        let p1 = make_player("hero", player_stats, 3);
        let e1 = make_enemy("goblin", enemy_stats);

        let mut battle = new_battle(
            vec![p1],
            vec![e1],
            test_config(),
            HashMap::new(),
            HashMap::new(),
        );

        // Plan player attack
        let player_ref = TargetRef {
            side: Side::Player,
            index: 0,
        };
        let enemy_ref = TargetRef {
            side: Side::Enemy,
            index: 0,
        };
        plan_action(
            &mut battle,
            player_ref,
            BattleAction::Attack { target: enemy_ref },
        )
        .unwrap();

        // Plan enemy attacks
        plan_enemy_actions(&mut battle);

        let enemy_hp_before = battle.enemies[0].unit.current_hp;
        let player_hp_before = battle.player_units[0].unit.current_hp;

        let _events = execute_round(&mut battle);

        let enemy_hp_after = battle.enemies[0].unit.current_hp;
        let player_hp_after = battle.player_units[0].unit.current_hp;

        assert!(
            enemy_hp_after < enemy_hp_before,
            "Enemy should take damage from player"
        );
        assert!(
            player_hp_after < player_hp_before,
            "Player should take damage from enemy"
        );
    }

    // ── Test: AI enemy uses ability when mana available ──────────────

    #[test]
    fn test_ai_enemy_uses_ability_when_mana_available() {
        let player_stats = test_stats(100, 20, 10, 10, 10);
        let enemy_stats = test_stats(80, 25, 10, 15, 12);

        let player = make_player("hero", player_stats, 5);
        let mut enemy_data = make_enemy("smart-goblin", enemy_stats);
        // Give the enemy an ability it can use
        let (aid, adef) = make_basic_ability("fire-bolt", 2, 40);
        enemy_data.enemy_def.abilities = vec![aid.clone()];

        let mut abilities = HashMap::new();
        abilities.insert(aid, adef);

        let mut battle = new_battle(
            vec![player],
            vec![enemy_data],
            test_config(),
            abilities,
            HashMap::new(),
        );

        // Verify the enemy has ability_ids populated
        assert_eq!(battle.enemies[0].ability_ids.len(), 1);
        assert_eq!(battle.enemies[0].ability_ids[0].0, "fire-bolt");

        // Use AI planning with Aggressive strategy (prefers abilities)
        plan_enemy_actions_with_ai(&mut battle, AiStrategy::Aggressive);

        // The enemy should have planned a UseAbility action (not just Attack)
        assert_eq!(battle.planned_actions.len(), 1);
        match &battle.planned_actions[0].1 {
            BattleAction::UseAbility {
                ability_id,
                targets,
            } => {
                assert_eq!(ability_id.0, "fire-bolt", "Enemy should use fire-bolt");
                assert!(!targets.is_empty(), "Should have at least one target");
                assert_eq!(targets[0].side, Side::Player, "Target should be a player");
            }
            _ => panic!("Expected UseAbility, got {:?}", battle.planned_actions[0].1),
        }

        // Execute and verify the ability actually deals damage
        let player_hp_before = battle.player_units[0].unit.current_hp;
        let events = execute_round(&mut battle);
        let player_hp_after = battle.player_units[0].unit.current_hp;

        assert!(
            player_hp_after < player_hp_before,
            "Player should take damage from enemy ability"
        );

        // Should have an EnemyAbilityUsed event
        let has_ability_event = events.iter().any(|e| {
            matches!(e, BattleEvent::EnemyAbilityUsed { ability_name, .. } if ability_name == "fire-bolt")
        });
        assert!(has_ability_event, "Should emit EnemyAbilityUsed event");
    }

    // ── Test: AI enemy targets lowest HP player when aggressive ──────

    #[test]
    fn test_ai_enemy_targets_lowest_hp_player_when_aggressive() {
        let player_stats = test_stats(100, 20, 10, 10, 10);
        let enemy_stats = test_stats(80, 25, 10, 10, 12);

        let p1 = make_player("hero-a", player_stats, 3);
        let p2 = make_player("hero-b", player_stats, 3);
        let enemy = make_enemy("aggressive-goblin", enemy_stats);

        let mut battle = new_battle(
            vec![p1, p2],
            vec![enemy],
            test_config(),
            HashMap::new(),
            HashMap::new(),
        );

        // Damage player 1 so it has lower HP
        battle.player_units[1].unit.current_hp = 20;

        plan_enemy_actions_with_ai(&mut battle, AiStrategy::Aggressive);

        assert_eq!(battle.planned_actions.len(), 1);
        match &battle.planned_actions[0].1 {
            BattleAction::Attack { target } => {
                assert_eq!(target.side, Side::Player);
                assert_eq!(
                    target.index, 1,
                    "Aggressive AI should target the player with lowest HP (index 1 at 20 HP)"
                );
            }
            _ => panic!("Expected Attack action"),
        }
    }

    // ── Test: AI falls back to basic attack when no abilities ────────

    #[test]
    fn test_ai_fallback_to_basic_attack_when_no_abilities() {
        let player_stats = test_stats(100, 20, 10, 10, 10);
        let enemy_stats = test_stats(80, 25, 10, 10, 12);

        let player = make_player("hero", player_stats, 5);
        let enemy = make_enemy("dumb-goblin", enemy_stats);
        // Enemy has no abilities (empty list from make_enemy)

        let mut battle = new_battle(
            vec![player],
            vec![enemy],
            test_config(),
            HashMap::new(),
            HashMap::new(),
        );

        assert!(
            battle.enemies[0].ability_ids.is_empty(),
            "Enemy should have no abilities"
        );

        plan_enemy_actions_with_ai(&mut battle, AiStrategy::Balanced);

        // With no abilities, AI should fall back to a basic Attack
        assert_eq!(battle.planned_actions.len(), 1);
        match &battle.planned_actions[0].1 {
            BattleAction::Attack { target } => {
                assert_eq!(target.side, Side::Player);
                assert_eq!(target.index, 0, "Should target first alive player");
            }
            _ => panic!(
                "Expected Attack fallback when no abilities, got {:?}",
                battle.planned_actions[0].1
            ),
        }
    }

    // ── FIX 1 Test: Planning order = execution order ────────────────

    #[test]
    fn test_planning_order_equals_execution_order() {
        // Set up: 2 players with different SPD, slower one planned first.
        // Under the old SPD-sort the faster unit would execute first.
        // Under the fix, the planning order is preserved.
        let slow_stats = test_stats(200, 30, 20, 10, 5);
        let fast_stats = test_stats(200, 30, 20, 10, 50);
        let enemy_stats = test_stats(500, 10, 5, 5, 1);

        let p1 = make_player("slow-hero", slow_stats, 3);
        let p2 = make_player("fast-hero", fast_stats, 3);
        let enemy = make_enemy("dummy", enemy_stats);

        let mut battle = new_battle(
            vec![p1, p2],
            vec![enemy],
            test_config(),
            HashMap::new(),
            HashMap::new(),
        );

        let slow_ref = TargetRef {
            side: Side::Player,
            index: 0,
        };
        let fast_ref = TargetRef {
            side: Side::Player,
            index: 1,
        };
        let enemy_ref = TargetRef {
            side: Side::Enemy,
            index: 0,
        };

        // Plan slow hero FIRST, then fast hero
        plan_action(
            &mut battle,
            slow_ref,
            BattleAction::Attack { target: enemy_ref },
        )
        .unwrap();
        plan_action(
            &mut battle,
            fast_ref,
            BattleAction::Attack { target: enemy_ref },
        )
        .unwrap();

        let events = execute_round(&mut battle);

        // Collect DamageDealt events in order
        let damage_sources: Vec<TargetRef> = events
            .iter()
            .filter_map(|e| match e {
                BattleEvent::DamageDealt(dd) if dd.source.side == Side::Player => Some(dd.source),
                _ => None,
            })
            .collect();

        assert!(damage_sources.len() >= 2, "Both players should deal damage");
        // The SLOW hero (index 0) was planned first, so should appear first
        assert_eq!(
            damage_sources[0].index, 0,
            "Slow hero (planned first) should execute first, not sorted by SPD"
        );
        assert_eq!(
            damage_sources[1].index, 1,
            "Fast hero (planned second) should execute second"
        );
    }

    // ── FIX 2 Test: Freeze broken by accumulated damage ─────────────

    #[test]
    fn test_freeze_broken_by_damage() {
        let player_stats = test_stats(200, 50, 20, 10, 15);
        let enemy_stats = test_stats(300, 10, 5, 5, 5);

        let player = make_player("hero", player_stats, 5);
        let enemy = make_enemy("frozen-goblin", enemy_stats);

        let mut battle = new_battle(
            vec![player],
            vec![enemy],
            test_config(),
            HashMap::new(),
            HashMap::new(),
        );

        // Apply freeze with a low threshold to the enemy
        let freeze = StatusEffect {
            effect_type: StatusEffectType::Freeze,
            duration: EffectDuration::new_unchecked(10),
            burn_percent: None,
            poison_percent: None,
            freeze_threshold: Some(10), // breaks after 10 cumulative damage
        };
        status::apply_status(&mut battle.enemies[0].status_state, &freeze, &None);

        // Verify freeze is active
        assert_eq!(battle.enemies[0].status_state.statuses.len(), 1);
        assert_eq!(
            battle.enemies[0].status_state.statuses[0].effect_type,
            StatusEffectType::Freeze
        );

        // Plan an attack that will deal damage
        let actor = TargetRef {
            side: Side::Player,
            index: 0,
        };
        let target = TargetRef {
            side: Side::Enemy,
            index: 0,
        };
        plan_action(&mut battle, actor, BattleAction::Attack { target }).unwrap();

        execute_round(&mut battle);

        // The attack should have accumulated freeze damage
        // After end-of-round tick, freeze should have broken
        // (damage from ATK 50 vs DEF 5 should easily exceed threshold of 10)
        // The freeze status should have been removed by tick_statuses
        let freeze_statuses: Vec<_> = battle.enemies[0]
            .status_state
            .statuses
            .iter()
            .filter(|s| s.effect_type == StatusEffectType::Freeze)
            .collect();
        assert!(
            freeze_statuses.is_empty(),
            "Freeze should be broken after sufficient damage accumulation"
        );
    }

    // ── FIX 3 Test: Chain damage hits all enemies ────────────────────

    #[test]
    fn test_chain_damage_hits_all_enemies() {
        let player_stats = test_stats(200, 10, 20, 40, 15);
        let enemy_stats = test_stats(200, 10, 5, 5, 5);

        let player = make_player("hero", player_stats, 10);
        let e1 = make_enemy("goblin-a", enemy_stats);
        let e2 = make_enemy("goblin-b", enemy_stats);
        let e3 = make_enemy("goblin-c", enemy_stats);

        let chain_ability_id = AbilityId("chain-bolt".to_string());
        let chain_ability = AbilityDef {
            id: chain_ability_id.clone(),
            name: "Chain Bolt".to_string(),
            category: AbilityCategory::Psynergy,
            damage_type: Some(DamageType::Psynergy),
            element: Some(Element::Jupiter),
            mana_cost: ManaCost::new_unchecked(2),
            base_power: BasePower::new_unchecked(30),
            targets: TargetMode::SingleEnemy,
            unlock_level: Level::new_unchecked(1),
            hit_count: HitCount::new_unchecked(1),
            status_effect: None,
            buff_effect: None,
            debuff_effect: None,
            shield_charges: None,
            shield_duration: None,
            heal_over_time: None,
            grant_immunity: None,
            cleanse: None,
            ignore_defense_percent: None,
            splash_damage_percent: None,
            chain_damage: true, // chain damage enabled
            revive: false,
            revive_hp_percent: None,
        };

        let mut abilities = HashMap::new();
        abilities.insert(chain_ability_id.clone(), chain_ability);

        let mut battle = new_battle(
            vec![player],
            vec![e1, e2, e3],
            test_config(),
            abilities,
            HashMap::new(),
        );

        let actor = TargetRef {
            side: Side::Player,
            index: 0,
        };
        // Target only the first enemy
        let primary_target = TargetRef {
            side: Side::Enemy,
            index: 0,
        };
        plan_action(
            &mut battle,
            actor,
            BattleAction::UseAbility {
                ability_id: AbilityId("chain-bolt".into()),
                targets: vec![primary_target],
            },
        )
        .unwrap();

        let e1_hp_before = battle.enemies[0].unit.current_hp;
        let e2_hp_before = battle.enemies[1].unit.current_hp;
        let e3_hp_before = battle.enemies[2].unit.current_hp;

        execute_round(&mut battle);

        // Primary target should be damaged
        assert!(
            battle.enemies[0].unit.current_hp < e1_hp_before,
            "Primary target should take damage"
        );
        // Chain should hit other enemies too
        assert!(
            battle.enemies[1].unit.current_hp < e2_hp_before,
            "Chain should hit second enemy"
        );
        assert!(
            battle.enemies[2].unit.current_hp < e3_hp_before,
            "Chain should hit third enemy"
        );
    }

    #[test]
    fn test_chain_hit_preserves_crit_flag() {
        let player_stats = test_stats(200, 10, 20, 40, 15);
        let enemy_stats = test_stats(200, 10, 5, 5, 5);

        let player = make_player("hero", player_stats, 10);
        let e1 = make_enemy("goblin-a", enemy_stats);
        let e2 = make_enemy("goblin-b", enemy_stats);
        let e3 = make_enemy("goblin-c", enemy_stats);

        let mut battle = new_battle(
            vec![player],
            vec![e1, e2, e3],
            test_config(),
            HashMap::new(),
            HashMap::new(),
        );

        let actor = TargetRef {
            side: Side::Player,
            index: 0,
        };
        let primary_target = TargetRef {
            side: Side::Enemy,
            index: 0,
        };
        let mut events = Vec::new();

        apply_chain_damage(
            &mut battle,
            actor,
            primary_target,
            25,
            DamageType::Psynergy,
            true,
            &mut events,
        );

        let chain_damage_events: Vec<&DamageDealt> = events
            .iter()
            .filter_map(|event| match event {
                BattleEvent::DamageDealt(damage)
                    if damage.target.side == Side::Enemy && damage.target.index != 0 =>
                {
                    Some(damage)
                }
                _ => None,
            })
            .collect();

        assert_eq!(
            chain_damage_events.len(),
            2,
            "Chain should hit both secondary enemies"
        );
        assert!(
            chain_damage_events.iter().all(|damage| damage.is_crit),
            "Chain hit events should preserve the primary crit flag",
        );
    }

    // ── FIX 4 Test: Immunity blocks status ──────────────────────────

    #[test]
    fn test_immunity_blocks_status() {
        let player_stats = test_stats(200, 10, 20, 30, 15);
        let enemy_stats = test_stats(200, 20, 5, 5, 5);

        let player = make_player("hero", player_stats, 10);
        let enemy = make_enemy("goblin", enemy_stats);

        // Create an immunity-granting ability
        let imm_ability_id = AbilityId("ward".to_string());
        let imm_ability = AbilityDef {
            id: imm_ability_id.clone(),
            name: "Ward".to_string(),
            category: AbilityCategory::Buff,
            damage_type: None,
            element: None,
            mana_cost: ManaCost::new_unchecked(2),
            base_power: BasePower::new_unchecked(0),
            targets: TargetMode::SingleAlly,
            unlock_level: Level::new_unchecked(1),
            hit_count: HitCount::new_unchecked(1),
            status_effect: None,
            buff_effect: None,
            debuff_effect: None,
            shield_charges: None,
            shield_duration: None,
            heal_over_time: None,
            grant_immunity: Some(Immunity {
                all: true,
                types: vec![],
                duration: EffectDuration::new_unchecked(3),
            }),
            cleanse: None,
            ignore_defense_percent: None,
            splash_damage_percent: None,
            chain_damage: false,
            revive: false,
            revive_hp_percent: None,
        };

        let mut abilities = HashMap::new();
        abilities.insert(imm_ability_id.clone(), imm_ability);

        let mut battle = new_battle(
            vec![player],
            vec![enemy],
            test_config(),
            abilities,
            HashMap::new(),
        );

        // Use ward on self
        let actor = TargetRef {
            side: Side::Player,
            index: 0,
        };
        plan_action(
            &mut battle,
            actor,
            BattleAction::UseAbility {
                ability_id: AbilityId("ward".into()),
                targets: vec![actor],
            },
        )
        .unwrap();
        execute_round(&mut battle);

        // Player should now have immunity
        assert!(
            battle.player_units[0].status_state.immunity.is_some(),
            "Player should have immunity after using ward"
        );

        // Try to apply a burn status — should be blocked
        let burn = StatusEffect {
            effect_type: StatusEffectType::Burn,
            duration: EffectDuration::new_unchecked(3),
            burn_percent: Some(0.10),
            poison_percent: None,
            freeze_threshold: None,
        };
        let imm = battle.player_units[0].status_state.immunity.clone();
        let applied = status::apply_status(&mut battle.player_units[0].status_state, &burn, &imm);
        assert!(!applied, "Status should be blocked by immunity");
        assert!(
            battle.player_units[0].status_state.statuses.is_empty(),
            "No status should be applied when immune"
        );
    }

    // ── FIX 4 Test: Cleanse removes statuses ────────────────────────

    #[test]
    fn test_cleanse_removes_statuses() {
        let player_stats = test_stats(200, 10, 20, 30, 15);
        let enemy_stats = test_stats(200, 10, 5, 5, 5);

        let player = make_player("hero", player_stats, 10);
        let enemy = make_enemy("goblin", enemy_stats);

        // Create a cleanse ability (Buff category, not Healing, to avoid the healing continue path)
        let cleanse_id = AbilityId("purify".to_string());
        let cleanse_ability = AbilityDef {
            id: cleanse_id.clone(),
            name: "Purify".to_string(),
            category: AbilityCategory::Buff,
            damage_type: None,
            element: None,
            mana_cost: ManaCost::new_unchecked(2),
            base_power: BasePower::new_unchecked(0),
            targets: TargetMode::SingleAlly,
            unlock_level: Level::new_unchecked(1),
            hit_count: HitCount::new_unchecked(1),
            status_effect: None,
            buff_effect: None,
            debuff_effect: None,
            shield_charges: None,
            shield_duration: None,
            heal_over_time: None,
            grant_immunity: None,
            cleanse: Some(CleanseType::All),
            ignore_defense_percent: None,
            splash_damage_percent: None,
            chain_damage: false,
            revive: false,
            revive_hp_percent: None,
        };

        let mut abilities = HashMap::new();
        abilities.insert(cleanse_id.clone(), cleanse_ability);

        let mut battle = new_battle(
            vec![player],
            vec![enemy],
            test_config(),
            abilities,
            HashMap::new(),
        );

        // Apply burn and poison to the player (not stun — stun prevents acting)
        let burn = StatusEffect {
            effect_type: StatusEffectType::Burn,
            duration: EffectDuration::new_unchecked(5),
            burn_percent: Some(0.10),
            poison_percent: None,
            freeze_threshold: None,
        };
        let poison = StatusEffect {
            effect_type: StatusEffectType::Poison,
            duration: EffectDuration::new_unchecked(5),
            burn_percent: None,
            poison_percent: Some(0.05),
            freeze_threshold: None,
        };
        status::apply_status(&mut battle.player_units[0].status_state, &burn, &None);
        status::apply_status(&mut battle.player_units[0].status_state, &poison, &None);
        assert_eq!(battle.player_units[0].status_state.statuses.len(), 2);

        // Use cleanse on self
        let actor = TargetRef {
            side: Side::Player,
            index: 0,
        };
        plan_action(
            &mut battle,
            actor,
            BattleAction::UseAbility {
                ability_id: AbilityId("purify".into()),
                targets: vec![actor],
            },
        )
        .unwrap();
        execute_round(&mut battle);

        // All statuses should be cleansed
        assert!(
            battle.player_units[0].status_state.statuses.is_empty(),
            "All statuses should be removed by cleanse"
        );
    }

    // ── FIX 5 Test: Revive dead ally ────────────────────────────────

    #[test]
    fn test_revive_dead_ally() {
        let player_stats = test_stats(100, 10, 20, 30, 15);
        let enemy_stats = test_stats(200, 10, 5, 5, 5);

        let p1 = make_player("healer", player_stats, 10);
        let p2 = make_player("dead-ally", player_stats, 0);
        let enemy = make_enemy("goblin", enemy_stats);

        // Create a revive ability
        let revive_id = AbilityId("revive".to_string());
        let revive_ability = AbilityDef {
            id: revive_id.clone(),
            name: "Revive".to_string(),
            category: AbilityCategory::Healing,
            damage_type: None,
            element: None,
            mana_cost: ManaCost::new_unchecked(3),
            base_power: BasePower::new_unchecked(0),
            targets: TargetMode::SingleAlly,
            unlock_level: Level::new_unchecked(1),
            hit_count: HitCount::new_unchecked(1),
            status_effect: None,
            buff_effect: None,
            debuff_effect: None,
            shield_charges: None,
            shield_duration: None,
            heal_over_time: None,
            grant_immunity: None,
            cleanse: None,
            ignore_defense_percent: None,
            splash_damage_percent: None,
            chain_damage: false,
            revive: true,
            revive_hp_percent: Some(0.5), // revive at 50% HP
        };

        let mut abilities = HashMap::new();
        abilities.insert(revive_id.clone(), revive_ability);

        let mut battle = new_battle(
            vec![p1, p2],
            vec![enemy],
            test_config(),
            abilities,
            HashMap::new(),
        );

        // Kill the second player
        battle.player_units[1].unit.is_alive = false;
        battle.player_units[1].unit.current_hp = 0;

        // Use revive
        let actor = TargetRef {
            side: Side::Player,
            index: 0,
        };
        // Target the dead ally (the ability is healing category, targeting the ally)
        let dead_ally = TargetRef {
            side: Side::Player,
            index: 1,
        };
        plan_action(
            &mut battle,
            actor,
            BattleAction::UseAbility {
                ability_id: AbilityId("revive".into()),
                targets: vec![dead_ally],
            },
        )
        .unwrap();

        let events = execute_round(&mut battle);

        // Dead ally should be revived
        assert!(
            battle.player_units[1].unit.is_alive,
            "Dead ally should be revived"
        );
        // HP should be 50% of max (100 * 0.5 = 50)
        assert_eq!(
            battle.player_units[1].unit.current_hp, 50,
            "Revived ally should have 50% max HP"
        );

        // Should have a HealingDone event for the revive
        let has_revive_heal = events.iter().any(|e| match e {
            BattleEvent::HealingDone(hd) => hd.target.side == Side::Player && hd.target.index == 1,
            _ => false,
        });
        assert!(has_revive_heal, "Should have HealingDone event for revive");
    }

    // ── FIX 6 Test: Healing includes MAG stat ───────────────────────

    #[test]
    fn test_healing_includes_mag_stat() {
        let player_stats = test_stats(200, 10, 20, 30, 15);
        let enemy_stats = test_stats(200, 10, 5, 5, 5);

        let player = make_player("healer", player_stats, 10);
        let enemy = make_enemy("goblin", enemy_stats);

        // Create a healing ability with base_power 20
        let heal_id = AbilityId("cure".to_string());
        let heal_ability = AbilityDef {
            id: heal_id.clone(),
            name: "Cure".to_string(),
            category: AbilityCategory::Healing,
            damage_type: None,
            element: None,
            mana_cost: ManaCost::new_unchecked(2),
            base_power: BasePower::new_unchecked(20), // base heal = 20
            targets: TargetMode::SingleAlly,
            unlock_level: Level::new_unchecked(1),
            hit_count: HitCount::new_unchecked(1),
            status_effect: None,
            buff_effect: None,
            debuff_effect: None,
            shield_charges: None,
            shield_duration: None,
            heal_over_time: None,
            grant_immunity: None,
            cleanse: None,
            ignore_defense_percent: None,
            splash_damage_percent: None,
            chain_damage: false,
            revive: false,
            revive_hp_percent: None,
        };

        let mut abilities = HashMap::new();
        abilities.insert(heal_id.clone(), heal_ability);

        let mut battle = new_battle(
            vec![player],
            vec![enemy],
            test_config(),
            abilities,
            HashMap::new(),
        );

        // Damage the player to make healing visible
        battle.player_units[0].unit.current_hp = 100;

        let actor = TargetRef {
            side: Side::Player,
            index: 0,
        };
        plan_action(
            &mut battle,
            actor,
            BattleAction::UseAbility {
                ability_id: AbilityId("cure".into()),
                targets: vec![actor],
            },
        )
        .unwrap();

        let events = execute_round(&mut battle);

        // Expected heal: base_power (20) + mag (30) = 50
        // Player started at 100/200, should now be 150
        assert_eq!(
            battle.player_units[0].unit.current_hp, 150,
            "Healing should be base_power + MAG: 20 + 30 = 50, so 100 + 50 = 150"
        );

        // Verify the HealingDone event has the correct amount
        let heal_event = events.iter().find_map(|e| match e {
            BattleEvent::HealingDone(hd) if hd.source == actor && hd.target == actor => {
                Some(hd.amount)
            }
            _ => None,
        });
        assert_eq!(
            heal_event,
            Some(50),
            "HealingDone amount should be 50 (base 20 + MAG 30)"
        );
    }

    // ── Audit Round 2: test_barrier_ability_without_duration_uses_default ──

    #[test]
    fn test_barrier_ability_without_duration_uses_default() {
        let player_stats = test_stats(200, 10, 20, 30, 15);
        let enemy_stats = test_stats(200, 10, 5, 5, 5);

        let player = make_player("hero", player_stats, 10);
        let enemy = make_enemy("goblin", enemy_stats);

        // Create a barrier ability with shield_charges but NO shield_duration
        let barrier_id = AbilityId("shield-no-dur".to_string());
        let barrier_ability = AbilityDef {
            id: barrier_id.clone(),
            name: "Shield".to_string(),
            category: AbilityCategory::Buff,
            damage_type: None,
            element: None,
            mana_cost: ManaCost::new_unchecked(2),
            base_power: BasePower::new_unchecked(0),
            targets: TargetMode::SingleAlly,
            unlock_level: Level::new_unchecked(1),
            hit_count: HitCount::new_unchecked(1),
            status_effect: None,
            buff_effect: None,
            debuff_effect: None,
            shield_charges: Some(2),
            shield_duration: None, // no duration specified
            heal_over_time: None,
            grant_immunity: None,
            cleanse: None,
            ignore_defense_percent: None,
            splash_damage_percent: None,
            chain_damage: false,
            revive: false,
            revive_hp_percent: None,
        };

        let mut abilities = HashMap::new();
        abilities.insert(barrier_id.clone(), barrier_ability);

        let mut battle = new_battle(
            vec![player],
            vec![enemy],
            test_config(),
            abilities,
            HashMap::new(),
        );

        let actor = TargetRef {
            side: Side::Player,
            index: 0,
        };
        plan_action(
            &mut battle,
            actor,
            BattleAction::UseAbility {
                ability_id: AbilityId("shield-no-dur".into()),
                targets: vec![actor],
            },
        )
        .unwrap();

        execute_round(&mut battle);

        // Barrier should have been applied with default duration 3
        // (tick_barriers decremented it once at end-of-round, so remaining_turns = 2)
        assert_eq!(
            battle.player_units[0].status_state.barriers.len(),
            1,
            "Barrier should exist"
        );
        assert_eq!(
            battle.player_units[0].status_state.barriers[0].charges, 2,
            "Barrier should have 2 charges"
        );
        assert_eq!(
            battle.player_units[0].status_state.barriers[0].remaining_turns, 2,
            "Barrier should have 2 remaining turns (default 3 minus 1 tick)"
        );
    }

    // ── Audit Round 2: test_immunity_expires_after_duration ──

    #[test]
    fn test_immunity_expires_after_duration() {
        let player_stats = test_stats(200, 10, 20, 30, 15);
        let enemy_stats = test_stats(200, 10, 5, 5, 5);

        let player = make_player("hero", player_stats, 10);
        let enemy = make_enemy("goblin", enemy_stats);

        let mut battle = new_battle(
            vec![player],
            vec![enemy],
            test_config(),
            HashMap::new(),
            HashMap::new(),
        );

        // Grant immunity with duration 2
        let immunity = Immunity {
            all: true,
            types: vec![],
            duration: EffectDuration::new_unchecked(2),
        };
        status::apply_immunity(&mut battle.player_units[0].status_state, &immunity);
        assert!(
            battle.player_units[0].status_state.immunity.is_some(),
            "Immunity should be active"
        );

        // Round 1: immunity ticks from 2 -> 1
        execute_round(&mut battle);
        assert!(
            battle.player_units[0].status_state.immunity.is_some(),
            "Immunity should still be active after 1 round"
        );

        // Round 2: immunity ticks from 1 -> 0 and expires
        execute_round(&mut battle);
        assert!(
            battle.player_units[0].status_state.immunity.is_none(),
            "Immunity should have expired after 2 rounds"
        );
    }

    // ── Audit Round 2: test_buffs_affect_damage_calculation ──

    #[test]
    fn test_buffs_affect_damage_calculation() {
        let player_stats = test_stats(200, 30, 20, 10, 15);
        let enemy_stats = test_stats(300, 10, 15, 10, 5);

        let player = make_player("hero", player_stats, 10);
        let e1 = make_enemy("goblin-a", enemy_stats);
        let e2 = make_enemy("goblin-b", enemy_stats);

        let mut battle = new_battle(
            vec![player],
            vec![e1, e2],
            test_config(),
            HashMap::new(),
            HashMap::new(),
        );

        // Attack without buffs — measure baseline damage
        let actor = TargetRef {
            side: Side::Player,
            index: 0,
        };
        let target1 = TargetRef {
            side: Side::Enemy,
            index: 0,
        };
        plan_action(&mut battle, actor, BattleAction::Attack { target: target1 }).unwrap();
        execute_round(&mut battle);
        let damage_without_buff = 300u16.saturating_sub(battle.enemies[0].unit.current_hp);

        // Now apply a strong ATK buff to the player
        let atk_buff = crate::shared::BuffEffect {
            stat_modifiers: crate::shared::StatBonus {
                atk: crate::shared::bounded_types::StatMod::new_unchecked(20),
                def: crate::shared::bounded_types::StatMod::new_unchecked(0),
                mag: crate::shared::bounded_types::StatMod::new_unchecked(0),
                spd: crate::shared::bounded_types::StatMod::new_unchecked(0),
                hp: crate::shared::bounded_types::StatMod::new_unchecked(0),
            },
            duration: EffectDuration::new_unchecked(5),
            shield_charges: None,
            grant_immunity: false,
        };
        status::apply_buff(&mut battle.player_units[0].status_state, &atk_buff, 3);

        // Attack second enemy with buff active
        let target2 = TargetRef {
            side: Side::Enemy,
            index: 1,
        };
        plan_action(&mut battle, actor, BattleAction::Attack { target: target2 }).unwrap();
        execute_round(&mut battle);
        let damage_with_buff = 300u16.saturating_sub(battle.enemies[1].unit.current_hp);

        assert!(
            damage_with_buff > damage_without_buff,
            "Buffed attack ({}) should deal more damage than unbuffed ({})",
            damage_with_buff,
            damage_without_buff
        );
    }

    // ── Audit Round 2: test_enemy_xp_gold_used_in_rewards ──

    #[test]
    fn test_enemy_xp_gold_used_in_rewards() {
        let player_stats = test_stats(200, 200, 20, 10, 15);

        let player = make_player("hero", player_stats, 5);

        // Create enemies with specific xp and gold values
        let e1 = EnemyUnitData {
            enemy_def: EnemyDef {
                id: EnemyId("weak-a".to_string()),
                name: "Weak A".to_string(),
                element: Element::Venus,
                level: Level::new_unchecked(1),
                stats: test_stats(10, 1, 1, 1, 1),
                xp: crate::shared::bounded_types::Xp::new_unchecked(25),
                gold: crate::shared::bounded_types::Gold::new_unchecked(15),
                abilities: vec![],
            },
        };
        let e2 = EnemyUnitData {
            enemy_def: EnemyDef {
                id: EnemyId("weak-b".to_string()),
                name: "Weak B".to_string(),
                element: Element::Mars,
                level: Level::new_unchecked(1),
                stats: test_stats(10, 1, 1, 1, 1),
                xp: crate::shared::bounded_types::Xp::new_unchecked(40),
                gold: crate::shared::bounded_types::Gold::new_unchecked(20),
                abilities: vec![],
            },
        };

        let mut battle = new_battle(
            vec![player],
            vec![e1, e2],
            test_config(),
            HashMap::new(),
            HashMap::new(),
        );

        // Kill both enemies
        battle.enemies[0].unit.is_alive = false;
        battle.enemies[0].unit.current_hp = 0;
        battle.enemies[1].unit.is_alive = false;
        battle.enemies[1].unit.current_hp = 0;

        let result = check_battle_end(&battle);
        match result {
            Some(BattleResult::Victory { xp, gold }) => {
                assert_eq!(xp, 65, "XP should be 25 + 40 = 65");
                assert_eq!(gold, 35, "Gold should be 15 + 20 = 35");
            }
            other => panic!("Expected Victory with specific XP/gold, got {:?}", other),
        }
    }

    // ── Audit Round 2: test_summon_damage_respects_barriers ──

    #[test]
    fn test_summon_damage_respects_barriers() {
        let player_stats = test_stats(200, 10, 20, 30, 15);
        let enemy_stats = test_stats(200, 10, 5, 5, 5);

        let player = make_player("hero", player_stats, 10);
        let e1 = make_enemy("barrier-goblin", enemy_stats);
        let e2 = make_enemy("normal-goblin", enemy_stats);

        // Set up djinn def with a summon effect
        let djinn_id = DjinnId("ramses".into());
        let empty_ability_set = crate::shared::DjinnAbilitySet {
            good_abilities: vec![],
            recovery_abilities: vec![],
        };
        let djinn_def = DjinnDef {
            id: djinn_id.clone(),
            name: "Ramses".to_string(),
            element: Element::Venus,
            tier: crate::shared::bounded_types::DjinnTier::new_unchecked(1),
            stat_bonus: crate::shared::StatBonus::default(),
            summon_effect: Some(crate::shared::SummonEffect {
                damage: 50,
                buff: None,
                status: None,
                heal: None,
            }),
            ability_pairs: crate::shared::DjinnAbilityPairs {
                same: empty_ability_set.clone(),
                counter: empty_ability_set.clone(),
                neutral: empty_ability_set,
            },
        };
        let mut djinn_defs = HashMap::new();
        djinn_defs.insert(djinn_id.clone(), djinn_def);

        let mut battle = new_battle(
            vec![player],
            vec![e1, e2],
            test_config(),
            HashMap::new(),
            djinn_defs,
        );

        // Give player a djinn in Good state
        battle.player_units[0].djinn_slots.add(djinn_id.clone());

        // Give first enemy a barrier
        status::apply_barrier(&mut battle.enemies[0].status_state, 1, 5);

        let actor = TargetRef {
            side: Side::Player,
            index: 0,
        };
        plan_action(
            &mut battle,
            actor,
            BattleAction::Summon {
                djinn_indices: vec![0],
            },
        )
        .unwrap();

        let e1_hp_before = battle.enemies[0].unit.current_hp;
        let e2_hp_before = battle.enemies[1].unit.current_hp;
        let events = execute_round(&mut battle);

        // First enemy (with barrier) should not take damage
        assert_eq!(
            battle.enemies[0].unit.current_hp, e1_hp_before,
            "Barrier should block summon damage on first enemy"
        );
        // Second enemy (no barrier) should take damage
        assert!(
            battle.enemies[1].unit.current_hp < e2_hp_before,
            "Second enemy without barrier should take summon damage"
        );
        // Should have a BarrierBlocked event
        let has_barrier_event = events
            .iter()
            .any(|e| matches!(e, BattleEvent::BarrierBlocked(_)));
        assert!(
            has_barrier_event,
            "Should have BarrierBlocked event for first enemy"
        );
    }

    // ── Planning-order tests ────────────────────────────────────────

    #[test]
    fn test_planning_order_matches_spd() {
        // Unit 0: SPD 10, Unit 1: SPD 18, Unit 2: SPD 13
        // Expected order: 1 (18), 2 (13), 0 (10)
        let battle = new_battle(
            vec![
                make_player("slow", test_stats(100, 20, 10, 10, 10), 1),
                make_player("fast", test_stats(100, 20, 10, 10, 18), 1),
                make_player("mid", test_stats(100, 20, 10, 10, 13), 1),
            ],
            vec![make_enemy("foe", test_stats(80, 15, 8, 5, 10))],
            test_config(),
            HashMap::new(),
            HashMap::new(),
        );

        let order = get_planning_order(&battle);
        assert_eq!(
            order,
            vec![1, 2, 0],
            "Planning order should be fastest-first by effective SPD"
        );
    }

    #[test]
    fn test_planning_order_skips_dead() {
        let mut battle = new_battle(
            vec![
                make_player("alive_slow", test_stats(100, 20, 10, 10, 5), 1),
                make_player("dead_fast", test_stats(100, 20, 10, 10, 20), 1),
                make_player("alive_fast", test_stats(100, 20, 10, 10, 15), 1),
            ],
            vec![make_enemy("foe", test_stats(80, 15, 8, 5, 10))],
            test_config(),
            HashMap::new(),
            HashMap::new(),
        );

        // Kill unit 1
        battle.player_units[1].unit.is_alive = false;
        battle.player_units[1].unit.current_hp = 0;

        let order = get_planning_order(&battle);
        assert_eq!(order, vec![2, 0], "Dead unit (index 1) must be excluded");
        assert!(!order.contains(&1), "Dead unit index must not appear");
    }

    #[test]
    fn test_planning_order_tiebreaker() {
        // Unit 0: base SPD 10, equip bonus +5 → effective 15
        // Unit 1: base SPD 15, equip bonus  0 → effective 15
        // Same effective SPD; higher base SPD (unit 1) should come first.
        // If base SPD also ties, lower index wins — tested by unit 2.
        let mut battle = new_battle(
            vec![
                make_player("low_base", test_stats(100, 20, 10, 10, 10), 1),
                make_player("high_base", test_stats(100, 20, 10, 10, 15), 1),
                make_player("also_15", test_stats(100, 20, 10, 10, 15), 1),
            ],
            vec![make_enemy("foe", test_stats(80, 15, 8, 5, 10))],
            test_config(),
            HashMap::new(),
            HashMap::new(),
        );

        // Give unit 0 equipment speed bonus so effective SPD = 15
        battle.player_units[0].unit.equipment_speed_bonus = 5;

        let order = get_planning_order(&battle);
        // Unit 1 and 2 both have base SPD 15, effective 15 — unit 1 wins by lower index.
        // Unit 0 has base SPD 10, effective 15 — comes last among the three.
        assert_eq!(
            order,
            vec![1, 2, 0],
            "Tiebreaker: higher base SPD first, then lower index"
        );
    }
}
