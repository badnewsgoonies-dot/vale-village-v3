# VALE VILLAGE — Complete Data Manifest
## What exists + how to make it flexible

---

## SUMMARY COUNTS

| Category | v2 Code | v2 Compendium | Rust Port | Notes |
|----------|---------|---------------|-----------|-------|
| Units (player) | 11 | 11 | 10 | Rust missing Tower Champion |
| Abilities | 241 | ~208 | 81 | Code exceeds compendium (code is canon) |
| Enemies | 105 | 104+ | 65 | Near-match |
| Equipment | 109 | 88 | 33 | Code exceeds compendium |
| Djinn | 23 | 20 | 12 | Code has 3 extras (Scorch, Serac, Eclipse) |
| Encounters | 54 | 28+ | basic tiers | Code far exceeds compendium docs |
| Sprites | 1,707 (v2) | — | 5,874 | Rust has more assets, 99.7% unwired |

---

## 1. UNITS (11 playable)

### Stat Profiles

| Unit | Element | Role | Mana | HP | PP | ATK | DEF | MAG | SPD |
|------|---------|------|------|----|----|-----|-----|-----|-----|
| Adept | Venus | Defensive Tank | 1 | 120 | 15 | 14 | 16 | 8 | 10 |
| War Mage | Mars | Elemental Mage | 2 | 80 | 25 | 10 | 8 | 18 | 12 |
| Mystic | Mercury | Healer | 2 | 90 | 30 | 8 | 10 | 16 | 11 |
| Ranger | Jupiter | Rogue Assassin | 1 | 85 | 20 | 16 | 9 | 10 | 18 |
| Sentinel | Venus | Support Buffer | 1 | 110 | 18 | 12 | 18 | 9 | 9 |
| Stormcaller | Jupiter | AoE Mage | 3 | 75 | 28 | 9 | 7 | 20 | 15 |
| Blaze | Mars | Balanced Warrior | 2 | 95 | 22 | 15 | 11 | 14 | 13 |
| Karis | Mercury | Versatile Scholar | 2 | 88 | 28 | 7 | 9 | 17 | 12 |
| Tyrell | Mars | Pure DPS | 1 | 92 | 18 | 18 | 10 | 12 | 16 |
| Felix | Venus | Master Warrior | 1 | 125 | 16 | 16 | 18 | 9 | 11 |
| Tower Champion | Venus | Boss/Ally | 2 | 620 | 180 | 320 | 210 | 140 | 185 |

### Growth Rates (per level)

| Unit | HP | PP | ATK | DEF | MAG | SPD |
|------|----|----|-----|-----|-----|-----|
| Adept | +25 | +4 | +3 | +4 | +2 | +1 |
| War Mage | +15 | +6 | +2 | +2 | +5 | +2 |
| Mystic | +18 | +7 | +1 | +2 | +4 | +2 |
| Ranger | +16 | +5 | +4 | +2 | +2 | +4 |
| Sentinel | +22 | +4 | +2 | +5 | +2 | +1 |
| Stormcaller | +14 | +7 | +1 | +1 | +6 | +3 |
| Blaze | +18 | +5 | +3 | +2 | +4 | +3 |
| Karis | +17 | +6 | +1 | +2 | +5 | +2 |
| Tyrell | +17 | +4 | +5 | +2 | +3 | +4 |
| Felix | +26 | +4 | +4 | +5 | +2 | +2 |

### Abilities Per Unit

Each unit has 44-48 ability references (including shared basics + unique progression). The ability list is the combination of:
- Shared basics (Strike, Heavy Strike, Guard Break, Precise Jab)
- Element-specific psynergy progression
- Counter-element abilities
- Neutral utility abilities
- Unit-unique abilities (gained per level)

---

## 2. ABILITIES (241 total)

### By Type

| Type | Count | Mana Range | Notes |
|------|-------|-----------|-------|
| Physical | 60 | 0-5 | Includes all auto-attacks (0 cost) |
| Psynergy | 91 | 1-5 | Elemental magic damage |
| Buff | 69 | 1-4 | Stat boosts, shields, regen |
| Healing | 17 | 2-5 | Single/party heals, revive |
| Debuff | 4 | 1-2 | Weaken, Blind, Taunt, Curse |

### 0-Cost Abilities (auto-attack class)

| Name | Power | Targets | Hits | Notes |
|------|-------|---------|------|-------|
| Strike | 0 | single | 1 | Universal basic |
| Heavy Strike | 15 | single | 1 | Stronger basic |
| Guard Break | 18 | single | 1 | DEF reduction |
| Precise Jab | 12 | single | 1 | Accuracy-focused |

