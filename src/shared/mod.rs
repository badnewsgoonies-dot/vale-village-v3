//! Vale Village v3 — Shared Type Contract
//! FROZEN after checksum. No worker edits this file.
//! Shapes only — tuning values live in data/tuning.
#![allow(dead_code)]

use serde::{Deserialize, Serialize};

// ── ID Types (string-branded for stable serialization) ──────────────

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct AbilityId(pub String);

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct UnitId(pub String);

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct EnemyId(pub String);

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct EquipmentId(pub String);

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct DjinnId(pub String);

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct EncounterId(pub String);

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct SetId(pub String);

// ── Core Enums ──────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Element {
    Venus,
    Mars,
    Mercury,
    Jupiter,
}

impl Element {
    pub fn counter(&self) -> Element {
        match self {
            Element::Venus => Element::Jupiter,
            Element::Jupiter => Element::Venus,
            Element::Mars => Element::Mercury,
            Element::Mercury => Element::Mars,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum DamageType {
    Physical,
    Psynergy,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TargetMode {
    SingleEnemy,
    AllEnemies,
    SingleAlly,
    AllAllies,
    SelfOnly,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AbilityCategory {
    Physical,
    Psynergy,
    Healing,
    Buff,
    Debuff,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum EquipmentSlot {
    Weapon,
    Helm,
    Armor,
    Boots,
    Accessory,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum EquipmentTier {
    Basic,
    Bronze,
    Iron,
    Steel,
    Silver,
    Mythril,
    Legendary,
    Artifact,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Difficulty {
    Easy,
    Medium,
    Hard,
    Boss,
}

// ── Status Effects (S07) — 6 types, fully deterministic ─────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum StatusEffectType {
    Stun,
    Null,
    Incapacitate,
    Burn,
    Poison,
    Freeze,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatusEffect {
    pub effect_type: StatusEffectType,
    pub duration: u8,
    /// Burn: % of max HP per turn (e.g. 0.10 = 10%)
    pub burn_percent: Option<f32>,
    /// Poison: % of MISSING HP per turn
    pub poison_percent: Option<f32>,
    /// Freeze: cumulative damage threshold to break
    pub freeze_threshold: Option<u16>,
}

// ── Djinn System ────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum DjinnState {
    Good,
    Recovery,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum DjinnCompatibility {
    Same,
    Counter,
    Neutral,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DjinnAbilitySet {
    pub good_abilities: Vec<AbilityId>,
    pub recovery_abilities: Vec<AbilityId>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DjinnAbilityPairs {
    pub same: DjinnAbilitySet,
    pub counter: DjinnAbilitySet,
    pub neutral: DjinnAbilitySet,
}

// ── Buff / Debuff (S08) ─────────────────────────────────────────────

#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize)]
pub struct StatBonus {
    pub atk: i16,
    pub def: i16,
    pub mag: i16,
    pub spd: i16,
    pub hp: i16,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuffEffect {
    pub stat_modifiers: StatBonus,
    pub duration: u8,
    pub shield_charges: Option<u8>,
    pub grant_immunity: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DebuffEffect {
    pub stat_modifiers: StatBonus,
    pub duration: u8,
}

// ── HoT (S10) ───────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealOverTime {
    pub amount: u16,
    pub duration: u8,
}

// ── Immunity (S12) ──────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Immunity {
    pub all: bool,
    pub types: Vec<StatusEffectType>,
    pub duration: u8,
}

// ── Cleanse (S13) ───────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CleanseType {
    All,
    Negative,
    ByType(Vec<StatusEffectType>),
}

// ── Stats ───────────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize)]
pub struct Stats {
    pub hp: u16,
    pub atk: u16,
    pub def: u16,
    pub mag: u16,
    pub spd: u16,
}

#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize)]
pub struct GrowthRates {
    pub hp: u16,
    pub atk: u16,
    pub def: u16,
    pub mag: u16,
    pub spd: u16,
}

// ── Data Definitions (loaded from RON at runtime) ───────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AbilityDef {
    pub id: AbilityId,
    pub name: String,
    pub category: AbilityCategory,
    pub damage_type: Option<DamageType>,
    pub element: Option<Element>,
    pub mana_cost: u8,
    pub base_power: u16,
    pub targets: TargetMode,
    pub unlock_level: u8,
    pub hit_count: u8,
    // S07: status
    pub status_effect: Option<StatusEffect>,
    // S08: buff/debuff
    pub buff_effect: Option<BuffEffect>,
    pub debuff_effect: Option<DebuffEffect>,
    // S09: barriers
    pub shield_charges: Option<u8>,
    pub shield_duration: Option<u8>,
    // S10: HoT
    pub heal_over_time: Option<HealOverTime>,
    // S12: immunity
    pub grant_immunity: Option<Immunity>,
    // S13: cleanse
    pub cleanse: Option<CleanseType>,
    // S14: defense penetration
    pub ignore_defense_percent: Option<f32>,
    // S15: splash damage
    pub splash_damage_percent: Option<f32>,
    // S16: chain damage
    pub chain_damage: bool,
    // S17: revive
    pub revive: bool,
    pub revive_hp_percent: Option<f32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AbilityProgression {
    pub level: u8,
    pub ability_id: AbilityId,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnitDef {
    pub id: UnitId,
    pub name: String,
    pub element: Element,
    pub mana_contribution: u8,
    pub base_stats: Stats,
    pub growth_rates: GrowthRates,
    pub abilities: Vec<AbilityProgression>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnemyDef {
    pub id: EnemyId,
    pub name: String,
    pub element: Element,
    pub level: u8,
    pub stats: Stats,
    pub xp: u32,
    pub gold: u32,
    pub abilities: Vec<AbilityId>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EquipmentDef {
    pub id: EquipmentId,
    pub name: String,
    pub slot: EquipmentSlot,
    pub tier: EquipmentTier,
    pub cost: u32,
    pub allowed_elements: Vec<Element>,
    pub stat_bonus: StatBonus,
    pub unlocks_ability: Option<AbilityId>,
    pub set_id: Option<SetId>,
    pub always_first_turn: bool,
    pub mana_bonus: Option<u8>,
    pub hit_count_bonus: Option<u8>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SummonEffect {
    pub damage: u16,
    pub buff: Option<BuffEffect>,
    pub status: Option<StatusEffect>,
    pub heal: Option<u16>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DjinnDef {
    pub id: DjinnId,
    pub name: String,
    pub element: Element,
    pub tier: u8,
    pub stat_bonus: StatBonus,
    pub summon_effect: Option<SummonEffect>,
    pub ability_pairs: DjinnAbilityPairs,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncounterEnemy {
    pub enemy_id: EnemyId,
    pub count: u8,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncounterDef {
    pub id: EncounterId,
    pub name: String,
    pub difficulty: Difficulty,
    pub enemies: Vec<EncounterEnemy>,
    pub xp_reward: u32,
    pub gold_reward: u32,
    pub recruit: Option<UnitId>,
    pub djinn_reward: Option<DjinnId>,
    pub equipment_rewards: Vec<EquipmentId>,
}

// ── Battle Flow ─────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum BattlePhase {
    Planning,
    Execution,
    RoundEnd,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TargetRef {
    pub side: Side,
    pub index: u8,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Side {
    Player,
    Enemy,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BattleAction {
    Attack { target: TargetRef },
    UseAbility { ability_id: AbilityId, targets: Vec<TargetRef> },
    ActivateDjinn { djinn_index: u8 },
    Summon { djinn_indices: Vec<u8> },
}

// ── Events (cross-domain messages) ──────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DamageDealt {
    pub source: TargetRef,
    pub target: TargetRef,
    pub amount: u16,
    pub damage_type: DamageType,
    pub is_crit: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealingDone {
    pub source: TargetRef,
    pub target: TargetRef,
    pub amount: u16,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatusApplied {
    pub source: TargetRef,
    pub target: TargetRef,
    pub effect: StatusEffect,
}

/// Reserved for future use: battle_engine currently emits
/// BattleEvent::BarrierBlocked(TargetRef) with a simpler shape.
/// This richer struct will replace it once barrier-charge tracking
/// is surfaced through the integration layer.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BarrierConsumed {
    pub unit: TargetRef,
    pub charges_remaining: u8,
}

/// Reserved for future use: battle_engine currently emits
/// BattleEvent::CritTriggered(TargetRef, u8) with a flat tuple.
/// This richer struct will replace it once crit events carry
/// semantic field names through the integration layer.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CritTriggered {
    pub unit: TargetRef,
    pub hit_number: u8,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ManaPoolChanged {
    pub old_value: u8,
    pub new_value: u8,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DjinnStateChanged {
    pub djinn_id: DjinnId,
    pub unit: TargetRef,
    pub old_state: DjinnState,
    pub new_state: DjinnState,
    pub recovery_turns: Option<u8>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnitDefeated {
    pub unit: TargetRef,
}

// ── Combat Config Shape (values live in data/tuning) ────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CombatConfig {
    pub physical_def_multiplier: f32,
    pub psynergy_def_multiplier: f32,
    pub crit_threshold: u8,
    pub crit_multiplier: f32,
    pub mana_gain_per_hit: u8,
    pub mana_resets_each_round: bool,
    pub max_party_size: u8,
    pub max_equipped_djinn: u8,
    pub max_level: u8,
    pub max_buff_stacks: u8,
    pub djinn_recovery_start_delay: u8,
    pub djinn_recovery_per_turn: u8,
}
