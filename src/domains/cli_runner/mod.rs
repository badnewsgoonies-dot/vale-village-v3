#![allow(dead_code, unused_imports)]
//! CLI Battle Runner — first player-reachable surface.
//!
//! Makes `cargo run` execute an actual battle using the battle_engine,
//! printing each round's events to the terminal.

use std::io::{self, Write};

use crate::domains::ai::AiStrategy;
use crate::domains::battle_engine::{
    check_battle_end, execute_round, new_battle, plan_action, plan_enemy_actions_with_ai, Battle,
    BattleEvent, BattleResult, EnemyUnitData, PlanError, PlayerUnitData,
};
use crate::domains::data_loader::GameData;
use crate::domains::djinn::DjinnSlots;
use crate::domains::equipment::{self, EquipmentEffects, EquipmentLoadout};
use crate::domains::djinn;
use crate::shared::{
    AbilityCategory, AbilityId, BattleAction, DjinnId, DjinnState, EncounterId, EnemyId,
    EquipmentId, EquipmentSlot, Side, TargetMode, TargetRef, UnitDef, UnitId,
};

// ── Public API ──────────────────────────────────────────────────────

/// Build a PlayerUnitData with equipment and djinn equipped.
///
/// Attempts to equip the given weapon/armor IDs and djinn ID.
/// Gracefully skips any item or djinn not found in game_data.
fn build_player_unit(
    unit_def: &UnitDef,
    weapon_id: &str,
    armor_id: &str,
    djinn_id_str: &str,
    game_data: &GameData,
) -> PlayerUnitData {
    let mut loadout = EquipmentLoadout::default();

    // Try to equip weapon
    let weapon_eid = EquipmentId(weapon_id.to_string());
    if let Some(weapon_def) = game_data.equipment.get(&weapon_eid) {
        let _ = equipment::equip(
            &mut loadout,
            EquipmentSlot::Weapon,
            weapon_eid,
            weapon_def,
            unit_def.element,
        );
    }

    // Try to equip armor
    let armor_eid = EquipmentId(armor_id.to_string());
    if let Some(armor_def) = game_data.equipment.get(&armor_eid) {
        let _ = equipment::equip(
            &mut loadout,
            EquipmentSlot::Armor,
            armor_eid,
            armor_def,
            unit_def.element,
        );
    }

    // Compute equipment effects
    let eq_effects = equipment::compute_equipment_effects(&loadout, &game_data.equipment);

    // Try to equip djinn
    let mut djinn_slots = DjinnSlots::new();
    let djinn_id = DjinnId(djinn_id_str.to_string());
    if game_data.djinn.contains_key(&djinn_id) {
        djinn_slots.add(djinn_id);
    }

    PlayerUnitData {
        id: unit_def.id.0.clone(),
        base_stats: unit_def.base_stats,
        equipment: loadout,
        djinn_slots,
        mana_contribution: unit_def.mana_contribution,
        equipment_effects: eq_effects,
    }
}

/// Format equipment and djinn info for a player unit at battle start.
fn format_unit_gear(
    unit_id: &str,
    hp: u16,
    loadout: &EquipmentLoadout,
    djinn_slots: &DjinnSlots,
    game_data: &GameData,
) -> String {
    let mut gear_parts: Vec<String> = Vec::new();

    // Equipment name lookup
    if let Some(ref wid) = loadout.weapon {
        if let Some(def) = game_data.equipment.get(wid) {
            gear_parts.push(def.name.clone());
        }
    }
    if let Some(ref aid) = loadout.armor {
        if let Some(def) = game_data.equipment.get(aid) {
            gear_parts.push(def.name.clone());
        }
    }

    // Djinn info
    for inst in &djinn_slots.slots {
        let name = game_data
            .djinn
            .get(&inst.djinn_id)
            .map(|d| d.name.clone())
            .unwrap_or_else(|| inst.djinn_id.0.clone());
        gear_parts.push(format!("{}({:?})", name, inst.state));
    }

    let gear_str = if gear_parts.is_empty() {
        String::new()
    } else {
        format!(" [{}]", gear_parts.join(", "))
    };

    format!("{} (HP:{}){}", unit_id, hp, gear_str)
}

/// Pick the first psynergy/mana-costing ability from a unit's ability list
/// that exists in game_data.abilities and has mana_cost > 0.
/// Returns None if no such ability is found.
fn find_first_psynergy_ability(unit_def: &UnitDef, game_data: &GameData) -> Option<AbilityId> {
    for ap in &unit_def.abilities {
        if let Some(adef) = game_data.abilities.get(&ap.ability_id) {
            if adef.mana_cost > 0 {
                return Some(ap.ability_id.clone());
            }
        }
    }
    None
}

fn format_djinn_states(djinn_slots: &DjinnSlots, game_data: &GameData) -> String {
    if djinn_slots.slots.is_empty() {
        return "none".to_string();
    }

    djinn_slots
        .slots
        .iter()
        .map(|inst| {
            let name = game_data
                .djinn
                .get(&inst.djinn_id)
                .map(|djinn| djinn.name.clone())
                .unwrap_or_else(|| inst.djinn_id.0.clone());
            format!("{}({:?})", name, inst.state)
        })
        .collect::<Vec<_>>()
        .join(", ")
}

fn ability_category_label(category: AbilityCategory) -> &'static str {
    match category {
        AbilityCategory::Physical => "physical",
        AbilityCategory::Psynergy => "psynergy",
        AbilityCategory::Healing => "healing",
        AbilityCategory::Buff => "buff",
        AbilityCategory::Debuff => "debuff",
    }
}

fn target_mode_label(target_mode: TargetMode) -> &'static str {
    match target_mode {
        TargetMode::SingleEnemy => "single-enemy",
        TargetMode::AllEnemies => "all-enemies",
        TargetMode::SingleAlly => "single-ally",
        TargetMode::AllAllies => "all-allies",
        TargetMode::SelfOnly => "self-only",
    }
}

