#![allow(dead_code)]
//! CLI Game Loop — text-based adventure runner integrating all beyond-battle domains.
//! Orchestrator-owned. This is the integration layer connecting screens, world map,
//! towns, shops, dungeons, quests, dialogue, and encounters into a playable loop.

use std::io::{self, Write};

use crate::game_state::{self, GameState};
use crate::shared::{
    bounded_types::{Gold, ItemCount},
    DialogueNode, DialogueNodeId, DialogueResponse, DialogueTree, DialogueTreeId,
    DjinnId, Direction, DungeonDef, DungeonId, GameScreen, MapNode, MapNodeId,
    MapNodeType, MenuScreen, NodeUnlockState, NpcId, QuestFlagId, QuestStage,
    RoomDef, RoomExit, RoomId, RoomItem, RoomType, ItemId, UnitId,
    ScreenTransition, ShopId, TownDef, TownId,
};

use crate::domains::dialogue::{self, ConditionContext};
use crate::domains::dungeon;
use crate::domains::puzzle;
use crate::domains::cli_runner;
use crate::domains::battle_engine::BattleResult;
use crate::domains::data_loader::GameData;
use crate::domains::quest::QuestManager;
use crate::domains::save;
use crate::domains::shop;
use crate::domains::world_map;
use crate::domains::progression;

// ── Starter Data (hardcoded until RON loader is built) ──────────────

