#![allow(dead_code)]
//! Dungeon Domain — Wave 2
//!
//! Manages dungeon navigation state, item collection, encounter triggering,
//! and conditional exit evaluation. Does NOT handle rendering, puzzle solving,
//! or battle execution.

use std::collections::{HashMap, HashSet};

use crate::shared::bounded_types::Gold;
use crate::shared::{
    DialogueCondition, Direction, DjinnId, DungeonDef, EncounterDef, ItemId, PuzzleDef,
    QuestFlagId, QuestStage, RoomDef, RoomId, UnitId,
};

// ── Condition evaluation ────────────────────────────────────────────

/// Implemented by the game state layer so dungeon exits can evaluate
/// shared dialogue-style conditions without importing another domain.
pub trait ConditionContext {
    fn has_item(&self, item: &ItemId) -> bool;
    fn has_djinn(&self, djinn: &DjinnId) -> bool;
    fn quest_at_stage(&self, flag: &QuestFlagId, stage: QuestStage) -> bool;
    fn gold_at_least(&self, amount: Gold) -> bool;
    fn party_contains(&self, unit: &UnitId) -> bool;
}

fn evaluate_condition(cond: &DialogueCondition, ctx: &dyn ConditionContext) -> bool {
    match cond {
        DialogueCondition::HasItem(id) => ctx.has_item(id),
        DialogueCondition::HasDjinn(id) => ctx.has_djinn(id),
        DialogueCondition::QuestAtStage(flag, stage) => ctx.quest_at_stage(flag, *stage),
        DialogueCondition::GoldAtLeast(amount) => ctx.gold_at_least(*amount),
        DialogueCondition::PartyContains(unit) => ctx.party_contains(unit),
    }
}

// ── Error type ───────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DungeonError {
    /// Attempted to move to a room not reachable from the current room.
    NoExitToRoom(RoomId),
    /// Dungeon definition references a room that does not exist.
    RoomNotFound(RoomId),
}

impl std::fmt::Display for DungeonError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DungeonError::NoExitToRoom(id) => write!(f, "no exit leads to room {:?}", id),
            DungeonError::RoomNotFound(id) => write!(f, "room {:?} not found in dungeon def", id),
        }
    }
}

impl std::error::Error for DungeonError {}

// ── DungeonState ─────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct DungeonState {
    /// The room the player is currently in.
    pub current_room: RoomId,
    /// All rooms the player has entered (including the current one).
    pub visited_rooms: HashSet<RoomId>,
    /// One-time items that have been picked up: `(room_id, item_index)`.
    pub collected_items: HashSet<(RoomId, usize)>,
    /// How many times each encounter slot has been triggered: `(room_id, slot_index)`.
    pub encounter_counts: HashMap<(RoomId, usize), u8>,
}

// ── Public API ────────────────────────────────────────────────────────

/// Create a fresh `DungeonState` positioned at the dungeon's entry room.
pub fn enter_dungeon(def: &DungeonDef) -> DungeonState {
    let entry = def.entry_room;
    let mut visited = HashSet::new();
    visited.insert(entry);
    DungeonState {
        current_room: entry,
        visited_rooms: visited,
        collected_items: HashSet::new(),
        encounter_counts: HashMap::new(),
    }
}

/// Return a reference to the `RoomDef` for the player's current room.
///
/// # Panics
/// Panics if the current room id is not present in `def.rooms` — this
/// represents a corrupted dungeon definition.
pub fn get_current_room<'d>(state: &DungeonState, def: &'d DungeonDef) -> &'d RoomDef {
    def.rooms
        .iter()
        .find(|r| r.id == state.current_room)
        .unwrap_or_else(|| panic!("current room {:?} not found in dungeon def", state.current_room))
}

/// Return exits whose condition (if any) currently passes.
pub fn get_available_exits(
    state: &DungeonState,
    def: &DungeonDef,
    ctx: &dyn ConditionContext,
) -> Vec<(Direction, RoomId)> {
    let room = get_current_room(state, def);
    room.exits
        .iter()
        .filter(|exit| {
            exit.requires
                .as_ref()
                .map(|cond| evaluate_condition(cond, ctx))
                .unwrap_or(true)
        })
        .map(|exit| (exit.direction, exit.target_room))
        .collect()
}

/// Move the player to `room_id`.
///
/// Returns `Err(DungeonError::NoExitToRoom)` if no exit (regardless of
/// condition) leads to `room_id` from the current room.
pub fn move_to_room(state: &mut DungeonState, def: &DungeonDef, room_id: RoomId) -> Result<(), DungeonError> {
    let room = get_current_room(state, def);
    let exit_exists = room.exits.iter().any(|e| e.target_room == room_id);
    if !exit_exists {
        return Err(DungeonError::NoExitToRoom(room_id));
    }
    // Verify the target room actually exists in the definition.
    if def.rooms.iter().all(|r| r.id != room_id) {
        return Err(DungeonError::RoomNotFound(room_id));
    }
    state.current_room = room_id;
    state.visited_rooms.insert(room_id);
    Ok(())
}