fn ability_menu_entry(battle: &Battle, ability_id: &AbilityId) -> String {
    battle
        .ability_defs
        .get(ability_id)
        .map(|ability| {
            let multi_hit = if ability.hit_count > 1 {
                format!(", {}-hit", ability.hit_count)
            } else {
                String::new()
            };
            format!(
                "{} (cost:{}, power:{}, {}, {}{})",
                ability.name,
                ability.mana_cost,
                ability.base_power,
                ability_category_label(ability.category),
                target_mode_label(ability.targets),
                multi_hit
            )
        })
        .unwrap_or_else(|| format!("{} (missing definition)", ability_id.0))
}

fn prompt_for_usize(prompt: &str, min: usize, max: usize, default_on_eof: usize) -> usize {
    loop {
        print!("{prompt}");
        let _ = io::stdout().flush();

        let mut input = String::new();
        match io::stdin().read_line(&mut input) {
            Ok(0) => {
                println!("  End of input detected. Using {default_on_eof}.");
                return default_on_eof;
            }
            Ok(_) => match input.trim().parse::<usize>() {
                Ok(value) if (min..=max).contains(&value) => return value,
                Ok(_) => {
                    println!("  Please enter a number from {min} to {max}.");
                }
                Err(_) => {
                    println!("  Please enter a valid number.");
                }
            },
            Err(error) => {
                println!("  Failed to read input: {error}. Try again.");
            }
        }
    }
}

fn alive_targets_for_side(battle: &Battle, side: Side) -> Vec<TargetRef> {
    match side {
        Side::Player => battle
            .player_units
            .iter()
            .enumerate()
            .filter(|(_, unit)| unit.unit.is_alive)
            .map(|(index, _)| TargetRef {
                side: Side::Player,
                index: index as u8,
            })
            .collect(),
        Side::Enemy => battle
            .enemies
            .iter()
            .enumerate()
            .filter(|(_, unit)| unit.unit.is_alive)
            .map(|(index, _)| TargetRef {
                side: Side::Enemy,
                index: index as u8,
            })
            .collect(),
    }
}

fn target_hp_snapshot(battle: &Battle, target: TargetRef) -> Option<(u16, u16)> {
    match target.side {
        Side::Player => battle
            .player_units
            .get(target.index as usize)
            .map(|unit| (unit.unit.current_hp, unit.unit.stats.hp)),
        Side::Enemy => battle
            .enemies
            .get(target.index as usize)
            .map(|unit| (unit.unit.current_hp, unit.unit.stats.hp)),
    }
}

fn print_target_menu(battle: &Battle, targets: &[TargetRef], heading: &str) {
    println!("{heading}");
    for (menu_index, target) in targets.iter().enumerate() {
        if let Some((current_hp, max_hp)) = target_hp_snapshot(battle, *target) {
            println!(
                "  [{}] {} (HP:{}/{}) {}",
                menu_index,
                format_target(target, battle),
                current_hp,
                max_hp,
                hp_bar(current_hp, max_hp)
            );
        }
    }
}

fn prompt_for_target(
    battle: &Battle,
    targets: &[TargetRef],
    heading: &str,
    prompt: &str,
) -> TargetRef {
    print_target_menu(battle, targets, heading);
    let selected = prompt_for_usize(prompt, 0, targets.len() - 1, 0);
    targets[selected]
}

fn print_planning_overview(battle: &Battle, current_player_index: usize, game_data: &GameData) {
    println!(
        "  Mana pool available: {}/{}",
        battle.mana_pool.projected_mana, battle.mana_pool.max_mana
    );
    println!("  Allies:");
    for (index, unit) in battle.player_units.iter().enumerate() {
        if !unit.unit.is_alive {
            continue;
        }

        let focus = if index == current_player_index {
            ">"
        } else {
            " "
        };
        println!(
            "  {} [{}] {}: {}/{} {} Djinn: {}",
            focus,
            index,
            unit.unit.id,
            unit.unit.current_hp,
            unit.unit.stats.hp,
            hp_bar(unit.unit.current_hp, unit.unit.stats.hp),
            format_djinn_states(&unit.djinn_slots, game_data)
        );
    }

    println!("  Enemies:");
    for (index, unit) in battle.enemies.iter().enumerate() {
        if !unit.unit.is_alive {
            continue;
        }

        println!(
            "    [{}] {}: {}/{} {}",
            index,
            unit.unit.id,
            unit.unit.current_hp,
            unit.unit.stats.hp,
            hp_bar(unit.unit.current_hp, unit.unit.stats.hp)
        );
    }
}

fn build_ability_targets(
    battle: &Battle,
    unit_ref: TargetRef,
    ability_id: &AbilityId,
) -> Option<Vec<TargetRef>> {
    let ability = battle.ability_defs.get(ability_id)?;
    match ability.targets {
        TargetMode::SingleEnemy => {
            let targets = alive_targets_for_side(battle, Side::Enemy);
            if targets.is_empty() {
                None
            } else {
                Some(vec![prompt_for_target(
                    battle,
                    &targets,
                    "  Targets:",
                    "Choose target: ",
                )])
            }
        }
        TargetMode::AllEnemies => {
            let targets = alive_targets_for_side(battle, Side::Enemy);
            if targets.is_empty() {
                None
            } else {
                println!("  {} will target all alive enemies.", ability.name);
                Some(targets)
            }
        }
        TargetMode::SingleAlly => {
            let targets = alive_targets_for_side(battle, Side::Player);
            if targets.is_empty() {
                None
            } else {
                Some(vec![prompt_for_target(
                    battle,
                    &targets,
                    "  Allies:",
                    "Choose target: ",
                )])
            }
        }
        TargetMode::AllAllies => {
            let targets = alive_targets_for_side(battle, Side::Player);
            if targets.is_empty() {
                None
            } else {
                println!("  {} will target all alive allies.", ability.name);
                Some(targets)
            }
        }
        TargetMode::SelfOnly => {
            println!(
                "  {} targets {}.",
                ability.name,
                format_target(&unit_ref, battle)
            );
            Some(vec![unit_ref])
        }
    }
}

