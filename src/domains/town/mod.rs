#![allow(dead_code)]
//! Town Domain — TownState, NPC/shop/exit lookups, djinn discovery.

use std::collections::HashSet;

use crate::shared::{
    DialogueTreeId, DjinnId, MapNodeId, NpcId, NpcPlacement, QuestStage, QuestState,
    ShopId, TownDef, TownId,
};

// ── TownState ────────────────────────────────────────────────────────

#[derive(Debug, Clone, Default)]
pub struct TownState {
    pub current_town: Option<TownId>,
    pub discovered_djinn: HashSet<DjinnId>,
}

// ── Load ─────────────────────────────────────────────────────────────

pub fn load_town(def: &TownDef) -> TownState {
    TownState {
        current_town: Some(def.id),
        discovered_djinn: HashSet::new(),
    }
}

// ── NPC Lookups ──────────────────────────────────────────────────────

pub fn get_npcs(def: &TownDef) -> &[NpcPlacement] {
    &def.npcs
}

pub fn get_npc_dialogue(def: &TownDef, npc_id: NpcId) -> Option<DialogueTreeId> {
    def.npcs
        .iter()
        .find(|npc| npc.npc_id == npc_id)
        .map(|npc| npc.dialogue_tree)
}

// ── Shop Lookups ─────────────────────────────────────────────────────

pub fn get_shops(def: &TownDef) -> &[ShopId] {
    &def.shops
}

// ── Exit Lookups ─────────────────────────────────────────────────────

pub fn get_exits(def: &TownDef) -> &[MapNodeId] {
    &def.exits
}

// ── Djinn Discovery ──────────────────────────────────────────────────

const DJINN_DISCOVERY_RADIUS: f32 = 1.5;

/// Returns the DjinnId if the player is within range of an undiscovered djinn point
/// whose quest gate (if any) is satisfied.
///
/// Puzzle-gated djinn require their linked quest flag to be Complete; otherwise
/// quest-gated djinn unlock once their linked quest reaches Active.
pub fn check_djinn_discovery(
    def: &TownDef,
    state: &TownState,
    position: (f32, f32),
    quest_state: &QuestState,
) -> Option<DjinnId> {
    def.djinn_points.iter().find_map(|point| {
        // Already discovered — skip.
        if state.discovered_djinn.contains(&point.djinn_id) {
            return None;
        }
        // Quest gate: puzzle-gated djinn require a completed quest; otherwise Active is enough.
        if let Some(flag) = point.quest_flag {
            let required_stage = if point.requires_puzzle {
                QuestStage::Complete
            } else {
                QuestStage::Active
            };
            if !quest_state.at_least(flag, required_stage) {
                return None;
            }
        }
        // Distance check.
        let dx = position.0 - point.position.0;
        let dy = position.1 - point.position.1;
        if dx * dx + dy * dy <= DJINN_DISCOVERY_RADIUS * DJINN_DISCOVERY_RADIUS {
            Some(point.djinn_id.clone())
        } else {
            None
        }
    })
}

/// Marks a djinn as discovered (one-time; subsequent calls are no-ops).
pub fn discover_djinn(state: &mut TownState, djinn_id: DjinnId) {
    state.discovered_djinn.insert(djinn_id);
}

