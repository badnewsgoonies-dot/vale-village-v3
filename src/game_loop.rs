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
use crate::domains::encounter;
use crate::domains::menu;
use crate::domains::puzzle;
use crate::domains::cli_runner;
use crate::domains::battle_engine::BattleResult;
use crate::domains::data_loader::GameData;
use crate::domains::quest::QuestManager;
use crate::domains::save;
use crate::domains::shop;
use crate::domains::world_map;
use crate::domains::progression;

use crate::starter_data::{starter_map_nodes, starter_towns, starter_shop_defs, starter_dialogue_trees, starter_dungeons};

// ── Runtime ConditionContext ──────────────────────────────────────────

/// Snapshot-based context to avoid holding borrows across mutations.
struct SnapshotCtx {
    inventory: Vec<ItemId>,
    quest_flags: std::collections::HashMap<QuestFlagId, QuestStage>,
    gold: u32,
    djinn_ids: Vec<DjinnId>,
    djinn_elements: Vec<crate::shared::Element>,
    party_unit_ids: Vec<crate::shared::UnitId>,
}

impl SnapshotCtx {
    fn from_state(state: &GameState, inventory: &[ItemId], save_data: &save::SaveData, game_data: &GameData) -> Self {
        let djinn_ids: Vec<DjinnId> = save_data.team_djinn.iter().map(|d| d.djinn_id.clone()).collect();
        let djinn_elements: Vec<crate::shared::Element> = save_data.team_djinn.iter()
            .filter_map(|d| game_data.djinn.get(&d.djinn_id).map(|def| def.element))
            .collect();
        let party_unit_ids: Vec<crate::shared::UnitId> = save_data.player_party.iter()
            .map(|u| u.unit_id.clone())
            .collect();
        Self {
            inventory: inventory.to_vec(),
            quest_flags: state.quest_state.flags.clone(),
            gold: state.gold.get(),
            djinn_ids,
            djinn_elements,
            party_unit_ids,
        }
    }
}

impl ConditionContext for SnapshotCtx {
    fn has_item(&self, item: &ItemId) -> bool {
        self.inventory.contains(item)
    }
    fn has_djinn(&self, djinn: &DjinnId) -> bool {
        self.djinn_ids.contains(djinn)
    }
    fn quest_at_stage(&self, flag: &QuestFlagId, stage: QuestStage) -> bool {
        self.quest_flags.get(flag).copied().unwrap_or(QuestStage::Unknown) >= stage
    }
    fn gold_at_least(&self, amount: Gold) -> bool {
        self.gold >= amount.get()
    }
    fn party_contains(&self, unit: &UnitId) -> bool {
        self.party_unit_ids.contains(unit)
    }
}

impl puzzle::PuzzleContext for SnapshotCtx {
    fn has_djinn(&self, id: &DjinnId) -> bool {
        self.djinn_ids.contains(id)
    }
    fn has_element_ability(&self, element: crate::shared::Element) -> bool {
        self.djinn_elements.contains(&element)
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
                run_world_map(state, &towns, game_data, save_data);
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
                run_menu(state, *menu, game_data, save_data, &inventory);
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
                run_dialogue(state, nid, &dialogue_trees, &towns, &mut inventory, game_data, save_data);
            }
        }
    }
}

