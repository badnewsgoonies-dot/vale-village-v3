#![allow(dead_code)]
//! Screens domain — pure state-machine logic for screen navigation.
//! No rendering, no input, no Bevy systems.

use crate::shared::{GameScreen, ScreenStack, ScreenTransition};

pub const MAX_STACK_DEPTH: usize = 8;

// ── ScreenManager ────────────────────────────────────────────────────

/// Owns the active screen and the history stack for navigation.
#[derive(Debug, Clone)]
pub struct ScreenManager {
    pub current: GameScreen,
    pub stack: ScreenStack,
}

impl Default for ScreenManager {
    fn default() -> Self {
        Self {
            current: GameScreen::default(),
            stack: ScreenStack::default(),
        }
    }
}

impl ScreenManager {
    pub fn new(initial: GameScreen) -> Self {
        Self {
            current: initial,
            stack: ScreenStack::default(),
        }
    }
}

// ── Stack operations ─────────────────────────────────────────────────

/// Push `screen` onto the stack. Returns `Err` if depth would exceed 8.
pub fn push_screen(stack: &mut ScreenStack, screen: GameScreen) -> Result<(), StackOverflowError> {
    if stack.stack.len() >= MAX_STACK_DEPTH {
        return Err(StackOverflowError { depth: stack.stack.len() });
    }
    stack.stack.push(screen);
    Ok(())
}

/// Pop the most-recent screen. Returns `None` when the stack is empty.
pub fn pop_screen(stack: &mut ScreenStack) -> Option<GameScreen> {
    stack.stack.pop()
}

/// Return a reference to the current active screen on the manager.
pub fn current_screen(manager: &ScreenManager) -> &GameScreen {
    &manager.current
}

// ── Transition logic ─────────────────────────────────────────────────

/// Apply a `ScreenTransition` to the manager, updating `current` and `stack`.
///
/// * Navigating *forward* pushes the old screen onto the stack.
/// * `ReturnToPrevious` pops the stack; if empty it stays on the current screen.
/// * Hard resets (`ToTitle`, `TriggerGameOver`, `TriggerVictory`) clear the stack.
pub fn apply_transition(
    manager: &mut ScreenManager,
    transition: ScreenTransition,
) -> GameScreen {
    match transition {
        ScreenTransition::ToTitle => {
            manager.stack.stack.clear();
            manager.current = GameScreen::Title;
        }
        ScreenTransition::ToWorldMap => {
            push_or_clamp(&mut manager.stack, manager.current);
            manager.current = GameScreen::WorldMap;
        }
        ScreenTransition::EnterTown(id) => {
            push_or_clamp(&mut manager.stack, manager.current);
            manager.current = GameScreen::Town(id);
        }
        ScreenTransition::EnterDungeon(id) => {
            push_or_clamp(&mut manager.stack, manager.current);
            manager.current = GameScreen::Dungeon(id);
        }
        ScreenTransition::StartBattle(_encounter) => {
            push_or_clamp(&mut manager.stack, manager.current);
            manager.current = GameScreen::Battle;
        }
        ScreenTransition::OpenMenu(menu) => {
            push_or_clamp(&mut manager.stack, manager.current);
            manager.current = GameScreen::Menu(menu);
        }
        ScreenTransition::OpenShop(id) => {
            push_or_clamp(&mut manager.stack, manager.current);
            manager.current = GameScreen::Shop(id);
        }
        ScreenTransition::StartDialogue(id) => {
            push_or_clamp(&mut manager.stack, manager.current);
            manager.current = GameScreen::Dialogue(id);
        }
        ScreenTransition::OpenSaveLoad => {
            push_or_clamp(&mut manager.stack, manager.current);
            manager.current = GameScreen::SaveLoad;
        }
        ScreenTransition::TriggerGameOver => {
            manager.stack.stack.clear();
            manager.current = GameScreen::GameOver;
        }
        ScreenTransition::TriggerVictory => {
            manager.stack.stack.clear();
            manager.current = GameScreen::Victory;
        }
        ScreenTransition::ReturnToPrevious => {
            if let Some(prev) = manager.stack.stack.pop() {
                manager.current = prev;
            }
            // If stack empty, stay on current screen.
        }
    }
    manager.current
}

