# VALE VILLAGE — Systems Foundation
## Every system the engine must support, derived from scanning all 241 abilities + 109 equipment + 105 enemies

---

## THE ANSWER TO YOUR QUESTION

**Yes, migrating the v2 data is feasible AND it's the right move.** Here's why:

Scanning every ability, item, and enemy in v2 reveals exactly **19 mechanical systems** that the engine must implement. These 19 systems are the complete foundation. Every piece of content — current and future — is a combination of these systems. Build them generically, and you can add unlimited content through data files.

The v2 data isn't something to recreate from scratch. It's 241 abilities + 109 equipment + 105 enemies that have already been balanced against each other across 183 commits. That's months of design work. The migration is: define schemas in Rust, run a conversion script on the TypeScript definitions, output RON files. One afternoon of scripting.

---

## THE 19 SYSTEMS (complete, derived from data)

Every property that appears on any ability, enemy, or item maps to one of these systems. Build all 19, and you can express any content the game needs — now or in the future.

### TIER 1: Core Combat (must exist for any battle to work)

**S01 — Damage Calculation**
- Physical: `basePower + ATK - (DEF × 0.5)`, floor 1
- Psynergy: `basePower + MAG - (DEF × 0.3)`, floor 1
- No randomness, no element modifiers
- Properties consumed: `basePower`, `type`, `atk`, `def`, `mag`

**S02 — Targeting**
- 5 target modes: `single-enemy`, `all-enemies`, `single-ally`, `all-allies`, `self`
- Properties consumed: `targets`

**S03 — Multi-Hit**
- Per-hit damage resolution (target can die mid-combo, remaining hits stop)
- Per-hit crit counter advancement (auto-attacks only)
- Per-hit mana generation (auto-attacks only)
- Properties consumed: `hitCount` (1-10, default 1)
- Used by: 26 abilities currently, unlimited future

**S04 — Queue Battle State Machine**
- Planning → Execution → Round End
- All actions committed before execution
- SPD-ordered execution, interleaved with enemies
- Summons execute first (supersede SPD)
- Properties consumed: `spd` (from units/equipment/djinn)

**S05 — Team Mana Pool**
- Pool = sum of living units' `manaContribution` + item `manaBonus`
- Resets to max each round
- Auto-attack hits generate +1 mid-execution (available to later SPD units)
- Next-turn mana variant (some chars/items delay generation)
- Properties consumed: `manaCost`, `manaContribution`, `manaBonus`

**S06 — Deterministic Crit**
- Per-unit counter, increments per auto-attack HIT (not per action)
- Threshold: 10 (configurable in constants)
- Crit multiplier applied to that hit's damage
- Counter resets on crit, resets per battle
- Visible CritMeter UI
- No ability/spell advances crit

### TIER 2: Status & Buff Framework (required for 126+ abilities)

**S07 — Status Effects (negative)**
- `poison`: 8% max HP/turn DoT
- `burn`: 10% max HP/turn DoT
- `freeze`: Skip turn, 30% break chance/turn
- `stun`: Skip turn completely, auto-expires
- `paralyze`: 25% action failure chance/turn
- Properties consumed: `statusEffect.type`, `statusEffect.duration`, `statusEffect.chance`
- Tick order: duration decay → DoT → HoT → expiration events

**S08 — Buff/Debuff Stacking**
- Stat modifiers: ATK, DEF, MAG, SPD (positive or negative)
- Stack limit: 3 per stat per unit
- Duration-based (turns)
- Properties consumed: `buffEffect.{stat, modifier, duration}`, `debuffEffect.{stat, modifier, duration}`
- Used by: 50 buff abilities + 7 debuff abilities

**S09 — Shield System**
- Charge-based damage absorption (2-6 charges)
- Duration-limited (default 3 turns)
- Checked before HP damage application
- Properties consumed: `shieldCharges`, `duration`
- Used by: 14 abilities

**S10 — Heal Over Time (HoT)**
- Fixed HP per turn (8-20 HP range in current data)
- Duration-limited
- Processed alongside DoT in status tick
- Properties consumed: `healOverTime.amount`, `healOverTime.duration`
- Used by: 10 abilities

**S11 — Damage Reduction**
- Percentage-based (15-40% in current data)
- Duration-limited
- Applied as multiplier during damage calculation
- Properties consumed: `damageReductionPercent`, `duration`
- Used by: 18 abilities

**S12 — Immunity**
- All-negative-status immunity OR type-specific immunity
- Duration-limited
- Checked before any status application
- Replacement behavior (new immunity overwrites old)
- Properties consumed: `grantImmunity.all`, `grantImmunity.types`, `grantImmunity.duration`
- Used by: 8 abilities