fn run_world_map(state: &mut GameState, towns: &[TownDef], game_data: &GameData, save_data: &mut save::SaveData) {
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
            println!("\n  Menu:");
            println!("  [1] Party    [2] Equipment  [3] Djinn");
            println!("  [4] Items    [5] Quest Log  [6] Status");
            let choice = prompt("  > ");
            let screen = match choice.as_str() {
                "1" => Some(MenuScreen::Party),
                "2" => Some(MenuScreen::Equipment),
                "3" => Some(MenuScreen::Djinn),
                "4" => Some(MenuScreen::Items),
                "5" => Some(MenuScreen::QuestLog),
                "6" => Some(MenuScreen::Status),
                _ => None,
            };
            if let Some(s) = screen {
                game_state::apply_transition(state, ScreenTransition::OpenMenu(s));
            }
        }
        "s" => {
            game_state::apply_transition(state, ScreenTransition::OpenSaveLoad);
        }
        other => {
            if let Ok(idx) = other.parse::<usize>() {
                if idx < accessible.len() {
                    let node = &accessible[idx];
                    // Increment travel steps and check for overworld encounter
                    state.steps_since_encounter += 1;
                    let mut enc_table = crate::starter_data::overworld_encounter_table();
                    if encounter::should_encounter(&enc_table, state.steps_since_encounter) {
                        if let Some(enc) = encounter::select_encounter(&enc_table, state.steps_since_encounter) {
                            let enc_id = enc.id.0.clone();
                            println!("\n  ⚔ Random encounter: {}!", enc.name);
                            let result = cli_runner::run_demo_battle(game_data, &enc_id);
                            match result {
                                BattleResult::Victory { xp, gold: bg } => {
                                    println!("  Victory! +{} XP, +{} gold", xp, bg);
                                    let ng = state.gold.get() + bg;
                                    state.gold = Gold::new(ng);
                                    save_data.gold = ng;
                                    save_data.xp += xp;
                                }
                                BattleResult::Defeat => {
                                    println!("  Defeat! You retreat to the world map.");
                                    return; // don't arrive at destination
                                }
                            }
                            state.steps_since_encounter = 0;
                        }
                    }
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

fn run_dialogue(state: &mut GameState, npc_id: NpcId, trees: &[DialogueTree], towns: &[TownDef], inventory: &mut Vec<ItemId>, game_data: &GameData, save_data: &save::SaveData) {
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
    let ctx = SnapshotCtx::from_state(state, &inventory, save_data, game_data);

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
        let ctx = SnapshotCtx::from_state(state, inventory, save_data, game_data);
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
                DungeonId(1) => ("house-17", QuestFlagId(1), "Kolima Curse", vec![(MapNodeId(2), vec![MapNodeId(4)])]),
                DungeonId(2) => ("house-18", QuestFlagId(1), "Mogall Forest", vec![(MapNodeId(4), vec![MapNodeId(5)])]),
                DungeonId(3) => ("house-19", QuestFlagId(2), "Gondowan Passage", vec![(MapNodeId(6), vec![MapNodeId(7)])]),
                DungeonId(4) => ("house-20", QuestFlagId(4), "Colosso Qualifier", vec![(MapNodeId(8), vec![MapNodeId(9)])]),
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
fn run_menu(state: &mut GameState, screen: MenuScreen, game_data: &GameData, save_data: &save::SaveData, inventory: &[ItemId]) {
    println!("\n── Menu: {:?} ──", screen);

    match screen {
        MenuScreen::Party => {
            let summaries: Vec<menu::UnitSummary> = save_data.player_party.iter().map(|su| {
                let def = game_data.units.get(&su.unit_id);
                menu::UnitSummary {
                    name: def.map(|d| d.name.clone()).unwrap_or_else(|| su.unit_id.0.clone()),
                    level: su.level,
                    hp_current: su.current_hp,
                    hp_max: def.map(|d| d.base_stats.hp.get()).unwrap_or(100),
                    element: def.map(|d| format!("{:?}", d.element)).unwrap_or_else(|| "???".into()),
                }
            }).collect();
            for line in menu::show_party(&summaries) {
                println!("  {}", line);
            }
        }
        MenuScreen::Equipment => {
            if let Some(first) = save_data.player_party.first() {
                let def = game_data.units.get(&first.unit_id);
                let unit_summary = menu::UnitSummary {
                    name: def.map(|d| d.name.clone()).unwrap_or_else(|| first.unit_id.0.clone()),
                    level: first.level,
                    hp_current: first.current_hp,
                    hp_max: def.map(|d| d.base_stats.hp.get()).unwrap_or(100),
                    element: def.map(|d| format!("{:?}", d.element)).unwrap_or_else(|| "???".into()),
                };
                // Show equipped items
                let mut eq_list = Vec::new();
                if let Some(ref wep) = first.equipment.weapon {
                    let name = game_data.equipment.get(wep).map(|e| e.name.clone()).unwrap_or_else(|| wep.0.clone());
                    eq_list.push(menu::EquipmentSummary { name, slot: "Weapon".into(), tier: 1, equipped: true });
                }
                if let Some(ref helm) = first.equipment.helm {
                    let name = game_data.equipment.get(helm).map(|e| e.name.clone()).unwrap_or_else(|| helm.0.clone());
                    eq_list.push(menu::EquipmentSummary { name, slot: "Helm".into(), tier: 1, equipped: true });
                }
                if let Some(ref armor) = first.equipment.armor {
                    let name = game_data.equipment.get(armor).map(|e| e.name.clone()).unwrap_or_else(|| armor.0.clone());
                    eq_list.push(menu::EquipmentSummary { name, slot: "Armor".into(), tier: 1, equipped: true });
                }
                for line in menu::show_equipment(&unit_summary, &eq_list) {
                    println!("  {}", line);
                }
            } else {
                println!("  No party members.");
            }
        }
        MenuScreen::Djinn => {
            let djinn_list: Vec<menu::DjinnSummary> = save_data.team_djinn.iter().map(|sd| {
                let def = game_data.djinn.get(&sd.djinn_id);
                menu::DjinnSummary {
                    name: def.map(|d| d.name.clone()).unwrap_or_else(|| sd.djinn_id.0.clone()),
                    element: def.map(|d| format!("{:?}", d.element)).unwrap_or_else(|| "???".into()),
                    state: match sd.state {
                        crate::shared::DjinnState::Good => menu::DjinnMenuState::Set,
                        crate::shared::DjinnState::Recovery => menu::DjinnMenuState::Recovery,
                    },
                    tier: def.map(|d| d.tier.get()).unwrap_or(1),
                }
            }).collect();
            for line in menu::show_djinn(&djinn_list) {
                println!("  {}", line);
            }
            if djinn_list.is_empty() {
                println!("  No djinn in party yet.");
            }
        }
        MenuScreen::Items => {
            // Count items in inventory
            let mut counts: std::collections::HashMap<&str, u8> = std::collections::HashMap::new();
            for item_id in inventory {
                *counts.entry(&item_id.0).or_insert(0) += 1;
            }
            let item_list: Vec<menu::ItemSummary> = counts.iter().map(|(name, count)| {
                menu::ItemSummary {
                    name: name.to_string(),
                    count: *count,
                    description: String::new(),
                }
            }).collect();
            for line in menu::show_items(&item_list) {
                println!("  {}", line);
            }
            if item_list.is_empty() {
                println!("  No items.");
            }
        }
        MenuScreen::Psynergy => {
            println!("  (Psynergy list not yet wired — requires ability data per unit)");
        }
        MenuScreen::Status => {
            println!("  Gold: {}", state.gold.get());
            println!("  Play time: {}s", state.play_time_seconds);
            println!("  Quests active: {}", state.quest_state.flags.len());
            if let Some(ref wm) = state.world_map {
                let accessible = world_map::get_accessible_nodes(wm);
                println!("  Locations unlocked: {}", accessible.len());
            }
        }
        MenuScreen::QuestLog => {
            let entries: Vec<menu::QuestLogEntry> = state.quest_state.flags.iter().map(|(flag, stage)| {
                menu::QuestLogEntry {
                    name: format!("Quest #{}", flag.0),
                    description: format!("Stage: {:?}", stage),
                    completed: *stage >= crate::shared::QuestStage::Complete,
                }
            }).collect();
            for line in menu::show_quest_log(&entries) {
                println!("  {}", line);
            }
            if entries.is_empty() {
                println!("  No quests yet.");
            }
        }
    }

    println!("\n  [Press ENTER to return]");
    prompt("");
    game_state::apply_transition(state, ScreenTransition::ReturnToPrevious);
}

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
