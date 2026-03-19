#![allow(dead_code)]
//! Enemy AI Domain — decision engine for enemy turn planning.
//!
//! Pure logic: reads battle state and returns a BattleAction.
//! Does not execute anything or mutate state.

use crate::shared::{
    AbilityCategory, AbilityDef, BattleAction, Side, Stats, TargetMode, TargetRef,
};

// ── AI Strategy ─────────────────────────────────────────────────────

/// High-level behaviour personality assigned to an enemy.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AiStrategy {
    /// Focus weakest player, use highest damage ability.
    Aggressive,
    /// Attack random player, prefer healing/buff when low HP.
    Defensive,
    /// Mix of attacks and abilities based on situation.
    Balanced,
}

// ── Target Selection ────────────────────────────────────────────────

/// Criteria for choosing a target among the opposing side.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TargetCriteria {
    LowestHp,
    HighestAtk,
    FirstAlive,
}

/// Lightweight view of one unit visible to the AI.
#[derive(Debug, Clone)]
pub struct AiUnitView {
    pub index: u8,
    pub stats: Stats,
    pub current_hp: u16,
    pub max_hp: u16,
    pub is_alive: bool,
}

/// Select a target from a list of opponent views based on criteria.
/// Returns None only if no unit is alive.
pub fn choose_target(units: &[AiUnitView], criteria: TargetCriteria) -> Option<TargetRef> {
    let alive: Vec<&AiUnitView> = units.iter().filter(|u| u.is_alive).collect();
    if alive.is_empty() {
        return None;
    }

    let chosen = match criteria {
        TargetCriteria::LowestHp => alive
            .iter()
            .min_by_key(|u| u.current_hp)
            .unwrap(),
        TargetCriteria::HighestAtk => alive
            .iter()
            .max_by_key(|u| u.stats.atk.get())
            .unwrap(),
        TargetCriteria::FirstAlive => alive.first().unwrap(),
    };

    Some(TargetRef {
        side: Side::Player,
        index: chosen.index,
    })
}

// ── Ability helpers ─────────────────────────────────────────────────

/// Find the damaging ability with the highest base_power that the enemy
/// can afford with the given mana budget.
fn best_damaging_ability<'a>(
    abilities: &'a [&'a AbilityDef],
    mana_available: u8,
) -> Option<&'a AbilityDef> {
    abilities
        .iter()
        .filter(|a| {
            matches!(
                a.category,
                AbilityCategory::Physical | AbilityCategory::Psynergy
            )
        })
        .filter(|a| a.mana_cost.get() <= mana_available)
        .max_by_key(|a| a.base_power.get())
        .copied()
}

/// Find the first healing ability the enemy can afford.
fn healing_ability<'a>(
    abilities: &'a [&'a AbilityDef],
    mana_available: u8,
) -> Option<&'a AbilityDef> {
    abilities
        .iter()
        .filter(|a| a.category == AbilityCategory::Healing)
        .filter(|a| a.mana_cost.get() <= mana_available)
        .max_by_key(|a| a.base_power.get())
        .copied()
}

/// Find the first buff ability the enemy can afford.
fn buff_ability<'a>(
    abilities: &'a [&'a AbilityDef],
    mana_available: u8,
) -> Option<&'a AbilityDef> {
    abilities
        .iter()
        .filter(|a| a.category == AbilityCategory::Buff)
        .filter(|a| a.mana_cost.get() <= mana_available)
        .max_by_key(|a| a.base_power.get())
        .copied()
}

/// Find an AoE damaging ability the enemy can afford.
fn aoe_damaging_ability<'a>(
    abilities: &'a [&'a AbilityDef],
    mana_available: u8,
) -> Option<&'a AbilityDef> {
    abilities
        .iter()
        .filter(|a| {
            matches!(
                a.category,
                AbilityCategory::Physical | AbilityCategory::Psynergy
            )
        })
        .filter(|a| a.targets == TargetMode::AllEnemies)
        .filter(|a| a.mana_cost.get() <= mana_available)
        .max_by_key(|a| a.base_power.get())
        .copied()
}

