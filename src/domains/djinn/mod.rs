#![allow(dead_code, clippy::new_without_default)]
//! Djinn state machine: Good/Recovery states, ability oscillation,
//! summon execution, and staggered recovery.

use crate::shared::bounded_types::{DjinnTier, StatMod};
use crate::shared::{
    AbilityId, DjinnCompatibility, DjinnDef, DjinnId, DjinnState, DjinnStateChanged, Element,
    StatBonus, SummonEffect, TargetRef,
};

// ── Structs ─────────────────────────────────────────────────────────

/// Runtime state of one equipped djinn.
#[derive(Debug, Clone)]
pub struct DjinnInstance {
    pub djinn_id: DjinnId,
    pub state: DjinnState,
    pub recovery_turns_remaining: u8,
    pub activation_order: u32,
}

/// Per-unit djinn equipment (max 3 slots).
#[derive(Debug, Clone)]
pub struct DjinnSlots {
    pub slots: Vec<DjinnInstance>,
    pub next_activation_order: u32,
}

impl DjinnSlots {
    pub fn new() -> Self {
        Self {
            slots: Vec::new(),
            next_activation_order: 0,
        }
    }

    /// Add a djinn to this unit's slots. Returns false if already at max (3).
    pub fn add(&mut self, djinn_id: DjinnId) -> bool {
        if self.slots.len() >= 3 {
            return false;
        }
        self.slots.push(DjinnInstance {
            djinn_id,
            state: DjinnState::Good,
            recovery_turns_remaining: 0,
            activation_order: 0,
        });
        true
    }

    /// Count how many djinn are currently in Good state.
    pub fn good_count(&self) -> usize {
        self.slots
            .iter()
            .filter(|d| d.state == DjinnState::Good)
            .count()
    }
}

/// Summon tier availability entry.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SummonTier {
    pub tier: u8,
    pub required_good: u8,
}

/// Immediate effect payload emitted when a djinn is activated.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DjinnActivationEffectType {
    Damage,
    Buff,
    Heal,
    Status,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DjinnActivationEvent {
    pub element: Element,
    pub base_damage: u16,
    pub effect_type: Option<DjinnActivationEffectType>,
}

// ── Functions ────────────────────────────────────────────────────────

/// Determine compatibility between a djinn's element and a unit's element.
/// - Same: djinn_element == unit_element
/// - Counter: djinn_element == unit_element.counter()
/// - Neutral: neither
pub fn determine_compatibility(
    djinn_element: Element,
    unit_element: Element,
) -> DjinnCompatibility {
    if djinn_element == unit_element {
        DjinnCompatibility::Same
    } else if djinn_element == unit_element.counter() {
        DjinnCompatibility::Counter
    } else {
        DjinnCompatibility::Neutral
    }
}

/// Get the abilities granted by a djinn given its definition, compatibility,
/// and current state.
///
/// - Same/Counter + Good  -> good_abilities from the matching set
/// - Same/Counter + Recovery -> recovery_abilities from the matching set
/// - Neutral -> good_abilities (always-on, same in both states)
pub fn get_granted_abilities(
    djinn_def: &DjinnDef,
    compatibility: DjinnCompatibility,
    state: DjinnState,
) -> Vec<AbilityId> {
    match compatibility {
        DjinnCompatibility::Same => match state {
            DjinnState::Good => djinn_def.ability_pairs.same.good_abilities.clone(),
            DjinnState::Recovery => djinn_def.ability_pairs.same.recovery_abilities.clone(),
        },
        DjinnCompatibility::Counter => match state {
            DjinnState::Good => djinn_def.ability_pairs.counter.good_abilities.clone(),
            DjinnState::Recovery => djinn_def.ability_pairs.counter.recovery_abilities.clone(),
        },
        DjinnCompatibility::Neutral => {
            // Neutral always returns good_abilities regardless of state
            djinn_def.ability_pairs.neutral.good_abilities.clone()
        }
    }
}