### Multi-Hit Abilities (26 total)

| Name | Hits | Cost | Power | Type |
|------|------|------|-------|------|
| Rock Breaker | 2 | 1 | 18 | physical |
| Swift Strike | 2 | 1 | 12 | physical |
| Inferno Strike | 2 | 2 | 16 | physical |
| Stone Fist | 2 | 2 | 16 | physical |
| Earthen Cleave | 2 | 2 | 28 | physical |
| Electric Overload | 2 | 4 | 20 | psynergy |
| Storm Sovereign | 2 | 5 | 65 | psynergy |
| Tempest Tyrant | 2 | 5 | 70 | psynergy |
| Supernova Strike | 2 | 5 | 65 | physical |
| Gaia Blade | 2 | 5 | 75 | physical |
| Blazing Fury | 3 | 3 | 20 | psynergy |
| Blazing Assault | 3 | 3 | 18 | physical |
| Rapid Strikes | 3 | 0 | 10 | physical |
| Plasma Shot | 3 | 4 | 18 | psynergy |
| Frozen Spikes | 3 | 4 | 20 | psynergy |
| Grand Impact | 3 | 4 | 25 | physical |
| Inferno Barrage | 3 | 5 | 22 | physical |
| Bolt Barrage | 3 | 5 | 18 | psynergy |
| Zeus (ability) | 3 | 5 | 25 | psynergy |
| Death Strike | 5 | 3 | 18 | physical |
| Thunder God | 4 | 3 | 16 | physical |
| Crimson Fury | 4 | 4 | 20 | physical |
| God Thunder | 4 | 5 | 25 | psynergy |
| Flurry | 4 | 2 | 14 | physical |
| Unstoppable Force | 6 | 4 | 20 | physical |
| One Thousand Cuts | 8 | 5 | 25 | physical |

**KEY NOTE:** Rapid Strikes (3 hits, 0 cost) is a multi-hit auto-attack. This is the mana engine ability — 3 mana generated + 3 crit ticks in one action slot.

### By Element

| Element | Physical | Psynergy | Buff | Heal | Debuff | Total |
|---------|----------|----------|------|------|--------|-------|
| Neutral | 38 | 0 | 32 | 6 | 4 | 80 |
| Venus | 10 | 22 | 10 | 3 | 0 | 45 |
| Mars | 8 | 24 | 8 | 2 | 0 | 42 |
| Mercury | 2 | 22 | 10 | 4 | 0 | 38 |
| Jupiter | 2 | 23 | 9 | 2 | 0 | 36 |

---

## 3. ENEMIES (105 total)

### By Element

| Element | Count | HP Range | ATK Range | Level Range |
|---------|-------|----------|-----------|-------------|
| Venus | 22 | 36-350 | 6-34 | 1-12 |
| Mars | 22 | 45-320 | 6-38 | 1-12 |
| Mercury | 31 | 40-450 | 4-36 | 1-15 |
| Jupiter | 30 | 28-400 | 6-42 | 1-15 |

### By Tier (approximate from levels)

| Tier | Levels | Count | Purpose |
|------|--------|-------|---------|
| Early (1-3) | 1-3 | ~30 | Scouts, wolves, basic beasts |
| Mid (4-6) | 4-6 | ~25 | Bears, soldiers, elementals |
| Late (7-9) | 7-9 | ~25 | Captains, commanders, mythic beasts |
| Boss (10+) | 10-15 | ~25 | Warlords, dragons, sovereigns, avatars |

### Notable Bosses

| Name | Element | HP | ATK | DEF | SPD | Level |
|------|---------|-----|-----|-----|-----|-------|
| The Overseer | Jupiter | 400 | 30 | 30 | 20 | 10 |
| Chimera | Mars | 320 | 32 | 28 | 16 | 10 |
| Hydra | Mercury | 280 | 26 | 26 | 13 | 9 |
| Kraken | Mercury | 300 | 30 | 28 | 14 | 11 |
| Storm Titan | Jupiter | 350 | 36 | 26 | 16 | 12 |
| Arctic Sovereign | Mercury | 380 | 32 | 34 | 18 | 13 |
| Celestial Fury | Jupiter | 400 | 42 | 28 | 26 | 15 |
| Abyssal Emperor | Mercury | 450 | 36 | 40 | 17 | 15 |
| Zeus Avatar | Jupiter | 370 | 40 | 26 | 24 | 14 |

