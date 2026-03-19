#![allow(dead_code)]
//! Equipment domain — loadout management, stat bonus computation,
//! ability unlocking, and set bonus detection.

use std::collections::HashMap;

use crate::shared::bounded_types::StatMod;
use crate::shared::{
    AbilityId, Element, EquipmentDef, EquipmentId, EquipmentSlot, SetId, StatBonus,
};

// ── Structs ──────────────────────────────────────────────────────────

/// Per-unit equipment loadout with 5 slots.
#[derive(Debug, Clone, Default)]
pub struct EquipmentLoadout {
    pub weapon: Option<EquipmentId>,
    pub helm: Option<EquipmentId>,
    pub armor: Option<EquipmentId>,
    pub boots: Option<EquipmentId>,
    pub accessory: Option<EquipmentId>,
}

/// Computed aggregate effects from an equipment loadout.
#[derive(Debug, Clone, Default)]
pub struct EquipmentEffects {
    pub total_stat_bonus: StatBonus,
    pub unlocked_abilities: Vec<AbilityId>,
    pub total_mana_bonus: u8,
    pub total_hit_count_bonus: u8,
    pub always_first_turn: bool,
    pub active_set_bonuses: Vec<SetId>,
}

// ── Errors ───────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EquipError {
    /// Equipment's slot does not match the requested slot.
    WrongSlot,
    /// Unit's element is not in the equipment's allowed_elements.
    WrongElement,
}

// ── Functions ────────────────────────────────────────────────────────

/// Returns true if `unit_element` is in `equipment_def.allowed_elements`.
pub fn can_equip(equipment_def: &EquipmentDef, unit_element: Element) -> bool {
    equipment_def.allowed_elements.contains(&unit_element)
}

/// Equip an item into the given slot of a loadout.
///
/// Validates that the equipment's slot matches and the unit's element is allowed.
pub fn equip(
    loadout: &mut EquipmentLoadout,
    slot: EquipmentSlot,
    equipment_id: EquipmentId,
    equipment_def: &EquipmentDef,
    unit_element: Element,
) -> Result<(), EquipError> {
    if equipment_def.slot != slot {
        return Err(EquipError::WrongSlot);
    }
    if !can_equip(equipment_def, unit_element) {
        return Err(EquipError::WrongElement);
    }
    match slot {
        EquipmentSlot::Weapon => loadout.weapon = Some(equipment_id),
        EquipmentSlot::Helm => loadout.helm = Some(equipment_id),
        EquipmentSlot::Armor => loadout.armor = Some(equipment_id),
        EquipmentSlot::Boots => loadout.boots = Some(equipment_id),
        EquipmentSlot::Accessory => loadout.accessory = Some(equipment_id),
    }
    Ok(())
}

/// Remove whatever is in the given slot, returning the removed id (if any).
pub fn unequip(loadout: &mut EquipmentLoadout, slot: EquipmentSlot) -> Option<EquipmentId> {
    match slot {
        EquipmentSlot::Weapon => loadout.weapon.take(),
        EquipmentSlot::Helm => loadout.helm.take(),
        EquipmentSlot::Armor => loadout.armor.take(),
        EquipmentSlot::Boots => loadout.boots.take(),
        EquipmentSlot::Accessory => loadout.accessory.take(),
    }
}

/// Return whatever is equipped in the given slot (without removing it).
pub fn get_equipped_in_slot(
    loadout: &EquipmentLoadout,
    slot: EquipmentSlot,
) -> Option<&EquipmentId> {
    match slot {
        EquipmentSlot::Weapon => loadout.weapon.as_ref(),
        EquipmentSlot::Helm => loadout.helm.as_ref(),
        EquipmentSlot::Armor => loadout.armor.as_ref(),
        EquipmentSlot::Boots => loadout.boots.as_ref(),
        EquipmentSlot::Accessory => loadout.accessory.as_ref(),
    }
}

