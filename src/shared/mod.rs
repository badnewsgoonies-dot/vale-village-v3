//! Vale Village v3 — Shared Type Contract
//! FROZEN after checksum. No worker edits this file.
//! Shapes only — tuning values live in data/tuning.
#![allow(dead_code)]
pub mod bounded_types;
pub mod entity_types;
pub mod lifecycle_types;

use crate::shared::bounded_types::{
    BasePower, BaseStat, DjinnTier, EffectDuration, EncounterRate, Gold, GrowthRate, HitCount,
    Hp, ItemCount, Level, ManaCost, MaxBuffStacks, MaxEquippedDjinn, MaxPartySize, StatMod, Xp,
};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

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

// ── New ID Types (Wave 1 contract extension) ────────────────────────

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ItemId(pub String);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TownId(pub u8);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct DungeonId(pub u8);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct MapNodeId(pub u8);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct NpcId(pub u16);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct DialogueTreeId(pub u16);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct DialogueNodeId(pub u16);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct QuestFlagId(pub u16);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ShopId(pub u8);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct RoomId(pub u16);

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
    pub duration: EffectDuration,
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

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct StatBonus {
    pub atk: StatMod,
    pub def: StatMod,
    pub mag: StatMod,
    pub spd: StatMod,
    pub hp: StatMod,
}

impl Default for StatBonus {
    fn default() -> Self {
        StatBonus {
            atk: StatMod::new(0),
            def: StatMod::new(0),
            mag: StatMod::new(0),
            spd: StatMod::new(0),
            hp: StatMod::new(0),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuffEffect {
    pub stat_modifiers: StatBonus,
    pub duration: EffectDuration,
    pub shield_charges: Option<u8>,
    pub grant_immunity: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DebuffEffect {
    pub stat_modifiers: StatBonus,
    pub duration: EffectDuration,
}

// ── HoT (S10) ───────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealOverTime {
    pub amount: u16,
    pub duration: EffectDuration,
}

// ── Immunity (S12) ──────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Immunity {
    pub all: bool,
    pub types: Vec<StatusEffectType>,
    pub duration: EffectDuration,
}

// ── Cleanse (S13) ───────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CleanseType {
    All,
    Negative,
    ByType(Vec<StatusEffectType>),
}

// ── Stats ───────────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Stats {
    pub hp: Hp,
    pub atk: BaseStat,
    pub def: BaseStat,
    pub mag: BaseStat,
    pub spd: BaseStat,
}

