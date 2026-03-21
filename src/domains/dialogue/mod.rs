#![allow(dead_code)]
//! Dialogue Domain — Wave 2
//!
//! Pure tree traversal: no rendering, no audio, no side-effect execution.
//! Returns side effects to the caller for dispatch.

use crate::shared::{
    DialogueCondition, DialogueNode, DialogueNodeId, DialogueResponse, DialogueSideEffect,
    DialogueTree, DjinnId, ItemId, QuestFlagId, QuestStage, UnitId,
};
use crate::shared::bounded_types::Gold;

// ── ConditionContext trait ───────────────────────────────────────────

/// Implemented by the game state layer so the dialogue runner can evaluate
/// conditions without depending on any concrete game-state type.
pub trait ConditionContext {
    fn has_item(&self, item: &ItemId) -> bool;
    fn has_djinn(&self, djinn: &DjinnId) -> bool;
    fn quest_at_stage(&self, flag: &QuestFlagId, stage: QuestStage) -> bool;
    fn gold_at_least(&self, amount: Gold) -> bool;
    fn party_contains(&self, unit: &UnitId) -> bool;
}

// ── Condition evaluation ─────────────────────────────────────────────

pub fn evaluate_condition(cond: &DialogueCondition, ctx: &dyn ConditionContext) -> bool {
    match cond {
        DialogueCondition::HasItem(id) => ctx.has_item(id),
        DialogueCondition::HasDjinn(id) => ctx.has_djinn(id),
        DialogueCondition::QuestAtStage(flag, stage) => ctx.quest_at_stage(flag, *stage),
        DialogueCondition::GoldAtLeast(amount) => ctx.gold_at_least(*amount),
        DialogueCondition::PartyContains(unit) => ctx.party_contains(unit),
    }
}

// ── DialogueRunner ───────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct DialogueRunner {
    /// The node currently being displayed.
    pub current_node: DialogueNodeId,
    /// Nodes visited in order (including current).
    pub history: Vec<DialogueNodeId>,
    /// When `None` the conversation is over.
    next_node: Option<DialogueNodeId>,
}

pub fn start_dialogue(tree: &DialogueTree) -> DialogueRunner {
    DialogueRunner {
        current_node: tree.root,
        history: vec![tree.root],
        next_node: Some(tree.root),
    }
}

pub fn get_current_node<'t>(runner: &DialogueRunner, tree: &'t DialogueTree) -> &'t DialogueNode {
    tree.nodes
        .iter()
        .find(|n| n.id == runner.current_node)
        .expect("current_node must exist in tree")
}

/// Returns `(response_index, &response)` pairs whose condition (if any) passes.
pub fn get_available_responses<'t>(
    runner: &DialogueRunner,
    tree: &'t DialogueTree,
    ctx: &dyn ConditionContext,
) -> Vec<(usize, &'t DialogueResponse)> {
    let node = get_current_node(runner, tree);
    node.responses
        .iter()
        .enumerate()
        .filter(|(_, r)| {
            r.condition
                .as_ref()
                .map(|c| evaluate_condition(c, ctx))
                .unwrap_or(true)
        })
        .collect()
}

/// Advance the runner by choosing a response (by its index in `node.responses`).
///
/// Returns `(next_node_id, side_effects)`.  The runner's `current_node` is
/// updated and `next_node` is set to `None` when the chosen response leads
/// nowhere (end of conversation).
pub fn choose_response(
    runner: &mut DialogueRunner,
    tree: &DialogueTree,
    response_index: usize,
) -> (Option<DialogueNodeId>, Vec<DialogueSideEffect>) {
    let node = get_current_node(runner, tree);
    let response = &node.responses[response_index];
    let side_effects = response.side_effects.clone();
    let next = response.next_node;

    runner.next_node = next;
    if let Some(id) = next {
        runner.current_node = id;
        runner.history.push(id);
    }

    (next, side_effects)
}

/// `true` once `choose_response` has been called and the last response had
/// `next_node: None`.
pub fn is_finished(runner: &DialogueRunner) -> bool {
    runner.next_node.is_none()
}

