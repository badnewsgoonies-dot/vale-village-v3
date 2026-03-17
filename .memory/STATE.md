# Vale Village v3 — Current State

**Phase:** Wave 4 — Integration (battle engine worker dispatched)
**HEAD:** a769111
**Date:** 2026-03-17

## Completed Domains (all structurally correct, all experientially dead)

| Domain | Tests | Status |
|--------|-------|--------|
| data_loader | 9 | Loads sample RON, validates cross-refs |
| combat | 23 | S01-S06, damage/targeting/multi-hit/mana/crit |
| status | 27 | S07-S13, all 6 status types + buffs/barriers/HoT/immunity/cleanse |
| djinn | 26 | State machine, ability oscillation, summons, recovery |
| equipment | 15 | 5-slot loadout, stat bonuses, ability unlocks |
| damage_mods | 14 | Defense pen, splash, chain |
| **Total** | **114** | |

## In Progress

- battle_engine — integration layer wiring all domains into playable battle loop (worker dispatched)

## Harden Assessment (HONEST)

- **Reachable:** NO. main.rs prints a string. No Bevy app. No game loop.
- **Player can fight a battle:** NO. Domains are isolated library code.
- **Full data loaded:** NO. Only 2-3 sample entries per type. Need 241 abilities, 137 enemies, 109 equipment.
- **Gates green:** YES. But this is the "beautiful dead product" the playbook warns about.

## [Assumed] Claims on Critical Path

1. [Assumed] Battle engine worker will successfully wire all 6 domains together
2. [Assumed] Djinn recovery delay = 1 turn — DESIGN_LOCK says "starting the turn after next" (could be 2)
3. [Assumed] SPD tiebreaker only uses 2 levels (effective → base) — DESIGN_LOCK specifies 4-level cascade
4. [Assumed] Chain damage = same damage to all targets (no decay specified)
5. [Assumed] Set bonus detection works but no set bonus EFFECTS are defined
6. [Assumed] Tick order (damage → healing → expiration) not explicitly in DESIGN_LOCK

## P0 Graduation Debt (must resolve before next campaign)

- [ ] Runnable battle (1 round minimum)
- [ ] Full ability data in RON (241 entries, not 4)
- [ ] main.rs starts actual game, not println

## Cadence Debt (process violations from Waves 1-3)

- Scope clamp not run after any worker
- No per-wave player traces until retroactive trace written
- No worker reports until retroactive reports written
- Document/Harden/Graduate phases skipped

## Contract Integrity
- [Observed] .contract.sha256 passes — contract unmodified
