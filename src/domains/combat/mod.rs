#![allow(dead_code)]
//! Combat Core Domain — Systems S01-S06
//!
//! Implements the 6 foundational battle systems:
//! S01 Damage Calculation, S02 Targeting, S03 Multi-Hit,
//! S04 Battle State Machine, S05 Team Mana Pool, S06 Deterministic Crit.

use crate::shared::{
    BattleAction, BattlePhase, CombatConfig, DamageType, Side, Stats, TargetMode, TargetRef,
};

// ── S03 — Hit Result ────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq)]
pub struct HitResult {
    pub damage: u16,
    pub is_crit: bool,
    pub target_killed: bool,
    pub mana_generated: u8,
    pub crit_counter_after: u8,
}

// ── S04 — Battle Unit ───────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct BattleUnit {
    pub id: String,
    pub stats: Stats,
    pub current_hp: u16,
    pub is_alive: bool,
    pub crit_counter: u8,
    pub equipment_speed_bonus: i16,
}

// ── S05 — Mana Pool ────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct ManaPool {
    pub max_mana: u8,
    pub current_mana: u8,
    pub projected_mana: u8,
}

impl ManaPool {
    pub fn new(max_mana: u8) -> Self {
        ManaPool {
            max_mana,
            current_mana: max_mana,
            projected_mana: max_mana,
        }
    }

    /// Reset current mana to max at the start of each round.
    pub fn reset_mana(&mut self) {
        self.current_mana = self.max_mana;
        self.projected_mana = self.max_mana;
    }

    /// Spend mana if affordable. Returns true on success.
    pub fn spend_mana(&mut self, cost: u8) -> bool {
        if self.current_mana >= cost {
            self.current_mana -= cost;
            true
        } else {
            false
        }
    }

    /// Generate mana from auto-attack hits mid-execution.
    pub fn generate_mana(&mut self, amount: u8) {
        self.current_mana = self.current_mana.saturating_add(amount).min(self.max_mana);
    }
}

// ── S04 — Battle State Machine ──────────────────────────────────────

/// Standalone combat-domain battle state used by unit tests in this module.
/// The integration layer (battle_engine) uses its own `Battle` struct which
/// wraps BattleUnit alongside status, djinn, and equipment state.
/// This struct remains as the canonical reference for S04 execution-order logic.
#[derive(Debug, Clone)]
pub struct BattleState {
    pub phase: BattlePhase,
    pub round: u32,
    pub player_units: Vec<BattleUnit>,
    pub enemies: Vec<BattleUnit>,
    pub planned_actions: Vec<(TargetRef, BattleAction)>,
    pub mana_pool: ManaPool,
    pub execution_order: Vec<TargetRef>,
}

impl BattleState {
    /// Compute execution order: sort all living actors by effective SPD descending.
    /// Tiebreaker: base SPD descending (equipment_speed_bonus is 0 for base SPD tie).
    pub fn compute_execution_order(&mut self) {
        let mut actors: Vec<(TargetRef, i32, u16)> = Vec::new();

        for (i, unit) in self.player_units.iter().enumerate() {
            if unit.is_alive {
                let effective_spd = unit.stats.spd.get() as i32 + unit.equipment_speed_bonus as i32;
                actors.push((
                    TargetRef {
                        side: Side::Player,
                        index: i as u8,
                    },
                    effective_spd,
                    unit.stats.spd.get(),
                ));
            }
        }

        for (i, unit) in self.enemies.iter().enumerate() {
            if unit.is_alive {
                let effective_spd = unit.stats.spd.get() as i32 + unit.equipment_speed_bonus as i32;
                actors.push((
                    TargetRef {
                        side: Side::Enemy,
                        index: i as u8,
                    },
                    effective_spd,
                    unit.stats.spd.get(),
                ));
            }
        }

        // Sort descending by effective SPD, then by base SPD as tiebreaker
        actors.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| b.2.cmp(&a.2)));

        self.execution_order = actors.into_iter().map(|(tr, _, _)| tr).collect();
    }
}

// ── S01 — Damage Calculation ────────────────────────────────────────

/// Calculate damage given base power, type, attacker/defender stats, and config.
/// Physical: base_power + atk - (def * physical_def_multiplier), floor 1
/// Psynergy: base_power + mag - (def * psynergy_def_multiplier), floor 1
pub fn calculate_damage(
    base_power: u16,
    damage_type: DamageType,
    attacker_stats: &Stats,
    defender_stats: &Stats,
    config: &CombatConfig,
) -> u16 {
    let raw = match damage_type {
        DamageType::Physical => {
            let offense = base_power as f32 + attacker_stats.atk.get() as f32;
            let defense = defender_stats.def.get() as f32 * config.physical_def_multiplier;
            offense - defense
        }
        DamageType::Psynergy => {
            let offense = base_power as f32 + attacker_stats.mag.get() as f32;
            let defense = defender_stats.def.get() as f32 * config.psynergy_def_multiplier;
            offense - defense
        }
    };

    if raw < 1.0 {
        1
    } else {
        raw as u16
    }
}

