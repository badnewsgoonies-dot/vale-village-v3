#![allow(dead_code)]
//! CLI Game Loop — text-based adventure runner integrating all beyond-battle domains.
//! Orchestrator-owned. This is the integration layer connecting screens, world map,
//! towns, shops, dungeons, quests, dialogue, and encounters into a playable loop.

use std::io::{self, Write};

use crate::game_state::{self, GameState};
use crate::shared::{
    bounded_types::Gold,
    DjinnId, Direction, DungeonDef, DungeonId, GameScreen, MapNode, MapNodeId,
    MapNodeType, MenuScreen, NodeUnlockState, NpcId, QuestFlagId, QuestStage,
    ScreenTransition, ShopId, TownDef, TownId,
};

use crate::domains::dungeon;
use crate::domains::quest::QuestManager;
use crate::domains::shop;
use crate::domains::world_map;

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

// ── Input Helper ─────────────────────────────────────────────────────

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

pub fn run_game_loop(state: &mut GameState) {
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

            GameScreen::Dungeon(_dungeon_id) => {
                println!("[Dungeon] (not yet implemented — returning to world map)");
                game_state::apply_transition(state, ScreenTransition::ToWorldMap);
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

            GameScreen::Dialogue(_npc_id) => {
                println!("[Dialogue] (not yet wired to dialogue trees)");
                game_state::apply_transition(state, ScreenTransition::ReturnToPrevious);
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
    println!("  [t] Talk to NPCs");
    if let Some(t) = town {
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
        "t" => {
            if let Some(t) = town {
                for npc in &t.npcs {
                    println!("  NPC #{}: (dialogue not yet wired)", npc.npc_id.0);
                }
            }
        }
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
        _ => {}
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
        // Can buy herb for 10 gold
        assert!(shop::can_buy(&shop_state, ShopId(0), &item, gold, &defs).is_ok());
        // Can't buy with 0 gold
        assert!(shop::can_buy(&shop_state, ShopId(0), &item, Gold::new(0), &defs).is_err());
    }
}