// ── Tests ─────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::shared::{Direction, DjinnDiscoveryPoint, QuestFlagId};

    fn make_def() -> TownDef {
        TownDef {
            id: TownId(1),
            name: "Vale Village".to_string(),
            npcs: vec![
                NpcPlacement {
                    npc_id: NpcId(10),
                    position: (5.0, 5.0),
                    facing: Direction::Down,
                    dialogue_tree: DialogueTreeId(100),
                },
                NpcPlacement {
                    npc_id: NpcId(11),
                    position: (8.0, 3.0),
                    facing: Direction::Up,
                    dialogue_tree: DialogueTreeId(101),
                },
            ],
            shops: vec![ShopId(1), ShopId(2)],
            djinn_points: vec![
                DjinnDiscoveryPoint {
                    djinn_id: DjinnId("Flint".to_string()),
                    position: (0.0, 0.0),
                    requires_puzzle: false,
                    quest_flag: None,
                },
                DjinnDiscoveryPoint {
                    djinn_id: DjinnId("Forge".to_string()),
                    position: (20.0, 20.0),
                    requires_puzzle: false,
                    quest_flag: Some(QuestFlagId(5)),
                },
            ],
            exits: vec![MapNodeId(3), MapNodeId(4)],
        }
    }

    #[test]
    fn test_load_town_sets_current_town() {
        let def = make_def();
        let state = load_town(&def);
        assert_eq!(state.current_town, Some(TownId(1)));
        assert!(state.discovered_djinn.is_empty());
    }

    #[test]
    fn test_get_npcs_returns_all() {
        let def = make_def();
        assert_eq!(get_npcs(&def).len(), 2);
    }

    #[test]
    fn test_get_npc_dialogue_found() {
        let def = make_def();
        assert_eq!(get_npc_dialogue(&def, NpcId(10)), Some(DialogueTreeId(100)));
        assert_eq!(get_npc_dialogue(&def, NpcId(11)), Some(DialogueTreeId(101)));
    }

    #[test]
    fn test_get_npc_dialogue_not_found() {
        let def = make_def();
        assert_eq!(get_npc_dialogue(&def, NpcId(99)), None);
    }

    #[test]
    fn test_get_shops_returns_all() {
        let def = make_def();
        let shops = get_shops(&def);
        assert_eq!(shops.len(), 2);
        assert!(shops.contains(&ShopId(1)));
        assert!(shops.contains(&ShopId(2)));
    }

    #[test]
    fn test_get_exits_returns_all() {
        let def = make_def();
        let exits = get_exits(&def);
        assert_eq!(exits.len(), 2);
        assert!(exits.contains(&MapNodeId(3)));
        assert!(exits.contains(&MapNodeId(4)));
    }

    #[test]
    fn test_djinn_discovery_within_range() {
        let def = make_def();
        let state = load_town(&def);
        let qs = QuestState::default();
        let result = check_djinn_discovery(&def, &state, (0.5, 0.5), &qs);
        assert_eq!(result, Some(DjinnId("Flint".to_string())));
    }

    #[test]
    fn test_djinn_discovery_out_of_range() {
        let def = make_def();
        let state = load_town(&def);
        let qs = QuestState::default();
        let result = check_djinn_discovery(&def, &state, (10.0, 10.0), &qs);
        assert_eq!(result, None);
    }

    #[test]
    fn test_djinn_discovery_one_time() {
        let def = make_def();
        let mut state = load_town(&def);
        let qs = QuestState::default();

        let first = check_djinn_discovery(&def, &state, (0.5, 0.5), &qs);
        assert_eq!(first, Some(DjinnId("Flint".to_string())));

        discover_djinn(&mut state, DjinnId("Flint".to_string()));

        let second = check_djinn_discovery(&def, &state, (0.5, 0.5), &qs);
        assert_eq!(second, None);
    }

    #[test]
    fn test_djinn_quest_gated_blocked() {
        let def = make_def();
        let state = load_town(&def);
        let qs = QuestState::default(); // quest not started
        // Forge is at (20, 20) — position it within range but quest gate fails
        let result = check_djinn_discovery(&def, &state, (20.0, 20.0), &qs);
        assert_eq!(result, None);
    }

    #[test]
    fn test_djinn_quest_gated_unlocked() {
        let def = make_def();
        let state = load_town(&def);
        let mut qs = QuestState::default();
        qs.advance(QuestFlagId(5), QuestStage::Active);
        let result = check_djinn_discovery(&def, &state, (20.0, 20.0), &qs);
        assert_eq!(result, Some(DjinnId("Forge".to_string())));
    }

    #[test]
    fn test_djinn_requires_puzzle_blocks_until_complete() {
        let mut def = make_def();
        def.djinn_points[1].requires_puzzle = true;

        let state = load_town(&def);
        let mut qs = QuestState::default();
        qs.advance(QuestFlagId(5), QuestStage::Active);

        let blocked = check_djinn_discovery(&def, &state, (20.0, 20.0), &qs);
        assert_eq!(blocked, None);

        qs.advance(QuestFlagId(5), QuestStage::Complete);

        let unlocked = check_djinn_discovery(&def, &state, (20.0, 20.0), &qs);
        assert_eq!(unlocked, Some(DjinnId("Forge".to_string())));
    }
}