impl Default for Stats {
    fn default() -> Self {
        Stats {
            hp: Hp::new(1),
            atk: BaseStat::new(0),
            def: BaseStat::new(0),
            mag: BaseStat::new(0),
            spd: BaseStat::new(0),
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct GrowthRates {
    pub hp: GrowthRate,
    pub atk: GrowthRate,
    pub def: GrowthRate,
    pub mag: GrowthRate,
    pub spd: GrowthRate,
}

impl Default for GrowthRates {
    fn default() -> Self {
        GrowthRates {
            hp: GrowthRate::new(0),
            atk: GrowthRate::new(0),
            def: GrowthRate::new(0),
            mag: GrowthRate::new(0),
            spd: GrowthRate::new(0),
        }
    }
}

// ── Data Definitions (loaded from RON at runtime) ───────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AbilityDef {
    pub id: AbilityId,
    pub name: String,
    pub category: AbilityCategory,
    pub damage_type: Option<DamageType>,
    pub element: Option<Element>,
    pub mana_cost: ManaCost,
    pub base_power: BasePower,
    pub targets: TargetMode,
    pub unlock_level: Level,
    pub hit_count: HitCount,
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
    pub level: Level,
    pub ability_id: AbilityId,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnitDef {
    pub id: UnitId,
    pub name: String,
    pub element: Element,
    pub mana_contribution: ManaCost,
    pub base_stats: Stats,
    pub growth_rates: GrowthRates,
    pub abilities: Vec<AbilityProgression>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnemyDef {
    pub id: EnemyId,
    pub name: String,
    pub element: Element,
    pub level: Level,
    pub stats: Stats,
    pub xp: Xp,
    pub gold: Gold,
    pub abilities: Vec<AbilityId>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EquipmentDef {
    pub id: EquipmentId,
    pub name: String,
    pub slot: EquipmentSlot,
    pub tier: EquipmentTier,
    pub cost: Gold,
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
    pub tier: DjinnTier,
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
    pub xp_reward: Xp,
    pub gold_reward: Gold,
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
    Attack {
        target: TargetRef,
    },
    UseAbility {
        ability_id: AbilityId,
        targets: Vec<TargetRef>,
    },
    ActivateDjinn {
        djinn_index: u8,
    },
    Summon {
        djinn_indices: Vec<u8>,
    },
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
    pub mana_gain_per_hit: ManaCost,
    pub mana_resets_each_round: bool,
    pub max_party_size: MaxPartySize,
    pub max_equipped_djinn: MaxEquippedDjinn,
    pub max_level: Level,
    pub max_buff_stacks: MaxBuffStacks,
    pub djinn_recovery_start_delay: u8,
    pub djinn_recovery_per_turn: u8,
}

// ═══════════════════════════════════════════════════════════════════════
// WAVE 1 CONTRACT EXTENSION — Full game surfaces beyond battle
// ═══════════════════════════════════════════════════════════════════════

// ── Game State Machine ──────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
pub enum GameScreen {
    #[default]
    Title,
    WorldMap,
    Town(TownId),
    Dungeon(DungeonId),
    Battle,
    Menu(MenuScreen),
    Shop(ShopId),
    Dialogue(NpcId),
    SaveLoad,
    GameOver,
    Victory,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum MenuScreen {
    Party,
    Equipment,
    Djinn,
    Items,
    Psynergy,
    Status,
    QuestLog,
}

// ── Screen Transition Events ────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ScreenTransition {
    ToTitle,
    ToWorldMap,
    EnterTown(TownId),
    EnterDungeon(DungeonId),
    StartBattle(EncounterDef),
    OpenMenu(MenuScreen),
    OpenShop(ShopId),
    StartDialogue(NpcId),
    OpenSaveLoad,
    TriggerGameOver,
    TriggerVictory,
    ReturnToPrevious,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct ScreenStack {
    pub stack: Vec<GameScreen>, // max depth 8
}

// ── Direction ───────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Direction {
    Up,
    Down,
    Left,
    Right,
}

// ── World Map ───────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MapNode {
    pub id: MapNodeId,
    pub name: String,
    pub position: (f32, f32),
    pub node_type: MapNodeType,
    pub connections: Vec<MapNodeId>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MapNodeType {
    Town(TownId),
    Dungeon(DungeonId),
    Landmark,
    Hidden,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum NodeUnlockState {
    Locked,
    Visible,
    Unlocked,
    Completed,
}

// ── Town and NPC ────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TownDef {
    pub id: TownId,
    pub name: String,
    pub npcs: Vec<NpcPlacement>,
    pub shops: Vec<ShopId>,
    pub djinn_points: Vec<DjinnDiscoveryPoint>,
    pub exits: Vec<MapNodeId>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NpcPlacement {
    pub npc_id: NpcId,
    pub position: (f32, f32),
    pub facing: Direction,
    pub dialogue_tree: DialogueTreeId,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DjinnDiscoveryPoint {
    pub djinn_id: DjinnId,
    pub position: (f32, f32),
    pub requires_puzzle: bool,
    pub quest_flag: Option<QuestFlagId>,
}

// ── Dialogue System ─────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DialogueTree {
    pub id: DialogueTreeId,
    pub root: DialogueNodeId,
    pub nodes: Vec<DialogueNode>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DialogueNode {
    pub id: DialogueNodeId,
    pub speaker: Option<NpcId>,
    pub text: String,
    pub responses: Vec<DialogueResponse>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DialogueResponse {
    pub text: String,
    pub condition: Option<DialogueCondition>,
    pub next_node: Option<DialogueNodeId>,
    pub side_effects: Vec<DialogueSideEffect>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DialogueCondition {
    HasItem(ItemId),
    HasDjinn(DjinnId),
    QuestAtStage(QuestFlagId, QuestStage),
    GoldAtLeast(Gold),
    PartyContains(UnitId),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DialogueSideEffect {
    GiveItem(ItemId, ItemCount),
    TakeItem(ItemId, ItemCount),
    GiveGold(Gold),
    TakeGold(Gold),
    SetQuestStage(QuestFlagId, QuestStage),
    UnlockMapNode(MapNodeId),
    AddDjinnToParty(DjinnId),
    StartBattle(EncounterDef),
    Heal,
}

// ── Quest / Progression ─────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum QuestStage {
    Unknown,
    Discovered,
    Active,
    InProgress,
    Complete,
    Rewarded,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuestDef {
    pub id: QuestFlagId,
    pub name: String,
    pub description: String,
    pub stages: Vec<QuestStageDef>,
    pub rewards: Vec<DialogueSideEffect>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuestStageDef {
    pub stage: QuestStage,
    pub description: String,
    pub unlock_condition: Option<DialogueCondition>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct QuestState {
    pub flags: HashMap<QuestFlagId, QuestStage>,
}

impl QuestState {
    /// Advance a quest. Can only move forward, never backward.
    pub fn advance(&mut self, quest: QuestFlagId, to: QuestStage) {
        let current = self.flags.get(&quest).copied().unwrap_or(QuestStage::Unknown);
        if to > current {
            self.flags.insert(quest, to);
        }
    }

    pub fn at_least(&self, quest: QuestFlagId, stage: QuestStage) -> bool {
        self.flags.get(&quest).copied().unwrap_or(QuestStage::Unknown) >= stage
    }
}

// ── Shop System ─────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShopDef {
    pub id: ShopId,
    pub name: String,
    pub inventory: Vec<ShopEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShopEntry {
    pub item_id: ItemId,
    pub price: Gold,
    pub stock: ShopStock,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum ShopStock {
    Unlimited,
    Limited(ItemCount),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ShopEvent {
    Buy { shop: ShopId, item: ItemId, count: ItemCount },
    Sell { item: ItemId, count: ItemCount },
}

// ── Dungeon Types ───────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DungeonDef {
    pub id: DungeonId,
    pub name: String,
    pub rooms: Vec<RoomDef>,
    pub entry_room: RoomId,
    pub boss_room: Option<RoomId>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoomDef {
    pub id: RoomId,
    pub room_type: RoomType,
    pub exits: Vec<RoomExit>,
    pub encounters: Vec<EncounterSlot>,
    pub items: Vec<RoomItem>,
    pub puzzles: Vec<PuzzleDef>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RoomType {
    Normal,
    Puzzle,
    MiniBoss,
    Boss,
    Treasure,
    Safe,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoomExit {
    pub direction: Direction,
    pub target_room: RoomId,
    pub requires: Option<DialogueCondition>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncounterSlot {
    pub encounter: EncounterDef,
    pub weight: u8,
    pub max_triggers: Option<u8>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoomItem {
    pub item_id: ItemId,
    pub position: (f32, f32),
    pub visible: bool,
    pub quest_flag: Option<QuestFlagId>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PuzzleDef {
    pub puzzle_type: PuzzleType,
    pub reward: Option<DialogueSideEffect>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum PuzzleType {
    PushBlock,
    ElementPillar(Element),
    DjinnPuzzle(DjinnId),
    SwitchSequence,
    IceSlide,
}

// ── Encounter Definitions (extending battle types) ──────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncounterTable {
    pub region_id: u8,
    pub base_rate: EncounterRate,
    pub entries: Vec<EncounterSlot>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BossEncounter {
    pub encounter: EncounterDef,
    pub pre_dialogue: Option<DialogueTreeId>,
    pub post_dialogue: Option<DialogueTreeId>,
    pub quest_advance: Option<(QuestFlagId, QuestStage)>,
    pub unlock_on_defeat: Vec<MapNodeId>,
}

// ── Player Overworld State ──────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OverworldState {
    pub current_location: GameScreen,
    pub current_room: Option<RoomId>,
    pub player_position: (f32, f32),
    pub player_facing: Direction,
    pub steps_since_encounter: u16,
    pub visited_rooms: HashSet<RoomId>,
    pub collected_items: HashSet<(DungeonId, RoomId, usize)>,
}

// ── Save System Extension ───────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SaveDataExtension {
    pub quest_state: HashMap<QuestFlagId, QuestStage>,
    pub map_unlock_state: HashMap<MapNodeId, NodeUnlockState>,
    pub overworld: OverworldSaveData,
    pub shop_stock: HashMap<ShopId, Vec<(ItemId, Option<ItemCount>)>>,
    pub visited_rooms: Vec<RoomId>,
    pub collected_items: Vec<(DungeonId, RoomId, usize)>,
    pub play_time_seconds: u64,
    pub save_timestamp: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OverworldSaveData {
    pub location: GameScreen,
    pub room: Option<RoomId>,
    pub position: (f32, f32),
    pub facing: Direction,
}
