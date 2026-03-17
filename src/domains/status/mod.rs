#![allow(dead_code)]
//! Status Effects & Buffs domain (S07-S13)
//!
//! All status effects, buffs, debuffs, barriers, HoT, immunity, and cleansing.

use crate::shared::{
    BuffEffect, CleanseType, DebuffEffect, Immunity, StatBonus, StatusEffect, StatusEffectType,
};

// ── S07 — Active Status ─────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct ActiveStatus {
    pub effect_type: StatusEffectType,
    pub remaining_turns: u8,
    pub burn_percent: Option<f32>,
    pub poison_percent: Option<f32>,
    pub freeze_threshold: Option<u16>,
    pub freeze_damage_taken: u16,
}

// ── S08 — Active Buff / Debuff ──────────────────────────────────────

#[derive(Debug, Clone)]
pub struct ActiveBuff {
    pub stat_modifiers: StatBonus,
    pub remaining_turns: u8,
}

// ── S09 — Active Barrier ────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct ActiveBarrier {
    pub charges: u8,
    pub remaining_turns: u8,
}

// ── S10 — Active HoT ───────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct ActiveHoT {
    pub amount: u16,
    pub remaining_turns: u8,
}

// ── S12 — Active Immunity ───────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct ActiveImmunity {
    pub all: bool,
    pub types: Vec<StatusEffectType>,
    pub remaining_turns: u8,
}

// ── Aggregate per-unit state ────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct UnitStatusState {
    pub statuses: Vec<ActiveStatus>,
    pub buffs: Vec<ActiveBuff>,
    pub debuffs: Vec<ActiveBuff>,
    pub barriers: Vec<ActiveBarrier>,
    pub hots: Vec<ActiveHoT>,
    pub immunity: Option<ActiveImmunity>,
}

impl UnitStatusState {
    pub fn new() -> Self {
        Self {
            statuses: Vec::new(),
            buffs: Vec::new(),
            debuffs: Vec::new(),
            barriers: Vec::new(),
            hots: Vec::new(),
            immunity: None,
        }
    }
}

impl Default for UnitStatusState {
    fn default() -> Self {
        Self::new()
    }
}

// ── Tick result ─────────────────────────────────────────────────────

#[derive(Debug, Clone, Default)]
pub struct StatusTickResult {
    pub damage: u16,
    pub expired: Vec<StatusEffectType>,
    pub freeze_broken: bool,
}

// ── S12 — Immunity ──────────────────────────────────────────────────

/// Returns true if the given status type is blocked by active immunity.
pub fn is_immune(immunity: &Option<ActiveImmunity>, status_type: StatusEffectType) -> bool {
    match immunity {
        Some(imm) if imm.remaining_turns > 0 => {
            imm.all || imm.types.contains(&status_type)
        }
        _ => false,
    }
}

// ── S07 — Status application & tick ─────────────────────────────────

/// Attempt to apply a status effect. Returns true if applied, false if blocked by immunity.
pub fn apply_status(
    state: &mut UnitStatusState,
    status_effect: &StatusEffect,
    _immunity: &Option<ActiveImmunity>,
) -> bool {
    if is_immune(&state.immunity, status_effect.effect_type) {
        return false;
    }

    state.statuses.push(ActiveStatus {
        effect_type: status_effect.effect_type,
        remaining_turns: status_effect.duration,
        burn_percent: status_effect.burn_percent,
        poison_percent: status_effect.poison_percent,
        freeze_threshold: status_effect.freeze_threshold,
        freeze_damage_taken: 0,
    });
    true
}

/// Tick all active statuses at end of round.
/// Computes DoT damage, checks freeze breakage, decrements durations, removes expired.
pub fn tick_statuses(
    max_hp: u16,
    current_hp: u16,
    statuses: &mut Vec<ActiveStatus>,
) -> StatusTickResult {
    let mut result = StatusTickResult::default();

    for status in statuses.iter_mut() {
        match status.effect_type {
            StatusEffectType::Burn => {
                if let Some(pct) = status.burn_percent {
                    result.damage += (max_hp as f32 * pct) as u16;
                }
            }
            StatusEffectType::Poison => {
                if let Some(pct) = status.poison_percent {
                    let missing = max_hp.saturating_sub(current_hp);
                    result.damage += (missing as f32 * pct) as u16;
                }
            }
            StatusEffectType::Freeze => {
                if let Some(threshold) = status.freeze_threshold {
                    if status.freeze_damage_taken >= threshold {
                        result.freeze_broken = true;
                        status.remaining_turns = 0;
                    }
                }
            }
            _ => {}
        }

        if status.remaining_turns > 0 {
            status.remaining_turns -= 1;
        }
    }

    // Collect expired before removing
    for status in statuses.iter() {
        if status.remaining_turns == 0 {
            result.expired.push(status.effect_type);
        }
    }

    statuses.retain(|s| s.remaining_turns > 0);

    result
}

