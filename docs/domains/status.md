# Domain: Status Effects & Buffs (S07-S13)

## Scope
`src/domains/status/`

## Purpose
All status effects, buffs, debuffs, barriers, HoT, damage reduction, immunity, and cleansing. This makes 126+ abilities functional.

## Required imports (from crate::shared)
StatusEffectType, StatusEffect, BuffEffect, DebuffEffect, StatBonus, HealOverTime, Immunity, CleanseType, TargetRef, Stats

## Systems to implement

### S07 — Status Effects (6 types, all deterministic)
```rust
pub struct ActiveStatus {
    pub effect_type: StatusEffectType,
    pub remaining_turns: u8,
    pub burn_percent: Option<f32>,
    pub poison_percent: Option<f32>,
    pub freeze_threshold: Option<u16>,
    pub freeze_damage_taken: u16,
}
```
- Stun: cannot act, duration turns
- Null: can only auto-attack, duration turns
- Incapacitate: can only use abilities, duration turns
- Burn: % of MAX HP per turn (e.g. 0.10 = 10%)
- Poison: % of MISSING HP per turn (e.g. 0.05 = 5%)
- Freeze: skip turn, breaks when cumulative damage >= threshold

`tick_statuses(unit_max_hp, unit_current_hp, statuses) -> StatusTickResult`
- Returns: damage dealt (burn/poison), statuses expired, freeze broken

### S08 — Buff/Debuff Stacking
```rust
pub struct ActiveBuff {
    pub stat_modifiers: StatBonus,
    pub remaining_turns: u8,
}
```
- Max 3 stacks per stat per unit
- Duration-based
- `apply_buff()`, `tick_buffs()`, `compute_stat_modifiers() -> StatBonus`

### S09 — Barrier System (replaces v2 damage reduction)
```rust
pub struct ActiveBarrier {
    pub charges: u8,
    pub remaining_turns: u8,
}
```
- Each charge absorbs one full hit (regardless of damage amount)
- Multi-hit attacks consume one charge per hit
- Status damage (burn/poison) does NOT consume barrier charges
- Barriers can stack from multiple sources
- `apply_barrier()`, `consume_barrier() -> bool`, `tick_barriers()`

### S10 — Heal over Time
```rust
pub struct ActiveHoT {
    pub amount: u16,
    pub remaining_turns: u8,
}
```
- Fixed HP per turn for N turns
- `apply_hot()`, `tick_hots() -> u16` (returns total healing)

### S12 — Immunity
```rust
pub struct ActiveImmunity {
    pub all: bool,
    pub types: Vec<StatusEffectType>,
    pub remaining_turns: u8,
}
```
- Blocks status application (checked before apply)
- `is_immune(immunity, status_type) -> bool`

### S13 — Status Cleansing
- `cleanse(statuses, cleanse_type) -> Vec<StatusEffectType>` (returns removed types)
- CleanseType::All, Negative, ByType

### UnitStatusState (aggregate per-unit)
```rust
pub struct UnitStatusState {
    pub statuses: Vec<ActiveStatus>,
    pub buffs: Vec<ActiveBuff>,
    pub debuffs: Vec<ActiveBuff>,
    pub barriers: Vec<ActiveBarrier>,
    pub hots: Vec<ActiveHoT>,
    pub immunity: Option<ActiveImmunity>,
}
```

### Tick order (end of round)
1. Duration decay on all effects
2. DoT damage (burn, poison)
3. HoT healing
4. Expiration removal

## Quantitative targets
- 7 systems (S07-S13, S09 replaces S11)
- 6 public structs
- 12+ public functions
- 20+ unit tests

## Does NOT handle
- Combat integration (applying effects during battle execution)
- UI display of status icons
- AI priority weighting of statuses

## Validation
```
cargo check
cargo test -- status
```
