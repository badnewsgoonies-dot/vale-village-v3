#![allow(dead_code)]
//! Progression domain — XP, leveling, stat growth, and ability unlocking.

use crate::shared::{
    bounded_types::{BaseStat, Hp, Level}, AbilityId, AbilityProgression, GrowthRates, Stats, UnitDef, UnitId,
};

// ── Constants ────────────────────────────────────────────────────────

const MAX_LEVEL: u8 = 20;

// ── Structs ──────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct UnitProgress {
    pub unit_id: UnitId,
    pub level: u8,
    pub current_xp: u32,
}

#[derive(Debug, Clone)]
pub struct LevelUpResult {
    pub levels_gained: Vec<u8>,
    pub new_stats: Stats,
    pub new_abilities: Vec<AbilityId>,
}

// ── Functions ────────────────────────────────────────────────────────

/// XP required to reach a given level.
/// Curve: level * level * 100
pub fn xp_for_level(level: u8) -> u32 {
    (level as u32) * (level as u32) * 100
}

/// XP remaining until the next level-up.
/// Returns 0 if already at max level.
pub fn xp_to_next_level(progress: &UnitProgress) -> u32 {
    if progress.level >= MAX_LEVEL {
        return 0;
    }
    let needed = xp_for_level(progress.level + 1);
    needed.saturating_sub(progress.current_xp)
}

/// Add XP to a unit. Returns a list of levels gained (can be multiple).
/// Caps at MAX_LEVEL.
pub fn add_xp(progress: &mut UnitProgress, xp: u32) -> Vec<u8> {
    let mut levels_gained = Vec::new();

    if progress.level >= MAX_LEVEL {
        return levels_gained;
    }

    progress.current_xp += xp;

    while progress.level < MAX_LEVEL {
        let next_level_xp = xp_for_level(progress.level + 1);
        if progress.current_xp >= next_level_xp {
            progress.level += 1;
            levels_gained.push(progress.level);
        } else {
            break;
        }
    }

    // Cap at max level
    if progress.level >= MAX_LEVEL {
        progress.level = MAX_LEVEL;
    }

    levels_gained
}

/// Calculate stats at a given level: base + growth * (level - 1).
pub fn calculate_stats_at_level(base_stats: &Stats, growth_rates: &GrowthRates, level: u8) -> Stats {
    let growth = if level > 1 { (level - 1) as u16 } else { 0 };
    Stats {
        hp: Hp::new_unchecked(
            base_stats
                .hp
                .get()
                .saturating_add(growth_rates.hp.get().saturating_mul(growth))
                .min(9999),
        ),
        atk: BaseStat::new_unchecked(
            base_stats
                .atk
                .get()
                .saturating_add(growth_rates.atk.get().saturating_mul(growth))
                .min(9999),
        ),
        def: BaseStat::new_unchecked(
            base_stats
                .def
                .get()
                .saturating_add(growth_rates.def.get().saturating_mul(growth))
                .min(9999),
        ),
        mag: BaseStat::new_unchecked(
            base_stats
                .mag
                .get()
                .saturating_add(growth_rates.mag.get().saturating_mul(growth))
                .min(9999),
        ),
        spd: BaseStat::new_unchecked(
            base_stats
                .spd
                .get()
                .saturating_add(growth_rates.spd.get().saturating_mul(growth))
                .min(9999),
        ),
    }
}

/// All abilities unlocked at or below the given level.
pub fn unlocked_abilities(abilities: &[AbilityProgression], level: u8) -> Vec<AbilityId> {
    let level = Level::new(level).unwrap_or_else(|_| Level::new_unchecked(level.clamp(1, 99)));
    abilities
        .iter()
        .filter(|ap| ap.level <= level)
        .map(|ap| ap.ability_id.clone())
        .collect()
}

/// Full battle-reward flow: add XP, recalculate stats, find newly unlocked abilities.
pub fn apply_battle_rewards(
    progress: &mut UnitProgress,
    xp: u32,
    unit_def: &UnitDef,
) -> LevelUpResult {
    let old_level = progress.level;
    let levels_gained = add_xp(progress, xp);

    let new_stats = calculate_stats_at_level(
        &unit_def.base_stats,
        &unit_def.growth_rates,
        progress.level,
    );

    // Abilities that were NOT unlocked before but ARE now
    let old_abilities = unlocked_abilities(&unit_def.abilities, old_level);
    let all_abilities = unlocked_abilities(&unit_def.abilities, progress.level);
    let new_abilities: Vec<AbilityId> = all_abilities
        .into_iter()
        .filter(|a| !old_abilities.contains(a))
        .collect();

    LevelUpResult {
        levels_gained,
        new_stats,
        new_abilities,
    }
}

