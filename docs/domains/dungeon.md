# Domain: dungeon
## Scope: src/domains/dungeon/
## Imports from contract
DungeonDef, DungeonId, RoomDef, RoomId, RoomType, RoomExit, EncounterSlot,
RoomItem, PuzzleDef, Direction, DialogueCondition, ItemId, QuestFlagId

## Deliverables
1. DungeonState struct: current_room (RoomId), visited_rooms (HashSet<RoomId>), collected_items (HashSet<(RoomId, usize)>), encounter_counts (HashMap<(RoomId, usize), u8>)
2. fn enter_dungeon(def: &DungeonDef) -> DungeonState — starts at entry_room
3. fn get_current_room(state, def) -> &RoomDef
4. fn get_available_exits(state, def, context: &dyn ConditionContext) -> Vec<(Direction, RoomId)> — filters by condition
5. fn move_to_room(state, room_id) -> Result<(), DungeonError> — validates exit exists
6. fn collect_item(state, item_index: usize) -> Option<ItemId> — one-time collection
7. fn trigger_encounter(state, slot_index: usize) -> Option<EncounterDef> — respects max_triggers
8. fn get_room_puzzles(state, def) -> Vec<&PuzzleDef>
9. fn is_boss_room(def, room_id) -> bool
10. Tests: entry room, room navigation, item collection one-time, encounter depletion, conditional exits, boss room detection

## Does NOT handle
Rendering rooms, puzzle solving (delegates to puzzle domain), battle execution.

## Quantitative targets
- 6 RoomType variants recognized
- One-time item collection enforced
- max_triggers encounter depletion
- ≥10 tests