**S13 — Status Cleansing**
- Remove all negative statuses, or by specific type
- Properties consumed: `removeStatusEffects.type` (all/negative/byType), `removeStatusEffects.statuses`
- Used by: 9 abilities

### TIER 3: Advanced Damage Modifiers (required for 60+ abilities)

**S14 — Defense Penetration**
- Percentage of target DEF ignored (20-50% in current data)
- Properties consumed: `ignoreDefensePercent`
- Used by: 27 abilities

**S15 — Splash Damage**
- Single-target ability deals percentage of damage to all other enemies
- Range: 30-100% of primary damage
- Properties consumed: `splashDamagePercent`
- Used by: 20 abilities

**S16 — Chain Damage**
- Damage chains between targets (AoE variant)
- Properties consumed: `chainDamage` (boolean)
- Used by: 13 abilities

### TIER 4: Life/Death (required for specific high-value abilities)

**S17 — Revive**
- Resurrect KO'd unit with percentage of max HP
- Range: 50-80% HP restored
- Properties consumed: `revive` (boolean), `reviveHPPercent`
- Used by: 3 abilities

**S18 — Auto-Revive**
- Status effect: on KO, auto-resurrect
- Uses-based (not duration)
- Properties consumed: attached as status effect with `uses` field
- Used by: referenced in status system, granted by buff abilities

### TIER 5: Equipment Integration

**S19 — Equipment Ability Unlock**
- Equipment grants access to specific abilities while equipped
- Properties consumed: `unlocksAbility` (ability ID reference)
- Used by: 56 of 109 equipment items (51%!)
- Also: `alwaysFirstTurn` (Hermes Sandals — guaranteed first action, 3 items)
- Also: `setId` (equipment set bonuses, 10 items)
- Also: `allowedElements` (element-gated equipment)

---

## WHAT'S NOT A SYSTEM (it's configuration)

These are NOT separate systems — they're parameters within the 19 systems above:

| Property | Lives in | System |
|----------|----------|--------|
| Element matching | Constants/data | Djinn compatibility rules |
| XP curve | Data file | Progression constants |
| Gold rewards | Encounter data | Reward processing |
| AI hints | Enemy/ability data | AI decision engine |
| Unlock levels | Ability data | Progression gating |
| Equipment tiers | Equipment data | Shop/progression |
| Difficulty multipliers | Constants | Scaling modifier |

---

## THE COMPLETE ABILITY PROPERTY TABLE

Every field that can appear on an ability, with which system processes it:

| Property | Type | System | Count in v2 |
|----------|------|--------|-------------|
| id | string | Identity | 241 |
| name | string | Display | 241 |
| type | enum | S01 Damage / S02 Targeting | 241 |
| manaCost | int 0-5 | S05 Mana Pool | 241 |
| basePower | int | S01 Damage | 241 |
| targets | enum | S02 Targeting | 241 |
| element | enum | Djinn compat (not damage) | 182 |
| unlockLevel | int 1-20 | Progression | 241 |
| hitCount | int 1-10 | S03 Multi-Hit | 26 |
| statusEffect | object | S07 Status Effects | 37 |
| statusEffect.chance | float | S07 Status Effects | 33 |
| buffEffect | object | S08 Buff/Debuff | 50 |
| debuffEffect | object | S08 Buff/Debuff | 7 |
| ignoreDefensePercent | float | S14 Defense Penetration | 27 |
| splashDamagePercent | float | S15 Splash Damage | 20 |
| chainDamage | bool | S16 Chain Damage | 13 |
| shieldCharges | int | S09 Shield | 14 |
| damageReductionPercent | float | S11 Damage Reduction | 18 |
| healOverTime | object | S10 HoT | 10 |
| grantImmunity | object | S12 Immunity | 8 |
| removeStatusEffects | object | S13 Cleansing | 9 |
| revive | bool | S17 Revive | 3 |
| reviveHPPercent | float | S17 Revive | 3 |
| aiHints | object | AI Engine | 241 |
| description | string | Display | 241 |

**This table IS the ability schema.** Any future ability is a combination of these fields. Want a new ability that does 3-hit splash damage with a burn chance and defense penetration? Set `hitCount: 3`, `splashDamagePercent: 0.5`, `statusEffect: {type: "burn", chance: 0.3, duration: 3}`, `ignoreDefensePercent: 0.25`. No new system needed.

---

## THE COMPLETE EQUIPMENT PROPERTY TABLE

