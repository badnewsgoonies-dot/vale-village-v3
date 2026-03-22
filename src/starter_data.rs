#![allow(dead_code)]
//! Starter game content — Acts 1-3 of the Golden Sun-inspired campaign.
//! 10 map nodes, 4 towns, 4 shops, 5 dungeons, 9 dialogue trees, 6 quests.

use crate::shared::{
    bounded_types::{Gold, ItemCount, EncounterRate},
    DialogueNode, DialogueNodeId, DialogueResponse, DialogueTree, DialogueTreeId,
    DjinnId, Direction, DungeonDef, DungeonId, MapNode, MapNodeId,
    MapNodeType, NpcId, QuestFlagId, QuestStage,
    RoomDef, RoomExit, RoomId, RoomItem, RoomType, ItemId,
    ShopId, TownDef, TownId,
};

// ── Encounter helper (creates stub EncounterDef — battle engine loads real data by ID) ──

fn enc(id: &str, name: &str, diff: crate::shared::Difficulty) -> crate::shared::EncounterDef {
    use crate::shared::{EncounterDef, EncounterId};
    use crate::shared::bounded_types::Xp;
    EncounterDef {
        id: EncounterId(id.into()), name: name.into(), difficulty: diff,
        enemies: vec![], xp_reward: Xp::new(0), gold_reward: Gold::new(0),
        recruit: None, djinn_reward: None, equipment_rewards: vec![],
    }
}

// ══════════════════════════════════════════════════════════════════════
//  WORLD MAP — 10 nodes across 3 acts
// ══════════════════════════════════════════════════════════════════════

pub fn starter_map_nodes() -> Vec<MapNode> {
    vec![
        MapNode { id: MapNodeId(0), name: "Vale Village".into(), position: (100.0, 300.0), node_type: MapNodeType::Town(TownId(0)), connections: vec![MapNodeId(1), MapNodeId(2)] },
        MapNode { id: MapNodeId(1), name: "Mercury Lighthouse".into(), position: (250.0, 150.0), node_type: MapNodeType::Dungeon(DungeonId(0)), connections: vec![MapNodeId(0), MapNodeId(3)] },
        MapNode { id: MapNodeId(2), name: "Kolima Forest".into(), position: (250.0, 450.0), node_type: MapNodeType::Dungeon(DungeonId(1)), connections: vec![MapNodeId(0), MapNodeId(4)] },
        MapNode { id: MapNodeId(3), name: "Imil".into(), position: (400.0, 100.0), node_type: MapNodeType::Town(TownId(1)), connections: vec![MapNodeId(1)] },
        MapNode { id: MapNodeId(4), name: "Mogall Forest".into(), position: (400.0, 450.0), node_type: MapNodeType::Dungeon(DungeonId(2)), connections: vec![MapNodeId(2), MapNodeId(5)] },
        MapNode { id: MapNodeId(5), name: "Kalay".into(), position: (550.0, 350.0), node_type: MapNodeType::Town(TownId(2)), connections: vec![MapNodeId(4), MapNodeId(6)] },
        MapNode { id: MapNodeId(6), name: "Gondowan Passage".into(), position: (700.0, 350.0), node_type: MapNodeType::Dungeon(DungeonId(3)), connections: vec![MapNodeId(5), MapNodeId(7)] },
        MapNode { id: MapNodeId(7), name: "Tolbi".into(), position: (850.0, 300.0), node_type: MapNodeType::Town(TownId(3)), connections: vec![MapNodeId(6), MapNodeId(8)] },
        MapNode { id: MapNodeId(8), name: "Altmiller Cave".into(), position: (850.0, 150.0), node_type: MapNodeType::Dungeon(DungeonId(4)), connections: vec![MapNodeId(7), MapNodeId(9)] },
        MapNode { id: MapNodeId(9), name: "Suhalla Gate".into(), position: (1000.0, 200.0), node_type: MapNodeType::Landmark, connections: vec![MapNodeId(8)] },
    ]
}

// ══════════════════════════════════════════════════════════════════════
//  TOWNS — 4 towns
// ══════════════════════════════════════════════════════════════════════

