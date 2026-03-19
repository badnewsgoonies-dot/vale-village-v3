#![allow(dead_code)]
//! Advanced Damage Modifiers (S14-S16)
//! Defense penetration, splash damage, chain damage.

use crate::shared::{DamageType, Side, Stats, TargetRef};

/// S14: Apply defense penetration.
/// Returns effective defense after ignoring `ignore_percent` of target DEF.
/// `ignore_percent` is 0.0..=1.0 (e.g. 0.2 = ignore 20% of DEF).
/// Result floors at 0.
pub fn apply_defense_penetration(target_def: u16, ignore_percent: f32) -> u16 {
    let effective = target_def as f32 * (1.0 - ignore_percent);
    if effective < 0.0 {
        0
    } else {
        effective as u16
    }
}

/// S15: Determine which targets receive splash damage.
/// Returns all enemies on the same side as the primary target, excluding the
/// primary target itself, each paired with the splash damage multiplier.
pub fn calculate_splash_targets(
    primary_target: TargetRef,
    splash_pct: f32,
    all_enemies: &[TargetRef],
) -> Vec<(TargetRef, f32)> {
    all_enemies
        .iter()
        .filter(|t| t.side == primary_target.side && t.index != primary_target.index)
        .map(|&t| (t, splash_pct))
        .collect()
}

/// S15: Compute splash damage from primary hit.
/// Floor at 1 if `splash_percent` > 0.
pub fn apply_splash_damage(primary_damage: u16, splash_percent: f32) -> u16 {
    if splash_percent <= 0.0 {
        return 0;
    }
    let raw = (primary_damage as f32 * splash_percent) as u16;
    if raw < 1 { 1 } else { raw }
}

/// S16: Determine chain-damage targets.
/// Chain hits every target on the opposing side of `source`, ordered by index.
pub fn calculate_chain_targets(source: TargetRef, all_targets: &[TargetRef]) -> Vec<TargetRef> {
    let opposing = match source.side {
        Side::Player => Side::Enemy,
        Side::Enemy => Side::Player,
    };
    let mut targets: Vec<TargetRef> = all_targets
        .iter()
        .filter(|t| t.side == opposing)
        .copied()
        .collect();
    targets.sort_by_key(|t| t.index);
    targets
}

/// Combined modifier pipeline.
/// 1. If `ignore_def_pct` is present, reduce effective DEF via penetration.
/// 2. Compute final damage based on `damage_type`:
///    - Physical: base_damage + atk - (effective_def * config_def_mult), floor 1
///    - Psynergy: base_damage + mag - (effective_def * config_def_mult), floor 1
pub fn apply_all_modifiers(
    base_damage: u16,
    target_def: u16,
    ignore_def_pct: Option<f32>,
    damage_type: DamageType,
    attacker_stats: &Stats,
    config_def_mult: f32,
) -> u16 {
    let effective_def = match ignore_def_pct {
        Some(pct) => apply_defense_penetration(target_def, pct),
        None => target_def,
    };

    let offense = match damage_type {
        DamageType::Physical => attacker_stats.atk.get() as f32,
        DamageType::Psynergy => attacker_stats.mag.get() as f32,
    };

    let raw = base_damage as f32 + offense - (effective_def as f32 * config_def_mult);
    if raw < 1.0 { 1 } else { raw as u16 }
}

