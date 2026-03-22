#![allow(dead_code)]
//! Starter game content — hardcoded world data for early game.
//! Will be replaced with RON data loader when the pipeline is built.

use crate::shared::{
    bounded_types::{Gold, ItemCount},
    DialogueNode, DialogueNodeId, DialogueResponse, DialogueTree, DialogueTreeId,
    DjinnId, Direction, DungeonDef, DungeonId, GameScreen, MapNode, MapNodeId,
    MapNodeType, NpcId, QuestFlagId, QuestStage,
    RoomDef, RoomExit, RoomId, RoomItem, RoomType, ItemId,
    ShopId, TownDef, TownId,
};

pub fn starter_map_nodes() -> Vec<MapNode> {
    vec![
        MapNode {
            id: MapNodeId(0),
            name: "Vale Village".into(),
            position: (100.0, 200.0),
            node_type: MapNodeType::Town(TownId(0)),
            connections: vec![MapNodeId(1), MapNodeId(2)],
        },
        MapNode {
            id: MapNodeId(1),
            name: "Mercury Lighthouse".into(),
            position: (300.0, 100.0),
            node_type: MapNodeType::Dungeon(DungeonId(0)),
            connections: vec![MapNodeId(0)],
        },
        MapNode {
            id: MapNodeId(2),
            name: "Kolima Forest".into(),
            position: (300.0, 300.0),
            node_type: MapNodeType::Dungeon(DungeonId(1)),
            connections: vec![MapNodeId(0), MapNodeId(3)],
        },
        MapNode {
            id: MapNodeId(3),
            name: "Imil".into(),
            position: (500.0, 100.0),
            node_type: MapNodeType::Town(TownId(1)),
            connections: vec![MapNodeId(2)],
        },
    ]
}

pub fn starter_towns() -> Vec<TownDef> {
    use crate::shared::{DialogueTreeId, NpcPlacement, ShopId, DjinnDiscoveryPoint};
    vec![
        TownDef {
            id: TownId(0),
            name: "Vale Village".into(),
            npcs: vec![
                NpcPlacement {
                    npc_id: NpcId(0),
                    position: (50.0, 50.0),
                    facing: Direction::Down,
                    dialogue_tree: DialogueTreeId(0),
                },
            ],
            shops: vec![ShopId(0)],
            djinn_points: vec![],
            exits: vec![MapNodeId(0)],
        },
        TownDef {
            id: TownId(1),
            name: "Imil".into(),
            npcs: vec![
                NpcPlacement {
                    npc_id: NpcId(2),
                    position: (40.0, 40.0),
                    facing: Direction::Down,
                    dialogue_tree: DialogueTreeId(2),
                },
            ],
            shops: vec![ShopId(1)],
            djinn_points: vec![
                DjinnDiscoveryPoint {
                    djinn_id: DjinnId("Fizz".into()),
                    position: (120.0, 80.0),
                    requires_puzzle: false,
                    quest_flag: None,
                },
            ],
            exits: vec![MapNodeId(3)],
        },
    ]
}

pub fn starter_shop_defs() -> Vec<crate::shared::ShopDef> {
    use crate::shared::{ItemId, ShopDef, ShopEntry, ShopStock};
    use crate::shared::bounded_types::ItemCount;
    vec![
        ShopDef {
            id: ShopId(0),
            name: "Vale General Store".into(),
            inventory: vec![
                ShopEntry { item_id: ItemId("herb".into()), price: Gold::new(10), stock: ShopStock::Unlimited },
                ShopEntry { item_id: ItemId("antidote".into()), price: Gold::new(15), stock: ShopStock::Unlimited },
            ],
        },
        ShopDef {
            id: ShopId(1),
            name: "Imil Outfitters".into(),
            inventory: vec![
                ShopEntry { item_id: ItemId("herb".into()), price: Gold::new(10), stock: ShopStock::Unlimited },
            ],
        },
    ]
}

// ── Dialogue Trees ───────────────────────────────────────────────────

