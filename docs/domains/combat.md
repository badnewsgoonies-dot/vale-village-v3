# Domain: Combat Core (S01-S06)

## Scope
`src/domains/combat/`

## Purpose
Implement the 6 core combat systems required for any battle to function. This is the minimum playable battle engine.

## Required imports (from crate::shared)
Stats, StatBonus, DamageType, TargetMode, AbilityDef, AbilityCategory, CombatConfig, BattlePhase, BattleAction, TargetRef, Side, DamageDealt, CritTriggered, ManaPoolChanged

## Systems to implement

### S01 — Damage Calculation
```rust
pub fn calculate_damage(
    base_power: u16,
    damage_type: DamageType,
    attacker_stats: &Stats,
    defender_stats: &Stats,
    config: &CombatConfig,
) -> u16
```
- Physical: `base_power + atk - (def * config.physical_def_multiplier)`, floor 1
- Psynergy: `base_power + mag - (def * config.psynergy_def_multiplier)`, floor 1
- No element modifiers. No randomness.

### S02 — Targeting
```rust
pub fn resolve_targets(
    mode: TargetMode,
    source: TargetRef,
    chosen_target: Option<TargetRef>,
    party_size: u8,
    enemy_count: u8,
) -> Vec<TargetRef>
```
- SingleEnemy/SingleAlly: return the chosen target
- AllEnemies/AllAllies: return all on that side
- SelfOnly: return source

### S03 — Multi-Hit
```rust
pub struct HitResult {
    pub damage: u16,
    pub is_crit: bool,
    pub target_killed: bool,
    pub mana_generated: u8,
    pub crit_counter_after: u8,
}

pub fn resolve_multi_hit(
    hit_count: u8,
    base_damage_per_hit: u16,
    target_hp: u16,
    crit_counter: u8,
    is_auto_attack: bool,
    config: &CombatConfig,
) -> Vec<HitResult>
```
- Each hit: deal damage, check if target dies (remaining hits stop)
- Auto-attacks only: each hit advances crit counter +1, generates +1 mana
- Abilities: no crit advance, no mana generation
- When crit_counter reaches config.crit_threshold: that hit crits (damage * config.crit_multiplier), counter resets to 0

### S04 — Battle State Machine
```rust
pub struct BattleState {
    pub phase: BattlePhase,
    pub round: u32,
    pub player_units: Vec<BattleUnit>,
    pub enemies: Vec<BattleUnit>,
    pub planned_actions: Vec<(TargetRef, BattleAction)>,
    pub mana_pool: ManaPool,
    pub execution_order: Vec<TargetRef>,
}

pub struct BattleUnit {
    pub id: String,
    pub stats: Stats,
    pub current_hp: u16,
    pub is_alive: bool,
    pub crit_counter: u8,
    pub equipment_speed_bonus: i16,
}
```
- Planning → Execution → RoundEnd → Planning
- In Execution: sort all actors by effective SPD (descending), execute in order
- Summons execute first (before SPD ordering)
- Speed tiebreaker: effective SPD → battle-start SPD → base SPD

### S05 — Team Mana Pool
```rust
pub struct ManaPool {
    pub max_mana: u8,
    pub current_mana: u8,
    pub projected_mana: u8,
}
```
- max_mana = sum of living units' mana_contribution + equipment mana bonuses
- Resets to max at start of each round
- Auto-attack hits generate +1 mana mid-execution (available to later units in SPD order)
- projected_mana tracks planned auto-attack generation during Planning phase

### S06 — Deterministic Crit
- Per-unit counter, stored in BattleUnit.crit_counter
- Increments per auto-attack HIT (not per action)
- At config.crit_threshold (10): that hit deals damage * config.crit_multiplier, counter resets
- Abilities do NOT advance crit
- Counter resets to 0 at battle start

## Quantitative targets
- 6 systems implemented (S01-S06)
- 4 public functions minimum (calculate_damage, resolve_targets, resolve_multi_hit, + speed ordering)
- 3 public structs (BattleState, BattleUnit, ManaPool)
- 15+ unit tests covering:
  - Physical damage formula (3 cases)
  - Psynergy damage formula (2 cases)
  - Damage floor at 1
  - All 5 target modes
  - Multi-hit with target dying mid-combo
  - Multi-hit crit triggering on 10th hit
  - Auto-attack mana generation
  - Ability NOT generating mana or crit
  - Mana pool reset per round
  - SPD ordering with tiebreakers

## Does NOT handle
- Status effects (S07-S13) — separate domain
- Defense penetration, splash, chain (S14-S16) — separate domain
- Djinn state machine — separate domain
- UI rendering
- Enemy AI

## Validation
```
cargo check
cargo test -- combat
```
