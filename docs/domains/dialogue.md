# Domain: dialogue
## Scope: src/domains/dialogue/
## Imports from contract
DialogueTree, DialogueNode, DialogueNodeId, DialogueResponse, DialogueCondition, DialogueSideEffect,
ItemId, DjinnId, QuestFlagId, QuestStage, Gold, ItemCount, UnitId, MapNodeId, EncounterDef

## Deliverables
1. DialogueRunner struct: current_tree, current_node, history
2. fn start_dialogue(tree: &DialogueTree) -> DialogueRunner
3. fn get_current_node(runner, tree) -> &DialogueNode
4. fn get_available_responses(runner, tree, context: &dyn ConditionContext) -> Vec<(usize, &DialogueResponse)>
5. fn choose_response(runner, index) -> (Option<DialogueNodeId>, Vec<DialogueSideEffect>)
6. fn is_finished(runner) -> bool (next_node is None)
7. ConditionContext trait: has_item, has_djinn, quest_at_stage, gold_at_least, party_contains
8. fn evaluate_condition(cond: &DialogueCondition, ctx: &dyn ConditionContext) -> bool
9. Tests: linear tree traversal, branching, conditions gate responses, side effects returned, finished detection

## Does NOT handle
Rendering text, playing audio, executing side effects (returns them). Pure tree traversal.

## Quantitative targets
- All 5 DialogueCondition variants evaluated
- All 9 DialogueSideEffect variants passed through
- ≥10 tests