/// Collect the item at `item_index` in the current room.
///
/// Returns `Some(ItemId)` on first collection, `None` if already collected
/// or the index is out of bounds.
pub fn collect_item(state: &mut DungeonState, def: &DungeonDef, item_index: usize) -> Option<ItemId> {
    let room = get_current_room(state, def);
    let room_id = room.id;
    let key = (room_id, item_index);
    if state.collected_items.contains(&key) {
        return None;
    }
    let item_id = room.items.get(item_index)?.item_id.clone();
    state.collected_items.insert(key);
    Some(item_id)
}

/// Trigger encounter slot `slot_index` in the current room.
///
/// Returns `Some(EncounterDef)` if the slot exists and has not been
/// exhausted (`max_triggers`). Increments the trigger counter.
pub fn trigger_encounter(state: &mut DungeonState, def: &DungeonDef, slot_index: usize) -> Option<EncounterDef> {
    let room = get_current_room(state, def);
    let room_id = room.id;
    let slot = room.encounters.get(slot_index)?;
    let key = (room_id, slot_index);
    let count = state.encounter_counts.entry(key).or_insert(0);
    if let Some(max) = slot.max_triggers {
        if *count >= max {
            return None;
        }
    }
    *count += 1;
    Some(slot.encounter.clone())
}

/// Return all puzzles in the current room.
pub fn get_room_puzzles<'d>(state: &DungeonState, def: &'d DungeonDef) -> Vec<&'d PuzzleDef> {
    let room = get_current_room(state, def);
    room.puzzles.iter().collect()
}

/// Return `true` if `room_id` is the designated boss room of this dungeon.
pub fn is_boss_room(def: &DungeonDef, room_id: RoomId) -> bool {
    def.boss_room == Some(room_id)
}