pub fn starter_dialogue_trees() -> Vec<DialogueTree> {
    vec![
        // NPC 0: Elder in Vale Village
        DialogueTree {
            id: DialogueTreeId(0),
            root: DialogueNodeId(0),
            nodes: vec![
                DialogueNode {
                    id: DialogueNodeId(0),
                    speaker: Some(NpcId(0)),
                    text: "Welcome, young adept. Dark forces stir at Mercury Lighthouse. Will you investigate?".into(),
                    responses: vec![
                        DialogueResponse {
                            text: "I'll go at once.".into(),
                            condition: None,
                            next_node: Some(DialogueNodeId(1)),
                            side_effects: vec![
                                crate::shared::DialogueSideEffect::SetQuestStage(QuestFlagId(0), QuestStage::Active),
                            ],
                        },
                        DialogueResponse {
                            text: "Tell me more first.".into(),
                            condition: None,
                            next_node: Some(DialogueNodeId(2)),
                            side_effects: vec![],
                        },
                    ],
                },
                DialogueNode {
                    id: DialogueNodeId(1),
                    speaker: Some(NpcId(0)),
                    text: "Brave soul. Take this gold for supplies. The lighthouse is north of town.".into(),
                    responses: vec![
                        DialogueResponse {
                            text: "[Accept]".into(),
                            condition: None,
                            next_node: None,
                            side_effects: vec![
                                crate::shared::DialogueSideEffect::GiveGold(Gold::new(200)),
                            ],
                        },
                    ],
                },
                DialogueNode {
                    id: DialogueNodeId(2),
                    speaker: Some(NpcId(0)),
                    text: "Saturos and his followers seek to light the beacon. If they succeed, the world will change forever.".into(),
                    responses: vec![
                        DialogueResponse {
                            text: "I understand. I'll stop them.".into(),
                            condition: None,
                            next_node: Some(DialogueNodeId(1)),
                            side_effects: vec![
                                crate::shared::DialogueSideEffect::SetQuestStage(QuestFlagId(0), QuestStage::Active),
                            ],
                        },
                    ],
                },
            ],
        },
        // NPC 1: Shop hint
        DialogueTree {
            id: DialogueTreeId(1),
            root: DialogueNodeId(0),
            nodes: vec![
                DialogueNode {
                    id: DialogueNodeId(0),
                    speaker: Some(NpcId(1)),
                    text: "The general store has herbs and equipment. Stock up before heading out!".into(),
                    responses: vec![
                        DialogueResponse {
                            text: "Thanks for the tip.".into(),
                            condition: None,
                            next_node: None,
                            side_effects: vec![],
                        },
                    ],
                },
            ],
        },
        // NPC 2: Imil resident
        DialogueTree {
            id: DialogueTreeId(2),
            root: DialogueNodeId(0),
            nodes: vec![
                DialogueNode {
                    id: DialogueNodeId(0),
                    speaker: Some(NpcId(2)),
                    text: "Imil is peaceful, but we rely on the lighthouse's power. Please don't let anything happen to it.".into(),
                    responses: vec![
                        DialogueResponse {
                            text: "I'll do my best.".into(),
                            condition: None,
                            next_node: None,
                            side_effects: vec![],
                        },
                    ],
                },
            ],
        },
    ]
}

// ── Dungeon Definitions ──────────────────────────────────────────────