/// Compute the aggregate effects of every equipped item in the loadout.
///
/// - Sums stat bonuses, mana bonuses, and hit-count bonuses.
/// - Collects unlocked abilities.
/// - Detects `always_first_turn` flag on any item.
/// - Detects set bonuses (2+ items sharing the same `set_id`).
pub fn compute_equipment_effects(
    loadout: &EquipmentLoadout,
    defs: &HashMap<EquipmentId, EquipmentDef>,
) -> EquipmentEffects {
    let mut effects = EquipmentEffects::default();

    // Collect all equipped ids in slot order.
    let equipped: Vec<&EquipmentId> = [
        loadout.weapon.as_ref(),
        loadout.helm.as_ref(),
        loadout.armor.as_ref(),
        loadout.boots.as_ref(),
        loadout.accessory.as_ref(),
    ]
    .into_iter()
    .flatten()
    .collect();

    // Track set counts for set-bonus detection.
    let mut set_counts: HashMap<SetId, u8> = HashMap::new();

    for eq_id in &equipped {
        if let Some(def) = defs.get(eq_id) {
            // Stat bonuses
            effects.total_stat_bonus.atk = StatMod::new_unchecked(
                (effects.total_stat_bonus.atk.get() + def.stat_bonus.atk.get()).clamp(-999, 999),
            );
            effects.total_stat_bonus.def = StatMod::new_unchecked(
                (effects.total_stat_bonus.def.get() + def.stat_bonus.def.get()).clamp(-999, 999),
            );
            effects.total_stat_bonus.mag = StatMod::new_unchecked(
                (effects.total_stat_bonus.mag.get() + def.stat_bonus.mag.get()).clamp(-999, 999),
            );
            effects.total_stat_bonus.spd = StatMod::new_unchecked(
                (effects.total_stat_bonus.spd.get() + def.stat_bonus.spd.get()).clamp(-999, 999),
            );
            effects.total_stat_bonus.hp = StatMod::new_unchecked(
                (effects.total_stat_bonus.hp.get() + def.stat_bonus.hp.get()).clamp(-999, 999),
            );

            // Ability unlock
            if let Some(ref ability_id) = def.unlocks_ability {
                effects.unlocked_abilities.push(ability_id.clone());
            }

            // Mana bonus
            if let Some(mana) = def.mana_bonus {
                effects.total_mana_bonus = effects.total_mana_bonus.saturating_add(mana);
            }

            // Hit-count bonus
            if let Some(hc) = def.hit_count_bonus {
                effects.total_hit_count_bonus = effects.total_hit_count_bonus.saturating_add(hc);
            }

            // Always-first-turn flag
            if def.always_first_turn {
                effects.always_first_turn = true;
            }

            // Set tracking
            if let Some(ref set_id) = def.set_id {
                *set_counts.entry(set_id.clone()).or_insert(0) += 1;
            }
        }
    }

    // Detect active set bonuses (2+ items with the same set_id).
    for (set_id, count) in set_counts {
        if count >= 2 {
            effects.active_set_bonuses.push(set_id);
        }
    }

    effects
}