pub fn starter_towns() -> Vec<TownDef> {
    use crate::shared::{DialogueTreeId, NpcPlacement, DjinnDiscoveryPoint};
    vec![
        TownDef {
            id: TownId(0), name: "Vale Village".into(),
            npcs: vec![
                NpcPlacement { npc_id: NpcId(0), position: (50.0, 50.0), facing: Direction::Down, dialogue_tree: DialogueTreeId(0) },
                NpcPlacement { npc_id: NpcId(1), position: (80.0, 60.0), facing: Direction::Left, dialogue_tree: DialogueTreeId(1) },
            ],
            shops: vec![ShopId(0)],
            djinn_points: vec![DjinnDiscoveryPoint { djinn_id: DjinnId("flint".into()), position: (30.0, 90.0), requires_puzzle: false, quest_flag: None }],
            exits: vec![MapNodeId(0)],
        },
        TownDef {
            id: TownId(1), name: "Imil".into(),
            npcs: vec![
                NpcPlacement { npc_id: NpcId(2), position: (40.0, 40.0), facing: Direction::Down, dialogue_tree: DialogueTreeId(2) },
                NpcPlacement { npc_id: NpcId(3), position: (70.0, 30.0), facing: Direction::Right, dialogue_tree: DialogueTreeId(3) },
            ],
            shops: vec![ShopId(1)],
            djinn_points: vec![DjinnDiscoveryPoint { djinn_id: DjinnId("fizz".into()), position: (120.0, 80.0), requires_puzzle: false, quest_flag: None }],
            exits: vec![MapNodeId(3)],
        },
        TownDef {
            id: TownId(2), name: "Kalay".into(),
            npcs: vec![
                NpcPlacement { npc_id: NpcId(4), position: (60.0, 40.0), facing: Direction::Down, dialogue_tree: DialogueTreeId(4) },
                NpcPlacement { npc_id: NpcId(5), position: (90.0, 50.0), facing: Direction::Left, dialogue_tree: DialogueTreeId(5) },
            ],
            shops: vec![ShopId(2)],
            djinn_points: vec![DjinnDiscoveryPoint { djinn_id: DjinnId("forge".into()), position: (110.0, 90.0), requires_puzzle: true, quest_flag: Some(QuestFlagId(3)) }],
            exits: vec![MapNodeId(5)],
        },
        TownDef {
            id: TownId(3), name: "Tolbi".into(),
            npcs: vec![
                NpcPlacement { npc_id: NpcId(6), position: (50.0, 50.0), facing: Direction::Down, dialogue_tree: DialogueTreeId(6) },
                NpcPlacement { npc_id: NpcId(7), position: (80.0, 40.0), facing: Direction::Right, dialogue_tree: DialogueTreeId(7) },
                NpcPlacement { npc_id: NpcId(8), position: (110.0, 60.0), facing: Direction::Left, dialogue_tree: DialogueTreeId(8) },
            ],
            shops: vec![ShopId(3)],
            djinn_points: vec![DjinnDiscoveryPoint { djinn_id: DjinnId("breeze".into()), position: (130.0, 100.0), requires_puzzle: false, quest_flag: Some(QuestFlagId(5)) }],
            exits: vec![MapNodeId(7)],
        },
    ]
}

// ══════════════════════════════════════════════════════════════════════
//  SHOPS — 4 with escalating equipment
// ══════════════════════════════════════════════════════════════════════

