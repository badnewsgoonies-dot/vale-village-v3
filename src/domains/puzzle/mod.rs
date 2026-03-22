#![allow(dead_code)]
//! Puzzle Domain — Wave 2
//!
//! Pure logic: no rendering, no animation, no player-input mapping.
//! Returns PuzzleResult to the caller for dispatch.

use crate::shared::{DialogueSideEffect, Direction, DjinnId, Element, PuzzleDef, PuzzleType};

// ── PuzzleState ──────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PuzzleState {
    Unsolved,
    InProgress,
    Solved,
}

// ── Per-type progress tracking ────────────────────────────────────────

#[derive(Debug, Clone)]
enum PuzzleProgress {
    /// Counts how many valid pushes have been made toward the goal.
    PushBlock { moves: u8 },
    /// Stateless: solved on the first correct element activation.
    ElementPillar,
    /// Stateless: solved on the first correct djinn use.
    DjinnPuzzle,
    /// Tracks the index of the next switch that must be toggled.
    SwitchSequence { next_switch: usize },
    /// Tracks how many correct slides have been made in the solution sequence.
    IceSlide { step: usize },
}

impl PuzzleProgress {
    fn for_type(puzzle_type: &PuzzleType) -> Self {
        match puzzle_type {
            PuzzleType::PushBlock => PuzzleProgress::PushBlock { moves: 0 },
            PuzzleType::ElementPillar(_) => PuzzleProgress::ElementPillar,
            PuzzleType::DjinnPuzzle(_) => PuzzleProgress::DjinnPuzzle,
            PuzzleType::SwitchSequence => PuzzleProgress::SwitchSequence { next_switch: 0 },
            PuzzleType::IceSlide => PuzzleProgress::IceSlide { step: 0 },
        }
    }
}

// ── PuzzleInstance ───────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct PuzzleInstance {
    pub def: PuzzleDef,
    pub state: PuzzleState,
    progress: PuzzleProgress,
}

impl PuzzleInstance {
    pub fn new(def: PuzzleDef) -> Self {
        let progress = PuzzleProgress::for_type(&def.puzzle_type);
        Self {
            def,
            state: PuzzleState::Unsolved,
            progress,
        }
    }
}

// ── PuzzleInput ───────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PuzzleInput {
    PushDirection(Direction),
    ActivateElement(Element),
    UseDjinn(DjinnId),
    ToggleSwitch(usize),
    SlideDirection(Direction),
}

// ── PuzzleResult ──────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub enum PuzzleResult {
    Solved(Option<DialogueSideEffect>),
    Progress,
    Failed,
    AlreadySolved,
}

// DialogueSideEffect does not implement PartialEq, so we compare via Debug.
impl PartialEq for PuzzleResult {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (PuzzleResult::Solved(a), PuzzleResult::Solved(b)) => {
                format!("{a:?}") == format!("{b:?}")
            }
            (PuzzleResult::Progress, PuzzleResult::Progress) => true,
            (PuzzleResult::Failed, PuzzleResult::Failed) => true,
            (PuzzleResult::AlreadySolved, PuzzleResult::AlreadySolved) => true,
            _ => false,
        }
    }
}

// ── PuzzleContext trait ───────────────────────────────────────────────

pub trait PuzzleContext {
    fn has_djinn(&self, id: &DjinnId) -> bool;
    fn has_element_ability(&self, element: Element) -> bool;
}

// ── Puzzle solve constants ────────────────────────────────────────────

/// Number of pushes required to solve a PushBlock puzzle.
const PUSH_BLOCK_REQUIRED: u8 = 3;

/// The required push direction sequence for PushBlock puzzles.
const PUSH_BLOCK_SOLUTION: [Direction; 3] = [
    Direction::Up,
    Direction::Left,
    Direction::Down,
];

/// Number of switches that must be toggled in sequence to solve SwitchSequence.
const SWITCH_COUNT: usize = 3;

/// The exact slide sequence required to solve IceSlide.
const ICE_SLIDE_SOLUTION: [Direction; 4] = [
    Direction::Up,
    Direction::Right,
    Direction::Down,
    Direction::Left,
];

// ── can_attempt ───────────────────────────────────────────────────────