---

## 4. EQUIPMENT (109 total)

### By Slot

| Slot | Count | Key Stats |
|------|-------|-----------|
| Weapons | 43 | ATK, MAG, SPD |
| Armor | 24 | DEF, HP, MAG |
| Helms | 14 | DEF, MAG, PP |
| Boots | 10 | SPD, DEF |
| Accessories | 18 | Various (mana, multi-hit, mixed) |

### Weapon Subtypes

| Type | Count | Element Affinity | Notes |
|------|-------|-----------------|-------|
| Swords | 8 | Venus/Jupiter | Standard progression, balanced |
| Axes | 5 | Mars | High ATK, SPD penalty |
| Maces | 3 | Mars | ATK + DEF hybrid |
| Staves | 7 | Mercury/Jupiter | MAG-focused, low ATK |
| Lances | 3 | Any | ATK + SPD |
| Bows | 3 | Any | ATK + SPD + range |
| Daggers | 3 | Any | SPD-focused |
| Special/Legendary | 11 | Various | Tower rewards, artifacts |

### Tier Progression

| Tier | ATK Range | DEF Range | Availability |
|------|-----------|-----------|-------------|
| Basic | 3-7 | 2-6 | Starting gear |
| Bronze | 9-12 | 6-10 | Early game |
| Iron | 14-18 | 9-15 | Act 1 |
| Steel | 22-30 | 14-24 | Mid game |
| Silver | 32-38 | 20-35 | Act 2 |
| Mythril | 40-48 | 28-52 | Late game |
| Legendary | 52-68 | 38-66 | Boss rewards |
| Artifact | 70-80 | 50-78 | Endgame / Tower |

---

## 5. DJINN (23 in code, 20 documented)

### Full Roster

| Name | Element | Tier | Summon Effect | Documented? |
|------|---------|------|---------------|-------------|
| Flint | Venus | 1 | Stone Barrage (80 AoE) | Yes |
| Granite | Venus | 2 | Terra Wall (DEF +10) | Yes |
| Bane | Venus | 3 | Earthquake (300 AoE) | Yes |
| Rockling | Venus | 1 | Earth Spike (46 single) | Yes |
| Serac | Venus | 3 | ? | Code-only |
| Forge | Mars | 1 | Firebolt Barrage (120 AoE) | Yes |
| Corona | Mars | 2 | Flame Field (MAG +8) | Yes |
| Fury | Mars | 3 | Blazing Torrent (220 AoE) | Yes |
| Ember | Mars | 1 | Fire Burst (46 single) | Yes |
| Nova | Mars | 3 | Starfire (320 AoE, Tower) | Yes |
| Scorch | Mars | 3 | ? | Code-only |
| Fizz | Mercury | 1 | Ice Shards (100 AoE) | Yes |
| Tonic | Mercury | 2 | Healing Mist (80 heal) | Yes |
| Crystal | Mercury | 3 | Crystal Prism (MAG +12) | Yes |
| Surge | Mercury | 2 | Tidal Wave (180 AoE) | Yes |
| Chill | Mercury | 3 | Absolute Zero (Stun) | Yes |
| Breeze | Jupiter | 1 | Gale Shards (110 AoE) | Yes |
| Squall | Jupiter | 2 | Storm Burst (160 AoE) | Yes |
| Storm | Jupiter | 3 | Tempest Swirl (Chaos) | Yes |
| Gust | Jupiter | 1 | Swift Winds (SPD +8) | Yes |
| Bolt | Jupiter | 2 | Chain Lightning (200 AoE) | Yes |
| Tempest | Jupiter | 3 | Ultimate Storm (350 AoE, Tower) | Yes |
| Eclipse | Jupiter | 3 | ? | Code-only |

### Distribution

| Element | T1 | T2 | T3 | Total |
|---------|----|----|-----|-------|
| Venus | 2 | 1 | 2 | 5 |
| Mars | 2 | 1 | 3 | 6 |
| Mercury | 1 | 2 | 2 | 5 |
| Jupiter | 2 | 2 | 3 | 7 |

---

## 6. ENCOUNTERS (54 total)

### Main Story (Houses 2-36)

35 scripted encounters, each with:
- Specific enemy composition
- Difficulty rating (easy/medium/hard/boss)
- Fixed XP + gold rewards
- Equipment rewards (fixed or choose-one)
- Recruitment triggers at specific houses
- Djinn acquisition at specific houses

### Post-Game / Tower (Houses 37-50)