// ── Tests ────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::shared::{
        bounded_types::*, AbilityId, AbilityProgression, Element, GrowthRates, Stats, UnitDef,
        UnitId,
    };

    fn make_progress(level: u8, xp: u32) -> UnitProgress {
        UnitProgress {
            unit_id: UnitId("test-unit".to_string()),
            level,
            current_xp: xp,
        }
    }

    fn test_base_stats() -> Stats {
        Stats {
            hp: Hp::new_unchecked(100),
            atk: BaseStat::new_unchecked(10),
            def: BaseStat::new_unchecked(8),
            mag: BaseStat::new_unchecked(12),
            spd: BaseStat::new_unchecked(10),
        }
    }

    fn test_growth_rates() -> GrowthRates {
        GrowthRates {
            hp: GrowthRate::new_unchecked(15),
            atk: GrowthRate::new_unchecked(3),
            def: GrowthRate::new_unchecked(2),
            mag: GrowthRate::new_unchecked(4),
            spd: GrowthRate::new_unchecked(2),
        }
    }

    fn test_abilities() -> Vec<AbilityProgression> {
        vec![
            AbilityProgression { level: Level::new_unchecked(1), ability_id: AbilityId("strike".to_string()) },
            AbilityProgression { level: Level::new_unchecked(1), ability_id: AbilityId("fireball".to_string()) },
            AbilityProgression { level: Level::new_unchecked(3), ability_id: AbilityId("heal".to_string()) },
            AbilityProgression { level: Level::new_unchecked(5), ability_id: AbilityId("earthquake".to_string()) },
            AbilityProgression { level: Level::new_unchecked(10), ability_id: AbilityId("meteor".to_string()) },
            AbilityProgression { level: Level::new_unchecked(15), ability_id: AbilityId("ultima".to_string()) },
            AbilityProgression { level: Level::new_unchecked(20), ability_id: AbilityId("genesis".to_string()) },
        ]
    }

    fn test_unit_def() -> UnitDef {
        UnitDef {
            id: UnitId("test-unit".to_string()),
            name: "Test Unit".to_string(),
            element: Element::Venus,
            mana_contribution: crate::shared::bounded_types::ManaCost::new_unchecked(2),
            base_stats: test_base_stats(),
            growth_rates: test_growth_rates(),
            abilities: test_abilities(),
        }
    }

    // ── xp_for_level ─────────────────────────────────────────────────

    #[test]
    fn xp_for_level_1() {
        assert_eq!(xp_for_level(1), 100);
    }

    #[test]
    fn xp_for_level_5() {
        assert_eq!(xp_for_level(5), 2500);
    }

    #[test]
    fn xp_for_level_20() {
        assert_eq!(xp_for_level(20), 40000);
    }

    // ── xp_to_next_level ─────────────────────────────────────────────

    #[test]
    fn xp_to_next_level_from_level_1() {
        let progress = make_progress(1, 100);
        // Need 400 for level 2, have 100 => 300 remaining
        assert_eq!(xp_to_next_level(&progress), 300);
    }

    #[test]
    fn xp_to_next_level_at_max() {
        let progress = make_progress(20, 50000);
        assert_eq!(xp_to_next_level(&progress), 0);
    }

    // ── add_xp: single level up ─────────────────────────────────────

    #[test]
    fn add_xp_single_level_up() {
        let mut progress = make_progress(1, 100);
        // Level 2 needs 400 XP. Give 300 more to hit exactly 400.
        let levels = add_xp(&mut progress, 300);
        assert_eq!(levels, vec![2]);
        assert_eq!(progress.level, 2);
        assert_eq!(progress.current_xp, 400);
    }

    // ── add_xp: multiple level ups ──────────────────────────────────

    #[test]
    fn add_xp_multiple_level_ups() {
        let mut progress = make_progress(1, 0);
        // Level 2 = 400, level 3 = 900, level 4 = 1600, level 5 = 2500
        // Give 2500 XP from 0 => should reach level 5
        let levels = add_xp(&mut progress, 2500);
        assert_eq!(levels, vec![2, 3, 4, 5]);
        assert_eq!(progress.level, 5);
    }

    // ── add_xp: no level up ─────────────────────────────────────────

    #[test]
    fn add_xp_no_level_up() {
        let mut progress = make_progress(1, 100);
        // Level 2 needs 400, give only 50 more (total 150)
        let levels = add_xp(&mut progress, 50);
        assert!(levels.is_empty());
        assert_eq!(progress.level, 1);
        assert_eq!(progress.current_xp, 150);
    }

    // ── add_xp: max level cap ───────────────────────────────────────

    #[test]
    fn add_xp_max_level_cap() {
        let mut progress = make_progress(19, 36100);
        // Level 20 = 40000. Give massive XP.
        let levels = add_xp(&mut progress, 100_000);
        assert_eq!(levels, vec![20]);
        assert_eq!(progress.level, 20);
    }

    #[test]
    fn add_xp_already_max_level() {
        let mut progress = make_progress(20, 50000);
        let levels = add_xp(&mut progress, 5000);
        assert!(levels.is_empty());
        assert_eq!(progress.level, 20);
        // XP doesn't change when already at max
        assert_eq!(progress.current_xp, 50000);
    }

    // ── calculate_stats_at_level ─────────────────────────────────────

    #[test]
    fn stats_at_level_1_equals_base() {
        let stats = calculate_stats_at_level(&test_base_stats(), &test_growth_rates(), 1);
        assert_eq!(stats.hp.get(), 100);
        assert_eq!(stats.atk.get(), 10);
        assert_eq!(stats.def.get(), 8);
        assert_eq!(stats.mag.get(), 12);
        assert_eq!(stats.spd.get(), 10);
    }

    #[test]
    fn stats_at_level_10_with_growth() {
        let stats = calculate_stats_at_level(&test_base_stats(), &test_growth_rates(), 10);
        // base + growth * 9
        assert_eq!(stats.hp.get(), 100 + 15 * 9);   // 235
        assert_eq!(stats.atk.get(), 10 + 3 * 9);    // 37
        assert_eq!(stats.def.get(), 8 + 2 * 9);     // 26
        assert_eq!(stats.mag.get(), 12 + 4 * 9);    // 48
        assert_eq!(stats.spd.get(), 10 + 2 * 9);    // 28
    }

    // ── unlocked_abilities ───────────────────────────────────────────

    #[test]
    fn unlocked_abilities_at_level_1() {
        let abilities = unlocked_abilities(&test_abilities(), 1);
        assert_eq!(abilities.len(), 2);
        assert!(abilities.contains(&AbilityId("strike".to_string())));
        assert!(abilities.contains(&AbilityId("fireball".to_string())));
    }

    #[test]
    fn unlocked_abilities_at_level_5() {
        let abilities = unlocked_abilities(&test_abilities(), 5);
        assert_eq!(abilities.len(), 4);
        assert!(abilities.contains(&AbilityId("strike".to_string())));
        assert!(abilities.contains(&AbilityId("fireball".to_string())));
        assert!(abilities.contains(&AbilityId("heal".to_string())));
        assert!(abilities.contains(&AbilityId("earthquake".to_string())));
    }

    #[test]
    fn unlocked_abilities_at_level_10() {
        let abilities = unlocked_abilities(&test_abilities(), 10);
        assert_eq!(abilities.len(), 5);
        assert!(abilities.contains(&AbilityId("meteor".to_string())));
    }

    // ── apply_battle_rewards ─────────────────────────────────────────

    #[test]
    fn apply_battle_rewards_full_flow() {
        let unit_def = test_unit_def();
        let mut progress = make_progress(1, 0);

        // Give enough XP to reach level 5 (2500)
        let result = apply_battle_rewards(&mut progress, 2500, &unit_def);

        assert_eq!(result.levels_gained, vec![2, 3, 4, 5]);
        assert_eq!(progress.level, 5);

        // Stats at level 5: base + growth * 4
        assert_eq!(result.new_stats.hp.get(), 100 + 15 * 4);   // 160
        assert_eq!(result.new_stats.atk.get(), 10 + 3 * 4);    // 22
        assert_eq!(result.new_stats.def.get(), 8 + 2 * 4);     // 16
        assert_eq!(result.new_stats.mag.get(), 12 + 4 * 4);    // 28
        assert_eq!(result.new_stats.spd.get(), 10 + 2 * 4);    // 18

        // New abilities: heal (lv3) and earthquake (lv5)
        // (strike and fireball were already unlocked at lv1)
        assert_eq!(result.new_abilities.len(), 2);
        assert!(result.new_abilities.contains(&AbilityId("heal".to_string())));
        assert!(result.new_abilities.contains(&AbilityId("earthquake".to_string())));
    }

    #[test]
    fn apply_battle_rewards_no_level_up() {
        let unit_def = test_unit_def();
        let mut progress = make_progress(1, 100);

        let result = apply_battle_rewards(&mut progress, 50, &unit_def);

        assert!(result.levels_gained.is_empty());
        assert!(result.new_abilities.is_empty());
        // Stats remain at level 1
        assert_eq!(result.new_stats.hp.get(), 100);
    }
}