/// Build the targets vec for a given ability aimed at one target.
fn ability_targets(ability: &AbilityDef, single_target: TargetRef, alive_count: u8) -> Vec<TargetRef> {
    match ability.targets {
        TargetMode::SingleEnemy => vec![single_target],
        TargetMode::AllEnemies => {
            // From enemy perspective, "all enemies" means all players
            (0..alive_count)
                .map(|i| TargetRef { side: Side::Player, index: i })
                .collect()
        }
        TargetMode::SelfOnly => vec![TargetRef { side: Side::Enemy, index: single_target.index }],
        TargetMode::SingleAlly => vec![TargetRef { side: Side::Enemy, index: single_target.index }],
        TargetMode::AllAllies => {
            // From enemy perspective, "all allies" means all enemies
            (0..alive_count)
                .map(|i| TargetRef { side: Side::Enemy, index: i })
                .collect()
        }
    }
}

// ── Main decision function ──────────────────────────────────────────

/// The enemy's own view of itself, enough for the AI to decide.
#[derive(Debug, Clone)]
pub struct AiSelfView {
    pub index: u8,
    pub current_hp: u16,
    pub max_hp: u16,
    pub mana_available: u8,
}

/// Choose a BattleAction for one enemy unit.
///
/// This is **pure logic** — it examines the state and returns an action.
/// The caller (battle_engine) is responsible for executing it.
pub fn choose_enemy_action(
    enemy: &AiSelfView,
    player_views: &[AiUnitView],
    abilities: &[&AbilityDef],
    strategy: AiStrategy,
) -> BattleAction {
    let alive_players: Vec<&AiUnitView> = player_views.iter().filter(|u| u.is_alive).collect();
    let alive_count = alive_players.len() as u8;

    if alive_count == 0 {
        // No targets; fallback auto-attack index 0 (will be a no-op at execution)
        return BattleAction::Attack {
            target: TargetRef { side: Side::Player, index: 0 },
        };
    }

    match strategy {
        AiStrategy::Aggressive => choose_aggressive(enemy, player_views, abilities, alive_count),
        AiStrategy::Defensive => choose_defensive(enemy, player_views, abilities, alive_count),
        AiStrategy::Balanced => choose_balanced(enemy, player_views, abilities, alive_count),
    }
}

// ── Strategy implementations ────────────────────────────────────────

fn choose_aggressive(
    enemy: &AiSelfView,
    player_views: &[AiUnitView],
    abilities: &[&AbilityDef],
    alive_count: u8,
) -> BattleAction {
    // Target: lowest HP player
    let target = choose_target(player_views, TargetCriteria::LowestHp)
        .expect("alive_count > 0 checked by caller");

    // Prefer highest power damaging ability if mana allows
    if let Some(ability) = best_damaging_ability(abilities, enemy.mana_available) {
        let targets = ability_targets(ability, target, alive_count);
        return BattleAction::UseAbility {
            ability_id: ability.id.clone(),
            targets,
        };
    }

    // Fallback: auto-attack
    BattleAction::Attack { target }
}

fn choose_defensive(
    enemy: &AiSelfView,
    player_views: &[AiUnitView],
    abilities: &[&AbilityDef],
    alive_count: u8,
) -> BattleAction {
    let hp_ratio = enemy.current_hp as f32 / enemy.max_hp.max(1) as f32;

    // If HP < 30%: try to heal
    if hp_ratio < 0.30 {
        if let Some(heal) = healing_ability(abilities, enemy.mana_available) {
            let self_ref = TargetRef { side: Side::Enemy, index: enemy.index };
            let targets = ability_targets(heal, self_ref, alive_count);
            return BattleAction::UseAbility {
                ability_id: heal.id.clone(),
                targets,
            };
        }
    }

    // If has a buff ability: buff self
    if let Some(buff) = buff_ability(abilities, enemy.mana_available) {
        let self_ref = TargetRef { side: Side::Enemy, index: enemy.index };
        let targets = ability_targets(buff, self_ref, alive_count);
        return BattleAction::UseAbility {
            ability_id: buff.id.clone(),
            targets,
        };
    }

    // Otherwise: attack first alive player
    let target = choose_target(player_views, TargetCriteria::FirstAlive)
        .expect("alive_count > 0 checked by caller");
    BattleAction::Attack { target }
}

