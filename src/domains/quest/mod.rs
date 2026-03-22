#![allow(dead_code)]
//! Quest Domain — QuestManager, stage queries, reward checking, availability.

use crate::shared::{
    DialogueCondition, DialogueSideEffect, QuestDef, QuestFlagId, QuestStage, QuestState,
};

// ── QuestManager ────────────────────────────────────────────────────

/// Owns quest runtime state and the catalogue of definitions.
#[derive(Debug, Clone, Default)]
pub struct QuestManager {
    pub state: QuestState,
    pub defs: Vec<QuestDef>,
}

impl QuestManager {
    pub fn new(defs: Vec<QuestDef>) -> Self {
        QuestManager {
            state: QuestState::default(),
            defs,
        }
    }

    /// Advance a quest monotonically. No-op if `to` is not strictly forward.
    pub fn advance(&mut self, quest_id: QuestFlagId, to: QuestStage) {
        self.state.advance(quest_id, to);
    }

    /// Current stage for a quest (Unknown if unseen).
    pub fn stage(&self, quest_id: QuestFlagId) -> QuestStage {
        self.state
            .flags
            .get(&quest_id)
            .copied()
            .unwrap_or(QuestStage::Unknown)
    }

    /// All defs whose quests are at least Active and not yet Rewarded.
    pub fn active_quests(&self) -> Vec<&QuestDef> {
        get_active_quests(&self.state, &self.defs)
    }

    /// All defs whose quests are Rewarded.
    pub fn completed_quests(&self) -> Vec<&QuestDef> {
        get_completed_quests(&self.state, &self.defs)
    }

    /// Returns rewards if the quest is Complete but not yet Rewarded.
    pub fn check_rewards(&self, def: &QuestDef) -> Option<Vec<DialogueSideEffect>> {
        check_rewards(&self.state, def)
    }

    /// Returns all defs that are currently available to the player.
    pub fn available_quests(&self) -> Vec<&QuestDef> {
        self.defs
            .iter()
            .filter(|d| is_quest_available(d, &self.state))
            .collect()
    }

    /// Mark rewards as claimed (advance to Rewarded).
    /// Returns false if quest is not at Complete stage.
    pub fn claim_rewards(&mut self, quest_id: QuestFlagId) -> bool {
        if self.stage(quest_id) != QuestStage::Complete {
            return false;
        }
        self.state.advance(quest_id, QuestStage::Rewarded);
        true
    }
}

// ── Free Functions ───────────────────────────────────────────────────

/// Advance quest `quest_id` to `stage` in `state` — monotonic only.
pub fn advance_quest(state: &mut QuestState, quest_id: QuestFlagId, stage: QuestStage) {
    state.advance(quest_id, stage);
}

/// Current stage for quest_id in state.
pub fn get_quest_stage(state: &QuestState, quest_id: QuestFlagId) -> QuestStage {
    state
        .flags
        .get(&quest_id)
        .copied()
        .unwrap_or(QuestStage::Unknown)
}

/// Defs whose quests are at least Active and not yet Rewarded.
pub fn get_active_quests<'a>(state: &QuestState, defs: &'a [QuestDef]) -> Vec<&'a QuestDef> {
    defs.iter()
        .filter(|d| {
            state.at_least(d.id, QuestStage::Active)
                && !state.at_least(d.id, QuestStage::Rewarded)
        })
        .collect()
}

/// Defs whose quests are Rewarded (fully completed).
pub fn get_completed_quests<'a>(state: &QuestState, defs: &'a [QuestDef]) -> Vec<&'a QuestDef> {
    defs.iter()
        .filter(|d| state.at_least(d.id, QuestStage::Rewarded))
        .collect()
}

/// If the quest is Complete but not yet Rewarded, returns the reward list.
pub fn check_rewards(state: &QuestState, def: &QuestDef) -> Option<Vec<DialogueSideEffect>> {
    let stage = get_quest_stage(state, def.id);
    if stage == QuestStage::Complete {
        Some(def.rewards.clone())
    } else {
        None
    }
}