/// Returns `true` if the player meets the prerequisites to attempt this puzzle.
pub fn can_attempt(puzzle: &PuzzleDef, context: &dyn PuzzleContext) -> bool {
    match &puzzle.puzzle_type {
        PuzzleType::ElementPillar(elem) => context.has_element_ability(*elem),
        PuzzleType::DjinnPuzzle(id) => context.has_djinn(id),
        _ => true,
    }
}

// ── attempt_solve ─────────────────────────────────────────────────────

/// Advance a puzzle by one input step. The puzzle instance is mutated in place.
pub fn attempt_solve(instance: &mut PuzzleInstance, input: PuzzleInput) -> PuzzleResult {
    if instance.state == PuzzleState::Solved {
        return PuzzleResult::AlreadySolved;
    }

    instance.state = PuzzleState::InProgress;

    // Clone the type so we can borrow `instance.progress` mutably below.
    let puzzle_type = instance.def.puzzle_type.clone();

    match (puzzle_type, input) {
        (PuzzleType::PushBlock, PuzzleInput::PushDirection(dir)) => {
            if let PuzzleProgress::PushBlock { moves } = &mut instance.progress {
                let expected = PUSH_BLOCK_SOLUTION.get(*moves as usize);
                if expected == Some(&dir) {
                    *moves += 1;
                    if *moves >= PUSH_BLOCK_REQUIRED {
                        instance.state = PuzzleState::Solved;
                        return PuzzleResult::Solved(instance.def.reward.clone());
                    }
                    PuzzleResult::Progress
                } else {
                    // Wrong direction — reset
                    *moves = 0;
                    instance.state = PuzzleState::Unsolved;
                    PuzzleResult::Failed
                }
            } else {
                PuzzleResult::Failed
            }
        }

        (PuzzleType::ElementPillar(required), PuzzleInput::ActivateElement(elem)) => {
            if elem == required {
                instance.state = PuzzleState::Solved;
                PuzzleResult::Solved(instance.def.reward.clone())
            } else {
                instance.state = PuzzleState::Unsolved;
                PuzzleResult::Failed
            }
        }

        (PuzzleType::DjinnPuzzle(required_id), PuzzleInput::UseDjinn(used_id)) => {
            if used_id == required_id {
                instance.state = PuzzleState::Solved;
                PuzzleResult::Solved(instance.def.reward.clone())
            } else {
                instance.state = PuzzleState::Unsolved;
                PuzzleResult::Failed
            }
        }

        (PuzzleType::SwitchSequence, PuzzleInput::ToggleSwitch(idx)) => {
            if let PuzzleProgress::SwitchSequence { next_switch } = &mut instance.progress {
                if idx == *next_switch {
                    *next_switch += 1;
                    if *next_switch >= SWITCH_COUNT {
                        instance.state = PuzzleState::Solved;
                        return PuzzleResult::Solved(instance.def.reward.clone());
                    }
                    PuzzleResult::Progress
                } else {
                    // Wrong switch — reset progress
                    *next_switch = 0;
                    instance.state = PuzzleState::Unsolved;
                    PuzzleResult::Failed
                }
            } else {
                PuzzleResult::Failed
            }
        }

        (PuzzleType::IceSlide, PuzzleInput::SlideDirection(dir)) => {
            if let PuzzleProgress::IceSlide { step } = &mut instance.progress {
                if dir == ICE_SLIDE_SOLUTION[*step] {
                    *step += 1;
                    if *step >= ICE_SLIDE_SOLUTION.len() {
                        instance.state = PuzzleState::Solved;
                        return PuzzleResult::Solved(instance.def.reward.clone());
                    }
                    PuzzleResult::Progress
                } else {
                    // Wrong direction — reset progress
                    *step = 0;
                    instance.state = PuzzleState::Unsolved;
                    PuzzleResult::Failed
                }
            } else {
                PuzzleResult::Failed
            }
        }

        // Input type doesn't match puzzle type
        _ => PuzzleResult::Failed,
    }
}