pub fn starter_shop_defs() -> Vec<crate::shared::ShopDef> {
    use crate::shared::{ShopDef, ShopEntry, ShopStock};
    vec![
        ShopDef { id: ShopId(0), name: "Vale General Store".into(), inventory: vec![
            ShopEntry { item_id: ItemId("herb".into()), price: Gold::new(10), stock: ShopStock::Unlimited },
            ShopEntry { item_id: ItemId("antidote".into()), price: Gold::new(15), stock: ShopStock::Unlimited },
            ShopEntry { item_id: ItemId("short-sword".into()), price: Gold::new(120), stock: ShopStock::Limited(ItemCount::new(2)) },
            ShopEntry { item_id: ItemId("leather-cap".into()), price: Gold::new(80), stock: ShopStock::Unlimited },
        ]},
        ShopDef { id: ShopId(1), name: "Imil Outfitters".into(), inventory: vec![
            ShopEntry { item_id: ItemId("herb".into()), price: Gold::new(10), stock: ShopStock::Unlimited },
            ShopEntry { item_id: ItemId("frost-blade".into()), price: Gold::new(500), stock: ShopStock::Limited(ItemCount::new(1)) },
            ShopEntry { item_id: ItemId("fur-coat".into()), price: Gold::new(200), stock: ShopStock::Unlimited },
            ShopEntry { item_id: ItemId("elixir".into()), price: Gold::new(100), stock: ShopStock::Unlimited },
        ]},
        ShopDef { id: ShopId(2), name: "Kalay Armory".into(), inventory: vec![
            ShopEntry { item_id: ItemId("herb".into()), price: Gold::new(10), stock: ShopStock::Unlimited },
            ShopEntry { item_id: ItemId("elixir".into()), price: Gold::new(100), stock: ShopStock::Unlimited },
            ShopEntry { item_id: ItemId("claymore".into()), price: Gold::new(1200), stock: ShopStock::Limited(ItemCount::new(1)) },
            ShopEntry { item_id: ItemId("silver-mail".into()), price: Gold::new(800), stock: ShopStock::Unlimited },
            ShopEntry { item_id: ItemId("iron-shield".into()), price: Gold::new(600), stock: ShopStock::Unlimited },
        ]},
        ShopDef { id: ShopId(3), name: "Tolbi Bazaar".into(), inventory: vec![
            ShopEntry { item_id: ItemId("elixir".into()), price: Gold::new(100), stock: ShopStock::Unlimited },
            ShopEntry { item_id: ItemId("water-of-life".into()), price: Gold::new(500), stock: ShopStock::Limited(ItemCount::new(3)) },
            ShopEntry { item_id: ItemId("gaia-blade".into()), price: Gold::new(3000), stock: ShopStock::Limited(ItemCount::new(1)) },
            ShopEntry { item_id: ItemId("spirit-armor".into()), price: Gold::new(2000), stock: ShopStock::Limited(ItemCount::new(1)) },
        ]},
    ]
}

// ══════════════════════════════════════════════════════════════════════
//  DIALOGUE TREES — 9 NPCs
// ══════════════════════════════════════════════════════════════════════

