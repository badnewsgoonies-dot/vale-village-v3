mod shared;
mod data;
mod domains;

use std::path::Path;

use domains::battle_engine::BattleResult;
use domains::cli_runner;
use domains::data_loader;
use domains::save;

const SAVE_PATH: &str = "saves/game.ron";

fn main() {
    // Ensure saves/ directory exists
    std::fs::create_dir_all("saves").expect("failed to create saves/ directory");

    let data_dir = Path::new("data/full");
    let game_data = match data_loader::load_game_data(data_dir) {
        Ok(data) => data,
        Err(errors) => {
            for e in &errors {
                eprintln!("Load error: {e:?}");
            }
            std::process::exit(1);
        }
    };
    println!(
        "Vale Village v3 — Loaded {} abilities, {} units, {} enemies",
        game_data.abilities.len(),
        game_data.units.len(),
        game_data.enemies.len()
    );

    // Check --new flag
    let force_new = std::env::args().any(|arg| arg == "--new");

    // Load or create save data
    let save_path = Path::new(SAVE_PATH);
    let mut save_data = if !force_new && save_path.exists() {
        match save::load_game(save_path) {
            Ok(data) => {
                println!(
                    "Loaded save: Level {}, {} gold, {} encounters completed",
                    data.player_party.first().map(|u| u.level).unwrap_or(1),
                    data.gold,
                    data.completed_encounters.len(),
                );
                data
            }
            Err(e) => {
                eprintln!("Warning: failed to load save ({e}), starting new game");
                let data = save::create_new_game();
                println!("Starting new game");
                data
            }
        }
    } else {
        let data = save::create_new_game();
        println!("Starting new game");
        data
    };

    let result = cli_runner::run_demo_battle(&game_data);
    match result {
        BattleResult::Victory { xp, gold } => {
            println!("\nVICTORY! +{xp} XP, +{gold} gold");

            // Update save data
            save_data.xp += xp;
            save_data.gold += gold;
            save_data.completed_encounters.push(
                shared::EncounterId("house-02".into()),
            );

            // Save to disk
            match save::save_game(&save_data, save_path) {
                Ok(()) => println!("Game saved."),
                Err(e) => eprintln!("Warning: failed to save game: {e}"),
            }
        }
        BattleResult::Defeat => println!("\nDEFEAT."),
    }
}
