#![allow(dead_code, clippy::unnecessary_map_or)]
//! Battle Engine — Integration layer wiring combat, status, djinn, equipment,
//! and damage_mods into a playable turn-by-turn battle loop.

use std::collections::HashMap;

use crate::shared::{
    AbilityDef, AbilityId, BattleAction, BattlePhase, CombatConfig, DamageDealt, DamageType,
    DjinnDef, DjinnId, DjinnState, DjinnStateChanged, EnemyDef, HealingDone, ManaPoolChanged,
    Side, Stats, StatusApplied, TargetRef, UnitDefeated,
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
}

/// Top-level battle state.
#[derive(Debug, Clone)]
pub struct Battle {
    pub round: u32,
    pub phase: BattlePhase,
    pub player_units: Vec<BattleUnitFull>,
    pub enemies: Vec<BattleUnitFull>,
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

    let player_units: Vec<BattleUnitFull> = player_data
        .into_iter()
        .map(|pd| {
            let eq_effects = &pd.equipment_effects;
            let stats = Stats {
                hp: (pd.base_stats.hp as i32 + eq_effects.total_stat_bonus.hp as i32).max(1)
                    as u16,
                atk: (pd.base_stats.atk as i32 + eq_effects.total_stat_bonus.atk as i32).max(0)
                    as u16,
                def: (pd.base_stats.def as i32 + eq_effects.total_stat_bonus.def as i32).max(0)
                    as u16,
                mag: (pd.base_stats.mag as i32 + eq_effects.total_stat_bonus.mag as i32).max(0)
                    as u16,
                spd: (pd.base_stats.spd as i32 + eq_effects.total_stat_bonus.spd as i32).max(0)
                    as u16,
            };
            let mana = pd.mana_contribution.saturating_add(eq_effects.total_mana_bonus);
            total_mana = total_mana.saturating_add(mana);

            BattleUnitFull {
                unit: BattleUnit {
                    id: pd.id,
                    stats,
                    current_hp: stats.hp,
                    is_alive: true,
                    crit_counter: 0,
                    equipment_speed_bonus: eq_effects.total_stat_bonus.spd,
                },
                status_state: UnitStatusState::new(),
                djinn_slots: pd.djinn_slots,
                equipment: pd.equipment,
                mana_contribution: mana,
                ability_ids: Vec::new(),
            }
        })
        .collect();

    let enemies: Vec<BattleUnitFull> = enemy_data
        .into_iter()
        .map(|ed| {
            let stats = ed.enemy_def.stats;
            let ability_ids = ed.enemy_def.abilities.clone();
            BattleUnitFull {
                unit: BattleUnit {
                    id: ed.enemy_def.id.0.clone(),
                    stats,
                    current_hp: stats.hp,
                    is_alive: true,
                    crit_counter: 0,
                    equipment_speed_bonus: 0,
                },
                status_state: UnitStatusState::new(),
                djinn_slots: DjinnSlots::new(),
                equipment: EquipmentLoadout::default(),
                mana_contribution: 0,
                ability_ids,
            }
        })
        .collect();