fn try_plan_action(
    battle: &mut Battle,
    unit_ref: TargetRef,
    action: BattleAction,
    unit_name: &str,
) -> bool {
    match plan_action(battle, unit_ref, action) {
        Ok(()) => true,
        Err(PlanError::InsufficientMana) => {
            println!("  Not enough mana for that choice. Pick again.");
            false
        }
        Err(PlanError::InvalidTarget) => {
            println!("  That target is not valid. Pick again.");
            false
        }
        Err(PlanError::UnitCannotAct) => {
            println!("  {unit_name} cannot act this round.");
            true
        }
        Err(error) => {
            println!("  Could not plan action: {error:?}. Pick again.");
            false
        }
    }
}

fn plan_auto_action_for_player(
    battle: &mut Battle,
    player_index: usize,
    current_round: u32,
    auto_ability_ids: &[Option<AbilityId>],
) {
    let target_idx = battle.enemies.iter().position(|enemy| enemy.unit.is_alive);
    let target_idx = match target_idx {
        Some(index) => index,
        None => return,
    };

    let unit_ref = TargetRef {
        side: Side::Player,
        index: player_index as u8,
    };
    let enemy_target = TargetRef {
        side: Side::Enemy,
        index: target_idx as u8,
    };

    // On round 4, activate a djinn if any are in Good state
    if current_round == 4 {
        let good_djinn_idx = battle.player_units[player_index]
            .djinn_slots
            .slots
            .iter()
            .position(|inst| inst.state == DjinnState::Good);

        if let Some(djinn_idx) = good_djinn_idx {
            let djinn_name = {
                let inst = &battle.player_units[player_index].djinn_slots.slots[djinn_idx];
                battle
                    .djinn_defs
                    .get(&inst.djinn_id)
                    .map(|d| d.name.clone())
                    .unwrap_or_else(|| inst.djinn_id.0.clone())
            };
            println!(
                "  >> {} activates {}!",
                battle.player_units[player_index].unit.id, djinn_name
            );
            let action = BattleAction::ActivateDjinn {
                djinn_index: djinn_idx as u8,
            };
            if plan_action(battle, unit_ref, action).is_ok() {
                return;
            }
        }
    }

    let mut used_ability = false;
    #[allow(clippy::manual_is_multiple_of)]
    if current_round % 3 == 0 {
        if let Some(ability_id) = auto_ability_ids
            .get(player_index)
            .and_then(|id| id.as_ref())
        {
            let can_use = battle
                .ability_defs
                .get(ability_id)
                .map(|ability| battle.mana_pool.current_mana >= ability.mana_cost)
                .unwrap_or(false);

            if can_use {
                let ability_name = battle
                    .ability_defs
                    .get(ability_id)
                    .map(|ability| ability.name.clone())
                    .unwrap_or_else(|| ability_id.0.clone());
                let enemy_name = battle
                    .enemies
                    .get(target_idx)
                    .map(|enemy| enemy.unit.id.clone())
                    .unwrap_or_else(|| format!("Enemy[{target_idx}]"));
                println!(
                    "  >> {} uses {} on {}!",
                    battle.player_units[player_index].unit.id, ability_name, enemy_name
                );
                let action = BattleAction::UseAbility {
                    ability_id: (*ability_id).clone(),
                    targets: vec![enemy_target],
                };
                if plan_action(battle, unit_ref, action).is_ok() {
                    used_ability = true;
                }
            }
        }
    }

    if !used_ability {
        let action = BattleAction::Attack {
            target: enemy_target,
        };
        let _ = plan_action(battle, unit_ref, action);
    }
}

