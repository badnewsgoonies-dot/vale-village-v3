//! Graduation tests — player-facing invariants verified during hardening.
//!
//! These are integration tests that exercise the actual game behaviour
//! end-to-end. Test names describe the player experience, not the
//! implementation detail.

use std::collections::HashMap;
use std::path::PathBuf;

use vale_village::data::default_combat_config;
use vale_village::domains::battle_engine::{
    self, BattleResult, EnemyUnitData, PlanError, PlayerUnitData,
};
use vale_village::domains::combat;
use vale_village::domains::data_loader;
use vale_village::domains::djinn::DjinnSlots;
use vale_village::domains::equipment::{self, EquipmentLoadout};
use vale_village::domains::status;
use vale_village::shared::{
    AbilityCategory, AbilityDef, AbilityId, BattleAction, CombatConfig, DamageType, Element,
    EncounterId, EnemyId, Side, Stats, StatusEffectType, TargetMode, TargetRef,
};

// ── Helpers ────────────────────────────────────────────────────────

fn full_data_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("data")
        .join("full")
}

fn config() -> CombatConfig {
    default_combat_config()
}

fn player(id: &str, stats: Stats, mana: u8) -> PlayerUnitData {
    PlayerUnitData {
        id: id.to_string(),
        base_stats: stats,
        equipment: EquipmentLoadout::default(),
        djinn_slots: DjinnSlots::new(),
        mana_contribution: mana,
        equipment_effects: equipment::EquipmentEffects::default(),
    }
}

fn enemy(id: &str, stats: Stats) -> EnemyUnitData {
    EnemyUnitData {
        enemy_def: vale_village::shared::EnemyDef {
            id: EnemyId(id.to_string()),
            name: id.to_string(),
            element: Element::Venus,
            level: 1,
            stats,
            xp: 10,
            gold: 5,
            abilities: vec![],
        },
    }
}

fn basic_ability(id: &str, cost: u8, power: u16) -> (AbilityId, AbilityDef) {
    let aid = AbilityId(id.to_string());
    let def = AbilityDef {
        id: aid.clone(),
        name: id.to_string(),
        category: AbilityCategory::Psynergy,
        damage_type: Some(DamageType::Psynergy),
        element: Some(Element::Venus),
        mana_cost: cost,
        base_power: power,
        targets: TargetMode::SingleEnemy,
        unlock_level: 1,
        hit_count: 1,
        status_effect: None,
        buff_effect: None,
        debuff_effect: None,
        shield_charges: None,
        shield_duration: None,
        heal_over_time: None,
        grant_immunity: None,
        cleanse: None,
        ignore_defense_percent: None,
        splash_damage_percent: None,
        chain_damage: false,
        revive: false,
        revive_hp_percent: None,
    };
    (aid, def)
}

// ════════════════════════════════════════════════════════════════════
// 1. Full data loads without error
// ════════════════════════════════════════════════════════════════════

#[test]
fn test_full_data_loads_without_error() {
    let data = data_loader::load_game_data(&full_data_dir())
        .expect("full data directory should load without errors");

    assert_eq!(data.abilities.len(), 346, "expected 346 abilities");
    assert_eq!(data.enemies.len(), 137, "expected 137 enemies");
    assert_eq!(data.equipment.len(), 109, "expected 109 equipment");
    assert_eq!(data.djinn.len(), 23, "expected 23 djinn");
    assert_eq!(data.encounters.len(), 55, "expected 55 encounters");
    assert_eq!(data.units.len(), 11, "expected 11 units");
}

// ════════════════════════════════════════════════════════════════════
// 2. Battle completes to victory or defeat
// ════════════════════════════════════════════════════════════════════

#[test]
fn test_battle_completes_to_victory_or_defeat() {
    let p1 = player(
        "hero",
        Stats {
            hp: 120,
            atk: 30,
            def: 20,
            mag: 25,
            spd: 15,
        },
        5,
    );
    let p2 = player(
        "mage",
        Stats {
            hp: 100,
            atk: 20,
            def: 15,
            mag: 35,
            spd: 13,
        },
        5,
    );
    let e1 = enemy(
        "goblin-a",
        Stats {
            hp: 60,
            atk: 15,
            def: 10,
            mag: 5,
            spd: 8,
        },
    );
    let e2 = enemy(
        "goblin-b",
        Stats {
            hp: 50,
            atk: 12,
            def: 8,
            mag: 5,
            spd: 7,
        },
    );

    let (aid, adef) = basic_ability("quake", 3, 40);
    let mut abilities = HashMap::new();
    abilities.insert(aid, adef);

    let mut battle = battle_engine::new_battle(
        vec![p1, p2],
        vec![e1, e2],
        config(),
        abilities,
        HashMap::new(),
    );

    let mut result: Option<BattleResult> = None;
    for _ in 0..30 {
        // Plan player attacks: each alive player attacks first alive enemy
        for pi in 0..battle.player_units.len() {
            if !battle.player_units[pi].unit.is_alive {
                continue;
            }
            let target_idx = battle.enemies.iter().position(|e| e.unit.is_alive);
            let target_idx = match target_idx {
                Some(i) => i,
                None => break,
            };
            let _ = battle_engine::plan_action(
                &mut battle,
                TargetRef {
                    side: Side::Player,
                    index: pi as u8,
                },
                BattleAction::Attack {
                    target: TargetRef {
                        side: Side::Enemy,
                        index: target_idx as u8,
                    },
                },
            );
        }

        // Plan enemy attacks
        battle_engine::plan_enemy_actions(&mut battle);

        // Execute
        battle_engine::execute_round(&mut battle);

        // Check end
        if let Some(r) = battle_engine::check_battle_end(&battle) {
            result = Some(r);
            break;
        }
    }

    assert!(result.is_some(), "battle must end within 30 rounds");
    match result.unwrap() {
        BattleResult::Victory { .. } => {} // ok
        BattleResult::Defeat => {}         // ok
    }
}