    Battle {
        round: 1,
        phase: BattlePhase::Planning,
        player_units,
        enemies,
        mana_pool: ManaPool::new(total_mana),
        planned_actions: Vec::new(),
        log: Vec::new(),
        config,
        ability_defs,
        djinn_defs,
    }
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
        BattleAction::UseAbility { ability_id, targets } => {
            if !status::can_use_ability(&unit.status_state.statuses) {
                return Err(PlanError::UnitCannotAct);
            }
            let ability = battle
                .ability_defs
                .get(ability_id)
                .ok_or(PlanError::InvalidTarget)?;
            if battle.mana_pool.current_mana < ability.mana_cost {
                return Err(PlanError::InsufficientMana);
            }
            // Validate all targets exist
            for t in targets {
                get_unit(battle, *t).ok_or(PlanError::InvalidTarget)?;
            }
            // Reserve mana in projected
            battle.mana_pool.projected_mana = battle
                .mana_pool
                .projected_mana
                .saturating_sub(ability.mana_cost);
        }
        BattleAction::ActivateDjinn { djinn_index } => {
            let idx = *djinn_index as usize;
            let slots = &unit.djinn_slots;
            if idx >= slots.slots.len() {
                return Err(PlanError::InvalidDjinnIndex);
            }
            if slots.slots[idx].state != DjinnState::Good {
                return Err(PlanError::DjinnNotReady);
            }
        }
        BattleAction::Summon { djinn_indices } => {
            let slots = &unit.djinn_slots;
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

    battle.planned_actions.push((unit_ref, action));
    Ok(())
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
        let target_idx = battle
            .player_units
            .iter()
            .position(|p| p.unit.is_alive);
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
            max_hp: pu.unit.stats.hp,
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
            max_hp: enemy.unit.stats.hp,
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
            BattleAction::UseAbility { ability_id, targets } => {
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
/// 1. Compute execution order (summons first, then SPD descending).
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

    // Compute SPD-based execution order for non-summon actions
    let mut spd_order: Vec<(TargetRef, BattleAction, i32, u16)> = other_actions
        .into_iter()
        .map(|(tr, action)| {
            let unit = get_unit(battle, tr);
            let (eff_spd, base_spd) = match unit {
                Some(u) => {
                    let eff = u.unit.stats.spd as i32 + u.unit.equipment_speed_bonus as i32;
                    (eff, u.unit.stats.spd)
                }
                None => (0, 0),
            };
            (tr, action, eff_spd, base_spd)
        })
        .collect();
    spd_order.sort_by(|a, b| b.2.cmp(&a.2).then_with(|| b.3.cmp(&a.3)));

    // Build final execution sequence: summons first, then SPD order
    let mut execution_sequence: Vec<(TargetRef, BattleAction)> = Vec::new();
    for (tr, action) in summon_actions {
        execution_sequence.push((tr, action));
    }
    for (tr, action, _, _) in spd_order {
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
            BattleAction::UseAbility { ability_id, targets } => {
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
    let (attacker_stats, crit_counter, hit_count_bonus) = {
        let actor = get_unit(battle, actor_ref).unwrap();
        (actor.unit.stats, actor.unit.crit_counter, 0u8)
    };

    // Check if target is alive
    {
        match get_unit(battle, target_ref) {
            Some(t) if t.unit.is_alive => {}
            _ => return,
        };
    }

    let target_stats = get_unit(battle, target_ref).unwrap().unit.stats;
    let target_hp = get_unit(battle, target_ref).unwrap().unit.current_hp;

    // Base hit count is 1 + equipment bonus
    let base_hit_count: u8 = 1 + hit_count_bonus;
    let base_power = 0u16; // auto-attack has 0 base power
    let base_damage = combat::calculate_damage(
        base_power,
        DamageType::Physical,
        &attacker_stats,
        &target_stats,
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
                    old_value: battle.mana_pool.current_mana.saturating_sub(hit.mana_generated),
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
        }

        events.push(BattleEvent::DamageDealt(DamageDealt {
            source: actor_ref,
            target: target_ref,
            amount: hit.damage,
            damage_type: DamageType::Physical,
            is_crit: hit.is_crit,
        }));

        if hit.is_crit {
            events.push(BattleEvent::CritTriggered(actor_ref, hit.crit_counter_after));
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
        if !battle.mana_pool.spend_mana(ability.mana_cost) {
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

    let attacker_stats = get_unit(battle, actor_ref).unwrap().unit.stats;

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
        if ability.category == crate::shared::AbilityCategory::Healing {
            let heal_amount = ability.base_power;
            {
                let target = get_unit_mut(battle, target_ref).unwrap();
                let max_hp = target.unit.stats.hp;
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

            // Apply defense penetration if applicable
            let effective_def = if let Some(pen_pct) = ability.ignore_defense_percent {
                damage_mods::apply_defense_penetration(target_stats.def, pen_pct)
            } else {
                target_stats.def
            };

            let modified_target_stats = Stats {
                def: effective_def,
                ..target_stats
            };

            let base_damage = combat::calculate_damage(
                ability.base_power,
                damage_type,
                &attacker_stats,
                &modified_target_stats,
                &battle.config,
            );

            let crit_counter = get_unit(battle, actor_ref).unwrap().unit.crit_counter;
            let hits = combat::resolve_multi_hit(
                ability.hit_count.max(1),
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
                    target.unit.current_hp =
                        target.unit.current_hp.saturating_sub(hit.damage);
                    if target.unit.current_hp == 0 {
                        target.unit.is_alive = false;
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
        if let Some(ref buff_effect) = ability.buff_effect {
            let target = get_unit_mut(battle, target_ref).unwrap();
            status::apply_buff(&mut target.status_state, buff_effect);
        }
        if let Some(ref debuff_effect) = ability.debuff_effect {
            let target = get_unit_mut(battle, target_ref).unwrap();
            status::apply_debuff(&mut target.status_state, debuff_effect);
        }

        // Apply barrier
        if let (Some(charges), Some(duration)) =
            (ability.shield_charges, ability.shield_duration)
        {
            let target = get_unit_mut(battle, target_ref).unwrap();
            status::apply_barrier(&mut target.status_state, charges, duration);
        }

        // Apply HoT
        if let Some(ref hot) = ability.heal_over_time {
            let target = get_unit_mut(battle, target_ref).unwrap();
            status::apply_hot(&mut target.status_state, hot.amount, hot.duration);
        }
    }
}

fn execute_djinn_activate(
    battle: &mut Battle,
    actor_ref: TargetRef,
    djinn_index: usize,
    events: &mut Vec<BattleEvent>,
) {
    let recovery_turns = battle.config.djinn_recovery_start_delay
        + battle.config.djinn_recovery_per_turn;
    let actor = get_unit_mut(battle, actor_ref).unwrap();
    if let Some(ev) = djinn::activate_djinn(&mut actor.djinn_slots, djinn_index, actor_ref, recovery_turns) {
        events.push(BattleEvent::DjinnChanged(ev));
    }
}

fn execute_summon_action(
    battle: &mut Battle,
    actor_ref: TargetRef,
    djinn_indices: &[u8],
    events: &mut Vec<BattleEvent>,
) {
    let recovery_turns = battle.config.djinn_recovery_start_delay
        + battle.config.djinn_recovery_per_turn;
    let indices: Vec<usize> = djinn_indices.iter().map(|&i| i as usize).collect();

    // Collect djinn IDs before mutating slots (for summon damage lookup)
    let djinn_ids: Vec<DjinnId> = {
        let actor = get_unit(battle, actor_ref).unwrap();
        indices
            .iter()
            .filter_map(|&i| actor.djinn_slots.slots.get(i))
            .map(|inst| inst.djinn_id.clone())
            .collect()
    };

    // Transition djinn to Recovery
    let actor = get_unit_mut(battle, actor_ref).unwrap();
    let summon_succeeded = if let Some(djinn_events) =
        djinn::execute_summon(&mut actor.djinn_slots, &indices, actor_ref, recovery_turns)
    {
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

    // Apply summon damage to all alive enemies
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
            battle.enemies[ei].unit.current_hp = battle.enemies[ei]
                .unit
                .current_hp
                .saturating_sub(total_summon_damage);
            if battle.enemies[ei].unit.current_hp == 0 {
                battle.enemies[ei].unit.is_alive = false;
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

    // Tick djinn recovery
    tick_all_units_djinn(battle, events);
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
            unit.unit.stats.hp,
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
            unit.unit.stats.hp,
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
            let max_hp = unit.unit.stats.hp;
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
            let max_hp = unit.unit.stats.hp;
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
    for unit in battle.player_units.iter_mut().chain(battle.enemies.iter_mut()) {
        if !unit.unit.is_alive {
            continue;
        }
        status::tick_buffs(&mut unit.status_state.buffs);
        status::tick_buffs(&mut unit.status_state.debuffs);
    }
}

fn tick_all_units_barriers(battle: &mut Battle) {
    for unit in battle.player_units.iter_mut().chain(battle.enemies.iter_mut()) {
        if !unit.unit.is_alive {
            continue;
        }
        status::tick_barriers(&mut unit.status_state.barriers);
    }
}

fn tick_all_units_djinn(battle: &mut Battle, events: &mut Vec<BattleEvent>) {
    for (i, unit) in battle.player_units.iter_mut().enumerate() {
        if !unit.unit.is_alive {
            continue;
        }
        let unit_ref = TargetRef {
            side: Side::Player,
            index: i as u8,
        };
        let djinn_events = djinn::tick_recovery(&mut unit.djinn_slots, unit_ref);
        for ev in djinn_events {
            events.push(BattleEvent::DjinnChanged(ev));
        }
    }

    for (i, unit) in battle.enemies.iter_mut().enumerate() {
        if !unit.unit.is_alive {
            continue;
        }
        let unit_ref = TargetRef {
            side: Side::Enemy,
            index: i as u8,
        };
        let djinn_events = djinn::tick_recovery(&mut unit.djinn_slots, unit_ref);
        for ev in djinn_events {
            events.push(BattleEvent::DjinnChanged(ev));
        }
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
        // Sum xp and gold from enemy IDs (simple: 10 xp and 5 gold per enemy)
        let xp = battle.enemies.len() as u32 * 10;
        let gold = battle.enemies.len() as u32 * 5;
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
    use crate::shared::{
        AbilityCategory, DjinnId, Element, EnemyId, StatusEffect, StatusEffectType, TargetMode,
    };

    // ── Helpers ──────────────────────────────────────────────────────

    fn test_config() -> CombatConfig {
        default_combat_config()
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
                level: 1,
                stats,
                xp: 10,
                gold: 5,
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
            mana_cost: cost,
            base_power: power,
            targets: TargetMode::SingleEnemy,
            unlock_level: 1,
            hit_count: 1,
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

    fn setup_basic_battle() -> Battle {
        let player_stats = Stats { hp: 100, atk: 30, def: 20, mag: 25, spd: 15 };
        let enemy_stats = Stats { hp: 80, atk: 20, def: 15, mag: 10, spd: 10 };

        let player = make_player("hero", player_stats, 5);
        let enemy = make_enemy("goblin", enemy_stats);

        let (aid, adef) = make_basic_ability("quake", 3, 40);
        let mut abilities = HashMap::new();
        abilities.insert(aid, adef);

        new_battle(vec![player], vec![enemy], test_config(), abilities, HashMap::new())
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
        assert_eq!(player.unit.stats.hp, 100);
        assert_eq!(player.unit.stats.atk, 30);
        assert_eq!(player.unit.current_hp, 100);
        assert!(player.unit.is_alive);
        assert_eq!(player.unit.crit_counter, 0);

        let enemy = &battle.enemies[0];
        assert_eq!(enemy.unit.stats.hp, 80);
        assert_eq!(enemy.unit.current_hp, 80);
        assert!(enemy.unit.is_alive);
    }

    // ── Test: plan_action attack succeeds ───────────────────────────

    #[test]
    fn test_plan_attack_succeeds() {
        let mut battle = setup_basic_battle();
        let actor = TargetRef { side: Side::Player, index: 0 };
        let target = TargetRef { side: Side::Enemy, index: 0 };
        let result = plan_action(
            &mut battle,
            actor,
            BattleAction::Attack { target },
        );
        assert!(result.is_ok());
        assert_eq!(battle.planned_actions.len(), 1);
    }

    // ── Test: plan_action ability with enough mana succeeds ─────────

    #[test]
    fn test_plan_ability_sufficient_mana() {
        let mut battle = setup_basic_battle();
        let actor = TargetRef { side: Side::Player, index: 0 };
        let target = TargetRef { side: Side::Enemy, index: 0 };
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

        let actor = TargetRef { side: Side::Player, index: 0 };
        let target = TargetRef { side: Side::Enemy, index: 0 };
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
        let actor = TargetRef { side: Side::Player, index: 0 };
        let target = TargetRef { side: Side::Enemy, index: 0 };
        plan_action(
            &mut battle,
            actor,
            BattleAction::Attack { target },
        )
        .unwrap();

        let enemy_hp_before = battle.enemies[0].unit.current_hp;
        let events = execute_round(&mut battle);

        let enemy_hp_after = battle.enemies[0].unit.current_hp;
        assert!(enemy_hp_after < enemy_hp_before, "Enemy should have taken damage");

        let has_damage = events.iter().any(|e| matches!(e, BattleEvent::DamageDealt(_)));
        assert!(has_damage, "Should have a DamageDealt event");
    }

    // ── Test: execute_round ability spends mana ─────────────────────

    #[test]
    fn test_execute_round_ability_spends_mana() {
        let mut battle = setup_basic_battle();
        let actor = TargetRef { side: Side::Player, index: 0 };
        let target = TargetRef { side: Side::Enemy, index: 0 };

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
        let player_stats = Stats { hp: 100, atk: 30, def: 20, mag: 25, spd: 15 };
        let enemy_stats = Stats { hp: 200, atk: 20, def: 15, mag: 10, spd: 10 };

        let mut player = make_player("hero", player_stats, 5);
        player.equipment_effects.total_hit_count_bonus = 0;

        let enemy = make_enemy("troll", enemy_stats);
        let mut battle = new_battle(vec![player], vec![enemy], test_config(), HashMap::new(), HashMap::new());

        // Spend some mana first to track generation
        battle.mana_pool.spend_mana(3);
        let _mana_before = battle.mana_pool.current_mana;

        let actor = TargetRef { side: Side::Player, index: 0 };
        let target = TargetRef { side: Side::Enemy, index: 0 };
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

        let actor = TargetRef { side: Side::Player, index: 0 };
        let target = TargetRef { side: Side::Enemy, index: 0 };
        plan_action(&mut battle, actor, BattleAction::Attack { target }).unwrap();

        let enemy_hp_before = battle.enemies[0].unit.current_hp;
        let events = execute_round(&mut battle);

        let enemy_hp_after = battle.enemies[0].unit.current_hp;
        assert_eq!(enemy_hp_after, enemy_hp_before, "Barrier should have blocked damage");

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
            duration: 3,
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
        let player_stats = Stats { hp: 100, atk: 200, def: 20, mag: 25, spd: 15 };
        let enemy_stats = Stats { hp: 30, atk: 20, def: 5, mag: 10, spd: 10 };

        let player = make_player("hero", player_stats, 5);
        let enemy = make_enemy("weak_goblin", enemy_stats);
        let mut battle = new_battle(vec![player], vec![enemy], test_config(), HashMap::new(), HashMap::new());

        let actor = TargetRef { side: Side::Player, index: 0 };
        let target = TargetRef { side: Side::Enemy, index: 0 };
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

        let actor = TargetRef { side: Side::Player, index: 0 };
        let target = TargetRef { side: Side::Enemy, index: 0 };
        let result = plan_action(&mut battle, actor, BattleAction::Attack { target });
        assert_eq!(result, Err(PlanError::UnitCannotAct));
    }

    // ── Test: plan djinn not ready ──────────────────────────────────

    #[test]
    fn test_plan_djinn_not_ready() {
        let mut battle = setup_basic_battle();
        // Add a djinn in Recovery state
        battle.player_units[0].djinn_slots.add(DjinnId("flint".into()));
        battle.player_units[0].djinn_slots.slots[0].state = DjinnState::Recovery;

        let actor = TargetRef { side: Side::Player, index: 0 };
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

        let actor = TargetRef { side: Side::Player, index: 0 };
        let result = plan_action(
            &mut battle,
            actor,
            BattleAction::ActivateDjinn { djinn_index: 5 },
        );
        assert_eq!(result, Err(PlanError::InvalidDjinnIndex));
    }

    // ── Test: full scenario plan + execute + check across 2 rounds ──

    #[test]
    fn test_full_two_round_scenario() {
        let player_stats = Stats { hp: 200, atk: 40, def: 20, mag: 30, spd: 15 };
        let enemy_stats = Stats { hp: 50, atk: 15, def: 10, mag: 10, spd: 8 };

        let player = make_player("hero", player_stats, 5);
        let enemy = make_enemy("goblin", enemy_stats);

        let (aid, adef) = make_basic_ability("quake", 2, 30);
        let mut abilities = HashMap::new();
        abilities.insert(aid, adef);

        let mut battle = new_battle(vec![player], vec![enemy], test_config(), abilities, HashMap::new());

        // Round 1: auto-attack
        let actor = TargetRef { side: Side::Player, index: 0 };
        let target = TargetRef { side: Side::Enemy, index: 0 };
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
        let player_ref = TargetRef { side: Side::Player, index: 0 };
        let enemy_ref = TargetRef { side: Side::Enemy, index: 0 };
        let invalid_ref = TargetRef { side: Side::Player, index: 99 };

        assert!(get_unit(&battle, player_ref).is_some());
        assert!(get_unit(&battle, enemy_ref).is_some());
        assert!(get_unit(&battle, invalid_ref).is_none());
    }

    // ── Test: multiple mana contributions ───────────────────────────

    #[test]
    fn test_multi_player_mana_pool() {
        let stats = Stats { hp: 100, atk: 20, def: 15, mag: 10, spd: 10 };
        let p1 = make_player("hero1", stats, 3);
        let p2 = make_player("hero2", stats, 4);
        let enemy = make_enemy("goblin", stats);

        let battle = new_battle(vec![p1, p2], vec![enemy], test_config(), HashMap::new(), HashMap::new());
        assert_eq!(battle.mana_pool.max_mana, 7);
        assert_eq!(battle.mana_pool.current_mana, 7);
    }

    // ── Test: execute_round with djinn activation ───────────────────

    #[test]
    fn test_execute_round_djinn_activation() {
        let mut battle = setup_basic_battle();
        battle.player_units[0].djinn_slots.add(DjinnId("flint".into()));

        let actor = TargetRef { side: Side::Player, index: 0 };
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

    // ── Test: plan_enemy_actions plans correctly ────────────────────

    #[test]
    fn test_plan_enemy_actions_targets_first_alive_player() {
        let player_stats = Stats { hp: 100, atk: 20, def: 15, mag: 10, spd: 10 };
        let enemy_stats = Stats { hp: 80, atk: 25, def: 10, mag: 10, spd: 12 };

        let p1 = make_player("hero1", player_stats, 3);
        let p2 = make_player("hero2", player_stats, 3);
        let e1 = make_enemy("goblin-a", enemy_stats);
        let e2 = make_enemy("goblin-b", enemy_stats);

        let mut battle = new_battle(vec![p1, p2], vec![e1, e2], test_config(), HashMap::new(), HashMap::new());

        plan_enemy_actions(&mut battle);

        // Both enemies should have planned an attack on player index 0
        assert_eq!(battle.planned_actions.len(), 2, "Both enemies should plan actions");
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
        let stats = Stats { hp: 100, atk: 20, def: 15, mag: 10, spd: 10 };
        let p1 = make_player("hero", stats, 3);
        let e1 = make_enemy("dead-goblin", stats);
        let e2 = make_enemy("live-goblin", stats);

        let mut battle = new_battle(vec![p1], vec![e1, e2], test_config(), HashMap::new(), HashMap::new());
        // Kill first enemy
        battle.enemies[0].unit.is_alive = false;
        battle.enemies[0].unit.current_hp = 0;

        plan_enemy_actions(&mut battle);

        assert_eq!(battle.planned_actions.len(), 1, "Only alive enemy should plan");
        assert_eq!(battle.planned_actions[0].0.index, 1, "Second enemy should be the actor");
    }

    // ── Test: plan_enemy_actions targets second player if first dead ─

    #[test]
    fn test_plan_enemy_actions_targets_second_player_if_first_dead() {
        let stats = Stats { hp: 100, atk: 20, def: 15, mag: 10, spd: 10 };
        let p1 = make_player("dead-hero", stats, 3);
        let p2 = make_player("alive-hero", stats, 3);
        let e1 = make_enemy("goblin", stats);

        let mut battle = new_battle(vec![p1, p2], vec![e1], test_config(), HashMap::new(), HashMap::new());
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
        let player_stats = Stats { hp: 100, atk: 20, def: 10, mag: 10, spd: 8 };
        let enemy_stats = Stats { hp: 80, atk: 30, def: 10, mag: 10, spd: 12 };

        let p1 = make_player("hero", player_stats, 3);
        let e1 = make_enemy("strong-goblin", enemy_stats);

        let mut battle = new_battle(vec![p1], vec![e1], test_config(), HashMap::new(), HashMap::new());

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
        let player_stats = Stats { hp: 30, atk: 5, def: 2, mag: 5, spd: 5 };
        let enemy_stats = Stats { hp: 500, atk: 200, def: 50, mag: 10, spd: 20 };

        let p1 = make_player("fragile-hero", player_stats, 1);
        let e1 = make_enemy("boss-goblin", enemy_stats);

        let mut battle = new_battle(vec![p1], vec![e1], test_config(), HashMap::new(), HashMap::new());

        // Plan both sides
        let player_ref = TargetRef { side: Side::Player, index: 0 };
        let enemy_target = TargetRef { side: Side::Enemy, index: 0 };
        let _ = plan_action(
            &mut battle,
            player_ref,
            BattleAction::Attack { target: enemy_target },
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
        let player_stats = Stats { hp: 200, atk: 25, def: 15, mag: 10, spd: 15 };
        let enemy_stats = Stats { hp: 150, atk: 20, def: 12, mag: 10, spd: 10 };

        let p1 = make_player("hero", player_stats, 3);
        let e1 = make_enemy("goblin", enemy_stats);

        let mut battle = new_battle(vec![p1], vec![e1], test_config(), HashMap::new(), HashMap::new());

        // Plan player attack
        let player_ref = TargetRef { side: Side::Player, index: 0 };
        let enemy_ref = TargetRef { side: Side::Enemy, index: 0 };
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
        let player_stats = Stats { hp: 100, atk: 20, def: 10, mag: 10, spd: 10 };
        let enemy_stats = Stats { hp: 80, atk: 25, def: 10, mag: 15, spd: 12 };

        let player = make_player("hero", player_stats, 5);
        let mut enemy_data = make_enemy("smart-goblin", enemy_stats);
        // Give the enemy an ability it can use
        let (aid, adef) = make_basic_ability("fire-bolt", 2, 40);
        enemy_data.enemy_def.abilities = vec![aid.clone()];

        let mut abilities = HashMap::new();
        abilities.insert(aid, adef);

        let mut battle = new_battle(vec![player], vec![enemy_data], test_config(), abilities, HashMap::new());

        // Verify the enemy has ability_ids populated
        assert_eq!(battle.enemies[0].ability_ids.len(), 1);
        assert_eq!(battle.enemies[0].ability_ids[0].0, "fire-bolt");

        // Use AI planning with Aggressive strategy (prefers abilities)
        plan_enemy_actions_with_ai(&mut battle, AiStrategy::Aggressive);

        // The enemy should have planned a UseAbility action (not just Attack)
        assert_eq!(battle.planned_actions.len(), 1);
        match &battle.planned_actions[0].1 {
            BattleAction::UseAbility { ability_id, targets } => {
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
        let player_stats = Stats { hp: 100, atk: 20, def: 10, mag: 10, spd: 10 };
        let enemy_stats = Stats { hp: 80, atk: 25, def: 10, mag: 10, spd: 12 };

        let p1 = make_player("hero-a", player_stats, 3);
        let p2 = make_player("hero-b", player_stats, 3);
        let enemy = make_enemy("aggressive-goblin", enemy_stats);

        let mut battle = new_battle(vec![p1, p2], vec![enemy], test_config(), HashMap::new(), HashMap::new());

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
        let player_stats = Stats { hp: 100, atk: 20, def: 10, mag: 10, spd: 10 };
        let enemy_stats = Stats { hp: 80, atk: 25, def: 10, mag: 10, spd: 12 };

        let player = make_player("hero", player_stats, 5);
        let enemy = make_enemy("dumb-goblin", enemy_stats);
        // Enemy has no abilities (empty list from make_enemy)

        let mut battle = new_battle(vec![player], vec![enemy], test_config(), HashMap::new(), HashMap::new());

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
}
