# Domain: quest
## Scope: src/domains/quest/
## Imports from contract
QuestState, QuestDef, QuestStageDef, QuestStage, QuestFlagId, DialogueCondition, DialogueSideEffect

## Deliverables
1. QuestManager struct wrapping QuestState + Vec<QuestDef>
2. fn advance_quest(state, quest_id, stage) — monotonic only (QuestState::advance exists in contract)
3. fn get_quest_stage(state, quest_id) -> QuestStage
4. fn get_active_quests(state, defs) -> Vec<&QuestDef>
5. fn get_completed_quests(state, defs) -> Vec<&QuestDef>
6. fn check_rewards(state, def) -> Option<Vec<DialogueSideEffect>> — if stage == Complete and rewards not claimed
7. fn is_quest_available(def, state) -> bool — checks unlock conditions on first stage
8. Tests: monotonic advance (can't go backward), stage queries, reward collection, available filtering

## Does NOT handle
Executing rewards (returns them), rendering quest log, dialogue integration.

## Quantitative targets
- 6 QuestStage variants: Unknown → Discovered → Active → InProgress → Complete → Rewarded
- Monotonic guarantee tested (advance from Complete to Active = no-op)
- ≥8 tests