// ════════════════════════════════════════════════════════════════════
// 3. Physical damage formula is deterministic
// ════════════════════════════════════════════════════════════════════

#[test]
fn test_physical_damage_formula_is_deterministic() {
    let cfg = config();
    let attacker = Stats {
        hp: 100,
        atk: 30,
        def: 10,
        mag: 5,
        spd: 10,
    };
    let defender = Stats {
        hp: 100,
        atk: 10,
        def: 20,
        mag: 5,
        spd: 10,
    };

    let dmg1 = combat::calculate_damage(50, DamageType::Physical, &attacker, &defender, &cfg);
    let dmg2 = combat::calculate_damage(50, DamageType::Physical, &attacker, &defender, &cfg);

    assert_eq!(dmg1, dmg2, "same inputs must produce identical damage");
}

// ════════════════════════════════════════════════════════════════════
// 4. Auto-attack generates mana, ability does not
// ════════════════════════════════════════════════════════════════════

#[test]
fn test_auto_attack_generates_mana_ability_does_not() {
    let cfg = config();

    // Multi-hit auto-attack: 3 hits
    let auto_hits = combat::resolve_multi_hit(3, 10, 200, 0, true, &cfg);
    let total_mana: u8 = auto_hits.iter().map(|h| h.mana_generated).sum();
    assert_eq!(
        total_mana, 3,
        "3-hit auto-attack should generate 3 mana (1 per hit)"
    );

    // Ability: 3 hits, same inputs
    let ability_hits = combat::resolve_multi_hit(3, 10, 200, 0, false, &cfg);
    let ability_mana: u8 = ability_hits.iter().map(|h| h.mana_generated).sum();
    assert_eq!(ability_mana, 0, "ability hits must not generate mana");
}

// ════════════════════════════════════════════════════════════════════
// 5. Crit triggers on the tenth hit
// ════════════════════════════════════════════════════════════════════

#[test]
fn test_crit_triggers_on_tenth_hit() {
    let cfg = config();
    // 10 auto-attack hits from counter=0, high HP so nothing dies
    let hits = combat::resolve_multi_hit(10, 10, 5000, 0, true, &cfg);

    assert_eq!(hits.len(), 10);

    let crit_count = hits.iter().filter(|h| h.is_crit).count();
    assert_eq!(crit_count, 1, "exactly 1 crit in 10 hits");
    assert!(hits[9].is_crit, "the 10th hit must be the crit");
    assert_eq!(
        hits[9].crit_counter_after, 0,
        "crit counter resets after crit"
    );
}

// ════════════════════════════════════════════════════════════════════
// 6. Dead unit cannot act
// ════════════════════════════════════════════════════════════════════

#[test]
fn test_dead_unit_cannot_act() {
    let p = player(
        "hero",
        Stats {
            hp: 100,
            atk: 30,
            def: 20,
            mag: 25,
            spd: 15,
        },
        5,
    );
    let e = enemy(
        "goblin",
        Stats {
            hp: 80,
            atk: 20,
            def: 15,
            mag: 10,
            spd: 10,
        },
    );

    let (aid, adef) = basic_ability("quake", 3, 40);
    let mut abilities = HashMap::new();
    abilities.insert(aid, adef);

    let mut battle =
        battle_engine::new_battle(vec![p], vec![e], config(), abilities, HashMap::new());

    // Kill the player unit
    battle.player_units[0].unit.current_hp = 0;
    battle.player_units[0].unit.is_alive = false;

    let result = battle_engine::plan_action(
        &mut battle,
        TargetRef {
            side: Side::Player,
            index: 0,
        },
        BattleAction::Attack {
            target: TargetRef {
                side: Side::Enemy,
                index: 0,
            },
        },
    );

    assert_eq!(
        result,
        Err(PlanError::UnitCannotAct),
        "dead unit must not be allowed to plan an action"
    );
}