| Property | Type | System | Count in v2 |
|----------|------|--------|-------------|
| id | string | Identity | 109 |
| name | string | Display | 109 |
| slot | enum | Equipment system | 109 |
| tier | enum | Progression | 109 |
| cost | int | Shop | 109 |
| allowedElements | enum[] | Element gating (S19) | 109 |
| statBonus.atk | int | Stat modification | varies |
| statBonus.def | int | Stat modification | varies |
| statBonus.mag | int | Stat modification | varies |
| statBonus.spd | int | Speed ordering (S04) | 27 |
| statBonus.hp | int | HP pool | 24 |
| statBonus.pp | int | (legacy, becomes mana?) | 16 |
| unlocksAbility | string | S19 Equipment Ability | 56 |
| setId | string | Set bonuses | 10 |
| alwaysFirstTurn | bool | Speed override (S04) | 3 |
| manaBonus | int | S05 Mana Pool | TBD (new) |
| hitCountBonus | int | S03 Multi-Hit | TBD (new) |

**Two new properties needed** that v2 doesn't have but your design requires:
- `manaBonus`: adds to team mana pool contribution
- `hitCountBonus`: grants multi-hit auto-attacks (or overrides base hitCount)

---

## MIGRATION FEASIBILITY

### What a conversion script does:

```
v2 TypeScript definitions → parse → validate → output RON files
```

| Source | Records | Complexity | Time estimate |
|--------|---------|-----------|---------------|
| abilities.ts → abilities/*.ron | 241 | Medium (nested objects) | 1-2 hours |
| enemies.ts → enemies/*.ron | 105 | Low (flat objects) | 30 min |
| equipment.ts → equipment/*.ron | 109 | Medium (stat objects) | 1 hour |
| units.ts → units/*.ron | 11 | High (ability refs, growth) | 1 hour |
| djinn.ts → djinn/*.ron | 23 | Medium | 30 min |
| encounters.ts → encounters/*.ron | 54 | High (enemy refs, rewards) | 2 hours |

**Total: ~6 hours of conversion scripting.** Not manual transcription — automated parsing. The TypeScript objects are regular enough to parse with regex.

### What has to be hand-designed (NOT migratable):

| Content | Count | Why |
|---------|-------|-----|
| Djinn SET/STANDBY ability pairs | ~46 pairs | New design not in v2 |
| Equipment manaBonus values | ~10-20 items | New property not in v2 |
| Equipment hitCountBonus values | ~10-15 items | New property not in v2 |
| Next-turn mana unit/item tagging | ~5-10 items | Underdeveloped in v2 |

**The hand-design work is ~80 data entries.** Everything else migrates.

---

## IMPLEMENTATION ORDER

### Phase 1: Schema + Data Loader (no gameplay yet)
Build Rust structs for all 19 systems. Build Bevy asset loader for RON files. Run conversion script. Validate all v2 data loads cleanly.

### Phase 2: Core Combat (S01-S06)
Damage calc, targeting, multi-hit, queue battle, mana pool, crit. This is the minimum playable battle.

### Phase 3: Status Framework (S07-S13)
All status effects, buffs, shields, HoT, damage reduction, immunity, cleansing. This makes 126 abilities functional.

### Phase 4: Advanced Damage (S14-S16)
Defense penetration, splash, chain. This makes the remaining 60 damage abilities fully functional.

### Phase 5: Life/Death + Equipment (S17-S19)
Revive, auto-revive, equipment abilities, set bonuses, speed overrides.

### Phase 6: Djinn Layer
Djinn state machine (SET/STANDBY/RECOVERY), ability oscillation, summon execution, staggered recovery. This is the system that makes the game unique — built last because it needs all other systems to exist first.

### Phase 7: Content Polish
Design djinn ability pairs. Tag equipment with mana/hit bonuses. Wire encounters to story. Balance pass.

---

## FUTURE-PROOFING

Want to add a new mechanic in the future? Here's the pattern:

1. **Does it fit an existing system?** (most do)
   - New status type → add to S07 enum, add processing in status tick
   - New targeting mode → add to S02 enum
   - New damage modifier → add as optional ability property

2. **Does it need a new system?** (rare)
   - Define the system (S20, S21, etc.)
   - Add optional properties to ability/equipment schema
   - Implement in Rust
   - Existing content is unaffected (new properties are optional)

**Example: Adding a "Stun on Crit" mechanic:**
- New optional ability property: `critEffect: { type: "stun", duration: 1 }`
- Processed in S06 (crit system) — when crit triggers, apply critEffect via S07
- No existing content changes. New abilities can opt into it.

**Example: Adding a "Taunt" targeting override:**
- Already exists as debuff (`Taunt` ability, 1 instance in v2)
- Would need S02 targeting to check for taunt status on the attacking unit
- System: when unit has taunt status, force single-target attacks toward taunter

The architecture handles this because every system is independent and every property is optional. The schema grows additively — nothing breaks.