fn choose_balanced(
    enemy: &AiSelfView,
    player_views: &[AiUnitView],
    abilities: &[&AbilityDef],
    alive_count: u8,
) -> BattleAction {
    // If any player < 25% HP: focus that unit (go for the kill)
    let low_hp_player: Option<&AiUnitView> = player_views
        .iter()
        .filter(|u| u.is_alive)
        .find(|u| (u.current_hp as f32 / u.max_hp.max(1) as f32) < 0.25);

    if let Some(weak) = low_hp_player {
        let target = TargetRef { side: Side::Player, index: weak.index };
        // Use best ability if available, else auto-attack
        if let Some(ability) = best_damaging_ability(abilities, enemy.mana_available) {
            if ability.targets == TargetMode::SingleEnemy {
                return BattleAction::UseAbility {
                    ability_id: ability.id.clone(),
                    targets: vec![target],
                };
            }
        }
        return BattleAction::Attack { target };
    }

    // If AoE ability available and 3+ players alive: use AoE
    if alive_count >= 3 {
        if let Some(aoe) = aoe_damaging_ability(abilities, enemy.mana_available) {
            let targets = ability_targets(aoe, TargetRef { side: Side::Player, index: 0 }, alive_count);
            return BattleAction::UseAbility {
                ability_id: aoe.id.clone(),
                targets,
            };
        }
    }

    // Otherwise: attack the player with highest ATK (remove biggest threat)
    let target = choose_target(player_views, TargetCriteria::HighestAtk)
        .expect("alive_count > 0 checked by caller");
    BattleAction::Attack { target }
}

