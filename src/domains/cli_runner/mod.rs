#![allow(dead_code)]
//! CLI Battle Runner — first player-reachable surface.
//!
//! Makes `cargo run` execute an actual battle using the battle_engine,
//! printing each round's events to the terminal.

use crate::domains::battle_engine::{
    check_battle_end, execute_round, new_battle, plan_action, Battle, BattleEvent, BattleResult,
    EnemyUnitData, PlayerUnitData,
};
use crate::domains::data_loader::GameData;
use crate::domains::djinn::DjinnSlots;
use crate::domains::equipment::{EquipmentEffects, EquipmentLoadout};
use crate::shared::{
    BattleAction, EnemyId, Side, TargetRef, UnitId,
};

// ── Public API ──────────────────────────────────────────────────────

/// Run a demo battle using loaded game data.
///
/// - 2 player units: Adept + Blaze
/// - 2 enemies: Mercury Slime + Earth Scout
/// - Auto-play loop: each player unit attacks first alive enemy
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

    let player_data: Vec<PlayerUnitData> = vec![
        PlayerUnitData {
            id: adept.id.0.clone(),
            base_stats: adept.base_stats,
            equipment: EquipmentLoadout::default(),
            djinn_slots: DjinnSlots::new(),
            mana_contribution: adept.mana_contribution,
            equipment_effects: EquipmentEffects::default(),
        },
        PlayerUnitData {
            id: blaze.id.0.clone(),
            base_stats: blaze.base_stats,
            equipment: EquipmentLoadout::default(),
            djinn_slots: DjinnSlots::new(),
            mana_contribution: blaze.mana_contribution,
            equipment_effects: EquipmentEffects::default(),
        },
    ];

    // Look up enemies
    let slime = game_data
        .enemies
        .get(&EnemyId("mercury-slime".to_string()))
        .expect("sample data must contain enemy 'mercury-slime'");
    let scout = game_data
        .enemies
        .get(&EnemyId("earth-scout".to_string()))
        .expect("sample data must contain enemy 'earth-scout'");

    let enemy_data: Vec<EnemyUnitData> = vec![
        EnemyUnitData {
            enemy_def: slime.clone(),
        },
        EnemyUnitData {
            enemy_def: scout.clone(),
        },
    ];

    // Build ability_defs map for the battle
    let ability_defs = game_data.abilities.clone();

    let mut battle = new_battle(
        player_data,
        enemy_data,
        game_data.config.clone(),
        ability_defs,
    );

    println!("\n=== BATTLE START ===");
    println!(
        "Party: {} (HP:{}) + {} (HP:{})",
        adept.name,
        adept.base_stats.hp,
        blaze.name,
        blaze.base_stats.hp
    );
    println!(
        "Enemies: {} (HP:{}) + {} (HP:{})",
        slime.name, slime.stats.hp, scout.name, scout.stats.hp
    );
    println!();

    loop {
        // Stalemate guard
        if battle.round > 20 {
            println!("\n*** STALEMATE after 20 rounds ***");
            return BattleResult::Defeat;
        }

        // Planning phase: each alive player unit attacks first alive enemy
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
            let action = BattleAction::Attack {
                target: TargetRef {
                    side: Side::Enemy,
                    index: target_idx as u8,
                },
            };
            let _ = plan_action(&mut battle, unit_ref, action);
        }

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

        // Make a custom enemy with absurd HP
        let mut tanky_enemy = game_data
            .enemies
            .get(&EnemyId("mercury-slime".to_string()))
            .unwrap()
            .clone();
        tanky_enemy.stats.hp = 60000;
        tanky_enemy.stats.def = 200;

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
