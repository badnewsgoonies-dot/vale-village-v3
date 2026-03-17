mod shared;
mod data;
mod domains;

use domains::battle_engine::BattleResult;
use domains::cli_runner;
use domains::data_loader;

fn main() {
    let data_dir = std::path::Path::new("data/full");
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

    let result = cli_runner::run_demo_battle(&game_data);
    match result {
        BattleResult::Victory { xp, gold } => println!("\nVICTORY! +{xp} XP, +{gold} gold"),
        BattleResult::Defeat => println!("\nDEFEAT."),
    }
}