/// True if the quest has not yet been seen AND its first stage's unlock condition (if any) passes.
pub fn is_quest_available(def: &QuestDef, state: &QuestState) -> bool {
    // Already known → not "available to discover"
    if state.at_least(def.id, QuestStage::Discovered) {
        return false;
    }
    // Check unlock condition on the first stage def, if present.
    let first = def.stages.first();
    match first {
        None => true,
        Some(stage_def) => match &stage_def.unlock_condition {
            None => true,
            Some(cond) => evaluate_condition(cond, state),
        },
    }
}

// ── Condition Evaluator ──────────────────────────────────────────────

fn evaluate_condition(cond: &DialogueCondition, state: &QuestState) -> bool {
    match cond {
        DialogueCondition::QuestAtStage(id, stage) => state.at_least(*id, *stage),
        // Other condition types cannot be evaluated without broader game state;
        // treat as satisfied so quest availability is not blocked.
        _ => true,
    }
}

// ── Tests ────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn make_def(id: u16) -> QuestDef {
        QuestDef {
            id: QuestFlagId(id),
            name: format!("Quest {id}"),
            description: String::from("A test quest."),
            stages: vec![
                crate::shared::QuestStageDef {
                    stage: QuestStage::Discovered,
                    description: String::from("Discovered"),
                    unlock_condition: None,
                },
                crate::shared::QuestStageDef {
                    stage: QuestStage::Active,
                    description: String::from("Active"),
                    unlock_condition: None,
                },
                crate::shared::QuestStageDef {
                    stage: QuestStage::InProgress,
                    description: String::from("InProgress"),
                    unlock_condition: None,
                },
                crate::shared::QuestStageDef {
                    stage: QuestStage::Complete,
                    description: String::from("Complete"),
                    unlock_condition: None,
                },
                crate::shared::QuestStageDef {
                    stage: QuestStage::Rewarded,
                    description: String::from("Rewarded"),
                    unlock_condition: None,
                },
            ],
            rewards: vec![DialogueSideEffect::GiveGold(crate::shared::bounded_types::Gold::new(100))],
        }
    }

    // T1: default stage is Unknown
    #[test]
    fn test_default_stage_unknown() {
        let mut mgr = QuestManager::new(vec![make_def(1)]);
        assert_eq!(mgr.stage(QuestFlagId(1)), QuestStage::Unknown);
        // not-tracked quest is also Unknown
        assert_eq!(mgr.stage(QuestFlagId(99)), QuestStage::Unknown);
        // advance and verify
        mgr.advance(QuestFlagId(1), QuestStage::Active);
        assert_eq!(mgr.stage(QuestFlagId(1)), QuestStage::Active);
    }

    // T2: advance is monotonic — cannot go backward
    #[test]
    fn test_advance_monotonic_no_backward() {
        let mut state = QuestState::default();
        let id = QuestFlagId(10);
        advance_quest(&mut state, id, QuestStage::Complete);
        assert_eq!(get_quest_stage(&state, id), QuestStage::Complete);

        // Try to go back to Active — must be no-op
        advance_quest(&mut state, id, QuestStage::Active);
        assert_eq!(get_quest_stage(&state, id), QuestStage::Complete);
    }

    // T3: advance from Complete to Active is no-op (spec example)
    #[test]
    fn test_advance_complete_to_active_noop() {
        let mut mgr = QuestManager::new(vec![make_def(2)]);
        mgr.advance(QuestFlagId(2), QuestStage::Complete);
        mgr.advance(QuestFlagId(2), QuestStage::Active);
        assert_eq!(mgr.stage(QuestFlagId(2)), QuestStage::Complete);
    }

    // T4: advance strictly forward works
    #[test]
    fn test_advance_forward_sequence() {
        let mut mgr = QuestManager::new(vec![make_def(3)]);
        let stages = [
            QuestStage::Discovered,
            QuestStage::Active,
            QuestStage::InProgress,
            QuestStage::Complete,
            QuestStage::Rewarded,
        ];
        for &s in &stages {
            mgr.advance(QuestFlagId(3), s);
            assert_eq!(mgr.stage(QuestFlagId(3)), s);
        }
    }

    // T5: active_quests returns Active/InProgress/Complete but not Unknown/Rewarded
    #[test]
    fn test_active_quests_filtering() {
        let defs = vec![make_def(1), make_def(2), make_def(3), make_def(4)];
        let mut mgr = QuestManager::new(defs);

        // 1 = Unknown (not active)
        mgr.advance(QuestFlagId(2), QuestStage::Active);
        mgr.advance(QuestFlagId(3), QuestStage::InProgress);
        mgr.advance(QuestFlagId(4), QuestStage::Rewarded); // completed, not active

        let active: Vec<u16> = mgr.active_quests().iter().map(|d| d.id.0).collect();
        assert!(active.contains(&2));
        assert!(active.contains(&3));
        assert!(!active.contains(&1));
        assert!(!active.contains(&4));
    }

    // T6: completed_quests returns only Rewarded
    #[test]
    fn test_completed_quests() {
        let defs = vec![make_def(5), make_def(6)];
        let mut mgr = QuestManager::new(defs);
        mgr.advance(QuestFlagId(5), QuestStage::Rewarded);

        let done: Vec<u16> = mgr.completed_quests().iter().map(|d| d.id.0).collect();
        assert_eq!(done, vec![5]);
    }

    // T7: check_rewards returns Some only at Complete stage
    #[test]
    fn test_check_rewards() {
        let def = make_def(7);
        let mut state = QuestState::default();

        assert!(check_rewards(&state, &def).is_none()); // Unknown

        advance_quest(&mut state, QuestFlagId(7), QuestStage::Active);
        assert!(check_rewards(&state, &def).is_none()); // Active — no rewards yet

        advance_quest(&mut state, QuestFlagId(7), QuestStage::Complete);
        let rewards = check_rewards(&state, &def);
        assert!(rewards.is_some());
        assert_eq!(rewards.unwrap().len(), 1);

        advance_quest(&mut state, QuestFlagId(7), QuestStage::Rewarded);
        assert!(check_rewards(&state, &def).is_none()); // Already rewarded
    }

    // T8: is_quest_available returns false once discovered
    #[test]
    fn test_availability_once_discovered() {
        let def = make_def(8);
        let mut state = QuestState::default();
        assert!(is_quest_available(&def, &state)); // not yet known

        advance_quest(&mut state, QuestFlagId(8), QuestStage::Discovered);
        assert!(!is_quest_available(&def, &state)); // already known
    }

    // T9: is_quest_available with a QuestAtStage unlock condition
    #[test]
    fn test_availability_with_condition() {
        use crate::shared::QuestStageDef;

        let prereq_id = QuestFlagId(100);
        let mut def = make_def(9);
        // First stage requires quest 100 to be at least Active
        def.stages[0].unlock_condition =
            Some(DialogueCondition::QuestAtStage(prereq_id, QuestStage::Active));

        let mut state = QuestState::default();
        // Prereq not met → unavailable
        assert!(!is_quest_available(&def, &state));

        // Satisfy prereq
        advance_quest(&mut state, prereq_id, QuestStage::Active);
        assert!(is_quest_available(&def, &state));
    }

    // T10: claim_rewards advances to Rewarded and check_rewards returns None after
    #[test]
    fn test_claim_rewards() {
        let defs = vec![make_def(10)];
        let mut mgr = QuestManager::new(defs);
        mgr.advance(QuestFlagId(10), QuestStage::Complete);
        let def = mgr.defs[0].clone();
        let rewards = mgr.check_rewards(&def);
        assert!(rewards.is_some());

        mgr.claim_rewards(QuestFlagId(10));
        assert!(mgr.check_rewards(&def).is_none());
        assert_eq!(mgr.stage(QuestFlagId(10)), QuestStage::Rewarded);
    }
}
