#![allow(dead_code, clippy::new_without_default)]
//! Djinn state machine: Good/Recovery states, ability oscillation,
//! summon execution, and staggered recovery.

use crate::shared::{
    AbilityId, DjinnCompatibility, DjinnDef, DjinnId, DjinnState, DjinnStateChanged, Element,
    StatBonus, TargetRef,
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
) -> Option<DjinnStateChanged> {
    let djinn = slots.slots.get_mut(index)?;
    if djinn.state != DjinnState::Good {
        return None;
    }

    let old_state = djinn.state;
    djinn.state = DjinnState::Recovery;
    djinn.recovery_turns_remaining = recovery_turns;
    djinn.activation_order = slots.next_activation_order;
    slots.next_activation_order += 1;

    Some(DjinnStateChanged {
        djinn_id: djinn.djinn_id.clone(),
        unit: unit_ref,
        old_state,
        new_state: DjinnState::Recovery,
        recovery_turns: Some(recovery_turns),
    })
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
            bonus.atk += def.stat_bonus.atk;
            bonus.def += def.stat_bonus.def;
            bonus.mag += def.stat_bonus.mag;
            bonus.spd += def.stat_bonus.spd;
            bonus.hp += def.stat_bonus.hp;
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
/// whose recovery_turns_remaining has reached 0.
/// Djinn with remaining turns > 0 get decremented by 1 first (1-turn delay).
/// Returns state-change events for any djinn that recovered this tick.
pub fn tick_recovery(slots: &mut DjinnSlots, unit_ref: TargetRef) -> Vec<DjinnStateChanged> {
    let mut events = Vec::new();

    // First, decrement recovery turns for all Recovery djinn
    for djinn in &mut slots.slots {
        if djinn.state == DjinnState::Recovery && djinn.recovery_turns_remaining > 0 {
            djinn.recovery_turns_remaining -= 1;
        }
    }

    // Find the Recovery djinn with lowest activation_order that is ready (turns == 0)
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
            tier: 1,
            stat_bonus: StatBonus {
                atk: 5,
                def: 3,
                mag: 2,
                spd: 1,
                hp: 10,
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
        for elem in &[Element::Venus, Element::Mars, Element::Mercury, Element::Jupiter] {
            assert_eq!(determine_compatibility(*elem, *elem), DjinnCompatibility::Same);
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
        let abilities =
            get_granted_abilities(&def, DjinnCompatibility::Same, DjinnState::Recovery);
        assert_eq!(abilities, vec![AbilityId("same_recovery".into())]);
    }

    #[test]
    fn abilities_counter_good() {
        let def = make_djinn_def("forge", Element::Mars);
        let abilities =
            get_granted_abilities(&def, DjinnCompatibility::Counter, DjinnState::Good);
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
        let abilities =
            get_granted_abilities(&def, DjinnCompatibility::Neutral, DjinnState::Good);
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
        let event = activate_djinn(&mut slots, 0, unit_ref(), 2);
        assert!(event.is_some());
        let ev = event.unwrap();
        assert_eq!(ev.old_state, DjinnState::Good);
        assert_eq!(ev.new_state, DjinnState::Recovery);
        assert_eq!(ev.recovery_turns, Some(2));
        assert_eq!(slots.slots[0].state, DjinnState::Recovery);
    }

    #[test]
    fn activate_recovery_djinn_fails() {
        let mut slots = make_slots_with(&["flint"], Element::Venus);
        slots.slots[0].state = DjinnState::Recovery;
        let event = activate_djinn(&mut slots, 0, unit_ref(), 2);
        assert!(event.is_none());
    }

    #[test]
    fn activate_out_of_range_fails() {
        let mut slots = make_slots_with(&["flint"], Element::Venus);
        let event = activate_djinn(&mut slots, 5, unit_ref(), 2);
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
        assert_eq!(bonus.atk, 10);
        assert_eq!(bonus.def, 6);

        // Set one to Recovery
        slots.slots[1].state = DjinnState::Recovery;
        let bonus = compute_djinn_stat_bonus(&slots, &defs);
        assert_eq!(bonus.atk, 5);
        assert_eq!(bonus.def, 3);
    }

    #[test]
    fn stat_bonus_empty_slots() {
        let slots = DjinnSlots::new();
        let defs: Vec<DjinnDef> = vec![];
        let bonus = compute_djinn_stat_bonus(&slots, &defs);
        assert_eq!(bonus.atk, 0);
        assert_eq!(bonus.def, 0);
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
        // Activate with recovery_turns=2 (1 turn delay means 2 ticks to recover)
        activate_djinn(&mut slots, 0, unit_ref(), 2);
        assert_eq!(slots.slots[0].state, DjinnState::Recovery);
        assert_eq!(slots.slots[0].recovery_turns_remaining, 2);

        // Tick 1: decrement to 1, no recovery yet
        let events = tick_recovery(&mut slots, unit_ref());
        assert!(events.is_empty());
        assert_eq!(slots.slots[0].recovery_turns_remaining, 1);

        // Tick 2: decrement to 0, still no recovery this tick (just reached 0)
        let events = tick_recovery(&mut slots, unit_ref());
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].new_state, DjinnState::Good);
    }

    #[test]
    fn tick_recovery_staggered_order() {
        let mut slots = make_slots_with(&["flint", "granite"], Element::Venus);

        // Activate flint first (order 0), then granite (order 1)
        activate_djinn(&mut slots, 0, unit_ref(), 1);
        activate_djinn(&mut slots, 1, unit_ref(), 1);

        // Tick 1: flint recovers first (lower activation_order)
        let events = tick_recovery(&mut slots, unit_ref());
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].djinn_id, DjinnId("flint".into()));
        assert_eq!(slots.slots[0].state, DjinnState::Good);
        assert_eq!(slots.slots[1].state, DjinnState::Recovery);

        // Tick 2: granite recovers (was already at 0 turns, now it's the lowest order)
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
