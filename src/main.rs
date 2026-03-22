mod data;
mod domains;
mod game_loop;
mod game_state;
mod shared;
mod starter_data;

use std::path::Path;

use domains::battle_engine::BattleResult;
use domains::cli_runner;
use domains::data_loader;
use domains::progression;
use domains::save;
use domains::ui;
use shared::{DjinnId, DjinnState, EncounterId};

const SAVE_PATH: &str = "saves/game.ron";

/// The story encounter sequence from house-01 through house-50.
const ENCOUNTER_SEQUENCE: &[&str] = &[
    "house-01", "house-02", "house-03", "house-04", "house-05", "house-06", "house-07", "house-08",
    "house-09", "house-10", "house-11", "house-12", "house-13", "house-14", "house-15", "house-16",
    "house-17", "house-18", "house-19", "house-20", "house-21", "house-22", "house-23", "house-24",
    "house-25", "house-26", "house-27", "house-28", "house-29", "house-30", "house-31", "house-32",
    "house-33", "house-34", "house-35", "house-36", "house-37", "house-38", "house-39", "house-40",
    "house-41", "house-42", "house-43", "house-44", "house-45", "house-46", "house-47", "house-48",
    "house-49", "house-50",
];

/// Find the next uncompleted encounter in the story sequence.
/// Returns None if all encounters have been completed.
fn next_encounter(completed: &[EncounterId]) -> Option<&'static str> {
    for &enc_id in ENCOUNTER_SEQUENCE {
        let eid = EncounterId(enc_id.into());
        if !completed.contains(&eid) {
            return Some(enc_id);
        }
    }
    None
}

fn load_encounter_id(save_data: &save::SaveData) -> Option<String> {
    save_data
        .current_encounter_id
        .as_ref()
        .map(|encounter_id| encounter_id.0.clone())
        .or_else(|| next_encounter(&save_data.completed_encounters).map(str::to_owned))
}

fn persist_djinn_reward(save_data: &mut save::SaveData, djinn_id: &DjinnId) {
    if save_data
        .team_djinn
        .iter()
        .any(|saved_djinn| saved_djinn.djinn_id == *djinn_id)
    {
        return;
    }

    save_data.team_djinn.push(save::SavedDjinn {
        djinn_id: djinn_id.clone(),
        state: DjinnState::Good,
    });
}

