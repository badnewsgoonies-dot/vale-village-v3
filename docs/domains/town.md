# Domain: town
## Scope: src/domains/town/
## Imports from contract
TownDef, TownId, NpcPlacement, NpcId, DjinnDiscoveryPoint, DjinnId, ShopId, MapNodeId,
DialogueTreeId, QuestFlagId, Direction

## Deliverables
1. TownState struct: current_town (Option<TownId>), discovered_djinn (HashSet<DjinnId>)
2. fn load_town(def: &TownDef) -> TownState
3. fn get_npcs(def: &TownDef) -> &[NpcPlacement]
4. fn get_npc_dialogue(def: &TownDef, npc_id: NpcId) -> Option<DialogueTreeId>
5. fn get_shops(def: &TownDef) -> &[ShopId]
6. fn check_djinn_discovery(def: &TownDef, state: &TownState, position: (f32, f32), quest_state: &QuestState) -> Option<DjinnId>
7. fn discover_djinn(state: &mut TownState, djinn_id: DjinnId) — marks discovered, one-time
8. fn get_exits(def: &TownDef) -> &[MapNodeId]
9. Tests: NPC lookup, shop listing, djinn discovery (one-time), exit listing, quest-gated djinn

## Does NOT handle
Rendering town, movement, dialogue execution, shop transactions.

## Quantitative targets
- One-time djinn discovery enforced
- Quest-gated djinn points tested
- ≥6 tests