// ── Tests ───────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::shared::bounded_types::{BasePower, HitCount, Level, ManaCost};
    use crate::shared::{
        AbilityCategory, AbilityDef, AbilityId, DamageType, TargetMode,
    };

    // ── Helpers ──────────────────────────────────────────────────────

    fn make_player_view(index: u8, hp: u16, max_hp: u16, atk: u16) -> AiUnitView {
        AiUnitView {
            index,
            stats: Stats { hp: max_hp, atk, def: 10, mag: 10, spd: 10 },
            current_hp: hp,
            max_hp,
            is_alive: hp > 0,
        }
    }

    fn make_enemy_self(index: u8, hp: u16, max_hp: u16, mana: u8) -> AiSelfView {
        AiSelfView { index, current_hp: hp, max_hp, mana_available: mana }
    }

    fn make_ability(id: &str, category: AbilityCategory, power: u16, cost: u8, targets: TargetMode) -> AbilityDef {
        AbilityDef {
            id: AbilityId(id.to_string()),
            name: id.to_string(),
            category,
            damage_type: match category {
                AbilityCategory::Physical => Some(DamageType::Physical),
                AbilityCategory::Psynergy => Some(DamageType::Psynergy),
                _ => None,
            },
            element: None,
            mana_cost: ManaCost::new_unchecked(cost),
            base_power: BasePower::new_unchecked(power),
            targets,
            unlock_level: Level::new_unchecked(1),
            hit_count: HitCount::new_unchecked(1),
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
        }
    }

    // ── choose_target tests ─────────────────────────────────────────

    #[test]
    fn target_lowest_hp() {
        let views = vec![
            make_player_view(0, 80, 100, 20),
            make_player_view(1, 30, 100, 20),
            make_player_view(2, 50, 100, 20),
        ];
        let result = choose_target(&views, TargetCriteria::LowestHp).unwrap();
        assert_eq!(result.index, 1);
        assert_eq!(result.side, Side::Player);
    }

    #[test]
    fn target_highest_atk() {
        let views = vec![
            make_player_view(0, 100, 100, 15),
            make_player_view(1, 100, 100, 40),
            make_player_view(2, 100, 100, 25),
        ];
        let result = choose_target(&views, TargetCriteria::HighestAtk).unwrap();
        assert_eq!(result.index, 1);
    }

    #[test]
    fn target_first_alive_skips_dead() {
        let views = vec![
            make_player_view(0, 0, 100, 20),  // dead
            make_player_view(1, 50, 100, 20),
            make_player_view(2, 80, 100, 20),
        ];
        let result = choose_target(&views, TargetCriteria::FirstAlive).unwrap();
        assert_eq!(result.index, 1);
    }

    #[test]
    fn target_returns_none_when_all_dead() {
        let views = vec![
            make_player_view(0, 0, 100, 20),
            make_player_view(1, 0, 100, 20),
        ];
        assert!(choose_target(&views, TargetCriteria::LowestHp).is_none());
    }

    // ── Aggressive strategy tests ───────────────────────────────────

    #[test]
    fn aggressive_targets_lowest_hp_player() {
        let players = vec![
            make_player_view(0, 90, 100, 20),
            make_player_view(1, 20, 100, 20),
            make_player_view(2, 60, 100, 20),
        ];
        let enemy = make_enemy_self(0, 100, 100, 0);
        let action = choose_enemy_action(&enemy, &players, &[], AiStrategy::Aggressive);

        match action {
            BattleAction::Attack { target } => assert_eq!(target.index, 1),
            _ => panic!("expected auto-attack"),
        }
    }

    #[test]
    fn aggressive_uses_highest_power_ability_with_mana() {
        let players = vec![
            make_player_view(0, 80, 100, 20),
            make_player_view(1, 30, 100, 20),
        ];
        let enemy = make_enemy_self(0, 100, 100, 5);
        let weak = make_ability("weak_atk", AbilityCategory::Physical, 20, 2, TargetMode::SingleEnemy);
        let strong = make_ability("strong_atk", AbilityCategory::Psynergy, 80, 4, TargetMode::SingleEnemy);
        let abilities: Vec<&AbilityDef> = vec![&weak, &strong];

        let action = choose_enemy_action(&enemy, &players, &abilities, AiStrategy::Aggressive);
        match action {
            BattleAction::UseAbility { ability_id, targets } => {
                assert_eq!(ability_id.0, "strong_atk");
                assert_eq!(targets.len(), 1);
                assert_eq!(targets[0].index, 1); // lowest HP
            }
            _ => panic!("expected UseAbility"),
        }
    }

    #[test]
    fn aggressive_falls_back_to_attack_without_mana() {
        let players = vec![
            make_player_view(0, 50, 100, 20),
            make_player_view(1, 80, 100, 20),
        ];
        let enemy = make_enemy_self(0, 100, 100, 0); // no mana
        let expensive = make_ability("nuke", AbilityCategory::Psynergy, 100, 5, TargetMode::SingleEnemy);
        let abilities: Vec<&AbilityDef> = vec![&expensive];

        let action = choose_enemy_action(&enemy, &players, &abilities, AiStrategy::Aggressive);
        match action {
            BattleAction::Attack { target } => assert_eq!(target.index, 0), // lowest HP
            _ => panic!("expected Attack fallback"),
        }
    }

    // ── Defensive strategy tests ────────────────────────────────────

    #[test]
    fn defensive_heals_when_low_hp() {
        let players = vec![make_player_view(0, 80, 100, 20)];
        let enemy = make_enemy_self(0, 20, 100, 5); // 20% HP
        let heal = make_ability("cure", AbilityCategory::Healing, 50, 3, TargetMode::SelfOnly);
        let abilities: Vec<&AbilityDef> = vec![&heal];

        let action = choose_enemy_action(&enemy, &players, &abilities, AiStrategy::Defensive);
        match action {
            BattleAction::UseAbility { ability_id, .. } => assert_eq!(ability_id.0, "cure"),
            _ => panic!("expected healing ability"),
        }
    }

    #[test]
    fn defensive_buffs_when_healthy() {
        let players = vec![make_player_view(0, 80, 100, 20)];
        let enemy = make_enemy_self(0, 90, 100, 5); // healthy
        let buff = make_ability("shield", AbilityCategory::Buff, 0, 2, TargetMode::SelfOnly);
        let abilities: Vec<&AbilityDef> = vec![&buff];

        let action = choose_enemy_action(&enemy, &players, &abilities, AiStrategy::Defensive);
        match action {
            BattleAction::UseAbility { ability_id, .. } => assert_eq!(ability_id.0, "shield"),
            _ => panic!("expected buff ability"),
        }
    }

    #[test]
    fn defensive_attacks_first_alive_when_no_abilities() {
        let players = vec![
            make_player_view(0, 0, 100, 20),  // dead
            make_player_view(1, 60, 100, 20),
            make_player_view(2, 80, 100, 20),
        ];
        let enemy = make_enemy_self(0, 90, 100, 0);
        let action = choose_enemy_action(&enemy, &players, &[], AiStrategy::Defensive);

        match action {
            BattleAction::Attack { target } => assert_eq!(target.index, 1),
            _ => panic!("expected Attack on first alive"),
        }
    }

    // ── Balanced strategy tests ─────────────────────────────────────

    #[test]
    fn balanced_focuses_low_hp_target() {
        let players = vec![
            make_player_view(0, 80, 100, 30),
            make_player_view(1, 10, 100, 20), // 10% HP — below 25%
            make_player_view(2, 90, 100, 40),
        ];
        let enemy = make_enemy_self(0, 100, 100, 0);
        let action = choose_enemy_action(&enemy, &players, &[], AiStrategy::Balanced);

        match action {
            BattleAction::Attack { target } => assert_eq!(target.index, 1),
            _ => panic!("expected Attack on low-HP target"),
        }
    }

    #[test]
    fn balanced_uses_aoe_when_three_plus_players() {
        let players = vec![
            make_player_view(0, 80, 100, 20),
            make_player_view(1, 70, 100, 20),
            make_player_view(2, 90, 100, 20),
        ];
        let enemy = make_enemy_self(0, 100, 100, 5);
        let aoe = make_ability("quake", AbilityCategory::Psynergy, 40, 3, TargetMode::AllEnemies);
        let abilities: Vec<&AbilityDef> = vec![&aoe];

        let action = choose_enemy_action(&enemy, &players, &abilities, AiStrategy::Balanced);
        match action {
            BattleAction::UseAbility { ability_id, targets } => {
                assert_eq!(ability_id.0, "quake");
                assert_eq!(targets.len(), 3); // hits all 3 players
            }
            _ => panic!("expected AoE ability"),
        }
    }

    #[test]
    fn balanced_targets_highest_atk_otherwise() {
        let players = vec![
            make_player_view(0, 80, 100, 15),
            make_player_view(1, 70, 100, 50), // highest ATK
        ];
        let enemy = make_enemy_self(0, 100, 100, 0);
        // No AoE, no low-HP players, fewer than 3 players
        let action = choose_enemy_action(&enemy, &players, &[], AiStrategy::Balanced);

        match action {
            BattleAction::Attack { target } => assert_eq!(target.index, 1),
            _ => panic!("expected Attack on highest ATK"),
        }
    }

    #[test]
    fn balanced_prefers_kill_over_aoe() {
        // Player 1 is at 10% HP — Balanced should prioritize the kill
        let players = vec![
            make_player_view(0, 80, 100, 20),
            make_player_view(1, 10, 100, 20), // low HP
            make_player_view(2, 90, 100, 20),
        ];
        let enemy = make_enemy_self(0, 100, 100, 10);
        let aoe = make_ability("quake", AbilityCategory::Psynergy, 40, 3, TargetMode::AllEnemies);
        let single = make_ability("slash", AbilityCategory::Physical, 60, 2, TargetMode::SingleEnemy);
        let abilities: Vec<&AbilityDef> = vec![&aoe, &single];

        let action = choose_enemy_action(&enemy, &players, &abilities, AiStrategy::Balanced);
        match action {
            BattleAction::UseAbility { ability_id, targets } => {
                assert_eq!(ability_id.0, "slash");
                assert_eq!(targets[0].index, 1); // focus the weak player
            }
            _ => panic!("expected single-target ability on weak player"),
        }
    }
}
