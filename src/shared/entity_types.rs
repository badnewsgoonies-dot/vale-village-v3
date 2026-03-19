use ironclad::game_entity;

#[game_entity(requires = [id, name, category, targets, mana_cost, base_power, hit_count, unlock_level])]
pub struct AbilityDefBuilder {
    pub id: super::AbilityId,
    pub name: String,
    pub category: super::AbilityCategory,
    pub damage_type: Option<super::DamageType>,
    pub element: Option<super::Element>,
    pub mana_cost: u8,
    pub base_power: u16,
    pub targets: super::TargetMode,
    pub unlock_level: super::bounded_types::Level,
    pub hit_count: u8,
}

#[game_entity(requires = [id, name, element, mana_contribution, base_stats, growth_rates])]
pub struct UnitDefBuilder {
    pub id: super::UnitId,
    pub name: String,
    pub element: super::Element,
    pub mana_contribution: u8,
    pub base_stats: super::Stats,
    pub growth_rates: super::GrowthRates,
    pub abilities: Vec<super::AbilityProgression>,
}

#[game_entity(requires = [id, name, element, level, stats])]
pub struct EnemyDefBuilder {
    pub id: super::EnemyId,
    pub name: String,
    pub element: super::Element,
    pub level: super::bounded_types::Level,
    pub stats: super::Stats,
    pub xp: super::bounded_types::Xp,
    pub gold: super::bounded_types::Gold,
    pub abilities: Vec<super::AbilityId>,
}

#[game_entity(requires = [id, name, slot, tier, cost, stat_bonus])]
pub struct EquipmentDefBuilder {
    pub id: super::EquipmentId,
    pub name: String,
    pub slot: super::EquipmentSlot,
    pub tier: super::EquipmentTier,
    pub cost: super::bounded_types::Gold,
    pub allowed_elements: Vec<super::Element>,
    pub stat_bonus: super::StatBonus,
}

#[game_entity(requires = [id, name, element, tier, stat_bonus, ability_pairs])]
pub struct DjinnDefBuilder {
    pub id: super::DjinnId,
    pub name: String,
    pub element: super::Element,
    pub tier: super::bounded_types::DjinnTier,
    pub stat_bonus: super::StatBonus,
    pub summon_effect: Option<super::SummonEffect>,
    pub ability_pairs: super::DjinnAbilityPairs,
}

#[game_entity(requires = [id, name, difficulty, enemies])]
pub struct EncounterDefBuilder {
    pub id: super::EncounterId,
    pub name: String,
    pub difficulty: super::Difficulty,
    pub enemies: Vec<super::EncounterEnemy>,
    pub xp_reward: super::bounded_types::Xp,
    pub gold_reward: super::bounded_types::Gold,
}
