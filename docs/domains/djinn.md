# Domain: Djinn System

## Scope
`src/domains/djinn/`

## Purpose
Djinn state machine (Good/Recovery), ability oscillation, summon execution, staggered recovery. The system that makes the game unique.

## Required imports (from crate::shared)
DjinnDef, DjinnId, DjinnState, DjinnCompatibility, DjinnAbilityPairs, DjinnAbilitySet, AbilityId, Element, StatBonus, SummonEffect, TargetRef, DjinnStateChanged

## Structs

### DjinnInstance (runtime state of one equipped djinn)
- djinn_id: DjinnId
- state: DjinnState (Good or Recovery)
- recovery_turns_remaining: u8
- activation_order: u32 (for staggered recovery sequencing)

### DjinnSlots (per-unit djinn equipment, max 3)
- slots: Vec<DjinnInstance> (max 3)
- next_activation_order: u32

### DjinnManager (team-wide)
- Tracks all djinn across all units
- Computes available summon tiers based on Good djinn count
- Manages staggered recovery queue

## Functions

### determine_compatibility(djinn_element, unit_element) -> DjinnCompatibility
- Same: djinn_element == unit_element
- Counter: djinn_element == unit_element.counter()
- Neutral: neither same nor counter

### get_granted_abilities(djinn_def, compatibility, state) -> Vec<AbilityId>
- Same/Counter + Good: return good_abilities from the matching set
- Same/Counter + Recovery: return recovery_abilities
- Neutral: return good_abilities (always-on, same in both states)

### activate_djinn(slots, djinn_index) -> Option<DjinnStateChanged>
- Djinn must be in Good state
- Fires immediate effect (returned for caller to apply)
- Transitions to Recovery state
- Records activation_order for recovery sequencing

### compute_djinn_stat_bonus(slots, djinn_defs) -> StatBonus
- Sum stat bonuses from all Good-state djinn only
- Recovery djinn contribute zero

### get_available_summons(all_unit_slots) -> Vec<SummonTier>
- Count total Good djinn across team
- Tier 1: needs 1 Good djinn, Tier 2: needs 2, Tier 3: needs 3

### execute_summon(slots, djinn_indices) -> SummonResult
- All specified djinn must be Good
- All transition to Recovery
- Returns combined summon effect
- Summons execute before SPD ordering (caller responsibility)

### tick_recovery(all_unit_slots)
- Each tick: 1 djinn recovers (the one with lowest activation_order)
- Recovery starts the turn AFTER the turn after activation (1 turn delay)
- Recovered djinn returns to Good state, stat bonus returns, abilities swap back

## Quantitative targets
- 3 structs (DjinnInstance, DjinnSlots, SummonTier)
- 7+ functions
- 15+ tests

## Tests
- determine_compatibility: same, counter, neutral for all 4 elements
- get_granted_abilities: same+good, same+recovery, counter+good, counter+recovery, neutral
- activate_djinn: success, already-in-recovery fails
- stat bonus: only Good djinn contribute
- summon availability: 0/1/2/3 Good djinn
- execute_summon: transitions all to Recovery
- tick_recovery: staggered order, 1 per turn, delay works
- ability oscillation: abilities change when state changes

## Validation
```
cargo check
cargo test -- djinn
```
