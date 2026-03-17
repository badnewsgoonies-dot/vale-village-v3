# Worker Report: status

## Files created
- src/domains/status/mod.rs (944 lines)

## What was implemented
- S07: 6 status types with tick logic
- S08: Buff/debuff stacking (max 3)
- S09: Barrier system (per-instance charge blockers)
- S10: HoT
- S12: Immunity (all + type-specific)
- S13: Cleanse (all/negative/by-type)
- Action restriction helpers (can_act, can_auto_attack, can_use_ability)

## Quantitative targets
- 7 systems: HIT
- Tests: 27 passing (exceeds 20+ target)

## Known risks
- [Observed] Burn = % maxHP, Poison = % missing HP — matches DESIGN_LOCK
- [Observed] Barriers don't block status damage — matches DESIGN_LOCK
- [Assumed] Tick order (damage → healing → expiration) is correct but not explicitly specified in DESIGN_LOCK