/// Calculate healing given base power and healer stats.
/// Healing: base_power + mag, floor 1
pub fn calculate_healing(base_power: u16, healer_stats: &Stats) -> u16 {
    (base_power + healer_stats.mag.get()).max(1)
}

// ── S02 — Targeting ─────────────────────────────────────────────────

/// Resolve targets based on targeting mode.
pub fn resolve_targets(
    mode: TargetMode,
    source: TargetRef,
    chosen_target: Option<TargetRef>,
    party_size: u8,
    enemy_count: u8,
) -> Vec<TargetRef> {
    match mode {
        TargetMode::SingleEnemy | TargetMode::SingleAlly => {
            // Return the chosen target
            match chosen_target {
                Some(t) => vec![t],
                None => vec![],
            }
        }
        TargetMode::AllEnemies => {
            let side = match source.side {
                Side::Player => Side::Enemy,
                Side::Enemy => Side::Player,
            };
            let count = match side {
                Side::Enemy => enemy_count,
                Side::Player => party_size,
            };
            (0..count).map(|i| TargetRef { side, index: i }).collect()
        }
        TargetMode::AllAllies => {
            let count = match source.side {
                Side::Player => party_size,
                Side::Enemy => enemy_count,
            };
            (0..count)
                .map(|i| TargetRef {
                    side: source.side,
                    index: i,
                })
                .collect()
        }
        TargetMode::SelfOnly => {
            vec![source]
        }
    }
}

// ── S03 + S06 — Multi-Hit with Deterministic Crit ──────────────────

/// Resolve a sequence of hits against a single target.
/// Auto-attacks: advance crit counter per hit, generate mana per hit.
/// Abilities: no crit advance, no mana.
/// At crit_threshold: that hit crits (damage * crit_multiplier), counter resets.
/// Stops early if target dies.
pub fn resolve_multi_hit(
    hit_count: u8,
    base_damage_per_hit: u16,
    target_hp: u16,
    crit_counter: u8,
    is_auto_attack: bool,
    config: &CombatConfig,
) -> Vec<HitResult> {
    let mut results = Vec::new();
    let mut remaining_hp = target_hp;
    let mut counter = crit_counter;

    for _hit in 0..hit_count {
        if remaining_hp == 0 {
            break;
        }

        let mut is_crit = false;
        let mut mana_generated: u8 = 0;

        if is_auto_attack {
            counter += 1;
            mana_generated = config.mana_gain_per_hit.get();

            if counter >= config.crit_threshold {
                is_crit = true;
                counter = 0;
            }
        }

        let damage = if is_crit {
            (base_damage_per_hit as f32 * config.crit_multiplier) as u16
        } else {
            base_damage_per_hit
        };

        let actual_damage = damage.min(remaining_hp);
        remaining_hp = remaining_hp.saturating_sub(actual_damage);
        let target_killed = remaining_hp == 0;

        results.push(HitResult {
            damage: actual_damage,
            is_crit,
            target_killed,
            mana_generated,
            crit_counter_after: counter,
        });
    }

    results
}