14 additional encounters for post-story content:
- Houses 37-39: Hard encounters
- House 40: Boss (Storm Gate)
- Houses 41-44: Hard encounters
- House 45: Boss (The Eye)
- Houses 46-48: Chaos sequence
- House 49: Boss (The Gatekeeper)
- House 50: Final Boss (Golden Sun)

### Side Encounters (4)

- training-dummy
- roadside-bandits
- merchant-guard
- abandoned-farm

---

## 7. DATA ARCHITECTURE — MAKING IT FLEXIBLE

### The Problem

The v2 TypeScript codebase has all game data hardcoded in TypeScript files. Abilities, enemies, units, equipment, djinn, and encounters are all exported constants. This means:
- Adding a new enemy requires editing enemies.ts and recompiling
- Balancing requires code changes
- No runtime loading of new content
- Can't mod or extend without source access

### The Solution: Data-Driven Architecture

Every piece of game content should be loadable from external data files (RON, JSON, or YAML) at runtime. The code defines the **schema and systems**. The data files define the **content**.

### Proposed File Structure

```
data/
├── schema/
│   ├── ability.schema.ron      # Ability field definitions + validation rules
│   ├── unit.schema.ron         # Unit field definitions
│   ├── enemy.schema.ron        # Enemy field definitions
│   ├── equipment.schema.ron    # Equipment field definitions
│   ├── djinn.schema.ron        # Djinn field definitions
│   └── encounter.schema.ron    # Encounter field definitions
│
├── content/
│   ├── abilities/
│   │   ├── physical.ron        # All physical abilities
│   │   ├── psynergy_venus.ron  # Venus psynergy
│   │   ├── psynergy_mars.ron   # Mars psynergy
│   │   ├── psynergy_mercury.ron
│   │   ├── psynergy_jupiter.ron
│   │   ├── buffs.ron           # All buffs
│   │   ├── healing.ron         # All healing
│   │   └── debuffs.ron         # All debuffs
│   │
│   ├── units/
│   │   ├── starters.ron        # Adept, War Mage, Mystic, Ranger
│   │   ├── recruits.ron        # Sentinel through Felix
│   │   └── special.ron         # Tower Champion
│   │
│   ├── enemies/
│   │   ├── venus_enemies.ron
│   │   ├── mars_enemies.ron
│   │   ├── mercury_enemies.ron
│   │   ├── jupiter_enemies.ron
│   │   └── bosses.ron
│   │
│   ├── equipment/
│   │   ├── weapons.ron
│   │   ├── armor.ron
│   │   ├── helms.ron
│   │   ├── boots.ron
│   │   └── accessories.ron
│   │
│   ├── djinn/
│   │   └── all_djinn.ron       # 23 djinn + summon data + abilities
│   │
│   └── encounters/
│       ├── story_act1.ron      # Houses 2-7
│       ├── story_act2.ron      # Houses 8-14
│       ├── story_act3.ron      # Houses 15-20
│       ├── story_post.ron      # Houses 21-36
│       ├── tower.ron           # Houses 37-50
│       └── side.ron            # Training, bandits, etc.
│
└── balance/
    ├── constants.ron           # Damage formulas, crit threshold, mana gen
    ├── difficulty.ron          # Easy/Normal/Hard multipliers
    └── progression.ron         # XP curve, level cap, tier thresholds
```

### Schema Example (what the code enforces)

```ron
// ability.schema.ron — defines what an ability CAN be
(
  required_fields: [
    ("id", String, unique: true),
    ("name", String),
    ("type", Enum(["physical", "psynergy", "healing", "buff", "debuff"])),
    ("mana_cost", Int, min: 0, max: 10),
    ("base_power", Int, min: 0),
    ("targets", Enum(["single-enemy", "all-enemies", "single-ally", "all-allies", "self"])),
  ],
  optional_fields: [
    ("element", Enum(["Venus", "Mars", "Mercury", "Jupiter", "Neutral"]), default: "Neutral"),
    ("hit_count", Int, min: 1, max: 10, default: 1),
    ("unlock_level", Int, min: 1, max: 20, default: 1),
    ("status_effect", StatusEffect),
    ("splash_damage_percent", Float, min: 0.0, max: 1.0),
    ("ignore_defense_percent", Float, min: 0.0, max: 1.0),
    ("description", String),
    ("ai_hints", AiHints),
  ],
  validation: [
    "if type == 'healing' then base_power > 0",
    "if mana_cost == 0 then type == 'physical'",  // 0-cost = auto-attack class
  ],
)
```