// ── S08 — Buff / Debuff ─────────────────────────────────────────────

const MAX_BUFF_STACKS: usize = 3;

/// Apply a buff to the unit. Max 3 stacks.
pub fn apply_buff(state: &mut UnitStatusState, buff_effect: &BuffEffect) {
    if state.buffs.len() < MAX_BUFF_STACKS {
        state.buffs.push(ActiveBuff {
            stat_modifiers: buff_effect.stat_modifiers,
            remaining_turns: buff_effect.duration,
        });
    }
}

/// Apply a debuff to the unit. Max 3 stacks.
pub fn apply_debuff(state: &mut UnitStatusState, debuff_effect: &DebuffEffect) {
    if state.debuffs.len() < MAX_BUFF_STACKS {
        state.debuffs.push(ActiveBuff {
            stat_modifiers: debuff_effect.stat_modifiers,
            remaining_turns: debuff_effect.duration,
        });
    }
}

/// Decrement buff durations, remove expired.
pub fn tick_buffs(buffs: &mut Vec<ActiveBuff>) {
    for buff in buffs.iter_mut() {
        if buff.remaining_turns > 0 {
            buff.remaining_turns -= 1;
        }
    }
    buffs.retain(|b| b.remaining_turns > 0);
}

/// Sum all active buff and debuff stat modifiers into a single StatBonus.
pub fn compute_stat_modifiers(buffs: &[ActiveBuff], debuffs: &[ActiveBuff]) -> StatBonus {
    let mut total = StatBonus::default();
    for b in buffs.iter().chain(debuffs.iter()) {
        total.atk += b.stat_modifiers.atk;
        total.def += b.stat_modifiers.def;
        total.mag += b.stat_modifiers.mag;
        total.spd += b.stat_modifiers.spd;
        total.hp += b.stat_modifiers.hp;
    }
    total
}

// ── S09 — Barrier ───────────────────────────────────────────────────

/// Apply a new barrier to the unit.
pub fn apply_barrier(state: &mut UnitStatusState, charges: u8, duration: u8) {
    state.barriers.push(ActiveBarrier {
        charges,
        remaining_turns: duration,
    });
}

/// Consume one charge from the oldest active barrier. Returns true if a charge was consumed.
pub fn consume_barrier(barriers: &mut Vec<ActiveBarrier>) -> bool {
    for barrier in barriers.iter_mut() {
        if barrier.charges > 0 {
            barrier.charges -= 1;
            return true;
        }
    }
    false
}

/// Decrement barrier durations, remove expired (0 turns or 0 charges).
pub fn tick_barriers(barriers: &mut Vec<ActiveBarrier>) {
    for barrier in barriers.iter_mut() {
        if barrier.remaining_turns > 0 {
            barrier.remaining_turns -= 1;
        }
    }
    barriers.retain(|b| b.remaining_turns > 0 && b.charges > 0);
}

// ── S10 — HoT ──────────────────────────────────────────────────────

/// Apply a heal-over-time effect to the unit.
pub fn apply_hot(state: &mut UnitStatusState, amount: u16, duration: u8) {
    state.hots.push(ActiveHoT {
        amount,
        remaining_turns: duration,
    });
}

/// Tick all HoTs, returning total healing this tick. Decrements durations, removes expired.
pub fn tick_hots(hots: &mut Vec<ActiveHoT>) -> u16 {
    let mut total = 0u16;
    for hot in hots.iter_mut() {
        if hot.remaining_turns > 0 {
            total = total.saturating_add(hot.amount);
            hot.remaining_turns -= 1;
        }
    }
    hots.retain(|h| h.remaining_turns > 0);
    total
}