// ── Tests ───────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::default_combat_config;
    use crate::shared::bounded_types::{BaseStat, Hp};

    fn test_config() -> CombatConfig {
        default_combat_config()
    }

    fn test_stats(hp: u16, atk: u16, def: u16, mag: u16, spd: u16) -> Stats {
        Stats {
            hp: Hp::new(hp).unwrap_or_else(|_| Hp::new_unchecked(hp.clamp(1, 9999))),
            atk: BaseStat::new(atk).unwrap_or_else(|_| BaseStat::new_unchecked(atk.clamp(0, 9999))),
            def: BaseStat::new(def).unwrap_or_else(|_| BaseStat::new_unchecked(def.clamp(0, 9999))),
            mag: BaseStat::new(mag).unwrap_or_else(|_| BaseStat::new_unchecked(mag.clamp(0, 9999))),
            spd: BaseStat::new(spd).unwrap_or_else(|_| BaseStat::new_unchecked(spd.clamp(0, 9999))),
        }
    }

    // ── S01: Physical damage — normal case ──────────────────────────

    #[test]
    fn physical_damage_normal() {
        let cfg = test_config();
        let attacker = test_stats(100, 30, 10, 5, 10);
        let defender = test_stats(100, 10, 20, 5, 10);
        // base_power(50) + atk(30) - def(20)*0.5 = 80 - 10 = 70
        let dmg = calculate_damage(50, DamageType::Physical, &attacker, &defender, &cfg);
        assert_eq!(dmg, 70);
    }

    // ── S01: Physical damage — high defense ─────────────────────────

    #[test]
    fn physical_damage_high_defense() {
        let cfg = test_config();
        let attacker = test_stats(100, 20, 10, 5, 10);
        let defender = test_stats(100, 10, 80, 5, 10);
        // base_power(10) + atk(20) - def(80)*0.5 = 30 - 40 = -10 -> floor 1
        let dmg = calculate_damage(10, DamageType::Physical, &attacker, &defender, &cfg);
        assert_eq!(dmg, 1);
    }

    // ── S01: Physical damage — floor at 1 ───────────────────────────

    #[test]
    fn physical_damage_floor_at_one() {
        let cfg = test_config();
        let attacker = test_stats(100, 1, 10, 5, 10);
        let defender = test_stats(100, 10, 200, 5, 10);
        // base_power(0) + atk(1) - def(200)*0.5 = 1 - 100 = -99 -> floor 1
        let dmg = calculate_damage(0, DamageType::Physical, &attacker, &defender, &cfg);
        assert_eq!(dmg, 1);
    }

    // ── S01: Psynergy damage — normal case ──────────────────────────

    #[test]
    fn psynergy_damage_normal() {
        let cfg = test_config();
        let attacker = test_stats(100, 10, 10, 40, 10);
        let defender = test_stats(100, 10, 30, 5, 10);
        // base_power(60) + mag(40) - def(30)*0.3 = 100 - 9 = 91
        let dmg = calculate_damage(60, DamageType::Psynergy, &attacker, &defender, &cfg);
        assert_eq!(dmg, 91);
    }

    // ── S01: Psynergy damage — floor at 1 ───────────────────────────

    #[test]
    fn psynergy_damage_floor_at_one() {
        let cfg = test_config();
        let attacker = test_stats(100, 10, 10, 1, 10);
        let defender = test_stats(100, 10, 200, 5, 10);
        // base_power(0) + mag(1) - def(200)*0.3 = 1 - 60 = -59 -> floor 1
        let dmg = calculate_damage(0, DamageType::Psynergy, &attacker, &defender, &cfg);
        assert_eq!(dmg, 1);
    }

    #[test]
    fn test_healing_uses_mag() {
        let healer = test_stats(100, 10, 10, 8, 10);
        let healing = calculate_healing(10, &healer);
        assert_eq!(healing, 18);
    }

    // ── S02: Targeting — SingleEnemy ────────────────────────────────

    #[test]
    fn targeting_single_enemy() {
        let source = TargetRef {
            side: Side::Player,
            index: 0,
        };
        let chosen = TargetRef {
            side: Side::Enemy,
            index: 2,
        };
        let targets = resolve_targets(TargetMode::SingleEnemy, source, Some(chosen), 4, 3);
        assert_eq!(targets, vec![chosen]);
    }

    // ── S02: Targeting — SingleAlly ─────────────────────────────────

    #[test]
    fn targeting_single_ally() {
        let source = TargetRef {
            side: Side::Player,
            index: 0,
        };
        let chosen = TargetRef {
            side: Side::Player,
            index: 1,
        };
        let targets = resolve_targets(TargetMode::SingleAlly, source, Some(chosen), 4, 3);
        assert_eq!(targets, vec![chosen]);
    }

    // ── S02: Targeting — AllEnemies ─────────────────────────────────

    #[test]
    fn targeting_all_enemies() {
        let source = TargetRef {
            side: Side::Player,
            index: 0,
        };
        let targets = resolve_targets(TargetMode::AllEnemies, source, None, 4, 3);
        assert_eq!(targets.len(), 3);
        assert_eq!(
            targets[0],
            TargetRef {
                side: Side::Enemy,
                index: 0
            }
        );
        assert_eq!(
            targets[1],
            TargetRef {
                side: Side::Enemy,
                index: 1
            }
        );
        assert_eq!(
            targets[2],
            TargetRef {
                side: Side::Enemy,
                index: 2
            }
        );
    }

    // ── S02: Targeting — AllAllies ──────────────────────────────────

    #[test]
    fn targeting_all_allies() {
        let source = TargetRef {
            side: Side::Player,
            index: 0,
        };
        let targets = resolve_targets(TargetMode::AllAllies, source, None, 4, 3);
        assert_eq!(targets.len(), 4);
        for i in 0..4 {
            assert_eq!(
                targets[i],
                TargetRef {
                    side: Side::Player,
                    index: i as u8
                }
            );
        }
    }

    // ── S02: Targeting — SelfOnly ───────────────────────────────────

    #[test]
    fn targeting_self_only() {
        let source = TargetRef {
            side: Side::Player,
            index: 2,
        };
        let targets = resolve_targets(TargetMode::SelfOnly, source, None, 4, 3);
        assert_eq!(targets, vec![source]);
    }

    // ── S03: Multi-hit — 3 hits normal ──────────────────────────────

    #[test]
    fn multi_hit_three_hits_normal() {
        let cfg = test_config();
        let results = resolve_multi_hit(3, 20, 200, 0, true, &cfg);
        assert_eq!(results.len(), 3);
        for r in &results {
            assert_eq!(r.damage, 20);
            assert!(!r.is_crit);
            assert!(!r.target_killed);
            assert_eq!(r.mana_generated, 1);
        }
        // Counter should advance: 1, 2, 3
        assert_eq!(results[0].crit_counter_after, 1);
        assert_eq!(results[1].crit_counter_after, 2);
        assert_eq!(results[2].crit_counter_after, 3);
    }

    // ── S03: Multi-hit — target dies on hit 2 ──────────────────────

    #[test]
    fn multi_hit_target_dies_early() {
        let cfg = test_config();
        let results = resolve_multi_hit(3, 30, 50, 0, true, &cfg);
        assert_eq!(results.len(), 2); // 3rd hit should not happen
        assert_eq!(results[0].damage, 30);
        assert!(!results[0].target_killed);
        assert_eq!(results[1].damage, 20); // only 20 HP remaining
        assert!(results[1].target_killed);
    }

    // ── S03 + S06: Crit triggers on 10th cumulative hit ────────────

    #[test]
    fn multi_hit_crit_on_tenth_hit() {
        let cfg = test_config();
        // Start with counter at 9, so the 1st hit becomes the 10th cumulative
        let results = resolve_multi_hit(2, 50, 500, 9, true, &cfg);
        assert_eq!(results.len(), 2);
        // First hit: counter goes 9->10, triggers crit
        assert!(results[0].is_crit);
        assert_eq!(results[0].damage, 100); // 50 * 2.0
        assert_eq!(results[0].crit_counter_after, 0); // reset
                                                      // Second hit: counter goes 0->1, normal
        assert!(!results[1].is_crit);
        assert_eq!(results[1].damage, 50);
        assert_eq!(results[1].crit_counter_after, 1);
    }

    // ── S03: Auto-attack generates mana per hit ────────────────────

    #[test]
    fn auto_attack_generates_mana() {
        let cfg = test_config();
        let results = resolve_multi_hit(3, 10, 200, 0, true, &cfg);
        for r in &results {
            assert_eq!(r.mana_generated, 1);
        }
    }

    // ── S03: Ability does NOT generate mana or advance crit ────────

    #[test]
    fn ability_no_mana_no_crit() {
        let cfg = test_config();
        let results = resolve_multi_hit(3, 25, 200, 5, false, &cfg);
        assert_eq!(results.len(), 3);
        for r in &results {
            assert_eq!(r.mana_generated, 0);
            assert!(!r.is_crit);
            assert_eq!(r.crit_counter_after, 5); // counter unchanged
        }
    }

    // ── S05: Mana pool — reset ──────────────────────────────────────

    #[test]
    fn mana_pool_reset() {
        let mut pool = ManaPool::new(10);
        pool.spend_mana(7);
        assert_eq!(pool.current_mana, 3);
        pool.reset_mana();
        assert_eq!(pool.current_mana, 10);
    }

    // ── S05: Mana pool — spend success ──────────────────────────────

    #[test]
    fn mana_pool_spend_success() {
        let mut pool = ManaPool::new(10);
        assert!(pool.spend_mana(5));
        assert_eq!(pool.current_mana, 5);
    }

    // ── S05: Mana pool — spend failure (insufficient) ───────────────

    #[test]
    fn mana_pool_spend_failure() {
        let mut pool = ManaPool::new(3);
        assert!(!pool.spend_mana(5));
        assert_eq!(pool.current_mana, 3); // unchanged
    }

    // ── S05: Mana pool — generate ───────────────────────────────────

    #[test]
    fn mana_pool_generate() {
        let mut pool = ManaPool::new(10);
        pool.spend_mana(5);
        assert_eq!(pool.current_mana, 5);
        pool.generate_mana(3);
        assert_eq!(pool.current_mana, 8);
    }

    // ── S05: Mana pool — generate caps at max ───────────────────────

    #[test]
    fn mana_pool_generate_caps_at_max() {
        let mut pool = ManaPool::new(10);
        pool.generate_mana(5); // already at 10, should stay 10
        assert_eq!(pool.current_mana, 10);
    }

    // ── S04: SPD ordering — basic ───────────────────────────────────

    #[test]
    fn speed_ordering_basic() {
        let mut state = BattleState {
            phase: BattlePhase::Planning,
            round: 1,
            player_units: vec![
                BattleUnit {
                    id: "slow".into(),
                    stats: test_stats(100, 10, 10, 10, 5),
                    current_hp: 100,
                    is_alive: true,
                    crit_counter: 0,
                    equipment_speed_bonus: 0,
                },
                BattleUnit {
                    id: "fast".into(),
                    stats: test_stats(100, 10, 10, 10, 20),
                    current_hp: 100,
                    is_alive: true,
                    crit_counter: 0,
                    equipment_speed_bonus: 0,
                },
            ],
            enemies: vec![BattleUnit {
                id: "medium".into(),
                stats: test_stats(100, 10, 10, 10, 12),
                current_hp: 100,
                is_alive: true,
                crit_counter: 0,
                equipment_speed_bonus: 0,
            }],
            planned_actions: vec![],
            mana_pool: ManaPool::new(5),
            execution_order: vec![],
        };

        state.compute_execution_order();

        // fast(20) > medium(12) > slow(5)
        assert_eq!(state.execution_order.len(), 3);
        assert_eq!(
            state.execution_order[0],
            TargetRef {
                side: Side::Player,
                index: 1
            }
        ); // fast
        assert_eq!(
            state.execution_order[1],
            TargetRef {
                side: Side::Enemy,
                index: 0
            }
        ); // medium
        assert_eq!(
            state.execution_order[2],
            TargetRef {
                side: Side::Player,
                index: 0
            }
        ); // slow
    }

    // ── S04: SPD ordering — tiebreaker by base SPD ──────────────────

    #[test]
    fn speed_ordering_tiebreaker() {
        let mut state = BattleState {
            phase: BattlePhase::Planning,
            round: 1,
            player_units: vec![
                BattleUnit {
                    id: "unit_a".into(),
                    stats: test_stats(100, 10, 10, 10, 10),
                    current_hp: 100,
                    is_alive: true,
                    crit_counter: 0,
                    equipment_speed_bonus: 5, // effective = 15
                },
                BattleUnit {
                    id: "unit_b".into(),
                    stats: test_stats(100, 10, 10, 10, 15),
                    current_hp: 100,
                    is_alive: true,
                    crit_counter: 0,
                    equipment_speed_bonus: 0, // effective = 15
                },
            ],
            enemies: vec![],
            planned_actions: vec![],
            mana_pool: ManaPool::new(5),
            execution_order: vec![],
        };

        state.compute_execution_order();

        // Both have effective SPD 15, but unit_b has base SPD 15 > unit_a base SPD 10
        assert_eq!(state.execution_order.len(), 2);
        assert_eq!(
            state.execution_order[0],
            TargetRef {
                side: Side::Player,
                index: 1
            }
        ); // unit_b
        assert_eq!(
            state.execution_order[1],
            TargetRef {
                side: Side::Player,
                index: 0
            }
        ); // unit_a
    }

    // ── S04: Dead units excluded from execution order ───────────────

    #[test]
    fn dead_units_excluded_from_order() {
        let mut state = BattleState {
            phase: BattlePhase::Planning,
            round: 1,
            player_units: vec![
                BattleUnit {
                    id: "alive".into(),
                    stats: test_stats(100, 10, 10, 10, 10),
                    current_hp: 100,
                    is_alive: true,
                    crit_counter: 0,
                    equipment_speed_bonus: 0,
                },
                BattleUnit {
                    id: "dead".into(),
                    stats: test_stats(100, 10, 10, 10, 50),
                    current_hp: 0,
                    is_alive: false,
                    crit_counter: 0,
                    equipment_speed_bonus: 0,
                },
            ],
            enemies: vec![],
            planned_actions: vec![],
            mana_pool: ManaPool::new(5),
            execution_order: vec![],
        };

        state.compute_execution_order();
        assert_eq!(state.execution_order.len(), 1);
        assert_eq!(
            state.execution_order[0],
            TargetRef {
                side: Side::Player,
                index: 0
            }
        );
    }
}