pub fn starter_dungeons() -> Vec<DungeonDef> {
    use crate::shared::{EncounterSlot, PuzzleDef, PuzzleType, Element, EncounterDef,
        EncounterId, Difficulty, EncounterEnemy, EnemyId};
    use crate::shared::bounded_types::{Xp, Hp};

    // Helper: create a minimal EncounterDef referencing a data-file encounter by ID
    let enc = |id: &str, name: &str, diff: Difficulty| -> EncounterDef {
        EncounterDef {
            id: EncounterId(id.into()),
            name: name.into(),
            difficulty: diff,
            enemies: vec![],  // battle engine loads from game_data by ID
            xp_reward: Xp::new(0),
            gold_reward: Gold::new(0),
            recruit: None,
            djinn_reward: None,
            equipment_rewards: vec![],
        }
    };

    vec![
        // ── Mercury Lighthouse (DungeonId 0) ─────────────────────────
        DungeonDef {
            id: DungeonId(0),
            name: "Mercury Lighthouse".into(),
            rooms: vec![
                RoomDef {
                    id: RoomId(0),
                    room_type: RoomType::Normal,
                    exits: vec![
                        RoomExit { direction: Direction::Up, target_room: RoomId(1), requires: None },
                    ],
                    encounters: vec![
                        EncounterSlot {
                            encounter: enc("house-01", "Wandering Spirits", Difficulty::Easy),
                            weight: 100,
                            max_triggers: Some(1),
                        },
                    ],
                    items: vec![
                        RoomItem {
                            item_id: ItemId("herb".into()),
                            position: (30.0, 30.0),
                            visible: true,
                            quest_flag: None,
                        },
                    ],
                    puzzles: vec![],
                },
                RoomDef {
                    id: RoomId(1),
                    room_type: RoomType::Puzzle,
                    exits: vec![
                        RoomExit { direction: Direction::Down, target_room: RoomId(0), requires: None },
                        RoomExit { direction: Direction::Up, target_room: RoomId(2), requires: None },
                    ],
                    encounters: vec![],
                    items: vec![],
                    puzzles: vec![
                        PuzzleDef {
                            puzzle_type: PuzzleType::ElementPillar(Element::Mercury),
                            reward: Some(crate::shared::DialogueSideEffect::GiveGold(Gold::new(100))),
                        },
                    ],
                },
                RoomDef {
                    id: RoomId(2),
                    room_type: RoomType::Boss,
                    exits: vec![
                        RoomExit { direction: Direction::Down, target_room: RoomId(1), requires: None },
                    ],
                    encounters: vec![],
                    items: vec![],
                    puzzles: vec![],
                },
            ],
            entry_room: RoomId(0),
            boss_room: Some(RoomId(2)),
        },

        // ── Kolima Forest (DungeonId 1) ──────────────────────────────
        DungeonDef {
            id: DungeonId(1),
            name: "Kolima Forest".into(),
            rooms: vec![
                // Room 0: Forest entrance
                RoomDef {
                    id: RoomId(0),
                    room_type: RoomType::Normal,
                    exits: vec![
                        RoomExit { direction: Direction::Up, target_room: RoomId(1), requires: None },
                        RoomExit { direction: Direction::Right, target_room: RoomId(2), requires: None },
                    ],
                    encounters: vec![
                        EncounterSlot {
                            encounter: enc("house-02", "Forest Spirits", Difficulty::Easy),
                            weight: 100,
                            max_triggers: Some(2),
                        },
                    ],
                    items: vec![
                        RoomItem {
                            item_id: ItemId("antidote".into()),
                            position: (40.0, 20.0),
                            visible: true,
                            quest_flag: None,
                        },
                    ],
                    puzzles: vec![],
                },
                // Room 1: Deep forest — push block puzzle
                RoomDef {
                    id: RoomId(1),
                    room_type: RoomType::Puzzle,
                    exits: vec![
                        RoomExit { direction: Direction::Down, target_room: RoomId(0), requires: None },
                        RoomExit { direction: Direction::Up, target_room: RoomId(3), requires: None },
                    ],
                    encounters: vec![
                        EncounterSlot {
                            encounter: enc("house-03", "Cursed Trees", Difficulty::Medium),
                            weight: 100,
                            max_triggers: Some(1),
                        },
                    ],
                    items: vec![],
                    puzzles: vec![
                        PuzzleDef {
                            puzzle_type: PuzzleType::PushBlock,
                            reward: Some(crate::shared::DialogueSideEffect::GiveItem(
                                ItemId("elixir".into()),
                                crate::shared::bounded_types::ItemCount::new(1),
                            )),
                        },
                    ],
                },
                // Room 2: Hidden grove — treasure room
                RoomDef {
                    id: RoomId(2),
                    room_type: RoomType::Treasure,
                    exits: vec![
                        RoomExit { direction: Direction::Left, target_room: RoomId(0), requires: None },
                    ],
                    encounters: vec![],
                    items: vec![
                        RoomItem {
                            item_id: ItemId("lucky-medal".into()),
                            position: (60.0, 60.0),
                            visible: false,
                            quest_flag: None,
                        },
                        RoomItem {
                            item_id: ItemId("herb".into()),
                            position: (20.0, 40.0),
                            visible: true,
                            quest_flag: None,
                        },
                    ],
                    puzzles: vec![],
                },
                // Room 3: Tret's lair — safe room before boss
                RoomDef {
                    id: RoomId(3),
                    room_type: RoomType::Safe,
                    exits: vec![
                        RoomExit { direction: Direction::Down, target_room: RoomId(1), requires: None },
                        RoomExit { direction: Direction::Up, target_room: RoomId(4), requires: None },
                    ],
                    encounters: vec![],
                    items: vec![
                        RoomItem {
                            item_id: ItemId("herb".into()),
                            position: (50.0, 50.0),
                            visible: true,
                            quest_flag: None,
                        },
                    ],
                    puzzles: vec![],
                },
                // Room 4: Tret boss room
                RoomDef {
                    id: RoomId(4),
                    room_type: RoomType::Boss,
                    exits: vec![
                        RoomExit { direction: Direction::Down, target_room: RoomId(3), requires: None },
                    ],
                    encounters: vec![],
                    items: vec![],
                    puzzles: vec![],
                },
            ],
            entry_room: RoomId(0),
            boss_room: Some(RoomId(4)),
        },
    ]
}

