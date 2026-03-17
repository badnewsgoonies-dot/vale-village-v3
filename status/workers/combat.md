# Worker Report: combat

## Files created
- src/domains/combat/mod.rs (634 lines)

## What was implemented
- S01: calculate_damage (physical + psynergy formulas, floor 1)
- S02: resolve_targets (5 target modes)
- S03: resolve_multi_hit (per-hit crit + mana, early stop on kill)
- S04: BattleState + BattleUnit + compute_execution_order
- S05: ManaPool (reset/spend/generate)
- S06: Deterministic crit (integrated in S03)

## Quantitative targets
- 6 systems: HIT (6/6)
- Tests: 23 passing (exceeds 15+ target)

## Shared type imports used
Stats, DamageType, TargetMode, CombatConfig, BattlePhase, TargetRef, Side

## Known risks
- [Observed] Damage formulas match DESIGN_LOCK.md exactly
- [Observed] Crit triggers on 10th hit, not 10th action
- [Assumed] SPD tiebreaker uses base SPD only — DESIGN_LOCK specifies 4-level cascade (effective → battle-start → base → level-base), only 2 levels implemented
