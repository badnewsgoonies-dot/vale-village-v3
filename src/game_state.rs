#![allow(dead_code)]
//! Game State — Integration layer holding all domain states.
//! Orchestrator-owned. Workers never touch this file.

use std::collections::HashMap;

use crate::shared::{
    bounded_types::Gold,
    DungeonId, GameScreen, MapNodeId, NodeUnlockState, QuestFlagId, QuestStage,
    ScreenTransition, ShopId, TownId,
};

use crate::domains::dungeon::DungeonState;
use crate::domains::quest::QuestManager;
use crate::domains::save;
use crate::domains::screens;
use crate::domains::shop::ShopState;
use crate::domains::world_map::WorldMap;

// ── Composite Game State ────────────────────────────────────────────

/// Holds all runtime state beyond the battle system.
#[derive(Debug)]
pub struct GameState {
    pub screen: GameScreen,
    pub screen_stack: crate::shared::ScreenStack,
    pub world_map: Option<WorldMap>,
    pub shop_state: ShopState,
    pub quest_state: crate::shared::QuestState,
    pub dungeon_state: Option<DungeonState>,
    pub gold: Gold,
    pub play_time_seconds: u64,
    pub steps_since_encounter: u16,
    pub active_encounter: Option<crate::shared::EncounterDef>,
}

impl GameState {
    /// Create a fresh game state for a new game.
    pub fn new_game() -> Self {
        Self {
            screen: GameScreen::Title,
            screen_stack: crate::shared::ScreenStack::default(),
            world_map: None,
            shop_state: ShopState::default(),
            quest_state: crate::shared::QuestState::default(),
            dungeon_state: None,
            gold: Gold::new(0),
            play_time_seconds: 0,
            steps_since_encounter: 0,
            active_encounter: None,
        }
    }

    /// Restore game state from save data extension.
    pub fn from_save(ext: &crate::shared::SaveDataExtension) -> Self {
        let mut quest_state = crate::shared::QuestState::default();
        for (flag, stage) in &ext.quest_state {
            quest_state.advance(*flag, *stage);
        }

        // Restore shop stock: Vec<(ItemId, Option<ItemCount>)> → HashMap<ItemId, Option<ItemCount>>
        let mut stock = std::collections::HashMap::new();
        for (sid, entries) in &ext.shop_stock {
            let items: std::collections::HashMap<crate::shared::ItemId, Option<crate::shared::bounded_types::ItemCount>> =
                entries.iter().map(|(iid, count)| (iid.clone(), *count)).collect();
            stock.insert(*sid, items);
        }

        Self {
            screen: ext.overworld.location.clone(),
            screen_stack: crate::shared::ScreenStack::default(),
            world_map: None, // loaded separately from data
            shop_state: ShopState { stock },
            quest_state,
            dungeon_state: None,
            gold: Gold::new(0), // gold lives in SaveData.gold
            play_time_seconds: ext.play_time_seconds,
            steps_since_encounter: 0,
            active_encounter: None,
        }
    }

    /// Convert current game state to save extension.
    pub fn to_save_extension(&self) -> crate::shared::SaveDataExtension {
        let map_unlock_state = self
            .world_map
            .as_ref()
            .map(|wm| wm.unlock_states.clone())
            .unwrap_or_default();

        let overworld = crate::shared::OverworldSaveData {
            location: self.screen.clone(),
            room: self.dungeon_state.as_ref().map(|ds| ds.current_room),
            position: (0.0, 0.0),
            facing: crate::shared::Direction::Down,
        };

        // Serialize shop stock: HashMap<ShopId, HashMap<ItemId, Option<ItemCount>>> → HashMap<ShopId, Vec<(ItemId, Option<ItemCount>)>>
        let shop_stock: HashMap<crate::shared::ShopId, Vec<(crate::shared::ItemId, Option<crate::shared::bounded_types::ItemCount>)>> = self.shop_state.stock.iter()
            .map(|(sid, items)| (*sid, items.iter().map(|(iid, count)| (iid.clone(), *count)).collect()))
            .collect();

        // Serialize dungeon collected items: need dungeon ID from current screen
        let collected_items = match (&self.screen, &self.dungeon_state) {
            (crate::shared::GameScreen::Dungeon(did), Some(ds)) => {
                ds.collected_items.iter().map(|(rid, idx)| (*did, *rid, *idx)).collect()
            }
            _ => Vec::new(),
        };

        crate::shared::SaveDataExtension {
            quest_state: self.quest_state.flags.clone(),
            map_unlock_state,
            overworld,
            shop_stock,
            visited_rooms: self
                .dungeon_state
                .as_ref()
                .map(|ds| ds.visited_rooms.iter().copied().collect())
                .unwrap_or_default(),
            collected_items,
            play_time_seconds: self.play_time_seconds,
            save_timestamp: String::new(),
        }
    }
}