/// Activate a djinn at the given index. The djinn must be in Good state.
/// Transitions it to Recovery, records activation order, sets recovery turns.
///
/// `unit_ref` is passed through to the returned event for the caller.
/// `recovery_turns` is the number of turns before recovery ticks start (typically 2).
pub fn activate_djinn(
    slots: &mut DjinnSlots,
    index: usize,
    unit_ref: TargetRef,
    recovery_turns: u8,
    djinn_def: &DjinnDef,
) -> Option<(DjinnStateChanged, DjinnActivationEvent)> {
    let djinn = slots.slots.get_mut(index)?;
    if djinn.state != DjinnState::Good {
        return None;
    }
    if djinn.djinn_id != djinn_def.id {
        return None;
    }

    let old_state = djinn.state;
    djinn.state = DjinnState::Recovery;
    djinn.recovery_turns_remaining = recovery_turns;
    djinn.activation_order = slots.next_activation_order;
    slots.next_activation_order += 1;

    let summon_effect = djinn_def.summon_effect.as_ref();

    Some((
        DjinnStateChanged {
            djinn_id: djinn.djinn_id.clone(),
            unit: unit_ref,
            old_state,
            new_state: DjinnState::Recovery,
            recovery_turns: Some(recovery_turns),
        },
        DjinnActivationEvent {
            element: djinn_def.element,
            base_damage: summon_effect.map_or(0, |effect| effect.damage),
            effect_type: summon_effect.and_then(DjinnActivationEffectType::from_summon_effect),
        },
    ))
}

impl DjinnActivationEffectType {
    fn from_summon_effect(effect: &SummonEffect) -> Option<Self> {
        if effect.damage > 0 {
            Some(Self::Damage)
        } else if effect.buff.is_some() {
            Some(Self::Buff)
        } else if effect.heal.is_some() {
            Some(Self::Heal)
        } else if effect.status.is_some() {
            Some(Self::Status)
        } else {
            None
        }
    }
}

/// Compute the total stat bonus from all Good-state djinn in the slots.
/// Recovery djinn contribute zero.
pub fn compute_djinn_stat_bonus(slots: &DjinnSlots, djinn_defs: &[DjinnDef]) -> StatBonus {
    let mut bonus = StatBonus::default();
    for inst in &slots.slots {
        if inst.state != DjinnState::Good {
            continue;
        }
        if let Some(def) = djinn_defs.iter().find(|d| d.id == inst.djinn_id) {
            bonus.atk = StatMod::new_unchecked(
                (bonus.atk.get() + def.stat_bonus.atk.get()).clamp(-999, 999),
            );
            bonus.def = StatMod::new_unchecked(
                (bonus.def.get() + def.stat_bonus.def.get()).clamp(-999, 999),
            );
            bonus.mag = StatMod::new_unchecked(
                (bonus.mag.get() + def.stat_bonus.mag.get()).clamp(-999, 999),
            );
            bonus.spd = StatMod::new_unchecked(
                (bonus.spd.get() + def.stat_bonus.spd.get()).clamp(-999, 999),
            );
            bonus.hp =
                StatMod::new_unchecked((bonus.hp.get() + def.stat_bonus.hp.get()).clamp(-999, 999));
        }
    }
    bonus
}

/// Get available summon tiers based on the count of Good djinn.
/// Tier 1 needs 1, Tier 2 needs 2, Tier 3 needs 3.
pub fn get_available_summons(good_count: usize) -> Vec<SummonTier> {
    let mut tiers = Vec::new();
    if good_count >= 1 {
        tiers.push(SummonTier {
            tier: 1,
            required_good: 1,
        });
    }
    if good_count >= 2 {
        tiers.push(SummonTier {
            tier: 2,
            required_good: 2,
        });
    }
    if good_count >= 3 {
        tiers.push(SummonTier {
            tier: 3,
            required_good: 3,
        });
    }
    tiers
}

/// Execute a summon using the specified djinn indices.
/// All specified djinn must be in Good state. They all transition to Recovery.
/// Returns the state-change events, or None if any djinn is invalid / not Good.
pub fn execute_summon(
    slots: &mut DjinnSlots,
    djinn_indices: &[usize],
    unit_ref: TargetRef,
    recovery_turns: u8,
) -> Option<Vec<DjinnStateChanged>> {
    // Validate: all indices in range and all Good
    for &idx in djinn_indices {
        let djinn = slots.slots.get(idx)?;
        if djinn.state != DjinnState::Good {
            return None;
        }
    }

    let mut events = Vec::new();
    for &idx in djinn_indices {
        let djinn = &mut slots.slots[idx];
        let old_state = djinn.state;
        djinn.state = DjinnState::Recovery;
        djinn.recovery_turns_remaining = recovery_turns;
        djinn.activation_order = slots.next_activation_order;
        slots.next_activation_order += 1;

        events.push(DjinnStateChanged {
            djinn_id: djinn.djinn_id.clone(),
            unit: unit_ref,
            old_state,
            new_state: DjinnState::Recovery,
            recovery_turns: Some(recovery_turns),
        });
    }

    Some(events)
}

