// encounter domain — Wave 2
#![allow(dead_code)]

use crate::shared::{
    BossEncounter, DialogueTreeId, EncounterDef, EncounterTable, MapNodeId,
};

/// Returns true when the player has walked enough steps to trigger an encounter check.
/// Deterministic: fires when steps_since_last >= base_rate threshold.
pub fn should_encounter(table: &EncounterTable, steps_since_last: u16) -> bool {
    steps_since_last >= table.base_rate.get() as u16
}

/// Deterministically selects an encounter from the table using step_count as a seed.
/// Only considers slots that are not exhausted (max_triggers != Some(0)).
/// Returns None if the table is empty or all slots are exhausted.
pub fn select_encounter(table: &EncounterTable, step_count: u16) -> Option<&EncounterDef> {
    let active: Vec<(usize, &crate::shared::EncounterSlot)> = table
        .entries
        .iter()
        .enumerate()
        .filter(|(_, slot)| slot.max_triggers != Some(0))
        .collect();

    if active.is_empty() {
        return None;
    }

    let total_weight: u32 = active.iter().map(|(_, s)| s.weight as u32).sum();
    if total_weight == 0 {
        return None;
    }

    let mut pick = (step_count as u32) % total_weight;
    for (_, slot) in &active {
        let w = slot.weight as u32;
        if pick < w {
            return Some(&slot.encounter);
        }
        pick -= w;
    }

    // Fallback: return last active slot (should never reach here with valid weights)
    active.last().map(|(_, s)| &s.encounter)
}

/// Decrements `max_triggers` for the slot at `encounter_idx`, if it has a finite trigger count.
pub fn decrement_slot(table: &mut EncounterTable, encounter_idx: usize) {
    if let Some(slot) = table.entries.get_mut(encounter_idx) {
        if let Some(ref mut triggers) = slot.max_triggers {
            if *triggers > 0 {
                *triggers -= 1;
            }
        }
    }
}

/// Prepares a boss encounter, returning pre-dialogue, the encounter def, and nodes to unlock.
pub fn prepare_boss(boss: &BossEncounter) -> (Option<DialogueTreeId>, EncounterDef, Vec<MapNodeId>) {
    (
        boss.pre_dialogue,
        boss.encounter.clone(),
        boss.unlock_on_defeat.clone(),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::shared::{
        bounded_types::{EncounterRate, Gold, Xp},
        BossEncounter, DialogueTreeId, EncounterId, EncounterDef, EncounterSlot, EncounterTable,
        MapNodeId,
    };

    fn make_encounter(id: &str) -> EncounterDef {
        EncounterDef {
            id: EncounterId(id.to_string()),
            name: id.to_string(),
            difficulty: crate::shared::Difficulty::Medium,
            enemies: vec![],
            xp_reward: Xp::new(10),
            gold_reward: Gold::new(5),
            recruit: None,
            djinn_reward: None,
            equipment_rewards: vec![],
        }
    }

    fn make_table(base_rate: u8, slots: Vec<EncounterSlot>) -> EncounterTable {
        EncounterTable {
            region_id: 1,
            base_rate: EncounterRate::new(base_rate),
            entries: slots,
        }
    }

    fn slot(id: &str, weight: u8, max_triggers: Option<u8>) -> EncounterSlot {
        EncounterSlot {
            encounter: make_encounter(id),
            weight,
            max_triggers,
        }
    }

    // ── should_encounter ────────────────────────────────────────────

    #[test]
    fn test_should_encounter_below_threshold() {
        let table = make_table(10, vec![slot("wolf", 1, None)]);
        assert!(!should_encounter(&table, 9));
    }

    #[test]
    fn test_should_encounter_at_threshold() {
        let table = make_table(10, vec![slot("wolf", 1, None)]);
        assert!(should_encounter(&table, 10));
    }

    #[test]
    fn test_should_encounter_above_threshold() {
        let table = make_table(5, vec![slot("wolf", 1, None)]);
        assert!(should_encounter(&table, 99));
    }

    // ── select_encounter ────────────────────────────────────────────

    #[test]
    fn test_select_encounter_deterministic() {
        let table = make_table(5, vec![slot("slime", 3, None), slot("wolf", 7, None)]);
        // Same step_count must always return the same encounter.
        let a = select_encounter(&table, 42).map(|e| e.id.0.clone());
        let b = select_encounter(&table, 42).map(|e| e.id.0.clone());
        assert_eq!(a, b);
    }

    #[test]
    fn test_select_encounter_weighted() {
        // weight 10 for "slime", 0 for "wolf" → always slime
        let table = make_table(5, vec![slot("slime", 10, None), slot("wolf", 0, None)]);
        for step in 0..20u16 {
            let enc = select_encounter(&table, step);
            assert_eq!(enc.map(|e| e.id.0.as_str()), Some("slime"));
        }
    }

    #[test]
    fn test_select_encounter_empty_table() {
        let table = make_table(5, vec![]);
        assert!(select_encounter(&table, 0).is_none());
    }

    #[test]
    fn test_select_encounter_all_exhausted() {
        let table = make_table(5, vec![slot("slime", 5, Some(0)), slot("wolf", 5, Some(0))]);
        assert!(select_encounter(&table, 7).is_none());
    }

    #[test]
    fn test_select_encounter_skips_exhausted_slot() {
        // slime exhausted, wolf still available
        let table = make_table(5, vec![slot("slime", 5, Some(0)), slot("wolf", 5, None)]);
        let enc = select_encounter(&table, 0);
        assert_eq!(enc.map(|e| e.id.0.as_str()), Some("wolf"));
    }

    // ── decrement_slot ──────────────────────────────────────────────

    #[test]
    fn test_decrement_slot_reduces_triggers() {
        let mut table = make_table(5, vec![slot("boss", 1, Some(3))]);
        decrement_slot(&mut table, 0);
        assert_eq!(table.entries[0].max_triggers, Some(2));
    }

    #[test]
    fn test_decrement_slot_no_underflow() {
        let mut table = make_table(5, vec![slot("boss", 1, Some(0))]);
        decrement_slot(&mut table, 0);
        assert_eq!(table.entries[0].max_triggers, Some(0));
    }

    #[test]
    fn test_decrement_slot_unlimited_unchanged() {
        let mut table = make_table(5, vec![slot("wolf", 1, None)]);
        decrement_slot(&mut table, 0);
        assert_eq!(table.entries[0].max_triggers, None);
    }

    // ── prepare_boss ────────────────────────────────────────────────

    #[test]
    fn test_prepare_boss_returns_correct_fields() {
        let boss = BossEncounter {
            encounter: make_encounter("dragon"),
            pre_dialogue: Some(DialogueTreeId(42)),
            post_dialogue: None,
            quest_advance: None,
            unlock_on_defeat: vec![MapNodeId(7), MapNodeId(8)],
        };
        let (dlg, enc, nodes) = prepare_boss(&boss);
        assert_eq!(dlg, Some(DialogueTreeId(42)));
        assert_eq!(enc.id.0, "dragon");
        assert_eq!(nodes, vec![MapNodeId(7), MapNodeId(8)]);
    }

    #[test]
    fn test_prepare_boss_no_dialogue() {
        let boss = BossEncounter {
            encounter: make_encounter("golem"),
            pre_dialogue: None,
            post_dialogue: None,
            quest_advance: None,
            unlock_on_defeat: vec![],
        };
        let (dlg, enc, nodes) = prepare_boss(&boss);
        assert_eq!(dlg, None);
        assert_eq!(enc.id.0, "golem");
        assert!(nodes.is_empty());
    }
}
