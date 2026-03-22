#![allow(dead_code)]
//! World Map domain — pure logic for map state, node unlocking, and travel.
//! No rendering, no input, no Bevy systems.

use crate::shared::{MapNode, MapNodeId, MapNodeType, NodeUnlockState};
use std::collections::HashMap;

// ── WorldMap ─────────────────────────────────────────────────────────

/// Owns the full map graph and each node's unlock state.
#[derive(Debug, Clone)]
pub struct WorldMap {
    pub nodes: Vec<MapNode>,
    pub unlock_states: HashMap<MapNodeId, NodeUnlockState>,
}

// ── Constructor ──────────────────────────────────────────────────────

/// Build a WorldMap from a list of nodes.
/// All nodes start Locked unless they are of type Town, which start Unlocked.
pub fn load_map(nodes: Vec<MapNode>) -> WorldMap {
    let unlock_states = nodes
        .iter()
        .map(|n| {
            let state = match n.node_type {
                MapNodeType::Town(_) => NodeUnlockState::Unlocked,
                _ => NodeUnlockState::Locked,
            };
            (n.id, state)
        })
        .collect();
    WorldMap { nodes, unlock_states }
}

// ── State transitions ────────────────────────────────────────────────

/// Locked → Visible, Visible → Unlocked. Other states are unchanged.
pub fn unlock_node(map: &mut WorldMap, node_id: MapNodeId) {
    let state = map.unlock_states.entry(node_id).or_insert(NodeUnlockState::Locked);
    *state = match *state {
        NodeUnlockState::Locked => NodeUnlockState::Visible,
        NodeUnlockState::Visible => NodeUnlockState::Unlocked,
        other => other,
    };
}

/// Marks a node as Completed (must already be Unlocked or Completed).
pub fn complete_node(map: &mut WorldMap, node_id: MapNodeId) {
    let state = map.unlock_states.entry(node_id).or_insert(NodeUnlockState::Locked);
    if matches!(*state, NodeUnlockState::Unlocked | NodeUnlockState::Completed) {
        *state = NodeUnlockState::Completed;
    }
}

// ── Queries ──────────────────────────────────────────────────────────

/// Returns all nodes that are Unlocked or Completed.
pub fn get_accessible_nodes(map: &WorldMap) -> Vec<&MapNode> {
    map.nodes
        .iter()
        .filter(|n| {
            matches!(
                map.unlock_states.get(&n.id),
                Some(NodeUnlockState::Unlocked) | Some(NodeUnlockState::Completed)
            )
        })
        .collect()
}

/// Returns nodes connected to `node_id` that are also accessible (Unlocked or Completed).
pub fn get_neighbors(map: &WorldMap, node_id: MapNodeId) -> Vec<&MapNode> {
    let connections: Vec<MapNodeId> = map
        .nodes
        .iter()
        .find(|n| n.id == node_id)
        .map(|n| n.connections.clone())
        .unwrap_or_default();

    map.nodes
        .iter()
        .filter(|n| {
            connections.contains(&n.id)
                && matches!(
                    map.unlock_states.get(&n.id),
                    Some(NodeUnlockState::Unlocked) | Some(NodeUnlockState::Completed)
                )
        })
        .collect()
}

/// True if `from` and `to` are directly connected and `to` is Unlocked or Completed.
pub fn can_travel(map: &WorldMap, from: MapNodeId, to: MapNodeId) -> bool {
    let connected = map
        .nodes
        .iter()
        .find(|n| n.id == from)
        .map(|n| n.connections.contains(&to))
        .unwrap_or(false);

    if !connected {
        return false;
    }

    matches!(
        map.unlock_states.get(&to),
        Some(NodeUnlockState::Unlocked) | Some(NodeUnlockState::Completed)
    )
}

/// Returns the MapNodeType for a node, if it exists.
pub fn get_node_type(map: &WorldMap, node_id: MapNodeId) -> Option<&MapNodeType> {
    map.nodes
        .iter()
        .find(|n| n.id == node_id)
        .map(|n| &n.node_type)
}