fn main() {
    // Launch Bevy GUI mode: always on WASM, or with --gui flag on native
    #[cfg(target_arch = "wasm32")]
    {
        console_error_panic_hook::set_once();
        bevy::app::App::new()
            .add_plugins(ui::plugin::ValeVillagePlugin)
            .run();
        return;
    }

    #[cfg(not(target_arch = "wasm32"))]
    if std::env::args().any(|arg| arg == "--gui") {
        bevy::app::App::new()
            .add_plugins(ui::plugin::ValeVillagePlugin)
            .run();
        return;
    }

    // Launch text adventure mode (world map, towns, shops, dungeons)
    if std::env::args().any(|arg| arg == "--adventure") {
        std::fs::create_dir_all("saves").expect("failed to create saves/ directory");

        let data_dir = Path::new("data/full");
        let game_data = match data_loader::load_game_data(data_dir) {
            Ok(data) => data,
            Err(errors) => {
                for e in &errors { eprintln!("Load error: {e:?}"); }
                std::process::exit(1);
            }
        };
        println!("Loaded {} abilities, {} enemies, {} encounters",
            game_data.abilities.len(), game_data.enemies.len(), game_data.encounters.len());

        let save_path = Path::new(SAVE_PATH);
        let mut save_data = if save_path.exists() {
            save::load_game(save_path).unwrap_or_else(|_| save::create_new_game())
        } else {
            save::create_new_game()
        };

        let mut state = if let Some(ref ext) = save_data.extension {
            game_state::GameState::from_save(ext)
        } else {
            game_state::GameState::new_game()
        };
        state.gold = shared::bounded_types::Gold::new(save_data.gold);

        game_loop::run_game_loop(&mut state, &game_data, &mut save_data);

        // Auto-save on exit
        save_data.gold = state.gold.get();
        save_data.extension = Some(state.to_save_extension());
        match save::save_game(&save_data, save_path) {
            Ok(()) => println!("Game saved."),
            Err(e) => eprintln!("Warning: failed to save: {e}"),
        }
        return;
    }

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
        "Vale Village v3 -- Loaded {} abilities, {} units, {} enemies",
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

    // Determine the next encounter in the story sequence
    let encounter_id = match load_encounter_id(&save_data) {
        Some(id) => id,
        None => {
            println!("All encounters completed! You've beaten the game!");
            return;
        }
    };

    // Show encounter info
    if let Some(enc) = game_data
        .encounters
        .get(&EncounterId(encounter_id.clone().into()))
    {
        println!("\n>> Next encounter: {} ({})", enc.name, encounter_id);
    } else {
        println!("\n>> Next encounter: {}", encounter_id);
    }

    let result = cli_runner::run_demo_battle(&game_data, &encounter_id);
    match result {
        BattleResult::Victory { xp, gold } => {
            println!("\nVICTORY! +{xp} XP, +{gold} gold");

            // Update save data: gold and global XP
            save_data.xp += xp;
            save_data.gold += gold;

            // Apply progression rewards to each party unit
            for saved_unit in &mut save_data.player_party {
                let unit_id = &saved_unit.unit_id;
                if let Some(unit_def) = game_data.units.get(unit_id) {
                    // Build a UnitProgress from saved level, using xp_for_level as baseline
                    let mut progress = progression::UnitProgress {
                        unit_id: unit_id.clone(),
                        level: saved_unit.level,
                        current_xp: progression::xp_for_level(saved_unit.level),
                    };
                    let old_stats = progression::calculate_stats_at_level(
                        &unit_def.base_stats,
                        &unit_def.growth_rates,
                        saved_unit.level,
                    );

                    let level_result =
                        progression::apply_battle_rewards(&mut progress, xp, unit_def);

                    println!("  {} gained {} XP", unit_def.name, xp);

                    if !level_result.levels_gained.is_empty() {
                        let new_level = progress.level;
                        let new_stats = &level_result.new_stats;
                        let hp_gain = new_stats.hp.get().saturating_sub(old_stats.hp.get());
                        let atk_gain = new_stats.atk.get().saturating_sub(old_stats.atk.get());

                        let ability_str = if level_result.new_abilities.is_empty() {
                            String::new()
                        } else {
                            let names: Vec<String> = level_result
                                .new_abilities
                                .iter()
                                .map(|aid| {
                                    game_data
                                        .abilities
                                        .get(aid)
                                        .map(|a| a.name.clone())
                                        .unwrap_or_else(|| aid.0.clone())
                                })
                                .collect();
                            format!(" Learned: {}", names.join(", "))
                        };

                        println!(
                            "  {} reached level {}! +{} HP, +{} ATK.{}",
                            unit_def.name, new_level, hp_gain, atk_gain, ability_str
                        );
                        saved_unit.level = new_level;
                    }
                }
            }

            // Mark encounter completed (avoid duplicates)
            let enc_id = EncounterId(encounter_id.clone().into());
            if !save_data.completed_encounters.contains(&enc_id) {
                save_data.completed_encounters.push(enc_id);
            }
            save_data.current_encounter_id = next_encounter(&save_data.completed_encounters)
                .map(|next_id| EncounterId(next_id.into()));

            // Show encounter-specific rewards
            if let Some(enc) = game_data
                .encounters
                .get(&EncounterId(encounter_id.clone().into()))
            {
                // Recruitment
                if let Some(ref recruit_id) = enc.recruit {
                    let recruit_name = game_data
                        .units
                        .get(recruit_id)
                        .map(|u| u.name.as_str())
                        .unwrap_or(&recruit_id.0);
                    println!("  {} joins your party!", recruit_name);

                    // Add recruited unit to roster if not already present
                    let already_in_party = save_data
                        .player_party
                        .iter()
                        .any(|u| u.unit_id == *recruit_id);
                    let already_in_roster =
                        save_data.roster.iter().any(|u| u.unit_id == *recruit_id);
                    if !already_in_party && !already_in_roster {
                        save_data.roster.push(save::SavedUnit {
                            unit_id: recruit_id.clone(),
                            level: 1,
                            current_hp: game_data
                                .units
                                .get(recruit_id)
                                .map(|u| u.base_stats.hp.get())
                                .unwrap_or(100),
                            equipment: save::SavedEquipment::default(),
                            djinn: vec![],
                        });
                    }
                }

                // Djinn reward
                if let Some(ref djinn_id) = enc.djinn_reward {
                    let djinn_name = game_data
                        .djinn
                        .get(djinn_id)
                        .map(|d| d.name.as_str())
                        .unwrap_or(&djinn_id.0);
                    println!("  Djinn acquired: {}!", djinn_name);
                    persist_djinn_reward(&mut save_data, djinn_id);
                }

                // Equipment rewards
                if !enc.equipment_rewards.is_empty() {
                    for eq_id in &enc.equipment_rewards {
                        let eq_name = game_data
                            .equipment
                            .get(eq_id)
                            .map(|e| e.name.as_str())
                            .unwrap_or(&eq_id.0);
                        println!("  Equipment acquired: {}!", eq_name);
                        save_data.inventory.push(eq_id.clone());
                    }
                }
            }

            // Save to disk
            match save::save_game(&save_data, save_path) {
                Ok(()) => println!("Game saved."),
                Err(e) => eprintln!("Warning: failed to save game: {e}"),
            }
        }
        BattleResult::Defeat => println!("\nDEFEAT."),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_djinn_reward_persisted_to_save() {
        let mut save_data = save::create_new_game();
        let djinn_id = DjinnId("flint".into());

        persist_djinn_reward(&mut save_data, &djinn_id);
        persist_djinn_reward(&mut save_data, &djinn_id);

        assert_eq!(save_data.team_djinn.len(), 1);
        assert_eq!(save_data.team_djinn[0].djinn_id, djinn_id);
        assert_eq!(save_data.team_djinn[0].state, DjinnState::Good);
    }
}
