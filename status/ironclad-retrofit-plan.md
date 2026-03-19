# Ironclad Retrofit Plan: Shared Types

Scope: analysis of every `struct` and `enum` in `src/shared/mod.rs` only.

Legend:
- `game_value`: fields that would benefit from bounded-value wrappers or validation.
- `lifecycle`: enums that model phases, transitions, or state machines.
- `game_entity`: structs with required fields that are easy to forget or partially initialize in worker-authored data/tests.

## Enums

| Type | lifecycle | Notes |
|---|---|---|
| `Element` | no | Taxonomy only. Stable category enum. |
| `DamageType` | no | Taxonomy only. |
| `TargetMode` | no | Targeting taxonomy, not a lifecycle. |
| `AbilityCategory` | no | Classification enum. |
| `EquipmentSlot` | no | Classification enum. |
| `EquipmentTier` | no | Ordered content tiering, but not a runtime state machine. |
| `Difficulty` | no | Encounter classification, not a state machine. |
| `StatusEffectType` | no | Effect taxonomy. |
| `DjinnState` | yes | Explicit 2-state machine: `Good -> Recovery -> Good`. Highest-priority lifecycle enum. |
| `DjinnCompatibility` | no | Relationship classification, not a lifecycle. |
| `CleanseType` | no | Behavior selection enum. |
| `BattlePhase` | yes | Explicit battle flow lifecycle: `Planning -> Execution -> RoundEnd -> Planning`. |
| `Side` | no | Team classification. |
| `BattleAction` | no | Command union; action selection shape, not a runtime lifecycle by itself. |

## Structs

| Type | game_value candidates | game_entity | Notes |
|---|---|---|---|
| `AbilityId` | none | yes | Required wrapped string identity. Easy to omit when hand-authoring fixtures/data. |
| `UnitId` | none | yes | Required wrapped string identity. |
| `EnemyId` | none | yes | Required wrapped string identity. |
| `EquipmentId` | none | yes | Required wrapped string identity. |
| `DjinnId` | none | yes | Required wrapped string identity. |
| `EncounterId` | none | yes | Required wrapped string identity. |
| `SetId` | none | yes | Required wrapped string identity. |
| `StatusEffect` | `duration`, `burn_percent`, `poison_percent`, `freeze_threshold` | yes | Required `effect_type` and `duration`; optional payload fields should be mutually constrained by `effect_type`. Strong candidate for bounded percent/turn wrappers. |
| `DjinnAbilitySet` | none | yes | Both ability vectors are required and easy to leave incomplete during content entry. |
| `DjinnAbilityPairs` | none | yes | All three compatibility buckets are required. High omission risk in manual content authoring. |
| `StatBonus` | `atk`, `def`, `mag`, `spd`, `hp` | no | Numeric deltas should likely be capped by design ranges even though negatives are valid. |
| `BuffEffect` | `duration`, `shield_charges` | yes | Required `stat_modifiers` and `duration`; optional charges should be bounded. `grant_immunity` is easy to default incorrectly if builders are loose. |
| `DebuffEffect` | `duration` | yes | Required `stat_modifiers` and `duration`. |
| `HealOverTime` | `amount`, `duration` | yes | Both fields are required and gameplay-sensitive. |
| `Immunity` | `duration` | yes | Required `duration`; `all` plus `types` likely need invariant checks to avoid contradictory states. |
| `Stats` | `hp`, `atk`, `def`, `mag`, `spd` | yes | Core stat block. Every field matters and partial/default initialization is risky. |
| `GrowthRates` | `hp`, `atk`, `def`, `mag`, `spd` | yes | Same risk profile as `Stats`; likely wants bounded progression ranges. |
| `AbilityDef` | `mana_cost`, `base_power`, `unlock_level`, `hit_count`, `shield_charges`, `shield_duration`, `ignore_defense_percent`, `splash_damage_percent`, `revive_hp_percent` | yes | Highest-risk content struct. Required identity/shape fields plus many optional numeric knobs with hidden invariants tied to `category`, `damage_type`, and effect payloads. |
| `AbilityProgression` | `level` | yes | Both fields required; missing `ability_id` or invalid level breaks unit progression. |
| `UnitDef` | `mana_contribution` | yes | Required `id`, `name`, `element`, `base_stats`, `growth_rates`, `abilities`. Full entity definition with multiple required composites. |
| `EnemyDef` | `level`, `xp`, `gold` | yes | Required combat/content entity. `stats` and `abilities` are easy omissions in test fixtures. |
| `EquipmentDef` | `cost`, `mana_bonus`, `hit_count_bonus` | yes | Required identity, slot, tier, allowed elements, and stat bonus. Good candidate for caps on economic/combat modifiers. |
| `SummonEffect` | `damage`, `heal` | yes | Optional payload members create shape ambiguity; bounded damage/heal values would help. |
| `DjinnDef` | `tier` | yes | Required entity with nested required `stat_bonus` and `ability_pairs`; likely wants bounded tier/slot semantics. |
| `EncounterEnemy` | `count` | yes | Required enemy reference plus bounded count. |
| `EncounterDef` | `xp_reward`, `gold_reward` | yes | Required identity, difficulty, enemy list, and reward vectors. High fixture/content omission risk. |
| `TargetRef` | `index` | yes | Required `side` and index; index should be bounded to valid party/enemy slots. |
| `DamageDealt` | `amount` | yes | Event payload with required source/target/type fields. |
| `HealingDone` | `amount` | yes | Event payload with required source/target fields. |
| `StatusApplied` | none | yes | Required source/target/effect payload. |
| `BarrierConsumed` | `charges_remaining` | yes | Required unit and post-consumption count; count should never go negative/out of range. |
| `CritTriggered` | `hit_number` | yes | Required event fields; hit count should stay within crit-threshold semantics. |
| `ManaPoolChanged` | `old_value`, `new_value` | yes | Strong candidate for bounded mana wrapper aligned with max mana rules. |
| `DjinnStateChanged` | `recovery_turns` | yes | Required IDs, refs, and states; optional turn count should be bounded when present. |
| `UnitDefeated` | none | yes | Minimal but required event wrapper. |
| `CombatConfig` | `physical_def_multiplier`, `psynergy_def_multiplier`, `crit_threshold`, `crit_multiplier`, `mana_gain_per_hit`, `max_party_size`, `max_equipped_djinn`, `max_level`, `max_buff_stacks`, `djinn_recovery_start_delay`, `djinn_recovery_per_turn` | yes | Central tuning struct. Nearly every numeric field benefits from explicit bounds; misconfiguration risk is high. |

## Priority Retrofit Targets

1. `DjinnState` and `BattlePhase` are the only explicit lifecycle/state-machine enums in the contract.
2. `AbilityDef`, `CombatConfig`, `StatusEffect`, `Stats`, `GrowthRates`, `UnitDef`, `EnemyDef`, `EquipmentDef`, `DjinnDef`, and `EncounterDef` are the highest-value `game_entity`/validation targets because they combine many required fields with gameplay-impacting numbers.
3. Percent-like fields should be wrapped or validated first: `burn_percent`, `poison_percent`, `ignore_defense_percent`, `splash_damage_percent`, and `revive_hp_percent`.
4. Count/index fields are the next strongest `game_value` candidates: `duration`, `hit_count`, `shield_charges`, `tier`, `count`, `index`, `mana_cost`, and the various `CombatConfig` caps.
5. Cross-field invariants matter as much as per-field bounds. The clearest examples are `StatusEffect` payloads keyed by `effect_type`, `AbilityDef` option sets keyed by category/effect behavior, and `Immunity { all, types }`.