/// Tick recovery for all djinn in the slots.
/// Each tick: 1 djinn recovers — the one with the lowest activation_order
/// whose recovery_turns_remaining was already 0 at the start of the tick.
/// After recovery, remaining Recovery djinn with turns > 0 are decremented.
///
/// This produces DESIGN_LOCK timing: "1 djinn recovers per turn, starting
/// the turn after next." With recovery_turns=2 and tick called at end of round:
///   Activated Turn N  -> recovery_turns_remaining = 2
///   End of Turn N     -> not 0, decrement to 1 (no recovery)
///   End of Turn N+1   -> not 0, decrement to 0 (no recovery)
///   End of Turn N+2   -> is 0, recovers
pub fn tick_recovery(slots: &mut DjinnSlots, unit_ref: TargetRef) -> Vec<DjinnStateChanged> {
    let mut events = Vec::new();

    // First, recover the eligible djinn (already at 0 from a previous tick).
    // Only 1 recovers per tick — the one with the lowest activation_order.
    let candidate_idx = slots
        .slots
        .iter()
        .enumerate()
        .filter(|(_, d)| d.state == DjinnState::Recovery && d.recovery_turns_remaining == 0)
        .min_by_key(|(_, d)| d.activation_order)
        .map(|(i, _)| i);

    if let Some(idx) = candidate_idx {
        let djinn = &mut slots.slots[idx];
        let old_state = djinn.state;
        djinn.state = DjinnState::Good;
        djinn.activation_order = 0;

        events.push(DjinnStateChanged {
            djinn_id: djinn.djinn_id.clone(),
            unit: unit_ref,
            old_state,
            new_state: DjinnState::Good,
            recovery_turns: None,
        });
    }

    // Then, decrement recovery turns for all remaining Recovery djinn
    for djinn in &mut slots.slots {
        if djinn.state == DjinnState::Recovery && djinn.recovery_turns_remaining > 0 {
            djinn.recovery_turns_remaining -= 1;
        }
    }

    events
}