// ── Tests ─────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::shared::{
        bounded_types::Xp,
        DialogueCondition, Difficulty, DjinnId, EncounterDef, EncounterId, EncounterSlot, ItemId,
        PuzzleDef, PuzzleType, RoomExit, RoomItem, RoomType, UnitId, QuestFlagId, QuestStage,
    };

    // ── Minimal ConditionContext for tests ────────────────────────────

    struct TestCtx {
        items: Vec<ItemId>,
    }

    impl ConditionContext for TestCtx {
        fn has_item(&self, item: &ItemId) -> bool {
            self.items.contains(item)
        }
        fn has_djinn(&self, _: &DjinnId) -> bool { false }
        fn quest_at_stage(&self, _: &QuestFlagId, _: QuestStage) -> bool { false }
        fn gold_at_least(&self, _: Gold) -> bool { false }
        fn party_contains(&self, _: &UnitId) -> bool { false }
    }

    fn empty_ctx() -> TestCtx {
        TestCtx { items: vec![] }
    }

    // ── Dungeon fixture ───────────────────────────────────────────────

    fn make_encounter(id: &str) -> EncounterDef {
        EncounterDef {
            id: EncounterId(id.to_string()),
            name: id.to_string(),
            difficulty: Difficulty::Easy,
            enemies: vec![],
            xp_reward: Xp::new(10),
            gold_reward: Gold::new(5),
            recruit: None,
            djinn_reward: None,
            equipment_rewards: vec![],
        }
    }

    fn make_dungeon() -> DungeonDef {
        let room0 = RoomDef {
            id: RoomId(0),
            room_type: RoomType::Normal,
            exits: vec![RoomExit {
                direction: Direction::Right,
                target_room: RoomId(1),
                requires: None,
            }],
            encounters: vec![EncounterSlot {
                encounter: make_encounter("slime"),
                weight: 10,
                max_triggers: Some(2),
            }],
            items: vec![RoomItem {
                item_id: ItemId("herb".to_string()),
                position: (0.0, 0.0),
                visible: true,
                quest_flag: None,
            }],
            puzzles: vec![],
        };

        let room1 = RoomDef {
            id: RoomId(1),
            room_type: RoomType::Normal,
            exits: vec![
                RoomExit {
                    direction: Direction::Left,
                    target_room: RoomId(0),
                    requires: None,
                },
                RoomExit {
                    direction: Direction::Right,
                    target_room: RoomId(2),
                    requires: Some(DialogueCondition::HasItem(ItemId("key".to_string()))),
                },
            ],
            encounters: vec![],
            items: vec![],
            puzzles: vec![PuzzleDef {
                puzzle_type: PuzzleType::PushBlock,
                reward: None,
            }],
        };

        let room2 = RoomDef {
            id: RoomId(2),
            room_type: RoomType::Boss,
            exits: vec![],
            encounters: vec![EncounterSlot {
                encounter: make_encounter("boss"),
                weight: 1,
                max_triggers: None, // unlimited
            }],
            items: vec![],
            puzzles: vec![],
        };

        DungeonDef {
            id: crate::shared::DungeonId(1),
            name: "Test Dungeon".to_string(),
            rooms: vec![room0, room1, room2],
            entry_room: RoomId(0),
            boss_room: Some(RoomId(2)),
        }
    }

    // ── Tests ─────────────────────────────────────────────────────────

    #[test]
    fn test_enter_dungeon_starts_at_entry_room() {
        let def = make_dungeon();
        let state = enter_dungeon(&def);
        assert_eq!(state.current_room, RoomId(0));
    }

    #[test]
    fn test_enter_dungeon_marks_entry_visited() {
        let def = make_dungeon();
        let state = enter_dungeon(&def);
        assert!(state.visited_rooms.contains(&RoomId(0)));
    }

    #[test]
    fn test_move_to_room_succeeds() {
        let def = make_dungeon();
        let mut state = enter_dungeon(&def);
        assert!(move_to_room(&mut state, &def, RoomId(1)).is_ok());
        assert_eq!(state.current_room, RoomId(1));
    }

    #[test]
    fn test_move_to_room_marks_visited() {
        let def = make_dungeon();
        let mut state = enter_dungeon(&def);
        move_to_room(&mut state, &def, RoomId(1)).unwrap();
        assert!(state.visited_rooms.contains(&RoomId(1)));
    }

    #[test]
    fn test_move_to_room_invalid_returns_error() {
        let def = make_dungeon();
        let mut state = enter_dungeon(&def);
        let result = move_to_room(&mut state, &def, RoomId(2));
        assert_eq!(result, Err(DungeonError::NoExitToRoom(RoomId(2))));
    }

    #[test]
    fn test_item_collection_one_time() {
        let def = make_dungeon();
        let mut state = enter_dungeon(&def);
        let first = collect_item(&mut state, &def, 0);
        assert_eq!(first, Some(ItemId("herb".to_string())));
        let second = collect_item(&mut state, &def, 0);
        assert_eq!(second, None, "item should not be collectible twice");
    }

    #[test]
    fn test_item_collection_out_of_bounds() {
        let def = make_dungeon();
        let mut state = enter_dungeon(&def);
        assert_eq!(collect_item(&mut state, &def, 99), None);
    }

    #[test]
    fn test_encounter_trigger_respects_max_triggers() {
        let def = make_dungeon();
        let mut state = enter_dungeon(&def);
        // max_triggers == 2
        assert!(trigger_encounter(&mut state, &def, 0).is_some());
        assert!(trigger_encounter(&mut state, &def, 0).is_some());
        assert!(
            trigger_encounter(&mut state, &def, 0).is_none(),
            "encounter should be exhausted after max_triggers"
        );
    }

    #[test]
    fn test_encounter_unlimited_triggers() {
        let def = make_dungeon();
        let mut state = enter_dungeon(&def);
        move_to_room(&mut state, &def, RoomId(1)).unwrap();
        // Need key to reach room 2; bypass by moving via def directly
        // Instead just manipulate state manually for test purposes
        state.current_room = RoomId(2);
        for _ in 0..10 {
            assert!(
                trigger_encounter(&mut state, &def, 0).is_some(),
                "unlimited encounter should always trigger"
            );
        }
    }

    #[test]
    fn test_conditional_exits_filtered_without_item() {
        let def = make_dungeon();
        let mut state = enter_dungeon(&def);
        move_to_room(&mut state, &def, RoomId(1)).unwrap();
        let ctx = empty_ctx();
        let exits = get_available_exits(&state, &def, &ctx);
        // Only the unconditional left exit should be visible
        assert_eq!(exits.len(), 1);
        assert_eq!(exits[0].0, Direction::Left);
    }

    #[test]
    fn test_conditional_exits_shown_with_item() {
        let def = make_dungeon();
        let mut state = enter_dungeon(&def);
        move_to_room(&mut state, &def, RoomId(1)).unwrap();
        let ctx = TestCtx { items: vec![ItemId("key".to_string())] };
        let exits = get_available_exits(&state, &def, &ctx);
        assert_eq!(exits.len(), 2, "both exits visible when player has key");
    }

    #[test]
    fn test_is_boss_room_true() {
        let def = make_dungeon();
        assert!(is_boss_room(&def, RoomId(2)));
    }

    #[test]
    fn test_is_boss_room_false() {
        let def = make_dungeon();
        assert!(!is_boss_room(&def, RoomId(0)));
        assert!(!is_boss_room(&def, RoomId(1)));
    }

    #[test]
    fn test_get_room_puzzles() {
        let def = make_dungeon();
        let mut state = enter_dungeon(&def);
        move_to_room(&mut state, &def, RoomId(1)).unwrap();
        let puzzles = get_room_puzzles(&state, &def);
        assert_eq!(puzzles.len(), 1);
        assert_eq!(puzzles[0].puzzle_type, PuzzleType::PushBlock);
    }
}
