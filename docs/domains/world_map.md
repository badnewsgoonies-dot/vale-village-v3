# Domain: world_map
## Scope: src/domains/world_map/
## Imports from contract
MapNode, MapNodeId, MapNodeType, NodeUnlockState, Direction, TownId, DungeonId

## Deliverables
1. WorldMap struct: nodes (Vec<MapNode>), unlock_states (HashMap<MapNodeId, NodeUnlockState>)
2. fn load_map(nodes: Vec<MapNode>) -> WorldMap
3. fn unlock_node(map, node_id) — transitions Locked→Visible or Visible→Unlocked
4. fn complete_node(map, node_id) — marks Completed
5. fn get_accessible_nodes(map) -> Vec<&MapNode> — Unlocked or Completed
6. fn get_neighbors(map, node_id) -> Vec<&MapNode> — connected + accessible
7. fn can_travel(map, from, to) -> bool — connected and target is Unlocked/Completed
8. fn get_node_type(map, node_id) -> Option<&MapNodeType>
9. Tests: unlock progression (Locked→Visible→Unlocked→Completed), travel validation, neighbor filtering, inaccessible nodes hidden

## Does NOT handle
Rendering map, player movement animation, input handling.

## Quantitative targets
- 4 NodeUnlockState transitions tested
- 4 MapNodeType variants handled
- ≥8 tests