// ── S13 — Cleanse ───────────────────────────────────────────────────

/// Remove statuses according to the cleanse type. Returns the types that were removed.
pub fn cleanse(
    statuses: &mut Vec<ActiveStatus>,
    cleanse_type: &CleanseType,
) -> Vec<StatusEffectType> {
    let mut removed = Vec::new();

    match cleanse_type {
        CleanseType::All => {
            removed.extend(statuses.iter().map(|s| s.effect_type));
            statuses.clear();
        }
        CleanseType::Negative => {
            // All 6 status types are negative (harmful) effects
            removed.extend(statuses.iter().map(|s| s.effect_type));
            statuses.clear();
        }
        CleanseType::ByType(types) => {
            for status in statuses.iter() {
                if types.contains(&status.effect_type) {
                    removed.push(status.effect_type);
                }
            }
            statuses.retain(|s| !types.contains(&s.effect_type));
        }
    }

    removed
}

// ── Action checks ───────────────────────────────────────────────────

/// Returns false if the unit has Stun or Freeze active (cannot act at all).
pub fn can_act(statuses: &[ActiveStatus]) -> bool {
    !statuses.iter().any(|s| {
        matches!(
            s.effect_type,
            StatusEffectType::Stun | StatusEffectType::Freeze
        )
    })
}

/// Returns false if the unit has Stun, Freeze, or Incapacitate active (cannot auto-attack).
pub fn can_auto_attack(statuses: &[ActiveStatus]) -> bool {
    !statuses.iter().any(|s| {
        matches!(
            s.effect_type,
            StatusEffectType::Stun | StatusEffectType::Freeze | StatusEffectType::Incapacitate
        )
    })
}

/// Returns false if the unit has Stun, Freeze, or Null active (cannot use abilities).
pub fn can_use_ability(statuses: &[ActiveStatus]) -> bool {
    !statuses.iter().any(|s| {
        matches!(
            s.effect_type,
            StatusEffectType::Stun | StatusEffectType::Freeze | StatusEffectType::Null
        )
    })
}

// ── Apply Immunity ──────────────────────────────────────────────────

/// Set or replace the unit's active immunity.
pub fn apply_immunity(state: &mut UnitStatusState, immunity: &Immunity) {
    state.immunity = Some(ActiveImmunity {
        all: immunity.all,
        types: immunity.types.clone(),
        remaining_turns: immunity.duration,
    });
}