// ── Tests ────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::shared::{
        bounded_types::{Gold, ItemCount},
        Difficulty, DialogueCondition, DialogueNode, DialogueNodeId, DialogueResponse, DialogueSideEffect,
        DialogueTree, DialogueTreeId, DjinnId, EncounterDef, EncounterId, ItemId, MapNodeId,
        QuestFlagId, QuestStage, UnitId,
    };

    // ── Minimal ConditionContext for tests ───────────────────────────

    struct TestCtx {
        items: Vec<ItemId>,
        djinn: Vec<DjinnId>,
        gold: u32,
        quests: Vec<(QuestFlagId, QuestStage)>,
        party: Vec<UnitId>,
    }

    impl TestCtx {
        fn empty() -> Self {
            TestCtx {
                items: vec![],
                djinn: vec![],
                gold: 0,
                quests: vec![],
                party: vec![],
            }
        }
    }

    impl ConditionContext for TestCtx {
        fn has_item(&self, item: &ItemId) -> bool {
            self.items.contains(item)
        }
        fn has_djinn(&self, djinn: &DjinnId) -> bool {
            self.djinn.contains(djinn)
        }
        fn quest_at_stage(&self, flag: &QuestFlagId, stage: QuestStage) -> bool {
            self.quests
                .iter()
                .any(|(f, s)| f == flag && *s >= stage)
        }
        fn gold_at_least(&self, amount: Gold) -> bool {
            self.gold >= amount.get()
        }
        fn party_contains(&self, unit: &UnitId) -> bool {
            self.party.contains(unit)
        }
    }

    // ── Tree helpers ─────────────────────────────────────────────────

    fn node(id: u16, responses: Vec<DialogueResponse>) -> DialogueNode {
        DialogueNode {
            id: DialogueNodeId(id),
            speaker: None,
            text: format!("Node {id}"),
            responses,
        }
    }

    fn response(next: Option<u16>, effects: Vec<DialogueSideEffect>) -> DialogueResponse {
        DialogueResponse {
            text: "Continue".into(),
            condition: None,
            next_node: next.map(DialogueNodeId),
            side_effects: effects,
        }
    }

    fn cond_response(
        cond: DialogueCondition,
        next: Option<u16>,
    ) -> DialogueResponse {
        DialogueResponse {
            text: "Conditional".into(),
            condition: Some(cond),
            next_node: next.map(DialogueNodeId),
            side_effects: vec![],
        }
    }

    fn linear_tree() -> DialogueTree {
        // 0 -> 1 -> 2 -> end
        DialogueTree {
            id: DialogueTreeId(1),
            root: DialogueNodeId(0),
            nodes: vec![
                node(0, vec![response(Some(1), vec![])]),
                node(1, vec![response(Some(2), vec![])]),
                node(2, vec![response(None, vec![])]),
            ],
        }
    }

    // ── Test 1: start positions at root ──────────────────────────────

    #[test]
    fn test_start_positions_at_root() {
        let tree = linear_tree();
        let runner = start_dialogue(&tree);
        assert_eq!(runner.current_node, DialogueNodeId(0));
        assert!(!is_finished(&runner));
    }

    // ── Test 2: get_current_node returns root node ───────────────────

    #[test]
    fn test_get_current_node_root() {
        let tree = linear_tree();
        let runner = start_dialogue(&tree);
        let n = get_current_node(&runner, &tree);
        assert_eq!(n.id, DialogueNodeId(0));
    }

    // ── Test 3: linear traversal advances correctly ──────────────────

    #[test]
    fn test_linear_traversal() {
        let tree = linear_tree();
        let mut runner = start_dialogue(&tree);
        let ctx = TestCtx::empty();

        let (next, _) = choose_response(&mut runner, &tree, 0);
        assert_eq!(next, Some(DialogueNodeId(1)));
        assert_eq!(runner.current_node, DialogueNodeId(1));
        assert!(!is_finished(&runner));

        let (next, _) = choose_response(&mut runner, &tree, 0);
        assert_eq!(next, Some(DialogueNodeId(2)));
        assert_eq!(runner.current_node, DialogueNodeId(2));

        let _ = ctx; // used above indirectly
    }

    // ── Test 4: finished detection ───────────────────────────────────

    #[test]
    fn test_finished_detection() {
        let tree = linear_tree();
        let mut runner = start_dialogue(&tree);
        choose_response(&mut runner, &tree, 0); // -> 1
        choose_response(&mut runner, &tree, 0); // -> 2
        assert!(!is_finished(&runner));
        choose_response(&mut runner, &tree, 0); // -> None
        assert!(is_finished(&runner));
    }

    // ── Test 5: history is recorded ──────────────────────────────────

    #[test]
    fn test_history_recorded() {
        let tree = linear_tree();
        let mut runner = start_dialogue(&tree);
        choose_response(&mut runner, &tree, 0);
        choose_response(&mut runner, &tree, 0);
        assert_eq!(
            runner.history,
            vec![DialogueNodeId(0), DialogueNodeId(1), DialogueNodeId(2)]
        );
    }

    // ── Test 6: condition gates responses (HasItem) ──────────────────

    #[test]
    fn test_condition_has_item_gates_response() {
        let sword = ItemId("sword".into());
        let tree = DialogueTree {
            id: DialogueTreeId(2),
            root: DialogueNodeId(0),
            nodes: vec![node(
                0,
                vec![
                    response(None, vec![]),
                    cond_response(
                        DialogueCondition::HasItem(sword.clone()),
                        None,
                    ),
                ],
            )],
        };
        let mut runner = start_dialogue(&tree);
        let ctx_no_item = TestCtx::empty();
        let available = get_available_responses(&runner, &tree, &ctx_no_item);
        assert_eq!(available.len(), 1); // only the unconditioned response

        let ctx_with_item = TestCtx {
            items: vec![sword],
            ..TestCtx::empty()
        };
        let available2 = get_available_responses(&mut runner, &tree, &ctx_with_item);
        assert_eq!(available2.len(), 2);
    }

    // ── Test 7: condition HasDjinn ────────────────────────────────────

    #[test]
    fn test_condition_has_djinn() {
        let djinn_id = DjinnId("Flint".into());
        let cond = DialogueCondition::HasDjinn(djinn_id.clone());
        let ctx_no = TestCtx::empty();
        assert!(!evaluate_condition(&cond, &ctx_no));

        let ctx_yes = TestCtx {
            djinn: vec![djinn_id],
            ..TestCtx::empty()
        };
        assert!(evaluate_condition(&cond, &ctx_yes));
    }

    // ── Test 8: condition QuestAtStage ───────────────────────────────

    #[test]
    fn test_condition_quest_at_stage() {
        let flag = QuestFlagId(1);
        let cond = DialogueCondition::QuestAtStage(flag, QuestStage::Active);

        let ctx_no = TestCtx::empty();
        assert!(!evaluate_condition(&cond, &ctx_no));

        let ctx_yes = TestCtx {
            quests: vec![(flag, QuestStage::Active)],
            ..TestCtx::empty()
        };
        assert!(evaluate_condition(&cond, &ctx_yes));
    }

    // ── Test 9: condition GoldAtLeast ────────────────────────────────

    #[test]
    fn test_condition_gold_at_least() {
        let cond = DialogueCondition::GoldAtLeast(Gold::new(100));
        let ctx_poor = TestCtx::empty();
        assert!(!evaluate_condition(&cond, &ctx_poor));

        let ctx_rich = TestCtx {
            gold: 200,
            ..TestCtx::empty()
        };
        assert!(evaluate_condition(&cond, &ctx_rich));
    }

    // ── Test 10: condition PartyContains ─────────────────────────────

    #[test]
    fn test_condition_party_contains() {
        let unit = UnitId("Isaac".into());
        let cond = DialogueCondition::PartyContains(unit.clone());
        let ctx_no = TestCtx::empty();
        assert!(!evaluate_condition(&cond, &ctx_no));

        let ctx_yes = TestCtx {
            party: vec![unit],
            ..TestCtx::empty()
        };
        assert!(evaluate_condition(&cond, &ctx_yes));
    }

    // ── Test 11: all 9 side effects are passed through ───────────────

    #[test]
    fn test_all_side_effects_passthrough() {
        let enc = EncounterDef {
            id: EncounterId("enc1".into()),
            name: "Test".into(),
            difficulty: Difficulty::Medium,
            enemies: vec![],
            xp_reward: crate::shared::bounded_types::Xp::new(0),
            gold_reward: Gold::new(0),
            recruit: None,
            djinn_reward: None,
            equipment_rewards: vec![],
        };
        let effects = vec![
            DialogueSideEffect::GiveItem(ItemId("herb".into()), ItemCount::new(1)),
            DialogueSideEffect::TakeItem(ItemId("herb".into()), ItemCount::new(1)),
            DialogueSideEffect::GiveGold(Gold::new(50)),
            DialogueSideEffect::TakeGold(Gold::new(10)),
            DialogueSideEffect::SetQuestStage(QuestFlagId(1), QuestStage::Complete),
            DialogueSideEffect::UnlockMapNode(MapNodeId(3)),
            DialogueSideEffect::AddDjinnToParty(DjinnId("Gust".into())),
            DialogueSideEffect::StartBattle(enc),
            DialogueSideEffect::Heal,
        ];
        assert_eq!(effects.len(), 9);

        let tree = DialogueTree {
            id: DialogueTreeId(3),
            root: DialogueNodeId(0),
            nodes: vec![node(0, vec![response(None, effects.clone())])],
        };
        let mut runner = start_dialogue(&tree);
        let (_, returned) = choose_response(&mut runner, &tree, 0);
        assert_eq!(returned.len(), 9);
    }

    // ── Test 12: branching — two paths diverge ────────────────────────

    #[test]
    fn test_branching_tree() {
        // node 0 has two responses: one leads to 1, the other to 2
        let tree = DialogueTree {
            id: DialogueTreeId(4),
            root: DialogueNodeId(0),
            nodes: vec![
                node(
                    0,
                    vec![
                        response(Some(1), vec![]),
                        response(Some(2), vec![]),
                    ],
                ),
                node(1, vec![response(None, vec![])]),
                node(2, vec![response(None, vec![])]),
            ],
        };

        // Take left branch
        let mut r1 = start_dialogue(&tree);
        choose_response(&mut r1, &tree, 0);
        assert_eq!(r1.current_node, DialogueNodeId(1));

        // Take right branch
        let mut r2 = start_dialogue(&tree);
        choose_response(&mut r2, &tree, 1);
        assert_eq!(r2.current_node, DialogueNodeId(2));
    }
}