// ── Tests ────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::shared::bounded_types::*;
    use crate::shared::{EquipmentTier, SetId};

    /// Helper: build a minimal EquipmentDef for testing.
    fn make_def(id: &str, slot: EquipmentSlot, elements: Vec<Element>) -> EquipmentDef {
        EquipmentDef {
            id: EquipmentId(id.to_string()),
            name: id.to_string(),
            slot,
            tier: EquipmentTier::Basic,
            cost: Gold::new_unchecked(100),
            allowed_elements: elements,
            stat_bonus: StatBonus::default(),
            unlocks_ability: None,
            set_id: None,
            always_first_turn: false,
            mana_bonus: None,
            hit_count_bonus: None,
        }
    }

    #[test]
    fn test_equip_success() {
        let mut loadout = EquipmentLoadout::default();
        let def = make_def("sword1", EquipmentSlot::Weapon, vec![Element::Venus]);
        let result = equip(
            &mut loadout,
            EquipmentSlot::Weapon,
            EquipmentId("sword1".into()),
            &def,
            Element::Venus,
        );
        assert!(result.is_ok());
        assert_eq!(loadout.weapon, Some(EquipmentId("sword1".into())));
    }

    #[test]
    fn test_equip_wrong_slot() {
        let mut loadout = EquipmentLoadout::default();
        let def = make_def("helm1", EquipmentSlot::Helm, vec![Element::Venus]);
        let result = equip(
            &mut loadout,
            EquipmentSlot::Weapon,
            EquipmentId("helm1".into()),
            &def,
            Element::Venus,
        );
        assert_eq!(result, Err(EquipError::WrongSlot));
    }

    #[test]
    fn test_equip_wrong_element() {
        let mut loadout = EquipmentLoadout::default();
        let def = make_def("sword2", EquipmentSlot::Weapon, vec![Element::Mars]);
        let result = equip(
            &mut loadout,
            EquipmentSlot::Weapon,
            EquipmentId("sword2".into()),
            &def,
            Element::Venus,
        );
        assert_eq!(result, Err(EquipError::WrongElement));
    }

    #[test]
    fn test_unequip_returns_item() {
        let mut loadout = EquipmentLoadout::default();
        loadout.weapon = Some(EquipmentId("sword1".into()));
        let removed = unequip(&mut loadout, EquipmentSlot::Weapon);
        assert_eq!(removed, Some(EquipmentId("sword1".into())));
        assert!(loadout.weapon.is_none());
    }

    #[test]
    fn test_unequip_empty_slot() {
        let mut loadout = EquipmentLoadout::default();
        let removed = unequip(&mut loadout, EquipmentSlot::Helm);
        assert!(removed.is_none());
    }

    #[test]
    fn test_stat_bonus_computation_two_items() {
        let mut loadout = EquipmentLoadout::default();
        let mut def_w = make_def("sword1", EquipmentSlot::Weapon, vec![Element::Venus]);
        def_w.stat_bonus = StatBonus {
            atk: StatMod::new_unchecked(10),
            def: StatMod::new_unchecked(0),
            mag: StatMod::new_unchecked(0),
            spd: StatMod::new_unchecked(5),
            hp: StatMod::new_unchecked(0),
        };
        let mut def_h = make_def("helm1", EquipmentSlot::Helm, vec![Element::Venus]);
        def_h.stat_bonus = StatBonus {
            atk: StatMod::new_unchecked(0),
            def: StatMod::new_unchecked(8),
            mag: StatMod::new_unchecked(0),
            spd: StatMod::new_unchecked(0),
            hp: StatMod::new_unchecked(20),
        };

        loadout.weapon = Some(EquipmentId("sword1".into()));
        loadout.helm = Some(EquipmentId("helm1".into()));

        let mut defs = HashMap::new();
        defs.insert(EquipmentId("sword1".into()), def_w);
        defs.insert(EquipmentId("helm1".into()), def_h);

        let effects = compute_equipment_effects(&loadout, &defs);
        assert_eq!(effects.total_stat_bonus.atk.get(), 10);
        assert_eq!(effects.total_stat_bonus.def.get(), 8);
        assert_eq!(effects.total_stat_bonus.spd.get(), 5);
        assert_eq!(effects.total_stat_bonus.hp.get(), 20);
    }

    #[test]
    fn test_ability_unlock_from_weapon() {
        let mut loadout = EquipmentLoadout::default();
        let mut def = make_def("magic_sword", EquipmentSlot::Weapon, vec![Element::Jupiter]);
        def.unlocks_ability = Some(AbilityId("ragnarok".into()));

        loadout.weapon = Some(EquipmentId("magic_sword".into()));

        let mut defs = HashMap::new();
        defs.insert(EquipmentId("magic_sword".into()), def);

        let effects = compute_equipment_effects(&loadout, &defs);
        assert_eq!(effects.unlocked_abilities.len(), 1);
        assert_eq!(effects.unlocked_abilities[0], AbilityId("ragnarok".into()));
    }

    #[test]
    fn test_mana_bonus_accumulation() {
        let mut loadout = EquipmentLoadout::default();
        let mut def_w = make_def("staff1", EquipmentSlot::Weapon, vec![Element::Mercury]);
        def_w.mana_bonus = Some(3);
        let mut def_a = make_def("robe1", EquipmentSlot::Armor, vec![Element::Mercury]);
        def_a.mana_bonus = Some(2);

        loadout.weapon = Some(EquipmentId("staff1".into()));
        loadout.armor = Some(EquipmentId("robe1".into()));

        let mut defs = HashMap::new();
        defs.insert(EquipmentId("staff1".into()), def_w);
        defs.insert(EquipmentId("robe1".into()), def_a);

        let effects = compute_equipment_effects(&loadout, &defs);
        assert_eq!(effects.total_mana_bonus, 5);
    }

    #[test]
    fn test_hit_count_bonus() {
        let mut loadout = EquipmentLoadout::default();
        let mut def = make_def("multi_blade", EquipmentSlot::Weapon, vec![Element::Venus]);
        def.hit_count_bonus = Some(2);

        loadout.weapon = Some(EquipmentId("multi_blade".into()));

        let mut defs = HashMap::new();
        defs.insert(EquipmentId("multi_blade".into()), def);

        let effects = compute_equipment_effects(&loadout, &defs);
        assert_eq!(effects.total_hit_count_bonus, 2);
    }

    #[test]
    fn test_always_first_turn_detection() {
        let mut loadout = EquipmentLoadout::default();
        let mut def = make_def("speed_boots", EquipmentSlot::Boots, vec![Element::Jupiter]);
        def.always_first_turn = true;

        loadout.boots = Some(EquipmentId("speed_boots".into()));

        let mut defs = HashMap::new();
        defs.insert(EquipmentId("speed_boots".into()), def);

        let effects = compute_equipment_effects(&loadout, &defs);
        assert!(effects.always_first_turn);
    }

    #[test]
    fn test_set_bonus_two_items_same_set() {
        let mut loadout = EquipmentLoadout::default();
        let mut def_w = make_def("set_sword", EquipmentSlot::Weapon, vec![Element::Mars]);
        def_w.set_id = Some(SetId("fire_set".into()));
        let mut def_h = make_def("set_helm", EquipmentSlot::Helm, vec![Element::Mars]);
        def_h.set_id = Some(SetId("fire_set".into()));

        loadout.weapon = Some(EquipmentId("set_sword".into()));
        loadout.helm = Some(EquipmentId("set_helm".into()));

        let mut defs = HashMap::new();
        defs.insert(EquipmentId("set_sword".into()), def_w);
        defs.insert(EquipmentId("set_helm".into()), def_h);

        let effects = compute_equipment_effects(&loadout, &defs);
        assert_eq!(effects.active_set_bonuses.len(), 1);
        assert_eq!(effects.active_set_bonuses[0], SetId("fire_set".into()));
    }

    #[test]
    fn test_empty_loadout_returns_zero_effects() {
        let loadout = EquipmentLoadout::default();
        let defs: HashMap<EquipmentId, EquipmentDef> = HashMap::new();
        let effects = compute_equipment_effects(&loadout, &defs);

        assert_eq!(effects.total_stat_bonus.atk.get(), 0);
        assert_eq!(effects.total_stat_bonus.def.get(), 0);
        assert_eq!(effects.total_stat_bonus.mag.get(), 0);
        assert_eq!(effects.total_stat_bonus.spd.get(), 0);
        assert_eq!(effects.total_stat_bonus.hp.get(), 0);
        assert!(effects.unlocked_abilities.is_empty());
        assert_eq!(effects.total_mana_bonus, 0);
        assert_eq!(effects.total_hit_count_bonus, 0);
        assert!(!effects.always_first_turn);
        assert!(effects.active_set_bonuses.is_empty());
    }

    #[test]
    fn test_can_equip_allowed() {
        let def = make_def(
            "sword",
            EquipmentSlot::Weapon,
            vec![Element::Venus, Element::Mars],
        );
        assert!(can_equip(&def, Element::Venus));
        assert!(can_equip(&def, Element::Mars));
    }

    #[test]
    fn test_can_equip_not_allowed() {
        let def = make_def("sword", EquipmentSlot::Weapon, vec![Element::Venus]);
        assert!(!can_equip(&def, Element::Jupiter));
    }

    #[test]
    fn test_get_equipped_in_slot() {
        let mut loadout = EquipmentLoadout::default();
        assert!(get_equipped_in_slot(&loadout, EquipmentSlot::Weapon).is_none());
        loadout.weapon = Some(EquipmentId("sword1".into()));
        assert_eq!(
            get_equipped_in_slot(&loadout, EquipmentSlot::Weapon),
            Some(&EquipmentId("sword1".into()))
        );
    }
}