// ── Tests ─────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::shared::{DungeonId, TownId};

    fn make_node(id: u8, node_type: MapNodeType, connections: Vec<u8>) -> MapNode {
        MapNode {
            id: MapNodeId(id),
            name: format!("Node {id}"),
            position: (0.0, 0.0),
            node_type,
            connections: connections.into_iter().map(MapNodeId).collect(),
        }
    }

    fn simple_map() -> WorldMap {
        load_map(vec![
            make_node(1, MapNodeType::Town(TownId(1)), vec![2, 3]),
            make_node(2, MapNodeType::Dungeon(DungeonId(1)), vec![1]),
            make_node(3, MapNodeType::Landmark, vec![1]),
            make_node(4, MapNodeType::Hidden, vec![]),
        ])
    }

    // ── load_map ──────────────────────────────────────────────────────

    #[test]
    fn test_load_map_town_starts_unlocked() {
        let map = simple_map();
        assert_eq!(
            map.unlock_states[&MapNodeId(1)],
            NodeUnlockState::Unlocked
        );
    }

    #[test]
    fn test_load_map_non_town_starts_locked() {
        let map = simple_map();
        assert_eq!(map.unlock_states[&MapNodeId(2)], NodeUnlockState::Locked);
        assert_eq!(map.unlock_states[&MapNodeId(3)], NodeUnlockState::Locked);
        assert_eq!(map.unlock_states[&MapNodeId(4)], NodeUnlockState::Locked);
    }

    // ── unlock_node progression ───────────────────────────────────────

    #[test]
    fn test_unlock_node_locked_to_visible() {
        let mut map = simple_map();
        unlock_node(&mut map, MapNodeId(2));
        assert_eq!(map.unlock_states[&MapNodeId(2)], NodeUnlockState::Visible);
    }

    #[test]
    fn test_unlock_node_visible_to_unlocked() {
        let mut map = simple_map();
        unlock_node(&mut map, MapNodeId(2));
        unlock_node(&mut map, MapNodeId(2));
        assert_eq!(map.unlock_states[&MapNodeId(2)], NodeUnlockState::Unlocked);
    }

    #[test]
    fn test_unlock_node_does_not_advance_completed() {
        let mut map = simple_map();
        unlock_node(&mut map, MapNodeId(2));
        unlock_node(&mut map, MapNodeId(2));
        complete_node(&mut map, MapNodeId(2));
        unlock_node(&mut map, MapNodeId(2)); // should stay Completed
        assert_eq!(map.unlock_states[&MapNodeId(2)], NodeUnlockState::Completed);
    }

    // ── complete_node ─────────────────────────────────────────────────

    #[test]
    fn test_complete_node_from_unlocked() {
        let mut map = simple_map();
        unlock_node(&mut map, MapNodeId(3));
        unlock_node(&mut map, MapNodeId(3));
        complete_node(&mut map, MapNodeId(3));
        assert_eq!(map.unlock_states[&MapNodeId(3)], NodeUnlockState::Completed);
    }

    #[test]
    fn test_complete_node_locked_does_nothing() {
        let mut map = simple_map();
        complete_node(&mut map, MapNodeId(2)); // still Locked
        assert_eq!(map.unlock_states[&MapNodeId(2)], NodeUnlockState::Locked);
    }

    // ── get_accessible_nodes ──────────────────────────────────────────

    #[test]
    fn test_get_accessible_nodes_only_unlocked_and_completed() {
        let mut map = simple_map();
        // node 1 = Town (Unlocked by default); unlock node 2 twice → Unlocked
        unlock_node(&mut map, MapNodeId(2));
        unlock_node(&mut map, MapNodeId(2));
        let accessible: Vec<MapNodeId> = get_accessible_nodes(&map)
            .into_iter()
            .map(|n| n.id)
            .collect();
        assert!(accessible.contains(&MapNodeId(1)));
        assert!(accessible.contains(&MapNodeId(2)));
        assert!(!accessible.contains(&MapNodeId(3)));
        assert!(!accessible.contains(&MapNodeId(4)));
    }

    #[test]
    fn test_inaccessible_nodes_hidden() {
        let map = simple_map();
        let accessible = get_accessible_nodes(&map);
        // Only the Town node should be accessible by default
        assert_eq!(accessible.len(), 1);
        assert_eq!(accessible[0].id, MapNodeId(1));
    }

    // ── get_neighbors ─────────────────────────────────────────────────

    #[test]
    fn test_get_neighbors_returns_accessible_connections() {
        let mut map = simple_map();
        unlock_node(&mut map, MapNodeId(2));
        unlock_node(&mut map, MapNodeId(2)); // node 2 now Unlocked
        let neighbors: Vec<MapNodeId> = get_neighbors(&map, MapNodeId(1))
            .into_iter()
            .map(|n| n.id)
            .collect();
        // node 3 is still Locked, so only node 2 should appear
        assert!(neighbors.contains(&MapNodeId(2)));
        assert!(!neighbors.contains(&MapNodeId(3)));
    }

    #[test]
    fn test_get_neighbors_excludes_locked_nodes() {
        let map = simple_map();
        let neighbors = get_neighbors(&map, MapNodeId(1));
        assert!(neighbors.is_empty());
    }

    // ── can_travel ────────────────────────────────────────────────────

    #[test]
    fn test_can_travel_connected_and_unlocked() {
        let mut map = simple_map();
        unlock_node(&mut map, MapNodeId(2));
        unlock_node(&mut map, MapNodeId(2));
        assert!(can_travel(&map, MapNodeId(1), MapNodeId(2)));
    }

    #[test]
    fn test_can_travel_connected_but_locked() {
        let map = simple_map();
        assert!(!can_travel(&map, MapNodeId(1), MapNodeId(2)));
    }

    #[test]
    fn test_can_travel_not_connected() {
        let mut map = simple_map();
        unlock_node(&mut map, MapNodeId(4));
        unlock_node(&mut map, MapNodeId(4));
        // node 4 has no connections, can't travel from node 1
        assert!(!can_travel(&map, MapNodeId(1), MapNodeId(4)));
    }

    #[test]
    fn test_can_travel_to_completed_node() {
        let mut map = simple_map();
        unlock_node(&mut map, MapNodeId(2));
        unlock_node(&mut map, MapNodeId(2));
        complete_node(&mut map, MapNodeId(2));
        assert!(can_travel(&map, MapNodeId(1), MapNodeId(2)));
    }

    // ── get_node_type ─────────────────────────────────────────────────

    #[test]
    fn test_get_node_type_known_node() {
        let map = simple_map();
        assert_eq!(
            get_node_type(&map, MapNodeId(1)),
            Some(&MapNodeType::Town(TownId(1)))
        );
        assert_eq!(
            get_node_type(&map, MapNodeId(2)),
            Some(&MapNodeType::Dungeon(DungeonId(1)))
        );
        assert_eq!(
            get_node_type(&map, MapNodeId(3)),
            Some(&MapNodeType::Landmark)
        );
        assert_eq!(
            get_node_type(&map, MapNodeId(4)),
            Some(&MapNodeType::Hidden)
        );
    }

    #[test]
    fn test_get_node_type_unknown_node() {
        let map = simple_map();
        assert_eq!(get_node_type(&map, MapNodeId(99)), None);
    }
}