// ── Tests ────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::shared::bounded_types::*;
    use crate::shared::{Side, TargetRef};

    fn test_stats(hp: u16, atk: u16, def: u16, mag: u16, spd: u16) -> Stats {
        Stats {
            hp: Hp::new_unchecked(hp),
            atk: BaseStat::new_unchecked(atk),
            def: BaseStat::new_unchecked(def),
            mag: BaseStat::new_unchecked(mag),
            spd: BaseStat::new_unchecked(spd),
        }
    }

    // ── Defense penetration (S14) ────────────────────────────────────

    #[test]
    fn defense_pen_zero_percent() {
        assert_eq!(apply_defense_penetration(100, 0.0), 100);
    }

    #[test]
    fn defense_pen_twenty_percent() {
        // 100 * 0.80 = 80
        assert_eq!(apply_defense_penetration(100, 0.2), 80);
    }

    #[test]
    fn defense_pen_fifty_percent() {
        assert_eq!(apply_defense_penetration(100, 0.5), 50);
    }

    #[test]
    fn defense_pen_hundred_percent() {
        assert_eq!(apply_defense_penetration(100, 1.0), 0);
    }

    // ── Splash targets (S15) ────────────────────────────────────────

    #[test]
    fn splash_excludes_primary_includes_others() {
        let primary = TargetRef { side: Side::Enemy, index: 1 };
        let all = vec![
            TargetRef { side: Side::Enemy, index: 0 },
            TargetRef { side: Side::Enemy, index: 1 },
            TargetRef { side: Side::Enemy, index: 2 },
        ];
        let result = calculate_splash_targets(primary, 0.5, &all);
        assert_eq!(result.len(), 2);
        assert!(result.iter().all(|(t, _)| t.index != 1));
        assert!(result.iter().all(|(_, pct)| (*pct - 0.5).abs() < f32::EPSILON));
    }

    #[test]
    fn splash_single_enemy_no_targets() {
        let primary = TargetRef { side: Side::Enemy, index: 0 };
        let all = vec![TargetRef { side: Side::Enemy, index: 0 }];
        let result = calculate_splash_targets(primary, 0.5, &all);
        assert!(result.is_empty());
    }

    #[test]
    fn splash_ignores_allies() {
        let primary = TargetRef { side: Side::Enemy, index: 0 };
        let all = vec![
            TargetRef { side: Side::Enemy, index: 0 },
            TargetRef { side: Side::Player, index: 0 },
            TargetRef { side: Side::Player, index: 1 },
        ];
        let result = calculate_splash_targets(primary, 0.3, &all);
        assert!(result.is_empty());
    }

    // ── Splash damage math (S15) ────────────────────────────────────

    #[test]
    fn splash_damage_math() {
        // 100 * 0.5 = 50
        assert_eq!(apply_splash_damage(100, 0.5), 50);
    }

    #[test]
    fn splash_damage_floor_at_one() {
        // 1 * 0.01 would truncate to 0, but floor is 1
        assert_eq!(apply_splash_damage(1, 0.01), 1);
    }

    #[test]
    fn splash_damage_zero_percent_returns_zero() {
        assert_eq!(apply_splash_damage(100, 0.0), 0);
    }

    // ── Chain targets (S16) ─────────────────────────────────────────

    #[test]
    fn chain_targets_returns_all_enemies() {
        let source = TargetRef { side: Side::Player, index: 0 };
        let all = vec![
            TargetRef { side: Side::Player, index: 0 },
            TargetRef { side: Side::Enemy, index: 2 },
            TargetRef { side: Side::Enemy, index: 0 },
            TargetRef { side: Side::Enemy, index: 1 },
        ];
        let result = calculate_chain_targets(source, &all);
        assert_eq!(result.len(), 3);
        // Ordered by index (left to right)
        assert_eq!(result[0].index, 0);
        assert_eq!(result[1].index, 1);
        assert_eq!(result[2].index, 2);
    }

    // ── Combined modifiers ──────────────────────────────────────────

    #[test]
    fn combined_physical_with_pen() {
        let stats = test_stats(100, 30, 10, 5, 10);
        // base 50 + atk 30 - (def 100 * (1 - 0.5) * 0.5) = 80 - 25 = 55
        let dmg = apply_all_modifiers(50, 100, Some(0.5), DamageType::Physical, &stats, 0.5);
        assert_eq!(dmg, 55);
    }

    #[test]
    fn combined_psynergy_no_pen() {
        let stats = test_stats(100, 10, 10, 40, 10);
        // base 60 + mag 40 - (def 50 * 1.0) = 100 - 50 = 50
        let dmg = apply_all_modifiers(60, 50, None, DamageType::Psynergy, &stats, 1.0);
        assert_eq!(dmg, 50);
    }

    #[test]
    fn combined_floor_at_one() {
        let stats = test_stats(100, 1, 10, 1, 10);
        // base 1 + atk 1 - (def 200 * 1.0) = 2 - 200 = -198 -> floor 1
        let dmg = apply_all_modifiers(1, 200, None, DamageType::Physical, &stats, 1.0);
        assert_eq!(dmg, 1);
    }
}