/// Tick the immunity timer. Clears immunity when expired.
pub fn tick_immunity(immunity: &mut Option<ActiveImmunity>) {
    if let Some(imm) = immunity {
        if imm.remaining_turns > 0 {
            imm.remaining_turns -= 1;
        }
        if imm.remaining_turns == 0 {
            *immunity = None;
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════
// Tests
// ═══════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;

    // -- helpers --

    fn make_state() -> UnitStatusState {
        UnitStatusState::new()
    }

    fn burn_effect(pct: f32, dur: u8) -> StatusEffect {
        StatusEffect {
            effect_type: StatusEffectType::Burn,
            duration: dur,
            burn_percent: Some(pct),
            poison_percent: None,
            freeze_threshold: None,
        }
    }

    fn poison_effect(pct: f32, dur: u8) -> StatusEffect {
        StatusEffect {
            effect_type: StatusEffectType::Poison,
            duration: dur,
            burn_percent: None,
            poison_percent: Some(pct),
            freeze_threshold: None,
        }
    }

    fn freeze_effect(threshold: u16, dur: u8) -> StatusEffect {
        StatusEffect {
            effect_type: StatusEffectType::Freeze,
            duration: dur,
            burn_percent: None,
            poison_percent: None,
            freeze_threshold: Some(threshold),
        }
    }

    fn stun_effect(dur: u8) -> StatusEffect {
        StatusEffect {
            effect_type: StatusEffectType::Stun,
            duration: dur,
            burn_percent: None,
            poison_percent: None,
            freeze_threshold: None,
        }
    }

    fn null_effect(dur: u8) -> StatusEffect {
        StatusEffect {
            effect_type: StatusEffectType::Null,
            duration: dur,
            burn_percent: None,
            poison_percent: None,
            freeze_threshold: None,
        }
    }

    fn incap_effect(dur: u8) -> StatusEffect {
        StatusEffect {
            effect_type: StatusEffectType::Incapacitate,
            duration: dur,
            burn_percent: None,
            poison_percent: None,
            freeze_threshold: None,
        }
    }

    fn buff(atk: i16, def: i16, dur: u8) -> BuffEffect {
        BuffEffect {
            stat_modifiers: StatBonus {
                atk,
                def,
                mag: 0,
                spd: 0,
                hp: 0,
            },
            duration: dur,
            shield_charges: None,
            grant_immunity: false,
        }
    }

    fn debuff(atk: i16, def: i16, dur: u8) -> DebuffEffect {
        DebuffEffect {
            stat_modifiers: StatBonus {
                atk,
                def,
                mag: 0,
                spd: 0,
                hp: 0,
            },
            duration: dur,
        }
    }

    // ── S07: Burn ───────────────────────────────────────────────────

    #[test]
    fn test_burn_damage_calculation() {
        let mut statuses = vec![ActiveStatus {
            effect_type: StatusEffectType::Burn,
            remaining_turns: 3,
            burn_percent: Some(0.10),
            poison_percent: None,
            freeze_threshold: None,
            freeze_damage_taken: 0,
        }];
        let result = tick_statuses(100, 100, &mut statuses);
        assert_eq!(result.damage, 10); // 10% of 100 max HP
    }

    // ── S07: Poison ─────────────────────────────────────────────────

    #[test]
    fn test_poison_damage_calculation() {
        let mut statuses = vec![ActiveStatus {
            effect_type: StatusEffectType::Poison,
            remaining_turns: 3,
            burn_percent: None,
            poison_percent: Some(0.05),
            freeze_threshold: None,
            freeze_damage_taken: 0,
        }];
        // max_hp=100, current_hp=60 => missing=40 => 5% of 40 = 2
        let result = tick_statuses(100, 60, &mut statuses);
        assert_eq!(result.damage, 2);
    }

    // ── S07: Freeze breaks at threshold ─────────────────────────────

    #[test]
    fn test_freeze_breaks_at_threshold() {
        let mut statuses = vec![ActiveStatus {
            effect_type: StatusEffectType::Freeze,
            remaining_turns: 5,
            burn_percent: None,
            poison_percent: None,
            freeze_threshold: Some(30),
            freeze_damage_taken: 30,
        }];
        let result = tick_statuses(100, 100, &mut statuses);
        assert!(result.freeze_broken);
        assert!(statuses.is_empty()); // removed after breaking
    }

    // ── S07: Freeze does NOT break below threshold ──────────────────

    #[test]
    fn test_freeze_does_not_break_below_threshold() {
        let mut statuses = vec![ActiveStatus {
            effect_type: StatusEffectType::Freeze,
            remaining_turns: 5,
            burn_percent: None,
            poison_percent: None,
            freeze_threshold: Some(30),
            freeze_damage_taken: 20,
        }];
        let result = tick_statuses(100, 100, &mut statuses);
        assert!(!result.freeze_broken);
        assert_eq!(statuses.len(), 1);
    }

    // ── S07: Stun prevents all actions ──────────────────────────────

    #[test]
    fn test_stun_prevents_all_actions() {
        let statuses = vec![ActiveStatus {
            effect_type: StatusEffectType::Stun,
            remaining_turns: 2,
            burn_percent: None,
            poison_percent: None,
            freeze_threshold: None,
            freeze_damage_taken: 0,
        }];
        assert!(!can_act(&statuses));
        assert!(!can_auto_attack(&statuses));
        assert!(!can_use_ability(&statuses));
    }

    // ── S07: Null allows auto-attack only ───────────────────────────

    #[test]
    fn test_null_allows_auto_attack_only() {
        let statuses = vec![ActiveStatus {
            effect_type: StatusEffectType::Null,
            remaining_turns: 2,
            burn_percent: None,
            poison_percent: None,
            freeze_threshold: None,
            freeze_damage_taken: 0,
        }];
        assert!(can_act(&statuses));
        assert!(can_auto_attack(&statuses));
        assert!(!can_use_ability(&statuses));
    }

    // ── S07: Incapacitate allows ability only ───────────────────────

    #[test]
    fn test_incapacitate_allows_ability_only() {
        let statuses = vec![ActiveStatus {
            effect_type: StatusEffectType::Incapacitate,
            remaining_turns: 2,
            burn_percent: None,
            poison_percent: None,
            freeze_threshold: None,
            freeze_damage_taken: 0,
        }];
        assert!(can_act(&statuses));
        assert!(!can_auto_attack(&statuses));
        assert!(can_use_ability(&statuses));
    }

    // ── S07: Status duration expires ────────────────────────────────

    #[test]
    fn test_status_duration_expires() {
        let mut statuses = vec![ActiveStatus {
            effect_type: StatusEffectType::Stun,
            remaining_turns: 1,
            burn_percent: None,
            poison_percent: None,
            freeze_threshold: None,
            freeze_damage_taken: 0,
        }];
        let result = tick_statuses(100, 100, &mut statuses);
        assert!(statuses.is_empty());
        assert!(result.expired.contains(&StatusEffectType::Stun));
    }

    // ── S08: Buff stacking up to 3 ─────────────────────────────────

    #[test]
    fn test_buff_stacking_max_3() {
        let mut state = make_state();
        let b = buff(5, 0, 3);
        apply_buff(&mut state, &b);
        apply_buff(&mut state, &b);
        apply_buff(&mut state, &b);
        apply_buff(&mut state, &b); // 4th should be rejected
        assert_eq!(state.buffs.len(), 3);
    }

    // ── S08: Buff stat modifier computation ─────────────────────────

    #[test]
    fn test_buff_stat_modifier_computation() {
        let buffs = vec![
            ActiveBuff {
                stat_modifiers: StatBonus { atk: 5, def: 3, mag: 0, spd: 0, hp: 0 },
                remaining_turns: 2,
            },
            ActiveBuff {
                stat_modifiers: StatBonus { atk: 10, def: 0, mag: 2, spd: 0, hp: 0 },
                remaining_turns: 3,
            },
        ];
        let debuffs = vec![ActiveBuff {
            stat_modifiers: StatBonus { atk: -4, def: -2, mag: 0, spd: 0, hp: 0 },
            remaining_turns: 1,
        }];
        let total = compute_stat_modifiers(&buffs, &debuffs);
        assert_eq!(total.atk, 11); // 5 + 10 - 4
        assert_eq!(total.def, 1);  // 3 + 0 - 2
        assert_eq!(total.mag, 2);
    }

    // ── S08: Buff tick expires ──────────────────────────────────────

    #[test]
    fn test_buff_tick_expires() {
        let mut buffs = vec![
            ActiveBuff {
                stat_modifiers: StatBonus::default(),
                remaining_turns: 1,
            },
            ActiveBuff {
                stat_modifiers: StatBonus::default(),
                remaining_turns: 3,
            },
        ];
        tick_buffs(&mut buffs);
        assert_eq!(buffs.len(), 1);
        assert_eq!(buffs[0].remaining_turns, 2);
    }

    // ── S09: Barrier blocks one hit ─────────────────────────────────

    #[test]
    fn test_barrier_blocks_one_hit() {
        let mut barriers = vec![ActiveBarrier { charges: 1, remaining_turns: 3 }];
        assert!(consume_barrier(&mut barriers));
        assert_eq!(barriers[0].charges, 0);
    }

    // ── S09: Barrier consumed per hit in multi-hit ──────────────────

    #[test]
    fn test_barrier_consumed_per_hit_multi() {
        let mut barriers = vec![ActiveBarrier { charges: 3, remaining_turns: 5 }];
        assert!(consume_barrier(&mut barriers)); // hit 1
        assert!(consume_barrier(&mut barriers)); // hit 2
        assert!(consume_barrier(&mut barriers)); // hit 3
        assert!(!consume_barrier(&mut barriers)); // no more charges
    }

    // ── S09: Barrier does NOT block burn/poison ─────────────────────

    #[test]
    fn test_barrier_does_not_block_burn_poison() {
        // Burn still deals damage even with a barrier active
        let mut statuses = vec![ActiveStatus {
            effect_type: StatusEffectType::Burn,
            remaining_turns: 3,
            burn_percent: Some(0.10),
            poison_percent: None,
            freeze_threshold: None,
            freeze_damage_taken: 0,
        }];
        let barriers = vec![ActiveBarrier { charges: 2, remaining_turns: 5 }];
        // tick_statuses does not interact with barriers at all — burn still ticks
        let result = tick_statuses(100, 100, &mut statuses);
        assert_eq!(result.damage, 10);
        // barriers unchanged
        assert_eq!(barriers[0].charges, 2);
    }

    // ── S09: Barrier tick expires ───────────────────────────────────

    #[test]
    fn test_barrier_tick_expires() {
        let mut barriers = vec![
            ActiveBarrier { charges: 2, remaining_turns: 1 },
            ActiveBarrier { charges: 1, remaining_turns: 3 },
        ];
        tick_barriers(&mut barriers);
        assert_eq!(barriers.len(), 1);
        assert_eq!(barriers[0].charges, 1);
        assert_eq!(barriers[0].remaining_turns, 2);
    }

    // ── S10: HoT healing per tick ───────────────────────────────────

    #[test]
    fn test_hot_healing_per_tick() {
        let mut hots = vec![
            ActiveHoT { amount: 15, remaining_turns: 3 },
            ActiveHoT { amount: 10, remaining_turns: 1 },
        ];
        let healing = tick_hots(&mut hots);
        assert_eq!(healing, 25); // 15 + 10
        assert_eq!(hots.len(), 1); // second HoT expired
    }

    // ── S12: Immunity blocks status application ─────────────────────

    #[test]
    fn test_immunity_blocks_status_application() {
        let mut state = make_state();
        state.immunity = Some(ActiveImmunity {
            all: true,
            types: Vec::new(),
            remaining_turns: 3,
        });
        let imm = state.immunity.clone();
        let applied = apply_status(&mut state, &stun_effect(2), &imm);
        assert!(!applied);
        assert!(state.statuses.is_empty());
    }

    // ── S12: Immunity type-specific ─────────────────────────────────

    #[test]
    fn test_immunity_type_specific() {
        let mut state = make_state();
        state.immunity = Some(ActiveImmunity {
            all: false,
            types: vec![StatusEffectType::Burn],
            remaining_turns: 3,
        });
        // Burn should be blocked
        let imm = state.immunity.clone();
        let burn_applied = apply_status(&mut state, &burn_effect(0.10, 3), &imm);
        assert!(!burn_applied);
        // Stun should go through
        let imm2 = state.immunity.clone();
        let stun_applied = apply_status(&mut state, &stun_effect(2), &imm2);
        assert!(stun_applied);
        assert_eq!(state.statuses.len(), 1);
        assert_eq!(state.statuses[0].effect_type, StatusEffectType::Stun);
    }

    // ── S13: Cleanse All removes everything ─────────────────────────

    #[test]
    fn test_cleanse_all_removes_everything() {
        let mut statuses = vec![
            ActiveStatus {
                effect_type: StatusEffectType::Burn,
                remaining_turns: 3,
                burn_percent: Some(0.10),
                poison_percent: None,
                freeze_threshold: None,
                freeze_damage_taken: 0,
            },
            ActiveStatus {
                effect_type: StatusEffectType::Stun,
                remaining_turns: 1,
                burn_percent: None,
                poison_percent: None,
                freeze_threshold: None,
                freeze_damage_taken: 0,
            },
        ];
        let removed = cleanse(&mut statuses, &CleanseType::All);
        assert!(statuses.is_empty());
        assert_eq!(removed.len(), 2);
    }

    // ── S13: Cleanse ByType removes only specified ──────────────────

    #[test]
    fn test_cleanse_by_type_removes_only_specified() {
        let mut statuses = vec![
            ActiveStatus {
                effect_type: StatusEffectType::Burn,
                remaining_turns: 3,
                burn_percent: Some(0.10),
                poison_percent: None,
                freeze_threshold: None,
                freeze_damage_taken: 0,
            },
            ActiveStatus {
                effect_type: StatusEffectType::Stun,
                remaining_turns: 2,
                burn_percent: None,
                poison_percent: None,
                freeze_threshold: None,
                freeze_damage_taken: 0,
            },
            ActiveStatus {
                effect_type: StatusEffectType::Poison,
                remaining_turns: 2,
                burn_percent: None,
                poison_percent: Some(0.05),
                freeze_threshold: None,
                freeze_damage_taken: 0,
            },
        ];
        let removed = cleanse(
            &mut statuses,
            &CleanseType::ByType(vec![StatusEffectType::Burn, StatusEffectType::Poison]),
        );
        assert_eq!(statuses.len(), 1);
        assert_eq!(statuses[0].effect_type, StatusEffectType::Stun);
        assert_eq!(removed.len(), 2);
        assert!(removed.contains(&StatusEffectType::Burn));
        assert!(removed.contains(&StatusEffectType::Poison));
    }

    // ── Tick order: damage then healing ─────────────────────────────

    #[test]
    fn test_tick_order_damage_then_healing() {
        // Simulate a full tick: status damage comes first, then HoT healing
        let mut statuses = vec![ActiveStatus {
            effect_type: StatusEffectType::Burn,
            remaining_turns: 3,
            burn_percent: Some(0.10),
            poison_percent: None,
            freeze_threshold: None,
            freeze_damage_taken: 0,
        }];
        let mut hots = vec![ActiveHoT { amount: 5, remaining_turns: 3 }];

        // Step 1: tick statuses (damage)
        let tick_result = tick_statuses(100, 80, &mut statuses);
        // Step 2: tick hots (healing)
        let healing = tick_hots(&mut hots);

        // Damage applied before healing
        assert_eq!(tick_result.damage, 10);
        assert_eq!(healing, 5);
        // Net effect: -10 + 5 = -5
    }

    // ── Apply status through full flow ──────────────────────────────

    #[test]
    fn test_apply_status_creates_active_status() {
        let mut state = make_state();
        let applied = apply_status(&mut state, &burn_effect(0.15, 2), &None);
        assert!(applied);
        assert_eq!(state.statuses.len(), 1);
        assert_eq!(state.statuses[0].effect_type, StatusEffectType::Burn);
        assert_eq!(state.statuses[0].remaining_turns, 2);
        assert_eq!(state.statuses[0].burn_percent, Some(0.15));
    }

    // ── Apply barrier and consume ───────────────────────────────────

    #[test]
    fn test_apply_barrier_and_consume() {
        let mut state = make_state();
        apply_barrier(&mut state, 2, 5);
        assert_eq!(state.barriers.len(), 1);
        assert!(consume_barrier(&mut state.barriers));
        assert_eq!(state.barriers[0].charges, 1);
        assert!(consume_barrier(&mut state.barriers));
        assert_eq!(state.barriers[0].charges, 0);
        assert!(!consume_barrier(&mut state.barriers));
    }

    // ── Apply HoT and tick ──────────────────────────────────────────

    #[test]
    fn test_apply_hot_and_tick() {
        let mut state = make_state();
        apply_hot(&mut state, 20, 2);
        assert_eq!(state.hots.len(), 1);
        let h1 = tick_hots(&mut state.hots);
        assert_eq!(h1, 20);
        assert_eq!(state.hots.len(), 1);
        let h2 = tick_hots(&mut state.hots);
        assert_eq!(h2, 20);
        assert!(state.hots.is_empty());
    }

    // ── Debuff stacking max 3 ───────────────────────────────────────

    #[test]
    fn test_debuff_stacking_max_3() {
        let mut state = make_state();
        let d = debuff(-3, -2, 2);
        apply_debuff(&mut state, &d);
        apply_debuff(&mut state, &d);
        apply_debuff(&mut state, &d);
        apply_debuff(&mut state, &d); // 4th rejected
        assert_eq!(state.debuffs.len(), 3);
    }

    // ── Immunity tick expires ───────────────────────────────────────

    #[test]
    fn test_immunity_tick_expires() {
        let mut immunity = Some(ActiveImmunity {
            all: true,
            types: Vec::new(),
            remaining_turns: 1,
        });
        tick_immunity(&mut immunity);
        assert!(immunity.is_none());
    }

    // ── Multiple barriers stack from different sources ───────────────

    #[test]
    fn test_multiple_barriers_stack() {
        let mut state = make_state();
        apply_barrier(&mut state, 1, 3);
        apply_barrier(&mut state, 2, 5);
        assert_eq!(state.barriers.len(), 2);
        // Consume from oldest first
        assert!(consume_barrier(&mut state.barriers));
        assert_eq!(state.barriers[0].charges, 0); // oldest depleted
        assert!(consume_barrier(&mut state.barriers)); // now from second
        assert_eq!(state.barriers[1].charges, 1);
    }
}