fn plan_interactive_action_for_player(
    battle: &mut Battle,
    player_index: usize,
    current_round: u32,
    auto_ability_ids: &[Option<AbilityId>],
    unit_ability_ids: &[AbilityId],
    game_data: &GameData,
) {
    if !battle.player_units[player_index].unit.is_alive
        || alive_targets_for_side(battle, Side::Enemy).is_empty()
    {
        return;
    }

    let unit_ref = TargetRef {
        side: Side::Player,
        index: player_index as u8,
    };

    loop {
        let unit = &battle.player_units[player_index].unit;
        println!(
            "\n── Planning: {} (HP:{}/{}, Mana: {}/{}) ──",
            unit.id,
            unit.current_hp,
            unit.stats.hp,
            battle.mana_pool.projected_mana,
            battle.mana_pool.max_mana
        );
        print_planning_overview(battle, player_index, game_data);
        println!("  [1] ATTACK -> target?");
        println!("  [2] ABILITY -> which ability?");
        println!("  [3] AUTO (let AI choose)");
        println!("  [4] DJINN -> activate a djinn");
        println!("  [5] SUMMON -> use standby djinn for summon");

        match prompt_for_usize("Choose action (1/2/3/4/5): ", 1, 5, 3) {
            1 => {
                let targets = alive_targets_for_side(battle, Side::Enemy);
                if targets.is_empty() {
                    return;
                }

                let target = prompt_for_target(battle, &targets, "  Targets:", "Choose target: ");
                let unit_name = battle.player_units[player_index].unit.id.clone();
                if try_plan_action(
                    battle,
                    unit_ref,
                    BattleAction::Attack { target },
                    &unit_name,
                ) {
                    break;
                }
            }
            2 => {
                let available_abilities: Vec<AbilityId> = unit_ability_ids
                    .iter()
                    .filter(|ability_id| battle.ability_defs.contains_key(*ability_id))
                    .cloned()
                    .collect();

                if available_abilities.is_empty() {
                    println!("  No abilities are available for this unit.");
                    continue;
                }

                println!("  Abilities:");
                for (menu_index, ability_id) in available_abilities.iter().enumerate() {
                    let mana_note = battle
                        .ability_defs
                        .get(ability_id)
                        .filter(|ability| battle.mana_pool.projected_mana < ability.mana_cost)
                        .map(|_| " [insufficient mana]")
                        .unwrap_or("");
                    println!(
                        "  [{}] {}{}",
                        menu_index,
                        ability_menu_entry(battle, ability_id),
                        mana_note
                    );
                }
                println!("  Mana available: {}", battle.mana_pool.projected_mana);

                let selected =
                    prompt_for_usize("Choose ability: ", 0, available_abilities.len() - 1, 0);
                let ability_id = available_abilities[selected].clone();

                if let Some(ability) = battle.ability_defs.get(&ability_id) {
                    if battle.mana_pool.projected_mana < ability.mana_cost {
                        println!("  Not enough mana left for {}.", ability.name);
                        continue;
                    }
                }

                let Some(targets) = build_ability_targets(battle, unit_ref, &ability_id) else {
                    println!("  No valid targets are available for that ability.");
                    continue;
                };

                let unit_name = battle.player_units[player_index].unit.id.clone();
                if try_plan_action(
                    battle,
                    unit_ref,
                    BattleAction::UseAbility {
                        ability_id,
                        targets,
                    },
                    &unit_name,
                ) {
                    break;
                }
            }
            3 => {
                plan_auto_action_for_player(battle, player_index, current_round, auto_ability_ids);
                break;
            }
            4 => {
                // DJINN -> activate a Good-state djinn
                let good_djinn: Vec<(usize, String)> = battle.player_units[player_index]
                    .djinn_slots
                    .slots
                    .iter()
                    .enumerate()
                    .filter(|(_, inst)| inst.state == DjinnState::Good)
                    .map(|(idx, inst)| {
                        let name = game_data
                            .djinn
                            .get(&inst.djinn_id)
                            .map(|d| d.name.clone())
                            .unwrap_or_else(|| inst.djinn_id.0.clone());
                        (idx, name)
                    })
                    .collect();

                if good_djinn.is_empty() {
                    println!("  No djinn in Good state to activate.");
                    continue;
                }

                println!("  Good-state djinn:");
                for (menu_i, (slot_idx, name)) in good_djinn.iter().enumerate() {
                    println!(
                        "  [{}] {} (slot {}) — activate for immediate effect",
                        menu_i, name, slot_idx
                    );
                }

                let selected =
                    prompt_for_usize("Choose djinn to activate: ", 0, good_djinn.len() - 1, 0);
                let (djinn_slot_index, djinn_name) = &good_djinn[selected];
                println!("  >> Activating {}!", djinn_name);

                let unit_name = battle.player_units[player_index].unit.id.clone();
                if try_plan_action(
                    battle,
                    unit_ref,
                    BattleAction::ActivateDjinn {
                        djinn_index: *djinn_slot_index as u8,
                    },
                    &unit_name,
                ) {
                    break;
                }
            }
            5 => {
                // SUMMON -> use Good djinn for a summon
                let good_count = battle.player_units[player_index]
                    .djinn_slots
                    .good_count();
                let tiers = djinn::get_available_summons(good_count);

                if tiers.is_empty() {
                    println!("  No djinn in Good state for summoning.");
                    continue;
                }

                println!("  Available summon tiers (Good djinn: {}):", good_count);
                for (menu_i, tier) in tiers.iter().enumerate() {
                    println!(
                        "  [{}] Tier {} (requires {} Good djinn)",
                        menu_i, tier.tier, tier.required_good
                    );
                }

                let selected =
                    prompt_for_usize("Choose summon tier: ", 0, tiers.len() - 1, 0);
                let chosen_tier = &tiers[selected];
                let needed = chosen_tier.required_good as usize;

                // Collect indices of Good djinn for selection
                let good_indices: Vec<(usize, String)> = battle.player_units[player_index]
                    .djinn_slots
                    .slots
                    .iter()
                    .enumerate()
                    .filter(|(_, inst)| inst.state == DjinnState::Good)
                    .map(|(idx, inst)| {
                        let name = game_data
                            .djinn
                            .get(&inst.djinn_id)
                            .map(|d| d.name.clone())
                            .unwrap_or_else(|| inst.djinn_id.0.clone());
                        (idx, name)
                    })
                    .collect();

                // Auto-select the first N Good djinn for the summon
                let selected_indices: Vec<u8> = good_indices
                    .iter()
                    .take(needed)
                    .map(|(idx, _)| *idx as u8)
                    .collect();
                let selected_names: Vec<&str> = good_indices
                    .iter()
                    .take(needed)
                    .map(|(_, name)| name.as_str())
                    .collect();
                println!(
                    "  >> Summoning with: {}!",
                    selected_names.join(" + ")
                );

                let unit_name = battle.player_units[player_index].unit.id.clone();
                if try_plan_action(
                    battle,
                    unit_ref,
                    BattleAction::Summon {
                        djinn_indices: selected_indices,
                    },
                    &unit_name,
                ) {
                    break;
                }
            }
            _ => unreachable!(),
        }
    }
}