/// Push onto the stack, silently dropping the oldest entry if already at max depth
/// so that forward navigation never hard-errors in gameplay code.
fn push_or_clamp(stack: &mut ScreenStack, screen: GameScreen) {
    if stack.stack.len() >= MAX_STACK_DEPTH {
        stack.stack.remove(0);
    }
    stack.stack.push(screen);
}

// ── Error type ───────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StackOverflowError {
    pub depth: usize,
}

impl std::fmt::Display for StackOverflowError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "ScreenStack overflow: depth {} already at maximum {}", self.depth, MAX_STACK_DEPTH)
    }
}

impl std::error::Error for StackOverflowError {}

// ── Tests ────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::shared::{
        bounded_types::{Gold, Xp},
        Difficulty, DjinnId, DungeonId, EncounterDef, EncounterEnemy, EncounterId,
        EquipmentId, MenuScreen, NpcId, ShopId, TownId, UnitId,
    };

    fn make_encounter() -> EncounterDef {
        EncounterDef {
            id: EncounterId("test-enc".into()),
            name: "Test".into(),
            difficulty: Difficulty::Easy,
            enemies: vec![],
            xp_reward: Xp::new(10),
            gold_reward: Gold::new(5),
            recruit: None,
            djinn_reward: None,
            equipment_rewards: vec![],
        }
    }

    // 1. Default manager starts on Title screen
    #[test]
    fn test_default_screen_is_title() {
        let mgr = ScreenManager::default();
        assert_eq!(*current_screen(&mgr), GameScreen::Title);
    }

    // 2. Push / pop roundtrip
    #[test]
    fn test_push_pop_roundtrip() {
        let mut stack = ScreenStack::default();
        push_screen(&mut stack, GameScreen::WorldMap).unwrap();
        push_screen(&mut stack, GameScreen::Battle).unwrap();
        assert_eq!(pop_screen(&mut stack), Some(GameScreen::Battle));
        assert_eq!(pop_screen(&mut stack), Some(GameScreen::WorldMap));
        assert_eq!(pop_screen(&mut stack), None);
    }

    // 3. Max depth enforcement via push_screen
    #[test]
    fn test_max_depth_enforcement() {
        let mut stack = ScreenStack::default();
        for i in 0..MAX_STACK_DEPTH {
            push_screen(&mut stack, GameScreen::Town(TownId(i as u8))).unwrap();
        }
        let result = push_screen(&mut stack, GameScreen::Battle);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().depth, MAX_STACK_DEPTH);
    }

    // 4. apply_transition — ToTitle clears stack
    #[test]
    fn test_to_title_clears_stack() {
        let mut mgr = ScreenManager::new(GameScreen::WorldMap);
        mgr.stack.stack.push(GameScreen::Title);
        let screen = apply_transition(&mut mgr, ScreenTransition::ToTitle);
        assert_eq!(screen, GameScreen::Title);
        assert!(mgr.stack.stack.is_empty());
    }

    // 5. apply_transition — ToWorldMap
    #[test]
    fn test_transition_to_world_map() {
        let mut mgr = ScreenManager::new(GameScreen::Title);
        let screen = apply_transition(&mut mgr, ScreenTransition::ToWorldMap);
        assert_eq!(screen, GameScreen::WorldMap);
        assert_eq!(mgr.stack.stack.last(), Some(&GameScreen::Title));
    }

    // 6. apply_transition — EnterTown / EnterDungeon
    #[test]
    fn test_transition_enter_town_and_dungeon() {
        let mut mgr = ScreenManager::new(GameScreen::WorldMap);
        apply_transition(&mut mgr, ScreenTransition::EnterTown(TownId(3)));
        assert_eq!(mgr.current, GameScreen::Town(TownId(3)));

        apply_transition(&mut mgr, ScreenTransition::EnterDungeon(DungeonId(7)));
        assert_eq!(mgr.current, GameScreen::Dungeon(DungeonId(7)));
    }

    // 7. apply_transition — StartBattle
    #[test]
    fn test_transition_start_battle() {
        let mut mgr = ScreenManager::new(GameScreen::WorldMap);
        let screen = apply_transition(&mut mgr, ScreenTransition::StartBattle(make_encounter()));
        assert_eq!(screen, GameScreen::Battle);
    }

    // 8. apply_transition — OpenMenu / OpenShop / StartDialogue / OpenSaveLoad
    #[test]
    fn test_transition_open_overlays() {
        let mut mgr = ScreenManager::new(GameScreen::WorldMap);

        apply_transition(&mut mgr, ScreenTransition::OpenMenu(MenuScreen::Items));
        assert_eq!(mgr.current, GameScreen::Menu(MenuScreen::Items));

        apply_transition(&mut mgr, ScreenTransition::OpenShop(ShopId(2)));
        assert_eq!(mgr.current, GameScreen::Shop(ShopId(2)));

        apply_transition(&mut mgr, ScreenTransition::StartDialogue(NpcId(9)));
        assert_eq!(mgr.current, GameScreen::Dialogue(NpcId(9)));

        apply_transition(&mut mgr, ScreenTransition::OpenSaveLoad);
        assert_eq!(mgr.current, GameScreen::SaveLoad);
    }

    // 9. apply_transition — TriggerGameOver and TriggerVictory clear stack
    #[test]
    fn test_transition_game_over_and_victory() {
        let mut mgr = ScreenManager::new(GameScreen::Battle);
        mgr.stack.stack.push(GameScreen::WorldMap);

        apply_transition(&mut mgr, ScreenTransition::TriggerGameOver);
        assert_eq!(mgr.current, GameScreen::GameOver);
        assert!(mgr.stack.stack.is_empty());

        mgr.current = GameScreen::Battle;
        mgr.stack.stack.push(GameScreen::WorldMap);
        apply_transition(&mut mgr, ScreenTransition::TriggerVictory);
        assert_eq!(mgr.current, GameScreen::Victory);
        assert!(mgr.stack.stack.is_empty());
    }

    // 10. apply_transition — ReturnToPrevious
    #[test]
    fn test_transition_return_to_previous() {
        let mut mgr = ScreenManager::new(GameScreen::Menu(MenuScreen::Party));
        mgr.stack.stack.push(GameScreen::WorldMap);
        let screen = apply_transition(&mut mgr, ScreenTransition::ReturnToPrevious);
        assert_eq!(screen, GameScreen::WorldMap);
        assert!(mgr.stack.stack.is_empty());
    }

    // 11. ReturnToPrevious on empty stack is a no-op
    #[test]
    fn test_return_to_previous_empty_stack_noop() {
        let mut mgr = ScreenManager::new(GameScreen::Title);
        let screen = apply_transition(&mut mgr, ScreenTransition::ReturnToPrevious);
        assert_eq!(screen, GameScreen::Title);
    }

    // 12. Deep navigation sequence and full unwind
    #[test]
    fn test_deep_navigation_unwind() {
        let mut mgr = ScreenManager::new(GameScreen::Title);
        apply_transition(&mut mgr, ScreenTransition::ToWorldMap);
        apply_transition(&mut mgr, ScreenTransition::EnterTown(TownId(1)));
        apply_transition(&mut mgr, ScreenTransition::OpenMenu(MenuScreen::Equipment));
        assert_eq!(mgr.stack.stack.len(), 3);

        apply_transition(&mut mgr, ScreenTransition::ReturnToPrevious);
        apply_transition(&mut mgr, ScreenTransition::ReturnToPrevious);
        apply_transition(&mut mgr, ScreenTransition::ReturnToPrevious);
        assert_eq!(mgr.current, GameScreen::Title);
        assert!(mgr.stack.stack.is_empty());
    }

    // 13. All 12 MenuScreen variants reachable
    #[test]
    fn test_all_menu_screen_variants() {
        let menus = [
            MenuScreen::Party,
            MenuScreen::Equipment,
            MenuScreen::Djinn,
            MenuScreen::Items,
            MenuScreen::Psynergy,
            MenuScreen::Status,
            MenuScreen::QuestLog,
        ];
        for menu in menus {
            let mut mgr = ScreenManager::new(GameScreen::WorldMap);
            apply_transition(&mut mgr, ScreenTransition::OpenMenu(menu));
            assert_eq!(mgr.current, GameScreen::Menu(menu));
        }
    }
}