// ── Tests ────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::shared::{DjinnAbilityPairs, DjinnAbilitySet, Side};

    fn unit_ref() -> TargetRef {
        TargetRef {
            side: Side::Player,
            index: 0,
        }
    }

    fn make_djinn_def(id: &str, element: Element) -> DjinnDef {
        DjinnDef {
            id: DjinnId(id.to_string()),
            name: id.to_string(),
            element,
            tier: DjinnTier::new_unchecked(1),
            stat_bonus: StatBonus {
                atk: StatMod::new_unchecked(5),
                def: StatMod::new_unchecked(3),
                mag: StatMod::new_unchecked(2),
                spd: StatMod::new_unchecked(1),
                hp: StatMod::new_unchecked(10),
            },
            summon_effect: None,
            ability_pairs: DjinnAbilityPairs {
                same: DjinnAbilitySet {
                    good_abilities: vec![AbilityId("same_good".into())],
                    recovery_abilities: vec![AbilityId("same_recovery".into())],
                },
                counter: DjinnAbilitySet {
                    good_abilities: vec![AbilityId("counter_good".into())],
                    recovery_abilities: vec![AbilityId("counter_recovery".into())],
                },
                neutral: DjinnAbilitySet {
                    good_abilities: vec![AbilityId("neutral_always".into())],
                    recovery_abilities: vec![AbilityId("neutral_always".into())],
                },
            },
        }
    }

    fn make_djinn_def_with_summon_effect(
        id: &str,
        element: Element,
        summon_effect: SummonEffect,
    ) -> DjinnDef {
        let mut def = make_djinn_def(id, element);
        def.summon_effect = Some(summon_effect);
        def
    }

    fn make_slots_with(ids: &[&str], element: Element) -> DjinnSlots {
        let mut slots = DjinnSlots::new();
        for id in ids {
            slots.add(DjinnId(id.to_string()));
            // Ensure element is carried by DjinnDef, not instance
            let _ = element;
        }
        slots
    }

    // ── Compatibility tests ──

    #[test]
    fn compatibility_same_all_elements() {
        for elem in &[
            Element::Venus,
            Element::Mars,
            Element::Mercury,
            Element::Jupiter,
        ] {
            assert_eq!(
                determine_compatibility(*elem, *elem),
                DjinnCompatibility::Same
            );
        }
    }

    #[test]
    fn compatibility_counter_all_pairs() {
        assert_eq!(
            determine_compatibility(Element::Jupiter, Element::Venus),
            DjinnCompatibility::Counter
        );
        assert_eq!(
            determine_compatibility(Element::Venus, Element::Jupiter),
            DjinnCompatibility::Counter
        );
        assert_eq!(
            determine_compatibility(Element::Mercury, Element::Mars),
            DjinnCompatibility::Counter
        );
        assert_eq!(
            determine_compatibility(Element::Mars, Element::Mercury),
            DjinnCompatibility::Counter
        );
    }

    #[test]
    fn compatibility_neutral() {
        assert_eq!(
            determine_compatibility(Element::Venus, Element::Mars),
            DjinnCompatibility::Neutral
        );
        assert_eq!(
            determine_compatibility(Element::Mercury, Element::Jupiter),
            DjinnCompatibility::Neutral
        );
        assert_eq!(
            determine_compatibility(Element::Mars, Element::Jupiter),
            DjinnCompatibility::Neutral
        );
        assert_eq!(
            determine_compatibility(Element::Venus, Element::Mercury),
            DjinnCompatibility::Neutral
        );
    }

    // ── Ability grant tests ──

    #[test]
    fn abilities_same_good() {
        let def = make_djinn_def("flint", Element::Venus);
        let abilities = get_granted_abilities(&def, DjinnCompatibility::Same, DjinnState::Good);
        assert_eq!(abilities, vec![AbilityId("same_good".into())]);
    }

    #[test]
    fn abilities_same_recovery() {
        let def = make_djinn_def("flint", Element::Venus);
        let abilities = get_granted_abilities(&def, DjinnCompatibility::Same, DjinnState::Recovery);
        assert_eq!(abilities, vec![AbilityId("same_recovery".into())]);
    }

    #[test]
    fn abilities_counter_good() {
        let def = make_djinn_def("forge", Element::Mars);
        let abilities = get_granted_abilities(&def, DjinnCompatibility::Counter, DjinnState::Good);
        assert_eq!(abilities, vec![AbilityId("counter_good".into())]);
    }

    #[test]
    fn abilities_counter_recovery() {
        let def = make_djinn_def("forge", Element::Mars);
        let abilities =
            get_granted_abilities(&def, DjinnCompatibility::Counter, DjinnState::Recovery);
        assert_eq!(abilities, vec![AbilityId("counter_recovery".into())]);
    }

    #[test]
    fn abilities_neutral_always_good() {
        let def = make_djinn_def("fizz", Element::Mercury);
        let abilities = get_granted_abilities(&def, DjinnCompatibility::Neutral, DjinnState::Good);
        assert_eq!(abilities, vec![AbilityId("neutral_always".into())]);
    }

    #[test]
    fn abilities_neutral_always_recovery() {
        let def = make_djinn_def("fizz", Element::Mercury);
        let abilities =
            get_granted_abilities(&def, DjinnCompatibility::Neutral, DjinnState::Recovery);
        assert_eq!(abilities, vec![AbilityId("neutral_always".into())]);
    }

    // ── Activate djinn tests ──

    #[test]
    fn activate_good_djinn_succeeds() {
        let mut slots = make_slots_with(&["flint"], Element::Venus);
        let def = make_djinn_def("flint", Element::Venus);
        let event = activate_djinn(&mut slots, 0, unit_ref(), 2, &def);
        assert!(event.is_some());
        let (ev, activation) = event.unwrap();
        assert_eq!(ev.old_state, DjinnState::Good);
        assert_eq!(ev.new_state, DjinnState::Recovery);
        assert_eq!(ev.recovery_turns, Some(2));
        assert_eq!(activation.element, Element::Venus);
        assert_eq!(activation.base_damage, 0);
        assert_eq!(activation.effect_type, None);
        assert_eq!(slots.slots[0].state, DjinnState::Recovery);
    }

    #[test]
    fn test_activation_damage_event() {
        let mut slots = make_slots_with(&["flint"], Element::Venus);
        let def = make_djinn_def_with_summon_effect(
            "flint",
            Element::Venus,
            SummonEffect {
                damage: 27,
                buff: None,
                status: None,
                heal: None,
            },
        );

        let (_, activation) = activate_djinn(&mut slots, 0, unit_ref(), 2, &def).unwrap();

        assert_eq!(activation.element, Element::Venus);
        assert_eq!(activation.base_damage, 27);
        assert_eq!(
            activation.effect_type,
            Some(DjinnActivationEffectType::Damage)
        );
    }

    #[test]
    fn test_activation_heal_event() {
        let mut slots = make_slots_with(&["mist"], Element::Mercury);
        let def = make_djinn_def_with_summon_effect(
            "mist",
            Element::Mercury,
            SummonEffect {
                damage: 0,
                buff: None,
                status: None,
                heal: Some(18),
            },
        );

        let (_, activation) = activate_djinn(&mut slots, 0, unit_ref(), 2, &def).unwrap();

        assert_eq!(activation.element, Element::Mercury);
        assert_eq!(activation.base_damage, 0);
        assert_eq!(
            activation.effect_type,
            Some(DjinnActivationEffectType::Heal)
        );
    }

    #[test]
    fn activate_recovery_djinn_fails() {
        let mut slots = make_slots_with(&["flint"], Element::Venus);
        slots.slots[0].state = DjinnState::Recovery;
        let def = make_djinn_def("flint", Element::Venus);
        let event = activate_djinn(&mut slots, 0, unit_ref(), 2, &def);
        assert!(event.is_none());
    }

    #[test]
    fn activate_out_of_range_fails() {
        let mut slots = make_slots_with(&["flint"], Element::Venus);
        let def = make_djinn_def("flint", Element::Venus);
        let event = activate_djinn(&mut slots, 5, unit_ref(), 2, &def);
        assert!(event.is_none());
    }

    // ── Stat bonus tests ──

    #[test]
    fn stat_bonus_only_good_djinn() {
        let mut slots = make_slots_with(&["flint", "granite"], Element::Venus);
        let defs = vec![
            make_djinn_def("flint", Element::Venus),
            make_djinn_def("granite", Element::Venus),
        ];

        // Both Good: both contribute
        let bonus = compute_djinn_stat_bonus(&slots, &defs);
        assert_eq!(bonus.atk.get(), 10);
        assert_eq!(bonus.def.get(), 6);

        // Set one to Recovery
        slots.slots[1].state = DjinnState::Recovery;
        let bonus = compute_djinn_stat_bonus(&slots, &defs);
        assert_eq!(bonus.atk.get(), 5);
        assert_eq!(bonus.def.get(), 3);
    }

    #[test]
    fn stat_bonus_empty_slots() {
        let slots = DjinnSlots::new();
        let defs: Vec<DjinnDef> = vec![];
        let bonus = compute_djinn_stat_bonus(&slots, &defs);
        assert_eq!(bonus.atk.get(), 0);
        assert_eq!(bonus.def.get(), 0);
    }

    // ── Summon availability tests ──

    #[test]
    fn summon_availability_zero_good() {
        let tiers = get_available_summons(0);
        assert!(tiers.is_empty());
    }

    #[test]
    fn summon_availability_one_good() {
        let tiers = get_available_summons(1);
        assert_eq!(tiers.len(), 1);
        assert_eq!(tiers[0].tier, 1);
    }

    #[test]
    fn summon_availability_two_good() {
        let tiers = get_available_summons(2);
        assert_eq!(tiers.len(), 2);
        assert_eq!(tiers[0].tier, 1);
        assert_eq!(tiers[1].tier, 2);
    }

    #[test]
    fn summon_availability_three_good() {
        let tiers = get_available_summons(3);
        assert_eq!(tiers.len(), 3);
        assert_eq!(tiers[2].tier, 3);
        assert_eq!(tiers[2].required_good, 3);
    }

    // ── Execute summon tests ──

    #[test]
    fn execute_summon_transitions_all_to_recovery() {
        let mut slots = make_slots_with(&["flint", "granite", "quartz"], Element::Venus);
        let events = execute_summon(&mut slots, &[0, 1, 2], unit_ref(), 2);
        assert!(events.is_some());
        let events = events.unwrap();
        assert_eq!(events.len(), 3);
        for ev in &events {
            assert_eq!(ev.new_state, DjinnState::Recovery);
        }
        for inst in &slots.slots {
            assert_eq!(inst.state, DjinnState::Recovery);
        }
    }

    #[test]
    fn execute_summon_fails_if_any_recovery() {
        let mut slots = make_slots_with(&["flint", "granite"], Element::Venus);
        slots.slots[1].state = DjinnState::Recovery;
        let events = execute_summon(&mut slots, &[0, 1], unit_ref(), 2);
        assert!(events.is_none());
        // First djinn should still be Good (transaction rolled back)
        assert_eq!(slots.slots[0].state, DjinnState::Good);
    }

    // ── Recovery tick tests ──

    #[test]
    fn tick_recovery_respects_delay() {
        let mut slots = make_slots_with(&["flint"], Element::Venus);
        // Activate with recovery_turns=2 (matches DESIGN_LOCK: "starting the turn after next")
        // Activated Turn N -> no recovery Turn N+1 -> no recovery Turn N+2's start -> recovers Turn N+2
        let def = make_djinn_def("flint", Element::Venus);
        activate_djinn(&mut slots, 0, unit_ref(), 2, &def);
        assert_eq!(slots.slots[0].state, DjinnState::Recovery);
        assert_eq!(slots.slots[0].recovery_turns_remaining, 2);

        // Tick 1 (end of activation turn): not at 0 yet, decrement 2->1, no recovery
        let events = tick_recovery(&mut slots, unit_ref());
        assert!(events.is_empty());
        assert_eq!(slots.slots[0].recovery_turns_remaining, 1);

        // Tick 2 (end of Turn N+1): not at 0 yet, decrement 1->0, no recovery
        // DESIGN_LOCK: "Turn 4: Nothing recovers yet."
        let events = tick_recovery(&mut slots, unit_ref());
        assert!(events.is_empty());
        assert_eq!(slots.slots[0].recovery_turns_remaining, 0);

        // Tick 3 (end of Turn N+2): already at 0, recovers!
        // DESIGN_LOCK: "Turn 5: Djinn 1 → GOOD."
        let events = tick_recovery(&mut slots, unit_ref());
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].new_state, DjinnState::Good);
    }

    #[test]
    fn tick_recovery_staggered_order() {
        let mut slots = make_slots_with(&["flint", "granite"], Element::Venus);

        // Activate flint first (order 0), then granite (order 1), both with 1 recovery turn
        let def_a = make_djinn_def("flint", Element::Venus);
        let def_b = make_djinn_def("granite", Element::Venus);
        activate_djinn(&mut slots, 0, unit_ref(), 1, &def_a);
        activate_djinn(&mut slots, 1, unit_ref(), 1, &def_b);

        // Tick 1: neither is at 0 yet, decrement both 1->0, no recovery
        let events = tick_recovery(&mut slots, unit_ref());
        assert!(events.is_empty());
        assert_eq!(slots.slots[0].recovery_turns_remaining, 0);
        assert_eq!(slots.slots[1].recovery_turns_remaining, 0);

        // Tick 2: both at 0. Flint recovers first (lower activation_order).
        let events = tick_recovery(&mut slots, unit_ref());
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].djinn_id, DjinnId("flint".into()));
        assert_eq!(slots.slots[0].state, DjinnState::Good);
        assert_eq!(slots.slots[1].state, DjinnState::Recovery);

        // Tick 3: granite recovers (still at 0, now lowest remaining order)
        let events = tick_recovery(&mut slots, unit_ref());
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].djinn_id, DjinnId("granite".into()));
        assert_eq!(slots.slots[1].state, DjinnState::Good);
    }

    #[test]
    fn tick_recovery_no_recovery_djinn() {
        let mut slots = make_slots_with(&["flint"], Element::Venus);
        // All Good, nothing to recover
        let events = tick_recovery(&mut slots, unit_ref());
        assert!(events.is_empty());
    }

    // ── DESIGN_LOCK compliance test ──

    /// Mirrors the exact 3-Djinn Summon Example from DESIGN_LOCK.md §4:
    ///   Turn 3: Summon fires. All 3 djinn → RECOVERY.
    ///   Turn 4: Nothing recovers yet.
    ///   Turn 5: Djinn 1 → GOOD.
    ///   Turn 6: Djinn 2 → GOOD.
    ///   Turn 7: Djinn 3 → GOOD.
    #[test]
    fn design_lock_3_djinn_summon_example() {
        let mut slots = make_slots_with(&["djinn1", "djinn2", "djinn3"], Element::Venus);

        // Turn 3: Summon fires with recovery_turns=2 (delay=1 + per_turn=1)
        let events = execute_summon(&mut slots, &[0, 1, 2], unit_ref(), 2);
        assert!(events.is_some());
        assert_eq!(slots.slots[0].state, DjinnState::Recovery);
        assert_eq!(slots.slots[1].state, DjinnState::Recovery);
        assert_eq!(slots.slots[2].state, DjinnState::Recovery);

        // End of Turn 3: tick — all at 2, decrement to 1, no recovery
        let events = tick_recovery(&mut slots, unit_ref());
        assert!(events.is_empty(), "End of Turn 3: nothing should recover");

        // End of Turn 4: tick — all at 1, decrement to 0, no recovery
        // DESIGN_LOCK: "Turn 4: Nothing recovers yet."
        let events = tick_recovery(&mut slots, unit_ref());
        assert!(
            events.is_empty(),
            "Turn 4: Nothing recovers yet (DESIGN_LOCK)"
        );

        // End of Turn 5: all at 0. Djinn 1 recovers (lowest activation_order).
        // DESIGN_LOCK: "Turn 5: Djinn 1 → GOOD."
        let events = tick_recovery(&mut slots, unit_ref());
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].djinn_id, DjinnId("djinn1".into()));
        assert_eq!(events[0].new_state, DjinnState::Good);
        assert_eq!(slots.slots[0].state, DjinnState::Good);
        assert_eq!(slots.slots[1].state, DjinnState::Recovery);
        assert_eq!(slots.slots[2].state, DjinnState::Recovery);

        // End of Turn 6: Djinn 2 → GOOD.
        let events = tick_recovery(&mut slots, unit_ref());
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].djinn_id, DjinnId("djinn2".into()));
        assert_eq!(events[0].new_state, DjinnState::Good);
        assert_eq!(slots.slots[1].state, DjinnState::Good);
        assert_eq!(slots.slots[2].state, DjinnState::Recovery);

        // End of Turn 7: Djinn 3 → GOOD. All restored.
        let events = tick_recovery(&mut slots, unit_ref());
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].djinn_id, DjinnId("djinn3".into()));
        assert_eq!(events[0].new_state, DjinnState::Good);
        assert_eq!(slots.slots[0].state, DjinnState::Good);
        assert_eq!(slots.slots[1].state, DjinnState::Good);
        assert_eq!(slots.slots[2].state, DjinnState::Good);
    }

    // ── Ability oscillation test ──

    #[test]
    fn ability_oscillation_on_state_change() {
        let def = make_djinn_def("flint", Element::Venus);
        let compat = DjinnCompatibility::Same;

        // Good state
        let abilities = get_granted_abilities(&def, compat, DjinnState::Good);
        assert_eq!(abilities, vec![AbilityId("same_good".into())]);

        // After activation -> Recovery
        let abilities = get_granted_abilities(&def, compat, DjinnState::Recovery);
        assert_eq!(abilities, vec![AbilityId("same_recovery".into())]);

        // After recovery tick -> Good again
        let abilities = get_granted_abilities(&def, compat, DjinnState::Good);
        assert_eq!(abilities, vec![AbilityId("same_good".into())]);
    }

    // ── Slot capacity test ──

    #[test]
    fn slots_max_capacity_is_three() {
        let mut slots = DjinnSlots::new();
        assert!(slots.add(DjinnId("a".into())));
        assert!(slots.add(DjinnId("b".into())));
        assert!(slots.add(DjinnId("c".into())));
        assert!(!slots.add(DjinnId("d".into())));
        assert_eq!(slots.slots.len(), 3);
    }

    #[test]
    fn good_count_tracks_correctly() {
        let mut slots = make_slots_with(&["a", "b", "c"], Element::Venus);
        assert_eq!(slots.good_count(), 3);
        slots.slots[0].state = DjinnState::Recovery;
        assert_eq!(slots.good_count(), 2);
        slots.slots[1].state = DjinnState::Recovery;
        assert_eq!(slots.good_count(), 1);
    }
}