// ════════════════════════════════════════════════════════════════════
// 7. Burn deals percent of max HP
// ════════════════════════════════════════════════════════════════════

#[test]
fn test_burn_deals_percent_of_max_hp() {
    let max_hp: u16 = 200;
    let burn_percent: f32 = 0.10; // 10%

    let mut statuses = vec![status::ActiveStatus {
        effect_type: StatusEffectType::Burn,
        remaining_turns: 3,
        burn_percent: Some(burn_percent),
        poison_percent: None,
        freeze_threshold: None,
        freeze_damage_taken: 0,
    }];

    let result = status::tick_statuses(max_hp, max_hp, &mut statuses);

    let expected = (max_hp as f32 * burn_percent) as u16; // 20
    assert_eq!(
        result.damage, expected,
        "burn damage = max_hp * burn_percent"
    );
}

// ════════════════════════════════════════════════════════════════════
// 8. Barrier blocks damage instance
// ════════════════════════════════════════════════════════════════════

#[test]
fn test_barrier_blocks_damage_instance() {
    let p = player(
        "hero",
        Stats {
            hp: 100,
            atk: 30,
            def: 20,
            mag: 25,
            spd: 15,
        },
        5,
    );
    let e = enemy(
        "goblin",
        Stats {
            hp: 80,
            atk: 20,
            def: 15,
            mag: 10,
            spd: 10,
        },
    );

    let mut abilities = HashMap::new();
    let (aid, adef) = basic_ability("quake", 3, 40);
    abilities.insert(aid, adef);

    let mut battle =
        battle_engine::new_battle(vec![p], vec![e], config(), abilities, HashMap::new());

    // Give player unit a barrier with 1 charge
    status::apply_barrier(&mut battle.player_units[0].status_state, 1, 5);

    let hp_before = battle.player_units[0].unit.current_hp;

    // Enemy attacks the player
    let _ = battle_engine::plan_action(
        &mut battle,
        TargetRef {
            side: Side::Enemy,
            index: 0,
        },
        BattleAction::Attack {
            target: TargetRef {
                side: Side::Player,
                index: 0,
            },
        },
    );

    battle_engine::execute_round(&mut battle);

    let hp_after = battle.player_units[0].unit.current_hp;
    assert_eq!(
        hp_after, hp_before,
        "HP must be unchanged when barrier absorbs the hit"
    );

    // Barrier charge consumed
    let charges: u8 = battle.player_units[0]
        .status_state
        .barriers
        .iter()
        .map(|b| b.charges)
        .sum();
    assert_eq!(charges, 0, "barrier charge must be consumed");
}

// ════════════════════════════════════════════════════════════════════
// 9. Enemies attack player units
// ════════════════════════════════════════════════════════════════════

#[test]
fn test_enemies_attack_player_units() {
    let p = player(
        "hero",
        Stats {
            hp: 200,
            atk: 10,
            def: 10,
            mag: 10,
            spd: 5,
        },
        5,
    );
    let e = enemy(
        "strong-goblin",
        Stats {
            hp: 200,
            atk: 30,
            def: 10,
            mag: 5,
            spd: 20,
        },
    );

    let abilities = HashMap::new();
    let mut battle =
        battle_engine::new_battle(vec![p], vec![e], config(), abilities, HashMap::new());

    let hp_before = battle.player_units[0].unit.current_hp;

    // Plan enemy actions (AI: attack first alive player)
    battle_engine::plan_enemy_actions(&mut battle);

    // Execute the round
    battle_engine::execute_round(&mut battle);

    let hp_after = battle.player_units[0].unit.current_hp;
    assert!(
        hp_after < hp_before,
        "player unit must take damage from enemy attack (before={}, after={})",
        hp_before,
        hp_after
    );
}

// ════════════════════════════════════════════════════════════════════
// 10. Encounter data loads house-02
// ════════════════════════════════════════════════════════════════════

#[test]
fn test_encounter_data_loads_house_02() {
    let data = data_loader::load_game_data(&full_data_dir()).expect("full data should load");

    let enc_id = EncounterId("house-02".to_string());
    let encounter = data
        .encounters
        .get(&enc_id)
        .expect("encounter house-02 must exist");

    assert_eq!(encounter.name, "House 2: The Bronze Trial");
    assert_eq!(encounter.enemies.len(), 2, "house-02 has 2 enemy groups");

    let enemy_ids: Vec<&str> = encounter
        .enemies
        .iter()
        .map(|e| e.enemy_id.0.as_str())
        .collect();
    assert!(
        enemy_ids.contains(&"earth-scout"),
        "house-02 must contain earth-scout"
    );
    assert!(
        enemy_ids.contains(&"venus-wolf"),
        "house-02 must contain venus-wolf"
    );
}