/// Process a screen transition, updating game state.
pub fn apply_transition(state: &mut GameState, transition: ScreenTransition) {
    let new_screen = match transition {
        ScreenTransition::ToTitle => GameScreen::Title,
        ScreenTransition::ToWorldMap => GameScreen::WorldMap,
        ScreenTransition::EnterTown(id) => GameScreen::Town(id),
        ScreenTransition::EnterDungeon(id) => {
            state.dungeon_state = None; // reset on entry
            GameScreen::Dungeon(id)
        }
        ScreenTransition::StartBattle(enc) => {
            state.active_encounter = Some(enc);
            GameScreen::Battle
        }
        ScreenTransition::OpenMenu(menu) => {
            screens::push_screen(&mut state.screen_stack, state.screen.clone()).ok();
            GameScreen::Menu(menu)
        }
        ScreenTransition::OpenShop(id) => {
            screens::push_screen(&mut state.screen_stack, state.screen.clone()).ok();
            GameScreen::Shop(id)
        }
        ScreenTransition::StartDialogue(npc) => {
            screens::push_screen(&mut state.screen_stack, state.screen.clone()).ok();
            GameScreen::Dialogue(npc)
        }
        ScreenTransition::OpenSaveLoad => {
            screens::push_screen(&mut state.screen_stack, state.screen.clone()).ok();
            GameScreen::SaveLoad
        }
        ScreenTransition::TriggerGameOver => GameScreen::GameOver,
        ScreenTransition::TriggerVictory => GameScreen::Victory,
        ScreenTransition::ReturnToPrevious => {
            screens::pop_screen(&mut state.screen_stack).unwrap_or(GameScreen::Title)
        }
    };
    state.screen = new_screen;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_game_starts_at_title() {
        let gs = GameState::new_game();
        assert_eq!(gs.screen, GameScreen::Title);
    }

    #[test]
    fn test_transition_to_world_map() {
        let mut gs = GameState::new_game();
        apply_transition(&mut gs, ScreenTransition::ToWorldMap);
        assert_eq!(gs.screen, GameScreen::WorldMap);
    }

    #[test]
    fn test_menu_push_pop() {
        let mut gs = GameState::new_game();
        apply_transition(&mut gs, ScreenTransition::ToWorldMap);
        assert_eq!(gs.screen, GameScreen::WorldMap);

        apply_transition(&mut gs, ScreenTransition::OpenMenu(crate::shared::MenuScreen::Party));
        assert_eq!(gs.screen, GameScreen::Menu(crate::shared::MenuScreen::Party));

        apply_transition(&mut gs, ScreenTransition::ReturnToPrevious);
        assert_eq!(gs.screen, GameScreen::WorldMap);
    }

    #[test]
    fn test_save_roundtrip() {
        let mut gs = GameState::new_game();
        apply_transition(&mut gs, ScreenTransition::ToWorldMap);
        gs.quest_state.advance(QuestFlagId(1), QuestStage::Active);
        gs.play_time_seconds = 3600;

        let ext = gs.to_save_extension();
        let restored = GameState::from_save(&ext);

        assert_eq!(restored.screen, GameScreen::WorldMap);
        assert!(restored.quest_state.at_least(QuestFlagId(1), QuestStage::Active));
        assert_eq!(restored.play_time_seconds, 3600);
    }
}
