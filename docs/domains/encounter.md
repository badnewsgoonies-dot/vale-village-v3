# Domain: encounter
## Scope: src/domains/encounter/
## Imports from contract
EncounterTable, EncounterSlot, EncounterDef, BossEncounter, EncounterRate,
DialogueTreeId, QuestFlagId, QuestStage, MapNodeId

## Deliverables
1. fn should_encounter(table: &EncounterTable, steps_since_last: u16) -> bool — deterministic: steps >= base_rate threshold
2. fn select_encounter(table: &EncounterTable, step_count: u16) -> Option<&EncounterDef> — weighted selection using step_count as seed (deterministic, no RNG)
3. fn decrement_slot(table: &mut EncounterTable, encounter_idx: usize) — reduces max_triggers if Some
4. fn prepare_boss(boss: &BossEncounter) -> (Option<DialogueTreeId>, EncounterDef, Vec<MapNodeId>)
5. Tests: encounter rate threshold, weighted selection determinism, max_triggers depletion, boss preparation

## Does NOT handle
Running battles, dialogue, quest advancement. Returns encounter definitions for caller to execute.

## Quantitative targets
- Deterministic encounter selection (same inputs = same output)
- max_triggers enforcement
- ≥6 tests