/// Run a demo battle using loaded game data for the specified encounter.
///
/// - 2 player units: Adept + Blaze
/// - Enemies loaded from the given encounter_id (falls back to house-02, then manual enemies)
/// - Auto-play loop: each player unit attacks first alive enemy
/// - Every 3rd round, first player uses an ability if mana allows
/// - Enemies attack back: each alive enemy attacks first alive player
/// - Both sides can win or lose
/// - Stalemate guard at round 20
pub fn run_demo_battle(game_data: &GameData, encounter_id: &str) -> BattleResult {
    // Look up player units
    let adept = game_data
        .units
        .get(&UnitId("adept".to_string()))
        .expect("sample data must contain unit 'adept'");
    let blaze = game_data
        .units
        .get(&UnitId("blaze".to_string()))
        .expect("sample data must contain unit 'blaze'");

    // Build player units with equipment and djinn
    // Adept (Venus): Bronze Sword + Leather Vest + Flint
    // Blaze (Mars): Wooden Axe + Leather Vest + Forge
    let adept_data = build_player_unit(adept, "bronze-sword", "leather-vest", "flint", game_data);
    let blaze_data = build_player_unit(blaze, "wooden-axe", "leather-vest", "forge", game_data);

    // Pre-compute ability IDs for auto-play usage
    let adept_ability = find_first_psynergy_ability(adept, game_data);
    let blaze_ability = find_first_psynergy_ability(blaze, game_data);

    // Stash gear info before moving into battle
    let adept_hp = (adept_data.base_stats.hp as i32
        + adept_data.equipment_effects.total_stat_bonus.hp as i32)
        .max(1) as u16;
    let adept_gear_info = format_unit_gear(
        &adept_data.id,
        adept_hp,
        &adept_data.equipment,
        &adept_data.djinn_slots,
        game_data,
    );
    let blaze_hp = (blaze_data.base_stats.hp as i32
        + blaze_data.equipment_effects.total_stat_bonus.hp as i32)
        .max(1) as u16;
    let blaze_gear_info = format_unit_gear(
        &blaze_data.id,
        blaze_hp,
        &blaze_data.equipment,
        &blaze_data.djinn_slots,
        game_data,
    );

    let player_data: Vec<PlayerUnitData> = vec![adept_data, blaze_data];

    // Load encounter by ID, fall back to house-02, then manual enemy list
    let encounter = game_data
        .encounters
        .get(&EncounterId(encounter_id.to_string()))
        .or_else(|| game_data.encounters.get(&EncounterId("house-02".to_string())));

    let enemy_data: Vec<EnemyUnitData> = if let Some(enc) = encounter {
        enc.enemies
            .iter()
            .flat_map(|enc_enemy| {
                let enemy_def = game_data.enemies.get(&enc_enemy.enemy_id);
                match enemy_def {
                    Some(def) => (0..enc_enemy.count)
                        .map(|_| EnemyUnitData {
                            enemy_def: def.clone(),
                        })
                        .collect::<Vec<_>>(),
                    None => {
                        eprintln!(
                            "Warning: encounter enemy '{}' not found in game_data, skipping",
                            enc_enemy.enemy_id.0
                        );
                        vec![]
                    }
                }
            })
            .collect()
    } else {
        // Fallback: use mercury-slime + earth-scout directly
        let slime = game_data
            .enemies
            .get(&EnemyId("mercury-slime".to_string()))
            .expect("sample data must contain enemy 'mercury-slime'");
        let scout = game_data
            .enemies
            .get(&EnemyId("earth-scout".to_string()))
            .expect("sample data must contain enemy 'earth-scout'");
        vec![
            EnemyUnitData {
                enemy_def: slime.clone(),
            },
            EnemyUnitData {
                enemy_def: scout.clone(),
            },
        ]
    };

    assert!(
        !enemy_data.is_empty(),
        "encounter must have at least one valid enemy"
    );

    // Build ability_defs and djinn_defs maps for the battle
    let ability_defs = game_data.abilities.clone();
    let djinn_defs = game_data.djinn.clone();

    let mut battle = new_battle(
        player_data,
        enemy_data,
        game_data.config.clone(),
        ability_defs,
        djinn_defs,
    );

    println!("\n=== BATTLE START ===");
    println!("Party: {} + {}", adept_gear_info, blaze_gear_info);
    print!("Enemies:");
    for eu in &battle.enemies {
        print!(" {} (HP:{})", eu.unit.id, eu.unit.stats.hp);
    }
    println!("\n");

    let auto_mode = cfg!(test) || std::env::args().any(|arg| arg == "--auto");

    // Collect ability IDs for auto-play and interactive menus (indexed by player index)
    let unit_auto_ability_ids: Vec<Option<AbilityId>> = vec![adept_ability, blaze_ability];
    let unit_ability_ids: Vec<Vec<AbilityId>> = vec![
        adept
            .abilities
            .iter()
            .map(|progression| progression.ability_id.clone())
            .collect(),
        blaze
            .abilities
            .iter()
            .map(|progression| progression.ability_id.clone())
            .collect(),
    ];

    loop {
        // Stalemate guard
        if battle.round > 20 {
            println!("\n*** STALEMATE after 20 rounds ***");
            return BattleResult::Defeat;
        }

        let current_round = battle.round;

        // Planning phase
        for pi in 0..battle.player_units.len() {
            if !battle.player_units[pi].unit.is_alive {
                continue;
            }
            if auto_mode {
                plan_auto_action_for_player(&mut battle, pi, current_round, &unit_auto_ability_ids);
            } else {
                plan_interactive_action_for_player(
                    &mut battle,
                    pi,
                    current_round,
                    &unit_auto_ability_ids,
                    unit_ability_ids.get(pi).map(Vec::as_slice).unwrap_or(&[]),
                    game_data,
                );
            }
        }

        // Enemy planning: use AI strategy for smart enemy decisions
        plan_enemy_actions_with_ai(&mut battle, AiStrategy::Balanced);

        // Execute
        let events = execute_round(&mut battle);

        // Print events
        for event in &events {
            println!("  {}", format_event(event, &battle));
        }

        // Show HP state
        format_battle_state(&battle);
        println!();

        // Check end
        if let Some(result) = check_battle_end(&battle) {
            match &result {
                BattleResult::Victory { .. } => {
                    println!("=== VICTORY ===");
                }
                BattleResult::Defeat => {
                    println!("=== DEFEAT — your party has fallen ===");
                }
            }
            return result;
        }
    }
}

// ── Event Formatting ────────────────────────────────────────────────

