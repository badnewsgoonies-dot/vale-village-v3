# Vale Village v3 — Build Manifest

## Current Phase: Wave 9 closeout — team-wide djinn correction

## Architecture
- Language: Rust
- Framework: Bevy
- Data format: RON
- Build method: AI-orchestrated with a frozen shared contract and wave gates

## Domain List

| Domain | Path | Owner | Status |
|--------|------|-------|--------|
| shared (contract) | src/shared/ | orchestrator | frozen |
| ai | src/domains/ai/ | worker | complete |
| battle_engine | src/domains/battle_engine/ | worker | complete |
| cli_runner | src/domains/cli_runner/ | worker | complete |
| combat | src/domains/combat/ | worker | complete |
| damage_mods | src/domains/damage_mods/ | worker | complete |
| data_loader | src/domains/data_loader/ | worker | complete |
| djinn | src/domains/djinn/ | worker | complete |
| equipment | src/domains/equipment/ | worker | complete |
| progression | src/domains/progression/ | worker | complete |
| save | src/domains/save/ | worker | complete |
| status | src/domains/status/ | worker | complete |
| ui | src/domains/ui/ | worker | in progress |

## Key Constants
- MAX_PARTY_SIZE = 4
- MAX_EQUIPPED_DJINN = 3
- MAX_LEVEL = 20
- CRIT_THRESHOLD = 10
- CRIT_MULTIPLIER = 2.0
- MANA_GAIN_PER_HIT = 1
- MANA_RESETS_EACH_ROUND = true
- Physical damage = basePower + ATK - (DEF × 0.5), floor 1
- Psynergy damage = basePower + MAG - (DEF × 0.3), floor 1

## Wave Plan
- Wave 1: Bevy bootstrap and placeholder battle scene — complete
- Wave 2: Data-driven battle scene population — complete
- Wave 3: HUD with HP, mana, and crit display — complete
- Wave 4: Planning panel and action selection UI — complete
- Wave 5: Animation/event playback systems — complete
- Wave 6: Root-state migration, script alignment, and lint hardening — complete
- Wave 7: GUI djinn menu, summon planning, and event playback wiring — complete
- Wave 8: battle-scene djinn interaction plus HUD/execution sync — complete
- Wave 9: team-wide djinn pool semantics in battle model and persistence schema — complete

## Key Decisions
- Root `STATE.md` is now the canonical state artifact; `.memory/STATE.md` is mirrored only for compatibility with older tooling — [Observed]
- Shared gameplay shapes stay frozen in `src/shared/mod.rs`; tuning values remain in data/config — [Observed]
- Mechanical scope clamping preserves `status/workers/` by default — [Observed]
- Combat remains deterministic with no randomness or element-based damage modifiers — [Observed]
- Djinn are team-wide equips with exactly 3 shared team slots, not per-unit equipment — [Observed]
- Any acting unit may activate or summon from the shared team djinn pool; summons still execute before speed order — [Observed]

## Blockers
- Manual click-through harden pass for djinn activation, summon timing, and recovery readability still blocks an interactive shipping claim — owner: orchestrator
- Pre-battle composition surface blocks the next UX milestone — owner: worker
- Battle setup and save/load still need to consume the team-wide djinn model end-to-end from a true pre-battle flow — owner: worker
- Top-center battle labels still duplicate bottom-HUD information and may need a cleanup pass — owner: worker
- `verify-state-claims.sh` is missing, so claim verification is not fully automated yet — owner: orchestrator