pub fn starter_dialogue_trees() -> Vec<DialogueTree> {
    vec![
        // NPC 0: Vale Elder — Mercury quest
        DialogueTree { id: DialogueTreeId(0), root: DialogueNodeId(0), nodes: vec![
            DialogueNode { id: DialogueNodeId(0), speaker: Some(NpcId(0)),
                text: "Welcome, young adept. Dark forces stir at Mercury Lighthouse. Will you investigate?".into(),
                responses: vec![
                    DialogueResponse { text: "I'll go at once.".into(), condition: None, next_node: Some(DialogueNodeId(1)),
                        side_effects: vec![crate::shared::DialogueSideEffect::SetQuestStage(QuestFlagId(0), QuestStage::Active)] },
                    DialogueResponse { text: "Tell me more.".into(), condition: None, next_node: Some(DialogueNodeId(2)), side_effects: vec![] },
                ] },
            DialogueNode { id: DialogueNodeId(1), speaker: Some(NpcId(0)),
                text: "Brave soul. Take this gold for supplies.".into(),
                responses: vec![DialogueResponse { text: "[Accept]".into(), condition: None, next_node: None,
                    side_effects: vec![crate::shared::DialogueSideEffect::GiveGold(Gold::new(200))] }] },
            DialogueNode { id: DialogueNodeId(2), speaker: Some(NpcId(0)),
                text: "Saturos seeks to light the beacon. If he succeeds, the world will change forever.".into(),
                responses: vec![DialogueResponse { text: "I'll stop them.".into(), condition: None, next_node: Some(DialogueNodeId(1)),
                    side_effects: vec![crate::shared::DialogueSideEffect::SetQuestStage(QuestFlagId(0), QuestStage::Active)] }] },
        ]},
        // NPC 1: Vale Villager — hint
        DialogueTree { id: DialogueTreeId(1), root: DialogueNodeId(0), nodes: vec![
            DialogueNode { id: DialogueNodeId(0), speaker: Some(NpcId(1)),
                text: "The general store has herbs and equipment. I heard there's a djinn hiding near the creek south of town.".into(),
                responses: vec![DialogueResponse { text: "Thanks!".into(), condition: None, next_node: None, side_effects: vec![] }] },
        ]},
        // NPC 2: Imil Healer — healing + Kolima quest
        DialogueTree { id: DialogueTreeId(2), root: DialogueNodeId(0), nodes: vec![
            DialogueNode { id: DialogueNodeId(0), speaker: Some(NpcId(2)),
                text: "You look weary. Let me heal your wounds.".into(),
                responses: vec![
                    DialogueResponse { text: "Please.".into(), condition: None, next_node: Some(DialogueNodeId(1)),
                        side_effects: vec![crate::shared::DialogueSideEffect::Heal] },
                    DialogueResponse { text: "Tell me about Kolima.".into(), condition: None, next_node: Some(DialogueNodeId(2)), side_effects: vec![] },
                ] },
            DialogueNode { id: DialogueNodeId(1), speaker: Some(NpcId(2)),
                text: "Rest well. The forest south has been cursed — the trees themselves are alive.".into(),
                responses: vec![DialogueResponse { text: "I'll investigate.".into(), condition: None, next_node: None,
                    side_effects: vec![crate::shared::DialogueSideEffect::SetQuestStage(QuestFlagId(1), QuestStage::Discovered)] }] },
            DialogueNode { id: DialogueNodeId(2), speaker: Some(NpcId(2)),
                text: "Kolima's people turned to trees. Tret was corrupted. You'll need Mercury Psynergy to reach him.".into(),
                responses: vec![DialogueResponse { text: "I'll find a way.".into(), condition: None, next_node: None,
                    side_effects: vec![crate::shared::DialogueSideEffect::SetQuestStage(QuestFlagId(1), QuestStage::Active)] }] },
        ]},
        // NPC 3: Imil Scholar — Mogall hint
        DialogueTree { id: DialogueTreeId(3), root: DialogueNodeId(0), nodes: vec![
            DialogueNode { id: DialogueNodeId(0), speaker: Some(NpcId(3)),
                text: "Beyond Kolima lies the Mogall Forest — a twisted maze. An ancient spirit guards the passage to Kalay.".into(),
                responses: vec![DialogueResponse { text: "Sounds dangerous.".into(), condition: None, next_node: None, side_effects: vec![] }] },
        ]},
        // NPC 4: Kalay Lord — Gondowan quest
        DialogueTree { id: DialogueTreeId(4), root: DialogueNodeId(0), nodes: vec![
            DialogueNode { id: DialogueNodeId(0), speaker: Some(NpcId(4)),
                text: "Welcome to Kalay. The passage to Gondowan is blocked by cave-ins. We need earth Psynergy to clear it.".into(),
                responses: vec![
                    DialogueResponse { text: "I can clear it.".into(), condition: None, next_node: Some(DialogueNodeId(1)),
                        side_effects: vec![
                            crate::shared::DialogueSideEffect::SetQuestStage(QuestFlagId(2), QuestStage::Active),
                            crate::shared::DialogueSideEffect::UnlockMapNode(MapNodeId(6)),
                        ] },
                    DialogueResponse { text: "I need to prepare.".into(), condition: None, next_node: None,
                        side_effects: vec![crate::shared::DialogueSideEffect::SetQuestStage(QuestFlagId(2), QuestStage::Discovered)] },
                ] },
            DialogueNode { id: DialogueNodeId(1), speaker: Some(NpcId(4)),
                text: "Take this gold. The passage is east of town — monsters nest in the rubble.".into(),
                responses: vec![DialogueResponse { text: "[Accept]".into(), condition: None, next_node: None,
                    side_effects: vec![crate::shared::DialogueSideEffect::GiveGold(Gold::new(500))] }] },
        ]},
        // NPC 5: Kalay Blacksmith — forge djinn hint
        DialogueTree { id: DialogueTreeId(5), root: DialogueNodeId(0), nodes: vec![
            DialogueNode { id: DialogueNodeId(0), speaker: Some(NpcId(5)),
                text: "My forge burns hotter than it should — could be a Mars djinn hiding in the furnace. Clear the Gondowan Passage first, then I'll trust you with my forge.".into(),
                responses: vec![
                    DialogueResponse { text: "Deal.".into(), condition: None, next_node: None,
                        side_effects: vec![crate::shared::DialogueSideEffect::SetQuestStage(QuestFlagId(3), QuestStage::Discovered)] },
                ] },
        ]},
        // NPC 6: Tolbi Mayor — Colosso tournament
        DialogueTree { id: DialogueTreeId(6), root: DialogueNodeId(0), nodes: vec![
            DialogueNode { id: DialogueNodeId(0), speaker: Some(NpcId(6)),
                text: "Tolbi's Colosso tournament begins soon! Champions compete for the Sol Blade and 5,000 gold. Will you enter?".into(),
                responses: vec![
                    DialogueResponse { text: "Sign me up!".into(), condition: None, next_node: Some(DialogueNodeId(1)),
                        side_effects: vec![crate::shared::DialogueSideEffect::SetQuestStage(QuestFlagId(4), QuestStage::Active)] },
                    DialogueResponse { text: "What's the qualifying round?".into(), condition: None, next_node: Some(DialogueNodeId(2)), side_effects: vec![] },
                ] },
            DialogueNode { id: DialogueNodeId(1), speaker: Some(NpcId(6)),
                text: "The Altmiller Cave north of town is the qualifying round. Defeat the guardians to earn your place.".into(),
                responses: vec![DialogueResponse { text: "I won't let you down.".into(), condition: None, next_node: None,
                    side_effects: vec![crate::shared::DialogueSideEffect::UnlockMapNode(MapNodeId(8))] }] },
            DialogueNode { id: DialogueNodeId(2), speaker: Some(NpcId(6)),
                text: "The champion earns the Sol Blade and passage to Suhalla Gate. The Altmiller Cave is the qualifier.".into(),
                responses: vec![DialogueResponse { text: "I'll enter.".into(), condition: None, next_node: Some(DialogueNodeId(1)),
                    side_effects: vec![crate::shared::DialogueSideEffect::SetQuestStage(QuestFlagId(4), QuestStage::Active)] }] },
        ]},
        // NPC 7: Tolbi Innkeeper — lore
        DialogueTree { id: DialogueTreeId(7), root: DialogueNodeId(0), nodes: vec![
            DialogueNode { id: DialogueNodeId(0), speaker: Some(NpcId(7)),
                text: "The Altmiller Cave is brutal — monsters far stronger than the northern lands. Make sure your djinn are set and equipment is top-notch.".into(),
                responses: vec![DialogueResponse { text: "I'll be ready.".into(), condition: None, next_node: None, side_effects: vec![] }] },
        ]},
        // NPC 8: Tolbi Sage — Jupiter djinn hint
        DialogueTree { id: DialogueTreeId(8), root: DialogueNodeId(0), nodes: vec![
            DialogueNode { id: DialogueNodeId(0), speaker: Some(NpcId(8)),
                text: "The winds whisper of a Jupiter djinn that appears to Colosso champions. Win the tournament, and Breeze may find you worthy.".into(),
                responses: vec![DialogueResponse { text: "I'll keep that in mind.".into(), condition: None, next_node: None,
                    side_effects: vec![crate::shared::DialogueSideEffect::SetQuestStage(QuestFlagId(5), QuestStage::Discovered)] }] },
        ]},
    ]
}