/// Produce human-readable text for a BattleEvent.
pub fn format_event(event: &BattleEvent, battle: &Battle) -> String {
    match event {
        BattleEvent::DamageDealt(dd) => {
            let crit_suffix = if dd.is_crit { " (CRIT!)" } else { "" };
            format!(
                "{} deals {} damage to {}{}",
                format_target(&dd.source, battle),
                dd.amount,
                format_target(&dd.target, battle),
                crit_suffix
            )
        }
        BattleEvent::HealingDone(hd) => {
            format!(
                "{} heals {} for {}",
                format_target(&hd.source, battle),
                format_target(&hd.target, battle),
                hd.amount
            )
        }
        BattleEvent::StatusApplied(sa) => {
            format!(
                "{} is afflicted with {:?}",
                format_target(&sa.target, battle),
                sa.effect.effect_type
            )
        }
        BattleEvent::CritTriggered(unit, hit_number) => {
            format!(
                "CRITICAL HIT! {} on hit #{}",
                format_target(unit, battle),
                hit_number
            )
        }
        BattleEvent::BarrierBlocked(unit) => {
            format!("{}'s barrier absorbs the hit!", format_target(unit, battle))
        }
        BattleEvent::UnitDefeated(ud) => {
            format!("{} is defeated!", format_target(&ud.unit, battle))
        }
        BattleEvent::ManaChanged(mc) => {
            format!("Mana pool: {} -> {}", mc.old_value, mc.new_value)
        }
        BattleEvent::RoundStarted(n) => {
            format!("══ Round {} ══", n)
        }
        BattleEvent::RoundEnded(n) => {
            format!("── End of Round {} ──", n)
        }
        BattleEvent::DjinnChanged(dc) => {
            let djinn_name = battle
                .djinn_defs
                .get(&dc.djinn_id)
                .map(|d| d.name.clone())
                .unwrap_or_else(|| dc.djinn_id.0.clone());
            let unit_name = format_target(&dc.unit, battle);
            match (dc.old_state, dc.new_state) {
                (DjinnState::Good, DjinnState::Recovery) => {
                    let turns = dc.recovery_turns.unwrap_or(0);
                    format!(
                        "{} activates {}! {} enters Recovery ({} turns)",
                        unit_name, djinn_name, djinn_name, turns
                    )
                }
                (DjinnState::Recovery, DjinnState::Good) => {
                    format!("{} recovers to Good state on {}", djinn_name, unit_name)
                }
                _ => {
                    format!(
                        "Djinn {} on {}: {:?} -> {:?}",
                        djinn_name, unit_name, dc.old_state, dc.new_state
                    )
                }
            }
        }
        BattleEvent::EnemyAbilityUsed {
            actor,
            ability_name,
            targets,
        } => {
            let actor_name = format_target(actor, battle);
            let target_names: Vec<String> = targets
                .iter()
                .map(|t| format_target(t, battle))
                .collect();
            format!(
                "{} uses {} on {}!",
                actor_name,
                ability_name,
                target_names.join(", ")
            )
        }
    }
}

/// Look up a unit's display name from battle state.
pub fn format_target(target: &TargetRef, battle: &Battle) -> String {
    match target.side {
        Side::Player => {
            if let Some(u) = battle.player_units.get(target.index as usize) {
                u.unit.id.clone()
            } else {
                format!("Player[{}]", target.index)
            }
        }
        Side::Enemy => {
            if let Some(e) = battle.enemies.get(target.index as usize) {
                e.unit.id.clone()
            } else {
                format!("Enemy[{}]", target.index)
            }
        }
    }
}

/// Print HP bars for all units between rounds.
pub fn format_battle_state(battle: &Battle) {
    println!("  ┌─ Party ─────────────────────────");
    for (i, u) in battle.player_units.iter().enumerate() {
        let bar = hp_bar(u.unit.current_hp, u.unit.stats.hp);
        let alive_mark = if u.unit.is_alive { " " } else { "X" };
        println!(
            "  │ [{}] {} {}: {}/{} {}",
            i, alive_mark, u.unit.id, u.unit.current_hp, u.unit.stats.hp, bar
        );
    }
    println!("  ├─ Enemies ─────────────────────────");
    for (i, e) in battle.enemies.iter().enumerate() {
        let bar = hp_bar(e.unit.current_hp, e.unit.stats.hp);
        let alive_mark = if e.unit.is_alive { " " } else { "X" };
        println!(
            "  │ [{}] {} {}: {}/{} {}",
            i, alive_mark, e.unit.id, e.unit.current_hp, e.unit.stats.hp, bar
        );
    }
    println!("  └───────────────────────────────────");
}

/// Build a simple ASCII HP bar.
fn hp_bar(current: u16, max: u16) -> String {
    let width = 20;
    if max == 0 {
        return format!("[{}]", ".".repeat(width));
    }
    let filled = ((current as u32 * width as u32) / max as u32) as usize;
    let empty = width - filled;
    format!("[{}{}]", "#".repeat(filled), ".".repeat(empty))
}