fn starter_map_nodes() -> Vec<MapNode> {
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

fn starter_towns() -> Vec<TownDef> {
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

fn starter_shop_defs() -> Vec<crate::shared::ShopDef> {
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

fn starter_dialogue_trees() -> Vec<DialogueTree> {
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

fn starter_dungeons() -> Vec<DungeonDef> {
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

// ── Runtime ConditionContext ──────────────────────────────────────────

/// Snapshot-based context to avoid holding borrows across mutations.
struct SnapshotCtx {
    inventory: Vec<ItemId>,
    quest_flags: std::collections::HashMap<QuestFlagId, QuestStage>,
    gold: u32,
}

impl SnapshotCtx {
    fn from_state(state: &GameState, inventory: &[ItemId]) -> Self {
        Self {
            inventory: inventory.to_vec(),
            quest_flags: state.quest_state.flags.clone(),
            gold: state.gold.get(),
        }
    }
}

impl ConditionContext for SnapshotCtx {
    fn has_item(&self, item: &ItemId) -> bool {
        self.inventory.contains(item)
    }
    fn has_djinn(&self, _djinn: &DjinnId) -> bool {
        false // TODO: check party djinn
    }
    fn quest_at_stage(&self, flag: &QuestFlagId, stage: QuestStage) -> bool {
        self.quest_flags.get(flag).copied().unwrap_or(QuestStage::Unknown) >= stage
    }
    fn gold_at_least(&self, amount: Gold) -> bool {
        self.gold >= amount.get()
    }
    fn party_contains(&self, _unit: &UnitId) -> bool {
        true // TODO: check party roster
    }
}

impl puzzle::PuzzleContext for SnapshotCtx {
    fn has_djinn(&self, _id: &DjinnId) -> bool {
        false // TODO
    }
    fn has_element_ability(&self, _element: crate::shared::Element) -> bool {
        true // Assume party has all elements for now
    }
}

fn prompt(msg: &str) -> String {
    print!("{}", msg);
    io::stdout().flush().ok();
    let mut input = String::new();
    io::stdin().read_line(&mut input).unwrap_or(0);
    input.trim().to_string()
}

fn prompt_number(msg: &str, max: usize) -> Option<usize> {
    let input = prompt(msg);
    input.parse::<usize>().ok().filter(|&n| n < max)
}

// ── Main Game Loop ───────────────────────────────────────────────────

pub fn run_game_loop(state: &mut GameState, game_data: &GameData, save_data: &mut save::SaveData) {
    // Set up world map
    let nodes = starter_map_nodes();
    let mut wm = world_map::load_map(nodes);
    // Unlock starting nodes
    world_map::unlock_node(&mut wm, MapNodeId(0));
    world_map::unlock_node(&mut wm, MapNodeId(0)); // Visible → Unlocked
    world_map::unlock_node(&mut wm, MapNodeId(1));
    world_map::unlock_node(&mut wm, MapNodeId(1));
    world_map::unlock_node(&mut wm, MapNodeId(2));
    world_map::unlock_node(&mut wm, MapNodeId(2));
    state.world_map = Some(wm);

    let towns = starter_towns();
    let shop_defs = starter_shop_defs();
    let dialogue_trees = starter_dialogue_trees();
    let dungeons = starter_dungeons();
    let mut inventory: Vec<ItemId> = vec![];
    state.shop_state = shop::init_shop_state(&shop_defs);

    // Start at title
    game_state::apply_transition(state, ScreenTransition::ToTitle);

    println!("\n╔══════════════════════════════════════╗");
    println!("║       VALE VILLAGE v3                ║");
    println!("║   A Golden Sun-inspired RPG          ║");
    println!("╚══════════════════════════════════════╝\n");

    loop {
        match &state.screen {
            GameScreen::Title => {
                println!("[Title Screen]");
                let choice = prompt("Press ENTER to start, or 'q' to quit: ");
                if choice == "q" {
                    break;
                }
                game_state::apply_transition(state, ScreenTransition::ToWorldMap);
            }

            GameScreen::WorldMap => {
                run_world_map(state, &towns);
            }

            GameScreen::Town(town_id) => {
                let tid = *town_id;
                run_town(state, tid, &towns, &shop_defs);
            }

            GameScreen::Shop(shop_id) => {
                let sid = *shop_id;
                run_shop(state, sid, &shop_defs);
            }

            GameScreen::Dungeon(dungeon_id) => {
                let did = *dungeon_id;
                run_dungeon(state, did, &dungeons, &mut inventory, game_data, save_data);
            }

            GameScreen::Menu(menu) => {
                println!("[Menu: {:?}] (not yet implemented)", menu);
                game_state::apply_transition(state, ScreenTransition::ReturnToPrevious);
            }

            GameScreen::Battle => {
                println!("[Battle] (use --cli for battle mode)");
                game_state::apply_transition(state, ScreenTransition::ReturnToPrevious);
            }

            GameScreen::SaveLoad => {
                println!("[Save/Load] (not yet implemented)");
                game_state::apply_transition(state, ScreenTransition::ReturnToPrevious);
            }

            GameScreen::GameOver => {
                println!("\n=== GAME OVER ===");
                break;
            }

            GameScreen::Victory => {
                println!("\n=== VICTORY! You've completed Vale Village! ===");
                break;
            }

            GameScreen::Dialogue(npc_id) => {
                let nid = *npc_id;
                run_dialogue(state, nid, &dialogue_trees, &towns, &mut inventory);
            }
        }
    }
}

fn run_world_map(state: &mut GameState, towns: &[TownDef]) {
    let wm = state.world_map.as_ref().expect("world map not loaded");
    let accessible = world_map::get_accessible_nodes(wm);

    println!("\n── World Map ──");
    for (i, node) in accessible.iter().enumerate() {
        let type_str = match &node.node_type {
            MapNodeType::Town(_) => "Town",
            MapNodeType::Dungeon(_) => "Dungeon",
            MapNodeType::Landmark => "Landmark",
            MapNodeType::Hidden => "???",
        };
        println!("  [{}] {} ({})", i, node.name, type_str);
    }
    println!("  [m] Menu  [s] Save  [q] Quit");

    let input = prompt("> ");
    match input.as_str() {
        "q" => {
            game_state::apply_transition(state, ScreenTransition::ToTitle);
        }
        "m" => {
            game_state::apply_transition(state, ScreenTransition::OpenMenu(MenuScreen::Party));
        }
        "s" => {
            game_state::apply_transition(state, ScreenTransition::OpenSaveLoad);
        }
        other => {
            if let Ok(idx) = other.parse::<usize>() {
                if idx < accessible.len() {
                    let node = &accessible[idx];
                    match &node.node_type {
                        MapNodeType::Town(tid) => {
                            game_state::apply_transition(state, ScreenTransition::EnterTown(*tid));
                        }
                        MapNodeType::Dungeon(did) => {
                            game_state::apply_transition(state, ScreenTransition::EnterDungeon(*did));
                        }
                        _ => println!("Nothing here yet."),
                    }
                }
            }
        }
    }
}

fn run_town(state: &mut GameState, town_id: TownId, towns: &[TownDef], shop_defs: &[crate::shared::ShopDef]) {
    let town = towns.iter().find(|t| t.id == town_id);
    let name = town.map(|t| t.name.as_str()).unwrap_or("Unknown Town");

    println!("\n── {} ──", name);
    if let Some(t) = town {
        for (i, npc) in t.npcs.iter().enumerate() {
            println!("  [{}] Talk to NPC #{}", i, npc.npc_id.0);
        }
        if !t.shops.is_empty() {
            println!("  [s] Shop");
        }
        if !t.djinn_points.is_empty() {
            println!("  [d] Search for Djinn");
        }
    }
    println!("  [l] Leave town");

    let input = prompt("> ");
    match input.as_str() {
        "s" => {
            if let Some(t) = town {
                if let Some(&shop_id) = t.shops.first() {
                    game_state::apply_transition(state, ScreenTransition::OpenShop(shop_id));
                    return;
                }
            }
            println!("  No shops here.");
        }
        "d" => {
            if let Some(t) = town {
                for djinn_point in &t.djinn_points {
                    println!("  Found Djinn: {}!", djinn_point.djinn_id.0);
                }
            }
        }
        "l" | "" => {
            game_state::apply_transition(state, ScreenTransition::ToWorldMap);
            return;
        }
        other => {
            if let Ok(idx) = other.parse::<usize>() {
                if let Some(t) = town {
                    if idx < t.npcs.len() {
                        let npc_id = t.npcs[idx].npc_id;
                        game_state::apply_transition(state, ScreenTransition::StartDialogue(npc_id));
                        return;
                    }
                }
            }
        }
    }
}

fn run_shop(state: &mut GameState, shop_id: ShopId, defs: &[crate::shared::ShopDef]) {
    let shop_def = defs.iter().find(|d| d.id == shop_id);
    let name = shop_def.map(|d| d.name.as_str()).unwrap_or("Shop");

    println!("\n── {} ── (Gold: {})", name, state.gold.get());
    if let Some(def) = shop_def {
        for (i, entry) in def.inventory.iter().enumerate() {
            let stock_str = match &entry.stock {
                crate::shared::ShopStock::Unlimited => "∞".to_string(),
                crate::shared::ShopStock::Limited(n) => format!("×{}", n.get()),
            };
            println!("  [{}] {} — {} gold ({})", i, entry.item_id.0, entry.price.get(), stock_str);
        }
    }
    println!("  [l] Leave");

    let input = prompt("Buy #> ");
    match input.as_str() {
        "l" | "" => {
            game_state::apply_transition(state, ScreenTransition::ReturnToPrevious);
        }
        other => {
            if let Ok(idx) = other.parse::<usize>() {
                if let Some(def) = shop_def {
                    if idx < def.inventory.len() {
                        let item = &def.inventory[idx];
                        match shop::can_buy(&state.shop_state, shop_id, &item.item_id, state.gold, defs) {
                            Ok(()) => {
                                let event = shop::execute_buy(&mut state.shop_state, shop_id, &item.item_id);
                                let new_gold = state.gold.get().saturating_sub(item.price.get());
                                state.gold = Gold::new(new_gold);
                                println!("  Bought {}! (Gold: {})", item.item_id.0, state.gold.get());
                            }
                            Err(e) => println!("  Can't buy: {:?}", e),
                        }
                    }
                }
            }
        }
    }
}

fn run_dialogue(state: &mut GameState, npc_id: NpcId, trees: &[DialogueTree], towns: &[TownDef], inventory: &mut Vec<ItemId>) {
    // Find dialogue tree for this NPC
    let tree_id = towns.iter()
        .flat_map(|t| t.npcs.iter())
        .find(|n| n.npc_id == npc_id)
        .map(|n| n.dialogue_tree);

    let tree = tree_id.and_then(|tid| trees.iter().find(|t| t.id == tid));

    let tree = match tree {
        Some(t) => t,
        None => {
            println!("  (This NPC has nothing to say.)");
            game_state::apply_transition(state, ScreenTransition::ReturnToPrevious);
            return;
        }
    };

    let mut runner = dialogue::start_dialogue(tree);
    let ctx = SnapshotCtx::from_state(state, &inventory);

    loop {
        if dialogue::is_finished(&runner) {
            break;
        }

        let node = dialogue::get_current_node(&runner, tree);
        let speaker = node.speaker.map(|id| format!("NPC #{}", id.0)).unwrap_or_else(|| "???".into());
        println!("\n  {}: \"{}\"", speaker, node.text);

        let responses = dialogue::get_available_responses(&runner, tree, &ctx);
        if responses.is_empty() {
            break;
        }

        for (i, (_idx, resp)) in responses.iter().enumerate() {
            println!("    [{}] {}", i, resp.text);
        }

        let choice = match prompt_number("  > ", responses.len()) {
            Some(c) => c,
            None => 0,
        };

        let (original_idx, _) = responses[choice];
        let (_next, side_effects) = dialogue::choose_response(&mut runner, tree, original_idx);

        // Process side effects
        for effect in &side_effects {
            match effect {
                crate::shared::DialogueSideEffect::GiveGold(amount) => {
                    let new_gold = state.gold.get() + amount.get();
                    state.gold = Gold::new(new_gold);
                    println!("  (Received {} gold!)", amount.get());
                }
                crate::shared::DialogueSideEffect::TakeGold(amount) => {
                    let new_gold = state.gold.get().saturating_sub(amount.get());
                    state.gold = Gold::new(new_gold);
                    println!("  (Lost {} gold.)", amount.get());
                }
                crate::shared::DialogueSideEffect::GiveItem(item_id, count) => {
                    for _ in 0..count.get() {
                        inventory.push(item_id.clone());
                    }
                    println!("  (Received {}!)", item_id.0);
                }
                crate::shared::DialogueSideEffect::SetQuestStage(flag, stage) => {
                    state.quest_state.advance(*flag, *stage);
                    println!("  (Quest updated!)");
                }
                crate::shared::DialogueSideEffect::UnlockMapNode(node_id) => {
                    if let Some(ref mut wm) = state.world_map {
                        world_map::unlock_node(wm, *node_id);
                        println!("  (New location discovered!)");
                    }
                }
                crate::shared::DialogueSideEffect::AddDjinnToParty(djinn_id) => {
                    println!("  (Djinn {} joined the party!)", djinn_id.0);
                }
                _ => {}
            }
        }
    }

    game_state::apply_transition(state, ScreenTransition::ReturnToPrevious);
}

fn run_dungeon(state: &mut GameState, dungeon_id: DungeonId, dungeons: &[DungeonDef], inventory: &mut Vec<ItemId>, game_data: &GameData, save_data: &mut save::SaveData) {
    let def = match dungeons.iter().find(|d| d.id == dungeon_id) {
        Some(d) => d,
        None => {
            println!("  (This dungeon doesn\'t exist yet.)");
            game_state::apply_transition(state, ScreenTransition::ToWorldMap);
            return;
        }
    };

    println!("\n╔══════════════════════════════════════╗");
    println!("║  Entering: {:30}║", def.name);
    println!("╚══════════════════════════════════════╝");

    let mut ds = dungeon::enter_dungeon(def);
    let mut step_count: u16 = 0;
    let mut puzzle_instances: std::collections::HashMap<(RoomId, usize), puzzle::PuzzleInstance> = std::collections::HashMap::new();

    loop {
        let ctx = SnapshotCtx::from_state(state, inventory);
        let room = dungeon::get_current_room(&ds, def);
        let room_type_str = match room.room_type {
            RoomType::Normal => "Room",
            RoomType::Puzzle => "Puzzle Room",
            RoomType::MiniBoss => "Mini-Boss Room",
            RoomType::Boss => "=== BOSS ROOM ===",
            RoomType::Treasure => "Treasure Room",
            RoomType::Safe => "Safe Room",
        };
        let is_new = !ds.visited_rooms.contains(&room.id);
        let visited_str = if is_new { " [NEW]" } else { "" };
        println!("\n── {} (Room {}) {} ──", room_type_str, room.id.0, visited_str);

        // Show items
        for (i, item) in room.items.iter().enumerate() {
            if !ds.collected_items.contains(&(room.id, i)) {
                let vis = if item.visible { "" } else { " [hidden]" };
                println!("  [i{}] Pick up: {}{}", i, item.item_id.0, vis);
            }
        }

        // Show puzzles
        let puzzles = dungeon::get_room_puzzles(&ds, def);
        for (i, puz) in puzzles.iter().enumerate() {
            let key = (room.id, i);
            let status = puzzle_instances.get(&key)
                .map(|pi| format!("{:?}", pi.state))
                .unwrap_or_else(|| "Unsolved".into());
            println!("  [p{}] {:?} [{}]", i, puz.puzzle_type, status);
        }

        // Show encounters
        for (i, slot) in room.encounters.iter().enumerate() {
            let count = ds.encounter_counts.get(&(room.id, i)).copied().unwrap_or(0);
            let max = slot.max_triggers.unwrap_or(255);
            if count < max {
                println!("  [e{}] Fight: {}", i, slot.encounter.name);
            }
        }

        // Show exits
        let exits = dungeon::get_available_exits(&ds, def, &ctx);
        for (dir, _target) in &exits {
            println!("  [{:?}] Go {:?}", dir, dir);
        }
        println!("  [flee] Leave dungeon");

        if dungeon::is_boss_room(def, room.id) {
            println!("\n  *** The boss awaits! ***");
            println!("  [fight] Challenge the boss");
        }

        let input = prompt("> ").to_lowercase();

        if input == "flee" {
            println!("  You escape the dungeon.");
            game_state::apply_transition(state, ScreenTransition::ToWorldMap);
            return;
        }

        // Boss fight with real battle
        if input == "fight" && dungeon::is_boss_room(def, room.id) {
            // Per-dungeon boss encounter, quest flag, and unlock
            let (boss_enc_id, quest_flag, quest_name, unlock_nodes) = match def.id {
                DungeonId(0) => ("house-16", QuestFlagId(0), "Mercury Lighthouse", vec![(MapNodeId(1), vec![MapNodeId(3)])]),
                DungeonId(1) => ("house-17", QuestFlagId(1), "Kolima Curse", vec![(MapNodeId(2), vec![])]),
                _ => ("house-20", QuestFlagId(0), "Unknown", vec![]),
            };

            println!("\n  ⚔ BOSS BATTLE ⚔");
            let result = cli_runner::run_demo_battle(game_data, boss_enc_id);
            match result {
                BattleResult::Victory { xp, gold: bg } => {
                    println!("\n  BOSS DEFEATED! +{} XP, +{} gold", xp, bg);
                    let ng = state.gold.get() + bg;
                    state.gold = Gold::new(ng);
                    save_data.gold = ng;
                    save_data.xp += xp;

                    // Apply XP to party
                    for unit in &mut save_data.player_party {
                        if let Some(unit_def) = game_data.units.get(&unit.unit_id) {
                            let mut progress = progression::UnitProgress {
                                unit_id: unit.unit_id.clone(),
                                level: unit.level,
                                current_xp: progression::xp_for_level(unit.level),
                            };
                            let level_result = progression::apply_battle_rewards(&mut progress, xp, unit_def);
                            if !level_result.levels_gained.is_empty() {
                                println!("  {} reached level {}!", unit_def.name, progress.level);
                                unit.level = progress.level;
                            }
                        }
                    }

                    state.quest_state.advance(quest_flag, QuestStage::Complete);
                    println!("  ({} quest completed!)", quest_name);

                    if let Some(ref mut wm) = state.world_map {
                        for (complete_node_id, unlock_ids) in &unlock_nodes {
                            world_map::complete_node(wm, *complete_node_id);
                            for uid in unlock_ids {
                                world_map::unlock_node(wm, *uid);
                                world_map::unlock_node(wm, *uid); // Visible → Unlocked
                                let node_name = wm.nodes.iter().find(|n| n.id == *uid).map(|n| n.name.as_str()).unwrap_or("???");
                                println!("  (New location unlocked: {}!)", node_name);
                            }
                        }
                    }

                    // Auto-save
                    save_data.extension = Some(state.to_save_extension());
                    let sp = std::path::Path::new("saves/game.ron");
                    if let Err(e) = save::save_game(save_data, sp) {
                        eprintln!("  Warning: auto-save failed: {}", e);
                    } else { println!("  (Game saved.)"); }
                }
                BattleResult::Defeat => {
                    println!("\n  DEFEATED by the boss... Retreating.");
                }
            }
            game_state::apply_transition(state, ScreenTransition::ToWorldMap);
            return;
        }

        // Encounter fight
        if input.starts_with('e') {
            if let Ok(idx) = input[1..].parse::<usize>() {
                if let Some(enc_def) = dungeon::trigger_encounter(&mut ds, def, idx) {
                    println!("\n  Battle: {}!", enc_def.name);
                    let result = cli_runner::run_demo_battle(game_data, &enc_def.id.0);
                    match result {
                        BattleResult::Victory { xp, gold: bg } => {
                            println!("  Victory! +{} XP, +{} gold", xp, bg);
                            let ng = state.gold.get() + bg;
                            state.gold = Gold::new(ng);
                            save_data.gold = ng;
                            save_data.xp += xp;
                        }
                        BattleResult::Defeat => {
                            println!("  Defeated! Fleeing the dungeon...");
                            game_state::apply_transition(state, ScreenTransition::ToWorldMap);
                            return;
                        }
                    }
                } else { println!("  No encounter available."); }
                continue;
            }
        }

        // Puzzle attempt
        if input.starts_with('p') {
            if let Ok(idx) = input[1..].parse::<usize>() {
                let puzzles = dungeon::get_room_puzzles(&ds, def);
                if idx < puzzles.len() {
                    let key = (room.id, idx);
                    let puz_def = puzzles[idx].clone();
                    let instance = puzzle_instances.entry(key).or_insert_with(|| {
                        puzzle::PuzzleInstance::new(puz_def)
                    });
                    let puzzle_input = match &puzzles[idx].puzzle_type {
                        crate::shared::PuzzleType::PushBlock => {
                            let d = prompt("  Push direction? (up/down/left/right) > ");
                            match d.as_str() {
                                "up" => Some(puzzle::PuzzleInput::PushDirection(Direction::Up)),
                                "down" => Some(puzzle::PuzzleInput::PushDirection(Direction::Down)),
                                "left" => Some(puzzle::PuzzleInput::PushDirection(Direction::Left)),
                                "right" => Some(puzzle::PuzzleInput::PushDirection(Direction::Right)),
                                _ => None,
                            }
                        }
                        crate::shared::PuzzleType::ElementPillar(elem) => {
                            println!("  Activate {:?} pillar? (y/n)", elem);
                            if prompt("  > ") == "y" { Some(puzzle::PuzzleInput::ActivateElement(*elem)) } else { None }
                        }
                        crate::shared::PuzzleType::SwitchSequence => {
                            let s = prompt("  Toggle switch #? (0-3) > ");
                            s.parse::<usize>().ok().map(puzzle::PuzzleInput::ToggleSwitch)
                        }
                        crate::shared::PuzzleType::IceSlide => {
                            let d = prompt("  Slide direction? > ");
                            match d.as_str() {
                                "up" => Some(puzzle::PuzzleInput::SlideDirection(Direction::Up)),
                                "down" => Some(puzzle::PuzzleInput::SlideDirection(Direction::Down)),
                                "left" => Some(puzzle::PuzzleInput::SlideDirection(Direction::Left)),
                                "right" => Some(puzzle::PuzzleInput::SlideDirection(Direction::Right)),
                                _ => None,
                            }
                        }
                        crate::shared::PuzzleType::DjinnPuzzle(djinn_id) => {
                            println!("  Use djinn {}? (y/n)", djinn_id.0);
                            if prompt("  > ") == "y" { Some(puzzle::PuzzleInput::UseDjinn(djinn_id.clone())) } else { None }
                        }
                    };
                    if let Some(pi) = puzzle_input {
                        let result = puzzle::attempt_solve(instance, pi);
                        match result {
                            puzzle::PuzzleResult::Solved(reward) => {
                                println!("  Puzzle SOLVED!");
                                if let Some(ref effect) = reward {
                                    apply_side_effect(state, inventory, effect);
                                }
                            }
                            puzzle::PuzzleResult::Progress => println!("  Making progress..."),
                            puzzle::PuzzleResult::Failed => println!("  That didn\'t work."),
                            puzzle::PuzzleResult::AlreadySolved => println!("  Already solved!"),
                        }
                    }
                }
                continue;
            }
        }

        // Item pickup
        if input.starts_with('i') {
            if let Ok(idx) = input[1..].parse::<usize>() {
                if let Some(item_id) = dungeon::collect_item(&mut ds, def, idx) {
                    inventory.push(item_id.clone());
                    println!("  Picked up: {}!", item_id.0);
                } else { println!("  Nothing to pick up."); }
                continue;
            }
        }

        // Direction movement with random encounter check
        let dir = match input.as_str() {
            "up" => Some(Direction::Up),
            "down" => Some(Direction::Down),
            "left" => Some(Direction::Left),
            "right" => Some(Direction::Right),
            _ => None,
        };

        if let Some(d) = dir {
            if let Some((_, target)) = exits.iter().find(|(ed, _)| *ed == d) {
                match dungeon::move_to_room(&mut ds, def, *target) {
                    Ok(()) => {
                        step_count += 1;
                        // Random encounter every 3 steps in rooms with encounters
                        let new_room = dungeon::get_current_room(&ds, def);
                        if !new_room.encounters.is_empty() && step_count % 3 == 0 {
                            if let Some(enc) = dungeon::trigger_encounter(&mut ds, def, 0) {
                                println!("\n  !! Random encounter: {}!", enc.name);
                                let result = cli_runner::run_demo_battle(game_data, &enc.id.0);
                                match result {
                                    BattleResult::Victory { xp, gold: bg } => {
                                        println!("  Victory! +{} XP, +{} gold", xp, bg);
                                        let ng = state.gold.get() + bg;
                                        state.gold = Gold::new(ng);
                                        save_data.gold = ng;
                                        save_data.xp += xp;
                                    }
                                    BattleResult::Defeat => {
                                        println!("  Defeated! Fleeing...");
                                        game_state::apply_transition(state, ScreenTransition::ToWorldMap);
                                        return;
                                    }
                                }
                            }
                        }
                    }
                    Err(e) => println!("  Can\'t go there: {:?}", e),
                }
            } else { println!("  No exit in that direction."); }
        }
    }
}

/// Apply a dialogue side effect to game state.
fn apply_side_effect(state: &mut GameState, inventory: &mut Vec<ItemId>, effect: &crate::shared::DialogueSideEffect) {
    match effect {
        crate::shared::DialogueSideEffect::GiveGold(amount) => {
            let ng = state.gold.get() + amount.get();
            state.gold = Gold::new(ng);
            println!("  (Received {} gold!)", amount.get());
        }
        crate::shared::DialogueSideEffect::TakeGold(amount) => {
            let ng = state.gold.get().saturating_sub(amount.get());
            state.gold = Gold::new(ng);
            println!("  (Lost {} gold.)", amount.get());
        }
        crate::shared::DialogueSideEffect::GiveItem(item_id, count) => {
            for _ in 0..count.get() { inventory.push(item_id.clone()); }
            println!("  (Received {}!)", item_id.0);
        }
        crate::shared::DialogueSideEffect::SetQuestStage(flag, stage) => {
            state.quest_state.advance(*flag, *stage);
            println!("  (Quest updated!)");
        }
        crate::shared::DialogueSideEffect::UnlockMapNode(node_id) => {
            if let Some(ref mut wm) = state.world_map {
                world_map::unlock_node(wm, *node_id);
                println!("  (New location discovered!)");
            }
        }
        crate::shared::DialogueSideEffect::AddDjinnToParty(djinn_id) => {
            println!("  (Djinn {} joined the party!)", djinn_id.0);
        }
        _ => {}
    }
}



#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_starter_data_loads() {
        let nodes = starter_map_nodes();
        assert_eq!(nodes.len(), 4);
        let towns = starter_towns();
        assert_eq!(towns.len(), 2);
        let shops = starter_shop_defs();
        assert_eq!(shops.len(), 2);
        let trees = starter_dialogue_trees();
        assert_eq!(trees.len(), 3);
        let dungeons = starter_dungeons();
        assert_eq!(dungeons.len(), 1);
        assert_eq!(dungeons[0].rooms.len(), 3);
    }

    #[test]
    fn test_world_map_setup() {
        let nodes = starter_map_nodes();
        let mut wm = world_map::load_map(nodes);
        // Initially all locked
        assert_eq!(world_map::get_accessible_nodes(&wm).len(), 0);
        // Unlock vale village (Locked → Visible → Unlocked)
        world_map::unlock_node(&mut wm, MapNodeId(0));
        world_map::unlock_node(&mut wm, MapNodeId(0));
        assert_eq!(world_map::get_accessible_nodes(&wm).len(), 1);
    }

    #[test]
    fn test_shop_buy_flow() {
        let defs = starter_shop_defs();
        let mut shop_state = shop::init_shop_state(&defs);
        let gold = Gold::new(100);
        let item = crate::shared::ItemId("herb".into());
        assert!(shop::can_buy(&shop_state, ShopId(0), &item, gold, &defs).is_ok());
        assert!(shop::can_buy(&shop_state, ShopId(0), &item, Gold::new(0), &defs).is_err());
    }

    #[test]
    fn test_dialogue_tree_traversal() {
        let trees = starter_dialogue_trees();
        let tree = &trees[0]; // Elder tree
        let mut runner = dialogue::start_dialogue(tree);
        assert!(!dialogue::is_finished(&runner));

        let node = dialogue::get_current_node(&runner, tree);
        assert!(node.text.contains("Mercury Lighthouse"));
        assert_eq!(node.responses.len(), 2);

        // Choose "I'll go at once"
        let (_next, effects) = dialogue::choose_response(&mut runner, tree, 0);
        assert_eq!(effects.len(), 1); // SetQuestStage

        // Should advance to goodbye node
        let node2 = dialogue::get_current_node(&runner, tree);
        assert!(node2.text.contains("Brave soul"));

        // Choose [Accept]
        let (_next, effects2) = dialogue::choose_response(&mut runner, tree, 0);
        assert_eq!(effects2.len(), 1); // GiveGold
        assert!(dialogue::is_finished(&runner));
    }

    #[test]
    fn test_dungeon_traversal() {
        let dungeons = starter_dungeons();
        let def = &dungeons[0];
        let mut ds = dungeon::enter_dungeon(def);

        // Start in entry room (0)
        let room = dungeon::get_current_room(&ds, def);
        assert_eq!(room.id, RoomId(0));
        assert_eq!(room.room_type, RoomType::Normal);

        // Collect herb
        let item = dungeon::collect_item(&mut ds, def, 0);
        assert_eq!(item, Some(ItemId("herb".into())));
        // Can't collect again
        let item2 = dungeon::collect_item(&mut ds, def, 0);
        assert_eq!(item2, None);

        // Move up to puzzle room
        assert!(dungeon::move_to_room(&mut ds, def, RoomId(1)).is_ok());
        let room1 = dungeon::get_current_room(&ds, def);
        assert_eq!(room1.room_type, RoomType::Puzzle);

        // Move up to boss room
        assert!(dungeon::move_to_room(&mut ds, def, RoomId(2)).is_ok());
        assert!(dungeon::is_boss_room(def, RoomId(2)));
    }
}
