# Player Trace — Wave 4 (Battle Engine Integration)

## What the player experiences from boot to first meaningful interaction

1. **[Observed]** `cargo run` still prints a string and exits. main.rs does not use battle_engine. No Bevy app. (src/main.rs)

2. **[Observed]** battle_engine::new_battle() creates a Battle with initialized units, mana pool, and stats. Verified by test_new_battle_stats_initialized. (src/domains/battle_engine/mod.rs)

3. **[Observed]** battle_engine::plan_action() validates actions against status restrictions, mana costs, and djinn states. Verified by 6 plan_* tests. (src/domains/battle_engine/mod.rs)

4. **[Observed]** battle_engine::execute_round() runs a full round: auto-attacks deal damage, abilities spend mana, multi-hit generates mana, barriers block, burn ticks, units die. Verified by 8 execute_* tests. (src/domains/battle_engine/mod.rs)

5. **[Observed]** A full 2-round battle scenario works end-to-end in tests (plan → execute → check → plan → execute → check). Verified by test_full_two_round_scenario.

## Harden Assessment

- Reachable end-to-end? **NO** — main.rs doesn't call it. Only reachable via tests.
- Feedback visible? **NO** — BattleEvent log exists but nothing renders it.
- Responsive? **N/A** — no runtime, only unit tests.
- Edge behavior sane? **PARTIAL** — 21 tests cover core paths, but no tests for: splash damage in battle, chain damage in battle, summon execution in battle, ability status effects applied to targets.
- Diagnosable? **YES** — BattleEvent log captures all actions.

## P0 Graduation Debt (updated)

- [x] Battle engine wires all domains together — DONE
- [ ] main.rs runs an actual battle (even text-based CLI) — NOT DONE
- [ ] Full data in RON (241 abilities, not 4) — NOT DONE
- [ ] Splash/chain/summon tested in integration — NOT DONE

## Triggered artifacts

- [Observed] Battle engine correctly wires combat + status + djinn + equipment + damage_mods
- [Assumed] Splash and chain damage work in integration — not tested in battle_engine, only in damage_mods unit tests
- [Assumed] Summon execution works in integration — djinn::execute_summon tested in isolation but battle_engine summon path untested
