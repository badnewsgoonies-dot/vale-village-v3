# Domain: Advanced Damage Modifiers (S14-S16)

## Scope
`src/domains/damage_mods/`

## Purpose
Defense penetration, splash damage, chain damage. Makes 60+ abilities fully functional.

## Required imports (from crate::shared)
Stats, DamageType, TargetRef, Side

## Functions

### apply_defense_penetration(target_def: u16, ignore_percent: f32) -> u16
- Returns effective defense: target_def * (1.0 - ignore_percent)
- ignore_percent range: 0.0 to 1.0 (0.2 = ignore 20% of DEF)
- Floor at 0

### calculate_splash_targets(primary_target: TargetRef, splash_percent: f32, all_enemies: &[TargetRef]) -> Vec<(TargetRef, f32)>
- Returns list of (target, damage_multiplier) for splash recipients
- Primary target NOT included (already takes full damage)
- All other enemies on same side receive splash_percent of damage

### apply_splash_damage(primary_damage: u16, splash_percent: f32) -> u16
- Returns splash damage amount: (primary_damage as f32 * splash_percent) as u16
- Floor at 1 if splash_percent > 0

### calculate_chain_targets(source: TargetRef, all_targets: &[TargetRef]) -> Vec<TargetRef>
- Chain hits all targets on the opposing side
- Order: left to right (by index)
- Each chain hit deals same damage as primary

### apply_all_modifiers(base_damage: u16, target_def: u16, ignore_def_pct: Option<f32>, damage_type: DamageType, attacker_stats: &Stats, config_def_mult: f32) -> u16
- Combines penetration with damage calculation
- If ignore_def_pct present: reduce effective DEF first, then calculate damage
- Physical: base_power + atk - (effective_def * physical_mult), floor 1
- Psynergy: base_power + mag - (effective_def * psynergy_mult), floor 1

## Quantitative targets
- 5 functions
- 10+ tests

## Tests
- Defense penetration: 0%, 20%, 50%, 100%
- Splash targets: excludes primary, includes all others
- Splash damage: math correct, floor at 1
- Chain targets: returns all enemies
- Combined: penetration + damage formula
- Edge: 0 splash percent = no splash damage
- Edge: single enemy = no splash targets

## Validation
```
cargo check
cargo test -- damage_mods
```