// ══════════════════════════════════════════════════════════════════════
//  DUNGEONS — 5 dungeons, 24 rooms
// ══════════════════════════════════════════════════════════════════════

pub fn starter_dungeons() -> Vec<DungeonDef> {
    use crate::shared::{Difficulty, EncounterSlot, PuzzleDef, PuzzleType, Element};
    vec![
        // ══ Mercury Lighthouse (Act 1) — 3 rooms ══
        DungeonDef { id: DungeonId(0), name: "Mercury Lighthouse".into(), entry_room: RoomId(0), boss_room: Some(RoomId(2)), rooms: vec![
            RoomDef { id: RoomId(0), room_type: RoomType::Normal,
                exits: vec![RoomExit { direction: Direction::Up, target_room: RoomId(1), requires: None }],
                encounters: vec![EncounterSlot { encounter: enc("house-01", "Wandering Spirits", Difficulty::Easy), weight: 100, max_triggers: Some(1) }],
                items: vec![RoomItem { item_id: ItemId("herb".into()), position: (30.0, 30.0), visible: true, quest_flag: None }],
                puzzles: vec![] },
            RoomDef { id: RoomId(1), room_type: RoomType::Puzzle,
                exits: vec![RoomExit { direction: Direction::Down, target_room: RoomId(0), requires: None }, RoomExit { direction: Direction::Up, target_room: RoomId(2), requires: None }],
                encounters: vec![], items: vec![],
                puzzles: vec![PuzzleDef { puzzle_type: PuzzleType::ElementPillar(Element::Mercury), reward: Some(crate::shared::DialogueSideEffect::GiveGold(Gold::new(100))) }] },
            RoomDef { id: RoomId(2), room_type: RoomType::Boss,
                exits: vec![RoomExit { direction: Direction::Down, target_room: RoomId(1), requires: None }],
                encounters: vec![], items: vec![], puzzles: vec![] },
        ]},
        // ══ Kolima Forest (Act 1) — 5 rooms ══
        DungeonDef { id: DungeonId(1), name: "Kolima Forest".into(), entry_room: RoomId(0), boss_room: Some(RoomId(4)), rooms: vec![
            RoomDef { id: RoomId(0), room_type: RoomType::Normal,
                exits: vec![RoomExit { direction: Direction::Up, target_room: RoomId(1), requires: None }, RoomExit { direction: Direction::Right, target_room: RoomId(2), requires: None }],
                encounters: vec![EncounterSlot { encounter: enc("house-02", "Forest Spirits", Difficulty::Easy), weight: 100, max_triggers: Some(2) }],
                items: vec![RoomItem { item_id: ItemId("antidote".into()), position: (40.0, 20.0), visible: true, quest_flag: None }],
                puzzles: vec![] },
            RoomDef { id: RoomId(1), room_type: RoomType::Puzzle,
                exits: vec![RoomExit { direction: Direction::Down, target_room: RoomId(0), requires: None }, RoomExit { direction: Direction::Up, target_room: RoomId(3), requires: None }],
                encounters: vec![EncounterSlot { encounter: enc("house-03", "Cursed Trees", Difficulty::Easy), weight: 100, max_triggers: Some(1) }],
                items: vec![],
                puzzles: vec![PuzzleDef { puzzle_type: PuzzleType::PushBlock, reward: Some(crate::shared::DialogueSideEffect::GiveItem(ItemId("elixir".into()), ItemCount::new(1))) }] },
            RoomDef { id: RoomId(2), room_type: RoomType::Treasure,
                exits: vec![RoomExit { direction: Direction::Left, target_room: RoomId(0), requires: None }],
                encounters: vec![],
                items: vec![
                    RoomItem { item_id: ItemId("lucky-medal".into()), position: (60.0, 60.0), visible: false, quest_flag: None },
                    RoomItem { item_id: ItemId("herb".into()), position: (20.0, 40.0), visible: true, quest_flag: None },
                ], puzzles: vec![] },
            RoomDef { id: RoomId(3), room_type: RoomType::Safe,
                exits: vec![RoomExit { direction: Direction::Down, target_room: RoomId(1), requires: None }, RoomExit { direction: Direction::Up, target_room: RoomId(4), requires: None }],
                encounters: vec![],
                items: vec![RoomItem { item_id: ItemId("herb".into()), position: (50.0, 50.0), visible: true, quest_flag: None }],
                puzzles: vec![] },
            RoomDef { id: RoomId(4), room_type: RoomType::Boss,
                exits: vec![RoomExit { direction: Direction::Down, target_room: RoomId(3), requires: None }],
                encounters: vec![], items: vec![], puzzles: vec![] },
        ]},
        // ══ Mogall Forest (Act 2) — 6 rooms ══
        DungeonDef { id: DungeonId(2), name: "Mogall Forest".into(), entry_room: RoomId(0), boss_room: Some(RoomId(5)), rooms: vec![
            RoomDef { id: RoomId(0), room_type: RoomType::Normal,
                exits: vec![RoomExit { direction: Direction::Right, target_room: RoomId(1), requires: None }, RoomExit { direction: Direction::Up, target_room: RoomId(2), requires: None }],
                encounters: vec![EncounterSlot { encounter: enc("house-06", "Mogall Lurkers", Difficulty::Medium), weight: 100, max_triggers: Some(2) }],
                items: vec![RoomItem { item_id: ItemId("herb".into()), position: (20.0, 30.0), visible: true, quest_flag: None }],
                puzzles: vec![] },
            RoomDef { id: RoomId(1), room_type: RoomType::Puzzle,
                exits: vec![RoomExit { direction: Direction::Left, target_room: RoomId(0), requires: None }, RoomExit { direction: Direction::Up, target_room: RoomId(3), requires: None }],
                encounters: vec![], items: vec![],
                puzzles: vec![PuzzleDef { puzzle_type: PuzzleType::SwitchSequence, reward: Some(crate::shared::DialogueSideEffect::GiveGold(Gold::new(300))) }] },
            RoomDef { id: RoomId(2), room_type: RoomType::Normal,
                exits: vec![RoomExit { direction: Direction::Down, target_room: RoomId(0), requires: None }, RoomExit { direction: Direction::Right, target_room: RoomId(3), requires: None }],
                encounters: vec![EncounterSlot { encounter: enc("house-07", "Thorned Sentinels", Difficulty::Medium), weight: 100, max_triggers: Some(1) }],
                items: vec![RoomItem { item_id: ItemId("elixir".into()), position: (50.0, 10.0), visible: true, quest_flag: None }],
                puzzles: vec![] },
            RoomDef { id: RoomId(3), room_type: RoomType::MiniBoss,
                exits: vec![RoomExit { direction: Direction::Left, target_room: RoomId(2), requires: None }, RoomExit { direction: Direction::Down, target_room: RoomId(1), requires: None }, RoomExit { direction: Direction::Up, target_room: RoomId(4), requires: None }],
                encounters: vec![EncounterSlot { encounter: enc("house-08", "Mogall Beast", Difficulty::Medium), weight: 100, max_triggers: Some(1) }],
                items: vec![], puzzles: vec![] },
            RoomDef { id: RoomId(4), room_type: RoomType::Treasure,
                exits: vec![RoomExit { direction: Direction::Down, target_room: RoomId(3), requires: None }, RoomExit { direction: Direction::Right, target_room: RoomId(5), requires: None }],
                encounters: vec![],
                items: vec![
                    RoomItem { item_id: ItemId("elixir".into()), position: (30.0, 30.0), visible: true, quest_flag: None },
                    RoomItem { item_id: ItemId("water-of-life".into()), position: (60.0, 40.0), visible: false, quest_flag: None },
                ], puzzles: vec![] },
            RoomDef { id: RoomId(5), room_type: RoomType::Boss,
                exits: vec![RoomExit { direction: Direction::Left, target_room: RoomId(4), requires: None }],
                encounters: vec![], items: vec![], puzzles: vec![] },
        ]},
        // ══ Gondowan Passage (Act 2) — 4 rooms ══
        DungeonDef { id: DungeonId(3), name: "Gondowan Passage".into(), entry_room: RoomId(0), boss_room: Some(RoomId(3)), rooms: vec![
            RoomDef { id: RoomId(0), room_type: RoomType::Normal,
                exits: vec![RoomExit { direction: Direction::Right, target_room: RoomId(1), requires: None }],
                encounters: vec![EncounterSlot { encounter: enc("house-09", "Cave Bats", Difficulty::Medium), weight: 100, max_triggers: Some(2) }],
                items: vec![RoomItem { item_id: ItemId("herb".into()), position: (15.0, 25.0), visible: true, quest_flag: None }],
                puzzles: vec![] },
            RoomDef { id: RoomId(1), room_type: RoomType::Puzzle,
                exits: vec![RoomExit { direction: Direction::Left, target_room: RoomId(0), requires: None }, RoomExit { direction: Direction::Right, target_room: RoomId(2), requires: None }],
                encounters: vec![], items: vec![],
                puzzles: vec![
                    PuzzleDef { puzzle_type: PuzzleType::ElementPillar(Element::Venus), reward: Some(crate::shared::DialogueSideEffect::GiveGold(Gold::new(200))) },
                    PuzzleDef { puzzle_type: PuzzleType::PushBlock, reward: Some(crate::shared::DialogueSideEffect::GiveItem(ItemId("elixir".into()), ItemCount::new(2))) },
                ] },
            RoomDef { id: RoomId(2), room_type: RoomType::Normal,
                exits: vec![RoomExit { direction: Direction::Left, target_room: RoomId(1), requires: None }, RoomExit { direction: Direction::Right, target_room: RoomId(3), requires: None }],
                encounters: vec![EncounterSlot { encounter: enc("house-10", "Rock Golems", Difficulty::Medium), weight: 100, max_triggers: Some(1) }],
                items: vec![RoomItem { item_id: ItemId("elixir".into()), position: (40.0, 50.0), visible: true, quest_flag: None }],
                puzzles: vec![] },
            RoomDef { id: RoomId(3), room_type: RoomType::Boss,
                exits: vec![RoomExit { direction: Direction::Left, target_room: RoomId(2), requires: None }],
                encounters: vec![], items: vec![], puzzles: vec![] },
        ]},
        // ══ Altmiller Cave (Act 3) — 6 rooms ══
        DungeonDef { id: DungeonId(4), name: "Altmiller Cave".into(), entry_room: RoomId(0), boss_room: Some(RoomId(5)), rooms: vec![
            RoomDef { id: RoomId(0), room_type: RoomType::Normal,
                exits: vec![RoomExit { direction: Direction::Up, target_room: RoomId(1), requires: None }, RoomExit { direction: Direction::Right, target_room: RoomId(2), requires: None }],
                encounters: vec![EncounterSlot { encounter: enc("house-11", "Crystal Sentinels", Difficulty::Hard), weight: 100, max_triggers: Some(2) }],
                items: vec![RoomItem { item_id: ItemId("elixir".into()), position: (25.0, 25.0), visible: true, quest_flag: None }],
                puzzles: vec![] },
            RoomDef { id: RoomId(1), room_type: RoomType::Puzzle,
                exits: vec![RoomExit { direction: Direction::Down, target_room: RoomId(0), requires: None }, RoomExit { direction: Direction::Up, target_room: RoomId(3), requires: None }],
                encounters: vec![], items: vec![],
                puzzles: vec![PuzzleDef { puzzle_type: PuzzleType::IceSlide, reward: Some(crate::shared::DialogueSideEffect::GiveGold(Gold::new(500))) }] },
            RoomDef { id: RoomId(2), room_type: RoomType::Treasure,
                exits: vec![RoomExit { direction: Direction::Left, target_room: RoomId(0), requires: None }],
                encounters: vec![],
                items: vec![
                    RoomItem { item_id: ItemId("water-of-life".into()), position: (50.0, 50.0), visible: true, quest_flag: None },
                    RoomItem { item_id: ItemId("lucky-medal".into()), position: (30.0, 20.0), visible: false, quest_flag: None },
                ], puzzles: vec![] },
            RoomDef { id: RoomId(3), room_type: RoomType::MiniBoss,
                exits: vec![RoomExit { direction: Direction::Down, target_room: RoomId(1), requires: None }, RoomExit { direction: Direction::Up, target_room: RoomId(4), requires: None }],
                encounters: vec![EncounterSlot { encounter: enc("house-13", "Cave Warden", Difficulty::Hard), weight: 100, max_triggers: Some(1) }],
                items: vec![], puzzles: vec![] },
            RoomDef { id: RoomId(4), room_type: RoomType::Safe,
                exits: vec![RoomExit { direction: Direction::Down, target_room: RoomId(3), requires: None }, RoomExit { direction: Direction::Up, target_room: RoomId(5), requires: None }],
                encounters: vec![],
                items: vec![RoomItem { item_id: ItemId("herb".into()), position: (40.0, 40.0), visible: true, quest_flag: None }, RoomItem { item_id: ItemId("elixir".into()), position: (60.0, 40.0), visible: true, quest_flag: None }],
                puzzles: vec![] },
            RoomDef { id: RoomId(5), room_type: RoomType::Boss,
                exits: vec![RoomExit { direction: Direction::Down, target_room: RoomId(4), requires: None }],
                encounters: vec![], items: vec![], puzzles: vec![] },
        ]},
    ]
}

// ══════════════════════════════════════════════════════════════════════
//  OVERWORLD ENCOUNTER TABLE
// ══════════════════════════════════════════════════════════════════════

pub fn overworld_encounter_table() -> crate::shared::EncounterTable {
    use crate::shared::{EncounterSlot, EncounterTable, Difficulty};
    EncounterTable {
        region_id: 0, base_rate: EncounterRate::new(3),
        entries: vec![
            EncounterSlot { encounter: enc("house-01", "Wandering Monsters", Difficulty::Easy), weight: 60, max_triggers: None },
            EncounterSlot { encounter: enc("house-04", "Road Bandits", Difficulty::Easy), weight: 40, max_triggers: None },
        ],
    }
}
