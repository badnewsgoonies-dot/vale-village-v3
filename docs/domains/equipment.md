# Domain: Equipment System (S19)

## Scope
`src/domains/equipment/`

## Purpose
Equipment loadout management, stat bonus computation, ability unlocking, set bonuses, speed overrides. 51% of equipment unlocks abilities.

## Required imports (from crate::shared)
EquipmentDef, EquipmentId, EquipmentSlot, AbilityId, Element, StatBonus, SetId

## Structs

### EquipmentLoadout (per-unit, 5 slots)
- weapon: Option<EquipmentId>
- helm: Option<EquipmentId>
- armor: Option<EquipmentId>
- boots: Option<EquipmentId>
- accessory: Option<EquipmentId>

### EquipmentEffects (computed from loadout)
- total_stat_bonus: StatBonus
- unlocked_abilities: Vec<AbilityId>
- total_mana_bonus: u8
- total_hit_count_bonus: u8
- always_first_turn: bool
- active_set_bonuses: Vec<SetId>

## Functions

### equip(loadout, slot, equipment_id, equipment_def, unit_element) -> Result<(), EquipError>
- Validate slot matches equipment_def.slot
- Validate unit_element is in equipment_def.allowed_elements
- Place in loadout

### unequip(loadout, slot) -> Option<EquipmentId>

### compute_equipment_effects(loadout, equipment_defs: &HashMap) -> EquipmentEffects
- Sum all stat_bonus from equipped items
- Collect all unlocked abilities
- Sum mana_bonus and hit_count_bonus
- Check always_first_turn flag
- Detect set bonuses (2+ items with same set_id)

### get_equipped_in_slot(loadout, slot) -> Option<EquipmentId>

### can_equip(equipment_def, unit_element) -> bool
- Check unit_element in allowed_elements

## Quantitative targets
- 2 structs (EquipmentLoadout, EquipmentEffects)
- 5+ functions
- 12+ tests

## Tests
- equip success
- equip wrong slot fails
- equip wrong element fails
- unequip returns item
- stat bonus computation (2 items)
- ability unlock from weapon
- mana bonus accumulation
- hit count bonus
- always_first_turn detection
- set bonus detection (2 items same set)
- empty loadout returns zero effects

## Validation
```
cargo check
cargo test -- equipment
```
