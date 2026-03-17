#![allow(dead_code, unused_imports)]
//! CLI Battle Runner — first player-reachable surface.
//!
//! Makes `cargo run` execute an actual battle using the battle_engine,
//! printing each round's events to the terminal.

use crate::domains::battle_engine::{
    check_battle_end, execute_round, new_battle, plan_action, plan_enemy_actions, Battle,
    BattleEvent, BattleResult, EnemyUnitData, PlayerUnitData,
};
use crate::domains::data_loader::GameData;
use crate::domains::djinn::DjinnSlots;
use crate::domains::equipment::{self, EquipmentEffects, EquipmentLoadout};
use crate::shared::{
    AbilityId, BattleAction, DjinnId, EncounterId, EnemyId, EquipmentId, EquipmentSlot, Side,
    TargetRef, UnitDef, UnitId,
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

/// Run a demo battle using loaded game data.
///
/// - 2 player units: Adept + Blaze
/// - Enemies loaded from encounter "house-02" (Earth Scout + Earthbound Wolf)
/// - Auto-play loop: each player unit attacks first alive enemy
/// - Every 3rd round, first player uses an ability if mana allows
/// - Enemies attack back: each alive enemy attacks first alive player
/// - Both sides can win or lose
/// - Stalemate guard at round 20
pub fn run_demo_battle(game_data: &GameData) -> BattleResult {
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

    // Load encounter "house-02", fall back to manual enemy list
    let encounter = game_data
        .encounters
        .get(&EncounterId("house-02".to_string()));

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

    // Build ability_defs map for the battle
    let ability_defs = game_data.abilities.clone();

    let mut battle = new_battle(
        player_data,
        enemy_data,
        game_data.config.clone(),
        ability_defs,
    );

    println!("\n=== BATTLE START ===");
    println!("Party: {} + {}", adept_gear_info, blaze_gear_info);
    print!("Enemies:");
    for eu in &battle.enemies {
        print!(" {} (HP:{})", eu.unit.id, eu.unit.stats.hp);
    }
    println!("\n");

    // Collect ability names for printing (indexed by player index)
    let unit_ability_ids: Vec<Option<AbilityId>> = vec![adept_ability, blaze_ability];

    loop {
        // Stalemate guard
        if battle.round > 20 {
            println!("\n*** STALEMATE after 20 rounds ***");
            return BattleResult::Defeat;
        }

        let current_round = battle.round;

        // Planning phase: each alive player unit attacks first alive enemy
        // Every 3rd round, first alive player uses an ability instead
        for pi in 0..battle.player_units.len() {
            if !battle.player_units[pi].unit.is_alive {
                continue;
            }
            // Find first alive enemy
            let target_idx = battle
                .enemies
                .iter()
                .position(|e| e.unit.is_alive);
            let target_idx = match target_idx {
                Some(i) => i,
                None => break, // no enemies left
            };

            let unit_ref = TargetRef {
                side: Side::Player,
                index: pi as u8,
            };
            let enemy_target = TargetRef {
                side: Side::Enemy,
                index: target_idx as u8,
            };

            // Every 3rd round: try to use an ability for this unit
            let mut used_ability = false;
            if current_round % 3 == 0 {
                if let Some(ref ability_id) = unit_ability_ids.get(pi).and_then(|a| a.as_ref()) {
                    // Check if the ability exists and mana allows
                    let can_use = battle
                        .ability_defs
                        .get(ability_id)
                        .map(|adef| battle.mana_pool.current_mana >= adef.mana_cost)
                        .unwrap_or(false);

                    if can_use {
                        let ability_name = battle
                            .ability_defs
                            .get(ability_id)
                            .map(|a| a.name.clone())
                            .unwrap_or_else(|| ability_id.0.clone());
                        let enemy_name = battle
                            .enemies
                            .get(target_idx)
                            .map(|e| e.unit.id.clone())
                            .unwrap_or_else(|| format!("Enemy[{}]", target_idx));
                        println!(
                            "  >> {} uses {} on {}!",
                            battle.player_units[pi].unit.id, ability_name, enemy_name
                        );
                        let action = BattleAction::UseAbility {
                            ability_id: (*ability_id).clone(),
                            targets: vec![enemy_target],
                        };
                        if plan_action(&mut battle, unit_ref, action).is_ok() {
                            used_ability = true;
                        }
                    }
                }
            }

            if !used_ability {
                let action = BattleAction::Attack {
                    target: enemy_target,
                };
                let _ = plan_action(&mut battle, unit_ref, action);
            }
        }

        // Enemy planning: each alive enemy attacks first alive player
        plan_enemy_actions(&mut battle);

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
            format!(
                "{}'s barrier absorbs the hit!",
                format_target(unit, battle)
            )
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
            format!(
                "Djinn {} on {}: {:?} -> {:?}",
                dc.djinn_id.0,
                format_target(&dc.unit, battle),
                dc.old_state,
                dc.new_state
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
        let result = run_demo_battle(&game_data);
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
        assert!(damage_text.contains("deals"), "damage event: {}", damage_text);
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
}
