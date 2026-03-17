# Vale Village v3 — Current State

**Phase:** Wave 5 complete — first playable surface exists
**HEAD:** 38f2371 (pre-commit, about to commit Wave 5)
**Date:** 2026-03-17

## Milestone
`cargo run` executes a full battle: load data → create battle → 5 rounds of combat → victory. First player-reachable surface.

## Completed Domains

| Domain | Tests | Surface reachable? |
|--------|-------|--------------------|
| data_loader | 9 | YES — loads sample RON |
| combat | 23 | YES — via battle_engine |
| status | 27 | YES — via battle_engine |
| djinn | 26 | NO — not exercised in demo |
| equipment | 15 | NO — not exercised in demo |
| damage_mods | 14 | NO — not exercised in demo |
| battle_engine | 21 | YES — wired and running |
| cli_runner | 3 | YES — cargo run works |
| **Total** | **138** | |

## Gate Status
- Contract checksum: PASS
- cargo check: PASS
- cargo test: 138 passing
- cargo clippy: PASS
- Connectivity: PASS
- cargo run: PASS (battle completes in 5 rounds)
- Scope clamp: PASS (verified clean)

## [Assumed] Claims on Critical Path
1. [Assumed] Djinn recovery delay = 1 turn (DESIGN_LOCK says "turn after next" = possibly 2)
2. [Assumed] SPD tiebreaker = 2 levels (DESIGN_LOCK specifies 4)
3. [Assumed] Chain damage = no decay
4. [Assumed] Ability usage works in battle_engine (only auto-attacks exercised)
5. [Assumed] Splash/chain/summon work in integration (unit tested only)

## P0 Debt
- [ ] Full data conversion (241 abilities, 137 enemies, 109 equipment → RON)
- [ ] Enemy actions in battle (enemies don't attack back)
- [ ] Ability usage in demo (only auto-attacks used)

## P1 Debt
- [ ] Bevy app with visual rendering
- [ ] Player input (interactive planning phase)
- [ ] Enemy AI decision engine
- [ ] Djinn/equipment exercised in demo
- [ ] Graduation tests for: battle completion, retargeting on death

## Process Status
- Wave cadence followed correctly from Wave 5 onward
- Scope clamp run after worker
- Player trace written
- Worker report written
- Harden assessment done with [Observed] tags