// ── Tests ─────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    struct MockCtx {
        djinn: Option<DjinnId>,
        elements: Vec<Element>,
    }

    impl PuzzleContext for MockCtx {
        fn has_djinn(&self, id: &DjinnId) -> bool {
            self.djinn.as_ref() == Some(id)
        }
        fn has_element_ability(&self, element: Element) -> bool {
            self.elements.contains(&element)
        }
    }

    fn push_block_def() -> PuzzleDef {
        PuzzleDef { puzzle_type: PuzzleType::PushBlock, reward: None }
    }

    fn element_pillar_def(elem: Element) -> PuzzleDef {
        PuzzleDef { puzzle_type: PuzzleType::ElementPillar(elem), reward: None }
    }

    fn djinn_def(id: &str) -> PuzzleDef {
        PuzzleDef { puzzle_type: PuzzleType::DjinnPuzzle(DjinnId(id.to_string())), reward: None }
    }

    fn switch_def() -> PuzzleDef {
        PuzzleDef { puzzle_type: PuzzleType::SwitchSequence, reward: None }
    }

    fn ice_slide_def() -> PuzzleDef {
        PuzzleDef { puzzle_type: PuzzleType::IceSlide, reward: None }
    }

    // ── PushBlock ──────────────────────────────────────────────────────

    #[test]
    fn push_block_solve_path() {
        let mut inst = PuzzleInstance::new(push_block_def());
        assert_eq!(attempt_solve(&mut inst, PuzzleInput::PushDirection(Direction::Up)), PuzzleResult::Progress);
        assert_eq!(attempt_solve(&mut inst, PuzzleInput::PushDirection(Direction::Left)), PuzzleResult::Progress);
        let result = attempt_solve(&mut inst, PuzzleInput::PushDirection(Direction::Down));
        assert_eq!(result, PuzzleResult::Solved(None));
        assert_eq!(inst.state, PuzzleState::Solved);
    }

    #[test]
    fn push_block_wrong_direction_resets() {
        let mut inst = PuzzleInstance::new(push_block_def());
        // First push correct
        assert_eq!(attempt_solve(&mut inst, PuzzleInput::PushDirection(Direction::Up)), PuzzleResult::Progress);
        // Wrong direction — should reset
        assert_eq!(attempt_solve(&mut inst, PuzzleInput::PushDirection(Direction::Right)), PuzzleResult::Failed);
        assert_eq!(inst.state, PuzzleState::Unsolved);
        // Must restart the full sequence
        assert_eq!(attempt_solve(&mut inst, PuzzleInput::PushDirection(Direction::Up)), PuzzleResult::Progress);
        assert_eq!(attempt_solve(&mut inst, PuzzleInput::PushDirection(Direction::Left)), PuzzleResult::Progress);
        assert_eq!(attempt_solve(&mut inst, PuzzleInput::PushDirection(Direction::Down)), PuzzleResult::Solved(None));
    }

    // ── ElementPillar ──────────────────────────────────────────────────

    #[test]
    fn element_pillar_correct_element() {
        let mut inst = PuzzleInstance::new(element_pillar_def(Element::Venus));
        let result = attempt_solve(&mut inst, PuzzleInput::ActivateElement(Element::Venus));
        assert_eq!(result, PuzzleResult::Solved(None));
    }

    #[test]
    fn element_pillar_wrong_element_fails() {
        let mut inst = PuzzleInstance::new(element_pillar_def(Element::Venus));
        let result = attempt_solve(&mut inst, PuzzleInput::ActivateElement(Element::Mars));
        assert_eq!(result, PuzzleResult::Failed);
        assert_eq!(inst.state, PuzzleState::Unsolved);
    }

    // ── DjinnPuzzle ───────────────────────────────────────────────────

    #[test]
    fn djinn_puzzle_correct_djinn() {
        let mut inst = PuzzleInstance::new(djinn_def("flint"));
        let result = attempt_solve(&mut inst, PuzzleInput::UseDjinn(DjinnId("flint".to_string())));
        assert_eq!(result, PuzzleResult::Solved(None));
    }

    #[test]
    fn djinn_puzzle_wrong_djinn_fails() {
        let mut inst = PuzzleInstance::new(djinn_def("flint"));
        let result = attempt_solve(&mut inst, PuzzleInput::UseDjinn(DjinnId("forge".to_string())));
        assert_eq!(result, PuzzleResult::Failed);
    }

    // ── SwitchSequence ────────────────────────────────────────────────

    #[test]
    fn switch_sequence_solve_path() {
        let mut inst = PuzzleInstance::new(switch_def());
        assert_eq!(attempt_solve(&mut inst, PuzzleInput::ToggleSwitch(0)), PuzzleResult::Progress);
        assert_eq!(attempt_solve(&mut inst, PuzzleInput::ToggleSwitch(1)), PuzzleResult::Progress);
        let result = attempt_solve(&mut inst, PuzzleInput::ToggleSwitch(2));
        assert_eq!(result, PuzzleResult::Solved(None));
    }

    #[test]
    fn switch_sequence_wrong_order_resets() {
        let mut inst = PuzzleInstance::new(switch_def());
        assert_eq!(attempt_solve(&mut inst, PuzzleInput::ToggleSwitch(0)), PuzzleResult::Progress);
        // Toggle switch 2 instead of 1 — should fail and reset
        assert_eq!(attempt_solve(&mut inst, PuzzleInput::ToggleSwitch(2)), PuzzleResult::Failed);
        // Should be able to restart from 0
        assert_eq!(attempt_solve(&mut inst, PuzzleInput::ToggleSwitch(0)), PuzzleResult::Progress);
    }

    // ── IceSlide ──────────────────────────────────────────────────────

    #[test]
    fn ice_slide_solve_path() {
        let mut inst = PuzzleInstance::new(ice_slide_def());
        assert_eq!(attempt_solve(&mut inst, PuzzleInput::SlideDirection(Direction::Up)), PuzzleResult::Progress);
        assert_eq!(attempt_solve(&mut inst, PuzzleInput::SlideDirection(Direction::Right)), PuzzleResult::Progress);
        assert_eq!(attempt_solve(&mut inst, PuzzleInput::SlideDirection(Direction::Down)), PuzzleResult::Progress);
        let result = attempt_solve(&mut inst, PuzzleInput::SlideDirection(Direction::Left));
        assert_eq!(result, PuzzleResult::Solved(None));
    }

    // ── Already-solved rejection ──────────────────────────────────────

    #[test]
    fn already_solved_returns_already_solved() {
        let mut inst = PuzzleInstance::new(push_block_def());
        // Solve it first with correct sequence
        attempt_solve(&mut inst, PuzzleInput::PushDirection(Direction::Up));
        attempt_solve(&mut inst, PuzzleInput::PushDirection(Direction::Left));
        attempt_solve(&mut inst, PuzzleInput::PushDirection(Direction::Down));
        assert_eq!(inst.state, PuzzleState::Solved);
        // Second attempt must be rejected
        let result = attempt_solve(&mut inst, PuzzleInput::PushDirection(Direction::Up));
        assert_eq!(result, PuzzleResult::AlreadySolved);
    }

    // ── Reward delivery ───────────────────────────────────────────────

    #[test]
    fn reward_is_delivered_on_solve() {
        use crate::shared::bounded_types::Gold;
        let reward = DialogueSideEffect::GiveGold(Gold::new(100));
        let def = PuzzleDef {
            puzzle_type: PuzzleType::ElementPillar(Element::Jupiter),
            reward: Some(reward.clone()),
        };
        let mut inst = PuzzleInstance::new(def);
        let result = attempt_solve(&mut inst, PuzzleInput::ActivateElement(Element::Jupiter));
        assert_eq!(result, PuzzleResult::Solved(Some(reward)));
    }

    // ── can_attempt prerequisites ─────────────────────────────────────

    #[test]
    fn can_attempt_requires_element_ability() {
        let def = element_pillar_def(Element::Mars);
        let ctx_no = MockCtx { djinn: None, elements: vec![] };
        let ctx_yes = MockCtx { djinn: None, elements: vec![Element::Mars] };
        assert!(!can_attempt(&def, &ctx_no));
        assert!(can_attempt(&def, &ctx_yes));
    }

    #[test]
    fn can_attempt_requires_djinn() {
        let def = djinn_def("zephyr");
        let ctx_no = MockCtx { djinn: None, elements: vec![] };
        let ctx_yes = MockCtx { djinn: Some(DjinnId("zephyr".to_string())), elements: vec![] };
        assert!(!can_attempt(&def, &ctx_no));
        assert!(can_attempt(&def, &ctx_yes));
    }
}