// ── Tests ───────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domains::data_loader::load_game_data;
    use std::path::PathBuf;

    fn sample_dir() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("data")
            .join("sample")
    }

    fn load_sample_data() -> GameData {
        load_game_data(&sample_dir()).expect("sample data must load")
    }

    #[test]
    fn test_demo_battle_completes_without_panic() {
        let game_data = load_sample_data();
        let result = run_demo_battle(&game_data, "training-dummy");
        // Battle must finish as Victory or Defeat — no panic
        match result {
            BattleResult::Victory { xp, gold } => {
                assert!(xp > 0 || gold > 0, "Victory should have rewards");
            }
            BattleResult::Defeat => {
                // Defeat is also acceptable (e.g. stalemate)
            }
        }
    }

    #[test]
    fn test_format_event_handles_all_variants() {
        let game_data = load_sample_data();
        let ability_defs = game_data.abilities.clone();

        // Create a minimal battle for name lookups
        let adept = game_data.units.get(&UnitId("adept".to_string())).unwrap();
        let slime = game_data
            .enemies
            .get(&EnemyId("mercury-slime".to_string()))
            .unwrap();

        let battle = new_battle(
            vec![PlayerUnitData {
                id: adept.id.0.clone(),
                base_stats: adept.base_stats,
                equipment: EquipmentLoadout::default(),
                djinn_slots: DjinnSlots::new(),
                mana_contribution: adept.mana_contribution,
                equipment_effects: EquipmentEffects::default(),
            }],
            vec![EnemyUnitData {
                enemy_def: slime.clone(),
            }],
            game_data.config.clone(),
            ability_defs,
            game_data.djinn.clone(),
        );

        let player_ref = TargetRef {
            side: Side::Player,
            index: 0,
        };
        let enemy_ref = TargetRef {
            side: Side::Enemy,
            index: 0,
        };

        // Test each variant
        let events: Vec<BattleEvent> = vec![
            BattleEvent::DamageDealt(crate::shared::DamageDealt {
                source: player_ref,
                target: enemy_ref,
                amount: 10,
                damage_type: crate::shared::DamageType::Physical,
                is_crit: false,
            }),
            BattleEvent::DamageDealt(crate::shared::DamageDealt {
                source: player_ref,
                target: enemy_ref,
                amount: 25,
                damage_type: crate::shared::DamageType::Physical,
                is_crit: true,
            }),
            BattleEvent::HealingDone(crate::shared::HealingDone {
                source: player_ref,
                target: player_ref,
                amount: 15,
            }),
            BattleEvent::StatusApplied(crate::shared::StatusApplied {
                source: player_ref,
                target: enemy_ref,
                effect: crate::shared::StatusEffect {
                    effect_type: crate::shared::StatusEffectType::Burn,
                    duration: 3,
                    burn_percent: Some(0.10),
                    poison_percent: None,
                    freeze_threshold: None,
                },
            }),
            BattleEvent::CritTriggered(player_ref, 3),
            BattleEvent::BarrierBlocked(enemy_ref),
            BattleEvent::UnitDefeated(crate::shared::UnitDefeated { unit: enemy_ref }),
            BattleEvent::ManaChanged(crate::shared::ManaPoolChanged {
                old_value: 2,
                new_value: 4,
            }),
            BattleEvent::RoundStarted(1),
            BattleEvent::RoundEnded(1),
            BattleEvent::DjinnChanged(crate::shared::DjinnStateChanged {
                djinn_id: crate::shared::DjinnId("test-djinn".to_string()),
                unit: player_ref,
                old_state: crate::shared::DjinnState::Good,
                new_state: crate::shared::DjinnState::Recovery,
                recovery_turns: Some(2),
            }),
        ];

        for event in &events {
            let text = format_event(event, &battle);
            assert!(!text.is_empty(), "format_event must produce non-empty text");
        }

        // Spot-check specific formats
        let damage_text = format_event(&events[0], &battle);
        assert!(
            damage_text.contains("deals"),
            "damage event: {}",
            damage_text
        );
        assert!(damage_text.contains("10"), "damage amount: {}", damage_text);

        let crit_text = format_event(&events[1], &battle);
        assert!(crit_text.contains("CRIT"), "crit suffix: {}", crit_text);

        let heal_text = format_event(&events[2], &battle);
        assert!(heal_text.contains("heals"), "heal event: {}", heal_text);

        let status_text = format_event(&events[3], &battle);
        assert!(
            status_text.contains("afflicted"),
            "status event: {}",
            status_text
        );

        let barrier_text = format_event(&events[5], &battle);
        assert!(
            barrier_text.contains("barrier"),
            "barrier event: {}",
            barrier_text
        );

        let defeated_text = format_event(&events[6], &battle);
        assert!(
            defeated_text.contains("defeated"),
            "defeated event: {}",
            defeated_text
        );

        let round_start_text = format_event(&events[8], &battle);
        assert!(
            round_start_text.contains("Round 1"),
            "round start: {}",
            round_start_text
        );

        let round_end_text = format_event(&events[9], &battle);
        assert!(
            round_end_text.contains("Round 1"),
            "round end: {}",
            round_end_text
        );
    }

    #[test]
    fn test_stalemate_guard_triggers() {
        let game_data = load_sample_data();

        // Create a battle where enemies have very high HP so stalemate triggers
        let adept = game_data.units.get(&UnitId("adept".to_string())).unwrap();

        // Make a custom enemy with absurd HP and low ATK so player survives 20 rounds
        let mut tanky_enemy = game_data
            .enemies
            .get(&EnemyId("mercury-slime".to_string()))
            .unwrap()
            .clone();
        tanky_enemy.stats.hp = 60000;
        tanky_enemy.stats.def = 200;
        tanky_enemy.stats.atk = 1;

        let mut battle = new_battle(
            vec![PlayerUnitData {
                id: adept.id.0.clone(),
                base_stats: adept.base_stats,
                equipment: EquipmentLoadout::default(),
                djinn_slots: DjinnSlots::new(),
                mana_contribution: adept.mana_contribution,
                equipment_effects: EquipmentEffects::default(),
            }],
            vec![EnemyUnitData {
                enemy_def: tanky_enemy,
            }],
            game_data.config.clone(),
            game_data.abilities.clone(),
            game_data.djinn.clone(),
        );

        // Run 21 rounds manually
        let mut stalemate = false;
        for _ in 0..25 {
            if battle.round > 20 {
                stalemate = true;
                break;
            }
            // Plan attack
            let unit_ref = TargetRef {
                side: Side::Player,
                index: 0,
            };
            let action = BattleAction::Attack {
                target: TargetRef {
                    side: Side::Enemy,
                    index: 0,
                },
            };
            let _ = plan_action(&mut battle, unit_ref, action);
            let _ = execute_round(&mut battle);

            if check_battle_end(&battle).is_some() {
                break;
            }
        }

        assert!(stalemate, "Stalemate guard should trigger at round > 20");
    }

    #[test]
    fn test_djinn_activation_via_interactive_menu() {
        let game_data = load_sample_data();
        let adept = game_data.units.get(&UnitId("adept".to_string())).unwrap();
        let slime = game_data
            .enemies
            .get(&EnemyId("mercury-slime".to_string()))
            .unwrap();

        let mut djinn_slots = DjinnSlots::new();
        djinn_slots.add(crate::shared::DjinnId("flint".to_string()));

        let mut battle = new_battle(
            vec![PlayerUnitData {
                id: adept.id.0.clone(),
                base_stats: adept.base_stats,
                equipment: EquipmentLoadout::default(),
                djinn_slots,
                mana_contribution: adept.mana_contribution,
                equipment_effects: EquipmentEffects::default(),
            }],
            vec![EnemyUnitData {
                enemy_def: slime.clone(),
            }],
            game_data.config.clone(),
            game_data.abilities.clone(),
            game_data.djinn.clone(),
        );

        // Verify djinn starts in Good state
        assert_eq!(
            battle.player_units[0].djinn_slots.slots[0].state,
            crate::shared::DjinnState::Good,
            "Djinn should start in Good state"
        );

        // Plan djinn activation (simulating option [4])
        let unit_ref = TargetRef {
            side: Side::Player,
            index: 0,
        };
        let result = plan_action(
            &mut battle,
            unit_ref,
            BattleAction::ActivateDjinn { djinn_index: 0 },
        );
        assert!(result.is_ok(), "Djinn activation should succeed");

        // Execute the round
        let events = execute_round(&mut battle);

        // Verify djinn transitioned to Recovery
        assert_eq!(
            battle.player_units[0].djinn_slots.slots[0].state,
            crate::shared::DjinnState::Recovery,
            "Djinn should be in Recovery after activation"
        );

        // Verify DjinnChanged event was emitted
        let has_djinn_event = events.iter().any(|e| matches!(e, BattleEvent::DjinnChanged(_)));
        assert!(has_djinn_event, "Should have DjinnChanged event");

        // Verify the event formats correctly
        let djinn_event = events
            .iter()
            .find(|e| matches!(e, BattleEvent::DjinnChanged(_)))
            .unwrap();
        let text = format_event(djinn_event, &battle);
        assert!(
            text.contains("activates") && text.contains("Flint"),
            "Event text should mention activation and djinn name: {}",
            text
        );
    }

    #[test]
    fn test_djinn_recovery_ticks_after_activation() {
        let game_data = load_sample_data();
        let adept = game_data.units.get(&UnitId("adept".to_string())).unwrap();
        let slime = game_data
            .enemies
            .get(&EnemyId("mercury-slime".to_string()))
            .unwrap();

        // Make enemy very tanky so battle lasts multiple rounds
        let mut tanky = slime.clone();
        tanky.stats.hp = 50000;
        tanky.stats.def = 200;
        tanky.stats.atk = 1;

        let mut djinn_slots = DjinnSlots::new();
        djinn_slots.add(crate::shared::DjinnId("flint".to_string()));

        let mut battle = new_battle(
            vec![PlayerUnitData {
                id: adept.id.0.clone(),
                base_stats: adept.base_stats,
                equipment: EquipmentLoadout::default(),
                djinn_slots,
                mana_contribution: adept.mana_contribution,
                equipment_effects: EquipmentEffects::default(),
            }],
            vec![EnemyUnitData {
                enemy_def: tanky,
            }],
            game_data.config.clone(),
            game_data.abilities.clone(),
            game_data.djinn.clone(),
        );

        // Round 1: activate djinn
        let unit_ref = TargetRef {
            side: Side::Player,
            index: 0,
        };
        plan_action(
            &mut battle,
            unit_ref,
            BattleAction::ActivateDjinn { djinn_index: 0 },
        )
        .unwrap();
        execute_round(&mut battle);

        assert_eq!(
            battle.player_units[0].djinn_slots.slots[0].state,
            crate::shared::DjinnState::Recovery,
            "Djinn should be in Recovery after round 1"
        );

        // Recovery turns = djinn_recovery_start_delay + djinn_recovery_per_turn = 1 + 1 = 2
        // After tick in round 1 end: remaining = 2 - 1 = 1 (no recovery yet)
        // Round 2: attack, then tick: remaining = 1 - 1 = 0, recovers!
        let _ = plan_action(
            &mut battle,
            unit_ref,
            BattleAction::Attack {
                target: TargetRef {
                    side: Side::Enemy,
                    index: 0,
                },
            },
        );
        let events_r2 = execute_round(&mut battle);

        // After round 2, djinn should have recovered to Good
        assert_eq!(
            battle.player_units[0].djinn_slots.slots[0].state,
            crate::shared::DjinnState::Good,
            "Djinn should recover to Good after sufficient ticks"
        );

        // Should have a recovery DjinnChanged event in round 2
        let has_recovery_event = events_r2.iter().any(|e| match e {
            BattleEvent::DjinnChanged(dc) => dc.new_state == crate::shared::DjinnState::Good,
            _ => false,
        });
        assert!(
            has_recovery_event,
            "Should have DjinnChanged event showing recovery to Good"
        );
    }

    #[test]
    fn test_auto_mode_activates_djinn_on_round_4() {
        let game_data = load_sample_data();
        let adept = game_data.units.get(&UnitId("adept".to_string())).unwrap();
        let slime = game_data
            .enemies
            .get(&EnemyId("mercury-slime".to_string()))
            .unwrap();

        // Make enemy tanky so battle lasts 4+ rounds
        let mut tanky = slime.clone();
        tanky.stats.hp = 50000;
        tanky.stats.def = 200;
        tanky.stats.atk = 1;

        let adept_ability = find_first_psynergy_ability(adept, &game_data);

        let mut djinn_slots = DjinnSlots::new();
        djinn_slots.add(crate::shared::DjinnId("flint".to_string()));

        let mut battle = new_battle(
            vec![PlayerUnitData {
                id: adept.id.0.clone(),
                base_stats: adept.base_stats,
                equipment: EquipmentLoadout::default(),
                djinn_slots,
                mana_contribution: adept.mana_contribution,
                equipment_effects: EquipmentEffects::default(),
            }],
            vec![EnemyUnitData {
                enemy_def: tanky,
            }],
            game_data.config.clone(),
            game_data.abilities.clone(),
            game_data.djinn.clone(),
        );

        let auto_abilities: Vec<Option<AbilityId>> = vec![adept_ability];

        // Rounds 1-3: should attack normally, djinn stays Good
        for round in 1..=3 {
            plan_auto_action_for_player(&mut battle, 0, round, &auto_abilities);
            execute_round(&mut battle);
            assert_eq!(
                battle.player_units[0].djinn_slots.slots[0].state,
                crate::shared::DjinnState::Good,
                "Djinn should still be Good before round 4 (round {})",
                round
            );
        }

        // Round 4: auto mode should activate djinn
        plan_auto_action_for_player(&mut battle, 0, 4, &auto_abilities);
        let events = execute_round(&mut battle);

        assert_eq!(
            battle.player_units[0].djinn_slots.slots[0].state,
            crate::shared::DjinnState::Recovery,
            "Djinn should be in Recovery after auto-activation on round 4"
        );

        let has_djinn_activation = events.iter().any(|e| match e {
            BattleEvent::DjinnChanged(dc) => {
                dc.old_state == crate::shared::DjinnState::Good
                    && dc.new_state == crate::shared::DjinnState::Recovery
            }
            _ => false,
        });
        assert!(
            has_djinn_activation,
            "Round 4 events should include djinn activation"
        );
    }
}