### Content Example (what gets loaded at runtime)

```ron
// content/abilities/physical.ron
[
  (id: "strike", name: "Strike", type: "physical", mana_cost: 0, base_power: 0, targets: "single-enemy",
   description: "Basic physical attack"),
  (id: "heavy-strike", name: "Heavy Strike", type: "physical", mana_cost: 0, base_power: 15, targets: "single-enemy",
   unlock_level: 2, description: "Powerful physical strike"),
  (id: "rapid-strikes", name: "Rapid Strikes", type: "physical", mana_cost: 0, base_power: 10, targets: "single-enemy",
   hit_count: 3, unlock_level: 5, description: "Triple-hit auto-attack"),
  // ... 57 more
]
```

### Why This Architecture

**1. Content is separate from systems.**
The Rust code implements: damage formulas, queue battle logic, mana pool, crit counter, djinn state machine, speed ordering, summons. It does NOT define what any specific ability, enemy, or item is. That's all in data files.

**2. Adding content = adding data, not code.**
New enemy? Add a RON entry to the right file. New ability? Same. New djinn? Same. No recompilation for content changes.

**3. Balance tuning is a data edit.**
Change `crit_threshold: 10` to `crit_threshold: 8` in constants.ron. Change an enemy's HP. Adjust a weapon's ATK. All without touching Rust source.

**4. Future-proof for expansion.**
Want 50 more enemies? Add them to the RON files. New element? Add it to the schema enum and create content files. Modding support? Expose the data directory.

**5. Agent-friendly.**
An AI agent can generate well-formed RON content files from a spec without needing to understand the Rust codebase. The schema validates everything. Workers create data, not systems.

### What the Rust Code Owns (never in data files)

- Damage formula implementation
- Queue battle state machine
- Mana pool calculation and mid-round generation
- Crit counter logic (per-hit, auto-attack only)
- Djinn state transitions (SET → STANDBY → RECOVERY)
- Summon execution priority
- Speed ordering and tiebreakers
- Save/load serialization
- UI rendering
- AI decision engine

### What Data Files Own (never hardcoded)

- All ability definitions (241+)
- All enemy definitions (105+)
- All unit stat profiles and growth rates (11+)
- All equipment stats and tier progression (109+)
- All djinn definitions and summon effects (23+)
- All encounter compositions and rewards (54+)
- Balance constants (damage multipliers, crit threshold, mana gen rate)
- Difficulty scaling curves
- XP curve and level cap

### Migration Path

1. **Define schemas in Rust** — structs with serde derive, validated on load
2. **Export v2 data to RON** — one-time conversion script from TypeScript definitions
3. **Build a data loader** — Bevy asset plugin that reads the content/ directory
4. **Validate on startup** — schemas catch malformed data before the game runs
5. **Remove hardcoded data from Rust** — everything loads from files
6. **Test with unit tests** — load data, run through formulas, verify outputs match v2

---

## 8. WHAT'S MISSING (gaps between v2 and what we need)

### Data gaps

| Gap | v2 status | Needed |
|-----|-----------|--------|
| Djinn SET/STANDBY abilities | Not implemented (only summon effects) | Full ability pairs per djinn per compatibility |
| Djinn-unit compatibility abilities | Mentioned in docs, not in code | Need ~60 djinn ability definitions |
| Multi-hit auto-attack items | No items grant multi-hit in v2 | Need equipment with hit_count property |
| Mana bonus items | No items add mana_contribution | Need equipment with mana_bonus property |
| Next-turn mana units/items | autoAttackTiming exists but underdeveloped | Need clear next-turn vs same-turn marking |
| Story battle sequence | Encounters exist but narrative framing is thin | Need story_battles.ron with narrative context |
| Tower floor definitions | 30 floors defined but rewards incomplete | Need full reward tables per floor |

### System gaps (Rust port)

| System | Status | Priority |
|--------|--------|----------|
| Queue-based battle | NOT built (standard turn-by-turn) | P0 |
| Team mana pool | NOT built (per-unit PP) | P0 |
| Deterministic crit (per-hit) | NOT built (random variance) | P0 |
| Djinn ability oscillation | NOT built | P0 |
| Summon priority execution | NOT built | P1 |
| Speed-ordered mana generation | NOT built | P1 |
| Data-driven content loading | NOT built (hardcoded) | P1 |
| Next-turn mana timing | Stub exists | P2 |
| Multi-hit equipment property | NOT built | P2 |
