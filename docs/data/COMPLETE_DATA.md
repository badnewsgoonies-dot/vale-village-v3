# VALE VILLAGE — Complete Game Data Reference
### Extracted by Codex workers from vale-village-v2 (canonical TypeScript)
### 137 enemies | 109 equipment | 23 djinn | 241 abilities | 55 encounters | 11 units
### March 2026 — All data worker-validated against source

---

# 1. BATTLE CONSTANTS

```
MAX_PARTY_SIZE        = 4
MAX_EQUIPPED_DJINN    = 3
MAX_LEVEL             = 20
CRIT_THRESHOLD        = 10 (every 10th auto-attack hit)
MANA_GAIN_PER_HIT     = 1
MANA_RESETS_EACH_ROUND = true

DAMAGE FORMULAS:
  Physical: basePower + ATK - (DEF × 0.5)    floor 1
  Psynergy: basePower + MAG - (DEF × 0.3)    floor 1
  Healing:  basePower + MAG                   floor 1

NOTE: v2 has element damage mods (1.5x advantage / 0.67x disadvantage).
      Geni's design: REMOVE element damage mods in Bevy port.
      Elements affect ONLY djinn compatibility, not combat damage.

STATUS EFFECTS:
  Poison:   8% maxHP/turn, duration-based
  Burn:     10% maxHP/turn, duration-based
  Freeze:   skip turn + 30% break/turn, duration-based
  Stun:     skip turn, auto-expires after 1 turn
  Paralyze: 25% action failure, duration-based

ACTIONS PER UNIT PER ROUND: 2 options only
  ATTACK:   0 cost, generates +1 mana/hit, +1 crit/hit
  PSYNERGY: costs mana, generates nothing
  (No defend. No items. No djinn menu action.)
```

---

# 2. UNITS (11 playable)

## 2.1 Base Stats & Growth

### Adept (adept) — Venus
- Abilities: 20
- Ability progression:
  - Lv.1: **Earth Spike** (psynergy)
  - Lv.1: **Strike** (physical)
  - Lv.2: **Focus Strike** (physical)
  - Lv.3: **Stone Skin** (buff)
  - Lv.5: **Ice Lance** (psynergy)
  - Lv.6: **Aqua Heal** (healing)
  - Lv.7: **Fortify** (buff)
  - Lv.8: **Tremor** (psynergy)
  - Lv.9: **Guardian Stance** (buff)
  - Lv.10: **Rock Breaker** (physical)
  - Lv.11: **Earthquake** (psynergy)
  - Lv.12: **Stone Wall** (buff)
  - Lv.13: **Unbreakable** (buff)
  - Lv.14: **Titan Grip** (physical)
  - Lv.15: **Gaia Shield** (buff)
  - Lv.16: **Petrify Strike** (physical)
  - Lv.17: **Mountain's Endurance** (buff)
  - Lv.18: **Landslide** (psynergy)
  - Lv.19: **Earth's Blessing** (healing)
  - Lv.20: **Gaia Rebirth** (healing)

### War Mage (war-mage) — Mars
- Abilities: 20
- Ability progression:
  - Lv.1: **Flame Burst** (psynergy)
  - Lv.1: **Strike** (physical)
  - Lv.2: **Fire Ward** (buff)
  - Lv.3: **Gale Force** (psynergy)
  - Lv.4: **Wind Barrier** (buff)
  - Lv.5: **Focus Strike** (physical)
  - Lv.7: **Ignite** (psynergy)
  - Lv.8: **Flame Wall** (buff)
  - Lv.9: **Inferno Slash** (physical)
  - Lv.10: **Blazing Fury** (psynergy)
  - Lv.11: **Pyroclasm** (psynergy)
  - Lv.12: **Fire Aura** (buff)
  - Lv.13: **Meteor Strike** (psynergy)
  - Lv.14: **Phoenix Flames** (healing)
  - Lv.15: **Magma Burst** (psynergy)
  - Lv.16: **Flame Shield** (buff)
  - Lv.17: **Supernova** (psynergy)
  - Lv.18: **Infernal Rage** (buff)
  - Lv.19: **Dragon Breath** (psynergy)
  - Lv.20: **Ragnarok Flames** (psynergy)

### Mystic (mystic) — Mercury
- Abilities: 20
- Ability progression:
  - Lv.1: **Ice Lance** (psynergy)
  - Lv.1: **Strike** (physical)
  - Lv.2: **Aqua Heal** (healing)
  - Lv.3: **Earth Spike** (psynergy)
  - Lv.4: **Stone Skin** (buff)
  - Lv.5: **Focus Strike** (physical)
  - Lv.7: **Cleanse** (healing)
  - Lv.8: **Frost Wave** (psynergy)
  - Lv.9: **Regen** (healing)
  - Lv.10: **Diamond Dust** (psynergy)
  - Lv.11: **Mass Regen** (healing)
  - Lv.12: **Glacial Shield** (buff)
  - Lv.13: **Deep Freeze** (psynergy)
  - Lv.14: **Sanctuary** (buff)
  - Lv.15: **Blizzard** (psynergy)
  - Lv.16: **Restoration** (healing)
  - Lv.17: **Frozen Tomb** (psynergy)
  - Lv.18: **Aqua Barrier** (buff)
  - Lv.19: **Absolute Zero** (psynergy)
  - Lv.20: **Leviathan's Grace** (healing)

### Ranger (ranger) — Jupiter
- Abilities: 20
- Ability progression:
  - Lv.1: **Gale Force** (psynergy)
  - Lv.1: **Strike** (physical)
  - Lv.2: **Wind Barrier** (buff)
  - Lv.3: **Flame Burst** (psynergy)
  - Lv.4: **Fire Ward** (buff)
  - Lv.5: **Focus Strike** (physical)
  - Lv.7: **Swift Strike** (physical)
  - Lv.8: **Shock Bolt** (psynergy)
  - Lv.9: **Tempest** (psynergy)
  - Lv.10: **Hurricane Slash** (physical)
  - Lv.11: **Plasma Shot** (psynergy)
  - Lv.12: **Cyclone** (psynergy)
  - Lv.13: **Thunder God's Fury** (physical)
  - Lv.14: **Storm Blessing** (buff)
  - Lv.15: **Judgment Bolt** (psynergy)
  - Lv.16: **Static Field** (psynergy)
  - Lv.17: **Wind Walker** (buff)
  - Lv.18: **Maelstrom** (psynergy)
  - Lv.19: **Zeus's Wrath** (psynergy)
  - Lv.20: **Storm Sovereign** (psynergy)

### Sentinel (sentinel) — Venus
- Abilities: 21
- Ability progression:
  - Lv.1: **Boost Defense** (buff)
  - Lv.1: **Strike** (physical)
  - Lv.2: **Guard Break** (physical)
  - Lv.3: **Quake** (psynergy)
  - Lv.4: **Taunt** (debuff)
  - Lv.5: **Shield Bash** (physical)
  - Lv.6: **Iron Wall** (buff)
  - Lv.7: **Counter Stance** (buff)
  - Lv.8: **Tremor Strike** (physical)
  - Lv.9: **Fortified Guard** (buff)
  - Lv.10: **Bulwark** (buff)
  - Lv.11: **Crushing Blow** (physical)
  - Lv.12: **Earthen Armor** (buff)
  - Lv.13: **Shockwave** (psynergy)
  - Lv.14: **Guardian's Resolve** (buff)
  - Lv.15: **Titan's Grip** (physical)
  - Lv.16: **Stone Fortress** (buff)
  - Lv.17: **Avalanche** (psynergy)
  - Lv.18: **Immortal Bulwark** (buff)
  - Lv.19: **Earth Splitter** (physical)
  - Lv.20: **Atlas's Stand** (buff)

### Stormcaller (stormcaller) — Jupiter
- Abilities: 21
- Ability progression:
  - Lv.1: **Gust** (psynergy)
  - Lv.1: **Strike** (physical)
  - Lv.2: **Chain Lightning** (psynergy)
  - Lv.3: **Blind** (debuff)
  - Lv.4: **Thunder Clap** (psynergy)
  - Lv.5: **Storm Call** (psynergy)
  - Lv.6: **Static Charge** (buff)
  - Lv.7: **Lightning Arc** (psynergy)
  - Lv.8: **Shock Pulse** (psynergy)
  - Lv.9: **Wind Mastery** (buff)
  - Lv.10: **Thunder Storm** (psynergy)
  - Lv.11: **Electric Overload** (psynergy)
  - Lv.12: **Storm Shield** (buff)
  - Lv.13: **Bolt Barrage** (psynergy)
  - Lv.14: **Hurricane Force** (psynergy)
  - Lv.15: **Thor's Hammer** (psynergy)
  - Lv.16: **Lightning Sanctuary** (buff)
  - Lv.17: **Apocalyptic Storm** (psynergy)
  - Lv.18: **God Thunder** (psynergy)
  - Lv.19: **World Storm** (psynergy)
  - Lv.20: **Tempest Tyrant** (psynergy)

### Blaze (blaze) — Mars
- Abilities: 21
- Ability progression:
  - Lv.1: **Heavy Strike** (physical)
  - Lv.1: **Strike** (physical)
  - Lv.2: **Fireball** (psynergy)
  - Lv.3: **Burn Touch** (psynergy)
  - Lv.4: **Flame Blade** (physical)
  - Lv.5: **Battle Cry** (buff)
  - Lv.6: **Inferno Strike** (physical)
  - Lv.7: **Pyroblast** (psynergy)
  - Lv.8: **Warrior's Flame** (buff)
  - Lv.9: **Molten Slash** (physical)
  - Lv.10: **Fire Nova** (psynergy)
  - Lv.11: **Blazing Assault** (physical)
  - Lv.12: **Magma Wave** (psynergy)
  - Lv.13: **Berserker Rage** (buff)
  - Lv.14: **Crimson Fury** (physical)
  - Lv.15: **Meteor Crash** (psynergy)
  - Lv.16: **Phoenix Aura** (buff)
  - Lv.17: **Solar Flare** (psynergy)
  - Lv.18: **Ultimate Warrior** (buff)
  - Lv.19: **Inferno Barrage** (physical)
  - Lv.20: **Supernova Strike** (physical)

### Karis (karis) — Mercury
- Abilities: 21
- Ability progression:
  - Lv.1: **Ice Shard** (psynergy)
  - Lv.1: **Strike** (physical)
  - Lv.2: **Heal** (healing)
  - Lv.3: **Freeze Blast** (psynergy)
  - Lv.4: **Party Heal** (healing)
  - Lv.5: **Frost Shield** (buff)
  - Lv.6: **Purify** (healing)
  - Lv.7: **Ice Spear** (psynergy)
  - Lv.8: **Scholar's Wisdom** (buff)
  - Lv.9: **Glacial Wave** (psynergy)
  - Lv.10: **Renewal** (healing)
  - Lv.11: **Frozen Spikes** (psynergy)
  - Lv.12: **Crystal Barrier** (buff)
  - Lv.13: **Ice Storm** (psynergy)
  - Lv.14: **Mass Restoration** (healing)
  - Lv.15: **Frozen Tomb** (psynergy)
  - Lv.16: **Scholar's Sanctuary** (buff)
  - Lv.17: **Blizzard Cascade** (psynergy)
  - Lv.18: **Divine Renewal** (healing)
  - Lv.19: **Absolute Zero** (psynergy)
  - Lv.20: **Aqua Resurrection** (healing)

### Tyrell (tyrell) — Mars
- Abilities: 21
- Ability progression:
  - Lv.1: **Precise Jab** (physical)
  - Lv.1: **Strike** (physical)
  - Lv.2: **Heavy Strike** (physical)
  - Lv.3: **Fireball** (psynergy)
  - Lv.4: **Burn Touch** (psynergy)
  - Lv.5: **Rapid Strikes** (physical)
  - Lv.6: **Flame Jab** (physical)
  - Lv.7: **Assassinate** (physical)
  - Lv.8: **Combat Focus** (buff)
  - Lv.9: **Flurry** (physical)
  - Lv.10: **Inferno Assault** (psynergy)
  - Lv.11: **Precision Strike** (physical)
  - Lv.12: **Blood Rush** (buff)
  - Lv.13: **Death Strike** (physical)
  - Lv.14: **Flame Tornado** (psynergy)
  - Lv.15: **Perfect Form** (buff)
  - Lv.16: **Obliterate** (physical)
  - Lv.17: **Unstoppable Force** (physical)
  - Lv.18: **Supreme Focus** (buff)
  - Lv.19: **Annihilation** (physical)
  - Lv.20: **One Thousand Cuts** (physical)

### Felix (felix) — Venus
- Abilities: 21
- Ability progression:
  - Lv.1: **Guard Break** (physical)
  - Lv.1: **Strike** (physical)
  - Lv.2: **Heavy Strike** (physical)
  - Lv.3: **Quake** (psynergy)
  - Lv.4: **Boost Defense** (buff)
  - Lv.5: **Earth Strike** (physical)
  - Lv.6: **Warrior's Resolve** (buff)
  - Lv.7: **Sunder Armor** (physical)
  - Lv.8: **Stone Fist** (physical)
  - Lv.9: **Earth Shaker** (psynergy)
  - Lv.10: **Battle Master** (buff)
  - Lv.11: **Crushing Impact** (physical)
  - Lv.12: **Mountain's Strength** (buff)
  - Lv.13: **Ragnarok Strike** (physical)
  - Lv.14: **Terra Shield** (buff)
  - Lv.15: **Grand Impact** (physical)
  - Lv.16: **Legendary Warrior** (buff)
  - Lv.17: **Titan Fall** (physical)
  - Lv.18: **Earth's Judgment** (psynergy)
  - Lv.19: **Master's Aura** (buff)
  - Lv.20: **Gaia Blade** (physical)

### Tower Champion (tower-champion) — Venus
- Abilities: 12
- Ability progression:
  - Lv.1: **Guard Break** (physical)
  - Lv.1: **Strike** (physical)
  - Lv.2: **Heavy Strike** (physical)
  - Lv.3: **Earth Spike** (psynergy)
  - Lv.4: **Gaia Shield** (buff)
  - Lv.5: **Earthquake** (psynergy)
  - Lv.7: **Ragnarok Strike** (physical)
  - Lv.9: **Stone Wall** (buff)
  - Lv.12: **Master's Aura** (buff)
  - Lv.15: **Gaia Blade** (physical)
  - Lv.18: **Titan Fall** (physical)
  - Lv.20: **Earth's Judgment** (psynergy)


## 2.2 Acquisition Timeline

| House | Recruit | Djinn Reward |
|-------|---------|-------------|
| Pre-game | Adept (Isaac) | Flint (Venus T1) |
| House 1 | War Mage (Garet) | Forge (Mars T1) |
| House 2-3 | Mystic (Mia), Ranger (Ivan) | — |
| House 5 | Blaze | — |
| House 7 | — | Breeze (Jupiter T1) |
| House 8 | Sentinel | Fizz (Mercury T1) |
| House 11 | Karis | — |
| House 12 | — | Granite (Venus T2) |
| House 14 | Tyrell | — |
| House 15 | Stormcaller | Squall (Jupiter T2) |
| House 17 | Felix | — |
| House 18 | — | Bane (Venus T3) |
| House 20 | — | Storm (Jupiter T3) |

---

# 3. ABILITIES (241)

## 3.1 PHYSICAL (60)

| Name | Elem | Cost | Power | Targets | Hits | Unlock | Special |
|------|------|------|-------|---------|------|--------|---------|
| Storm Slash | Jupiter | 1 | 28 | single-enemy | 1 | 1 | — |
| Swift Strike | Jupiter | 1 | 12 | single-enemy | 2 | 7 | 2-hit |
| Hurricane Slash | Jupiter | 2 | 25 | single-enemy | 1 | 10 | pen:0.3 |
| Thunder God's Fury | Jupiter | 3 | 16 | single-enemy | 4 | 13 | 4-hit |
| Axe Cleave | Mars | 1 | 22 | single-enemy | 1 | 1 | — |
| Great Cleave | Mars | 2 | 32 | single-enemy | 1 | 1 | — |
| Volcanic Smash | Mars | 1 | 30 | single-enemy | 1 | 1 | burn |
| Inferno Fist | Mars | 2 | 38 | single-enemy | 1 | 1 | burn |
| Flame Blade | Mars | 1 | 20 | single-enemy | 1 | 4 | — |
| Inferno Strike | Mars | 2 | 16 | single-enemy | 2 | 6 | 2-hit |
| Flame Jab | Mars | 1 | 18 | single-enemy | 1 | 6 | — |
| Inferno Slash | Mars | 1 | 22 | single-enemy | 1 | 9 | — |
| Molten Slash | Mars | 2 | 28 | single-enemy | 1 | 9 | pen:0.3, burn |
| Flurry | Mars | 2 | 14 | single-enemy | 4 | 9 | 4-hit |
| Blazing Assault | Mars | 3 | 18 | single-enemy | 3 | 11 | 3-hit |
| Crimson Fury | Mars | 4 | 20 | single-enemy | 4 | 14 | 4-hit |
| Obliterate | Mars | 4 | 55 | single-enemy | 1 | 16 | pen:0.6 |
| Inferno Barrage | Mars | 5 | 22 | all-enemies | 3 | 19 | 3-hit |
| Annihilation | Mars | 5 | 70 | single-enemy | 1 | 19 | pen:0.7 |
| Supernova Strike | Mars | 5 | 65 | all-enemies | 2 | 20 | 2-hit, pen:0.5, splash:1.0, burn |
| One Thousand Cuts | Mars | 5 | 25 | single-enemy | 8 | 20 | 8-hit, pen:0.5, burn |
| Frost Strike | Mercury | 1 | 26 | single-enemy | 1 | 1 | freeze |
| Strike | — | 0 | 0 | single-enemy | 1 | 1 | — |
| Precise Jab | — | 0 | 12 | single-enemy | 1 | 1 | — |
| Wooden Strike | — | 0 | 8 | single-enemy | 1 | 1 | — |
| Mythril Cleave | — | 2 | 45 | single-enemy | 1 | 1 | — |
| Mythril Pierce | — | 2 | 35 | single-enemy | 1 | 1 | pen:0.35 |
| Mythril Strike | — | 1 | 38 | single-enemy | 1 | 1 | — |
| Astral Strike | — | 3 | 55 | single-enemy | 1 | 1 | pen:0.25 |
| Focus Strike | — | 0 | 20 | single-enemy | 1 | 1 | — |
| Heavy Strike | — | 0 | 15 | single-enemy | 1 | 2 | — |
| Guard Break | — | 0 | 18 | single-enemy | 1 | 2 | — |
| Poison Strike | — | 1 | 10 | single-enemy | 1 | 2 | poison |
| Rapid Strikes | — | 0 | 10 | single-enemy | 3 | 5 | 3-hit |
| Assassinate | — | 2 | 30 | single-enemy | 1 | 7 | pen:0.4 |
| Precision Strike | — | 2 | 35 | single-enemy | 1 | 11 | pen:0.5 |
| Death Strike | — | 3 | 18 | single-enemy | 5 | 13 | 5-hit |
| Unstoppable Force | — | 4 | 20 | single-enemy | 6 | 17 | 6-hit |
| Bronze Slash | Venus | 0 | 12 | single-enemy | 1 | 1 | — |
| Iron Thrust | Venus | 0 | 18 | single-enemy | 1 | 1 | pen:0.2 |
| Mythril Edge | Venus | 2 | 35 | single-enemy | 1 | 1 | pen:0.3 |
| Steel Slash | Venus | 1 | 25 | single-enemy | 1 | 1 | — |
| Silver Strike | Venus | 1 | 30 | single-enemy | 1 | 1 | pen:0.25 |
| Shield Bash | Venus | 0 | 16 | single-enemy | 1 | 5 | stun |
| Earth Strike | Venus | 1 | 22 | single-enemy | 1 | 5 | — |
| Sunder Armor | Venus | 2 | 25 | single-enemy | 1 | 7 | — |
| Tremor Strike | Venus | 1 | 20 | single-enemy | 1 | 8 | — |
| Stone Fist | Venus | 2 | 16 | single-enemy | 2 | 8 | 2-hit |
| Rock Breaker | Venus | 1 | 18 | single-enemy | 2 | 10 | 2-hit |
| Earthen Cleave | Venus | 2 | 28 | single-enemy | 2 | 10 | 2-hit |
| Crushing Blow | Venus | 2 | 35 | single-enemy | 1 | 11 | pen:0.4 |
| Crushing Impact | Venus | 3 | 40 | single-enemy | 1 | 11 | pen:0.4 |
| Ragnarok Strike | Venus | 4 | 50 | single-enemy | 1 | 13 | pen:0.5 |
| Titan Grip | Venus | 2 | 40 | single-enemy | 1 | 14 | pen:0.5 |
| Titan's Grip | Venus | 3 | 45 | single-enemy | 1 | 15 | pen:0.6 |
| Grand Impact | Venus | 4 | 25 | single-enemy | 3 | 15 | 3-hit |
| Petrify Strike | Venus | 2 | 25 | single-enemy | 1 | 16 | freeze |
| Titan Fall | Venus | 5 | 60 | single-enemy | 1 | 17 | pen:0.6, splash:0.4 |
| Earth Splitter | Venus | 4 | 55 | single-enemy | 1 | 19 | pen:0.7, splash:0.3 |
| Gaia Blade | Venus | 5 | 75 | single-enemy | 2 | 20 | 2-hit, pen:0.7, splash:0.6 |

## 3.2 PSYNERGY (91)

| Name | Elem | Cost | Power | Targets | Hits | Unlock | Special |
|------|------|------|-------|---------|------|--------|---------|
| Gust | Jupiter | 2 | 30 | single-enemy | 1 | 1 | — |
| Lightning Shot | Jupiter | 2 | 32 | single-enemy | 1 | 1 | — |
| Gale Force | Jupiter | 3 | 34 | single-enemy | 1 | 1 | — |
| Chain Lightning | Jupiter | 4 | 25 | all-enemies | 1 | 3 | chain |
| Paralyze Shock | Jupiter | 2 | 15 | single-enemy | 1 | 3 | paralyze |
| Thunder Clap | Jupiter | 2 | 20 | all-enemies | 1 | 4 | — |
| Storm Call | Jupiter | 3 | 25 | all-enemies | 1 | 5 | — |
| Lightning Arc | Jupiter | 3 | 35 | single-enemy | 1 | 7 | — |
| Zephyr Burst | Jupiter | 2 | 26 | single-enemy | 1 | 7 | stun |
| Shock Bolt | Jupiter | 3 | 30 | single-enemy | 1 | 8 | paralyze |
| Shock Pulse | Jupiter | 3 | 22 | all-enemies | 1 | 8 | paralyze |
| Tempest | Jupiter | 3 | 28 | all-enemies | 1 | 9 | — |
| Thunder Storm | Jupiter | 4 | 38 | all-enemies | 1 | 10 | chain |
| Plasma Shot | Jupiter | 4 | 18 | single-enemy | 3 | 11 | 3-hit |
| Electric Overload | Jupiter | 4 | 20 | all-enemies | 2 | 11 | 2-hit |
| Cyclone | Jupiter | 4 | 35 | all-enemies | 1 | 12 | splash:1.0 |
| Gale Blast | Jupiter | 3 | 34 | all-enemies | 1 | 12 | — |
| Bolt Barrage | Jupiter | 5 | 18 | all-enemies | 3 | 13 | 3-hit, chain |
| Hurricane Force | Jupiter | 5 | 45 | all-enemies | 1 | 14 | splash:1.0 |
| Judgment Bolt | Jupiter | 5 | 55 | single-enemy | 1 | 15 | pen:0.4 |
| Thor's Hammer | Jupiter | 5 | 50 | single-enemy | 1 | 15 | pen:0.4, splash:0.5 |
| Static Field | Jupiter | 4 | 25 | all-enemies | 1 | 16 | paralyze |
| Apocalyptic Storm | Jupiter | 5 | 55 | all-enemies | 1 | 17 | splash:1.0, chain |
| Maelstrom | Jupiter | 5 | 50 | all-enemies | 1 | 18 | chain, paralyze |
| God Thunder | Jupiter | 5 | 25 | all-enemies | 4 | 18 | 4-hit, chain |
| Zeus's Wrath | Jupiter | 5 | 25 | all-enemies | 3 | 19 | 3-hit, chain |
| World Storm | Jupiter | 5 | 60 | all-enemies | 1 | 19 | splash:1.0, chain, paralyze |
| Storm Sovereign | Jupiter | 5 | 65 | all-enemies | 2 | 20 | 2-hit, splash:1.0, chain, paralyze |
| Tempest Tyrant | Jupiter | 5 | 70 | all-enemies | 2 | 20 | 2-hit, splash:1.0, chain, stun |
| Fireball | Mars | 2 | 35 | single-enemy | 1 | 1 | — |
| Fire Burst | Mars | 0 | 46 | single-enemy | 1 | 1 | — |
| Flame Burst | Mars | 2 | 35 | single-enemy | 1 | 1 | — |
| Shadowflame | Mars | 3 | 50 | single-enemy | 1 | 1 | burn |
| Flame Burst | Mars | 3 | 38 | single-enemy | 1 | 1 | — |
| Burn Touch | Mars | 2 | 25 | single-enemy | 1 | 2 | burn |
| Flare | Mars | 5 | 30 | all-enemies | 1 | 3 | — |
| Ignite | Mars | 2 | 25 | single-enemy | 1 | 7 | burn |
| Pyroblast | Mars | 3 | 35 | single-enemy | 1 | 7 | — |
| Ember Shower | Mars | 3 | 30 | all-enemies | 1 | 8 | burn |
| Blazing Fury | Mars | 3 | 20 | single-enemy | 3 | 10 | 3-hit |
| Fire Nova | Mars | 4 | 30 | all-enemies | 1 | 10 | — |
| Inferno Assault | Mars | 3 | 40 | single-enemy | 1 | 10 | — |
| Pyroclasm | Mars | 4 | 40 | all-enemies | 1 | 11 | — |
| Magma Wave | Mars | 4 | 40 | all-enemies | 1 | 12 | burn |
| Solar Smite | Mars | 4 | 42 | single-enemy | 1 | 12 | splash:0.3 |
| Meteor Strike | Mars | 4 | 45 | single-enemy | 1 | 13 | splash:0.4 |
| Flame Tornado | Mars | 4 | 45 | single-enemy | 1 | 14 | pen:0.3 |
| Magma Burst | Mars | 5 | 55 | single-enemy | 1 | 15 | burn |
| Meteor Crash | Mars | 5 | 55 | single-enemy | 1 | 15 | pen:0.4, splash:0.4 |
| Magma Core | Mars | 5 | 58 | single-enemy | 1 | 16 | pen:0.2, burn |
| Supernova | Mars | 5 | 48 | all-enemies | 1 | 17 | burn |
| Solar Flare | Mars | 5 | 50 | all-enemies | 1 | 17 | burn |
| Dragon Breath | Mars | 5 | 60 | all-enemies | 1 | 19 | chain |
| Ragnarok Flames | Mars | 5 | 70 | all-enemies | 1 | 20 | splash:1.0, burn |
| Ice Shard | Mercury | 2 | 32 | single-enemy | 1 | 1 | — |
| Frost Nova | Mercury | 3 | 28 | all-enemies | 1 | 1 | freeze |
| Ice Lance | Mercury | 3 | 36 | single-enemy | 1 | 1 | — |
| Freeze Blast | Mercury | 2 | 20 | single-enemy | 1 | 3 | freeze |
| Aqua Lance | Mercury | 2 | 28 | single-enemy | 1 | 6 | — |
| Ice Spear | Mercury | 3 | 35 | single-enemy | 1 | 7 | — |
| Frost Wave | Mercury | 3 | 30 | all-enemies | 1 | 8 | — |
| Glacial Wave | Mercury | 4 | 32 | all-enemies | 1 | 9 | — |
| Diamond Dust | Mercury | 3 | 25 | all-enemies | 1 | 10 | freeze |
| Frozen Spikes | Mercury | 4 | 20 | single-enemy | 3 | 11 | 3-hit |
| Tidal Wave | Mercury | 5 | 40 | all-enemies | 1 | 12 | — |
| Deep Freeze | Mercury | 4 | 35 | single-enemy | 1 | 13 | freeze |
| Ice Storm | Mercury | 5 | 45 | all-enemies | 1 | 13 | freeze |
| Blizzard | Mercury | 5 | 45 | all-enemies | 1 | 15 | chain |
| Frozen Tomb | Mercury | 4 | 35 | single-enemy | 1 | 15 | freeze |
| Frozen Tomb | Mercury | 4 | 30 | single-enemy | 1 | 17 | freeze |
| Blizzard Cascade | Mercury | 5 | 55 | all-enemies | 1 | 17 | chain |
| Absolute Zero | Mercury | 5 | 60 | all-enemies | 1 | 19 | freeze |
| Absolute Zero | Mercury | 5 | 65 | all-enemies | 1 | 19 | freeze |
| Shadow Bolt | Neutral | 3 | 34 | single-enemy | 1 | 9 | — |
| Luminous Ray | Neutral | 3 | 36 | single-enemy | 1 | 10 | — |
| Holy Lance | Neutral | 4 | 48 | single-enemy | 1 | 14 | pen:0.25 |
| Arcane Bolt | — | 1 | 20 | single-enemy | 1 | 1 | — |
| Crystal Blast | — | 2 | 28 | single-enemy | 1 | 1 | — |
| Zodiac Bolt | — | 3 | 40 | single-enemy | 1 | 1 | — |
| Mythril Surge | — | 2 | 42 | single-enemy | 1 | 1 | — |
| Earth Spike | Venus | 0 | 46 | single-enemy | 1 | 1 | — |
| Earth Spike | Venus | 3 | 35 | single-enemy | 1 | 1 | — |
| Quake | Venus | 3 | 30 | all-enemies | 1 | 2 | — |
| Tremor | Venus | 3 | 28 | all-enemies | 1 | 8 | — |
| Earth Shaker | Venus | 4 | 38 | all-enemies | 1 | 9 | — |
| Earthquake | Venus | 4 | 35 | all-enemies | 1 | 11 | splash:1.0 |
| Shockwave | Venus | 4 | 32 | all-enemies | 1 | 13 | splash:1.0 |
| Seismic Roar | Venus | 4 | 36 | all-enemies | 1 | 14 | — |
| Avalanche | Venus | 5 | 50 | all-enemies | 1 | 17 | splash:1.0 |
| Landslide | Venus | 5 | 50 | all-enemies | 1 | 18 | splash:1.0 |
| Earth's Judgment | Venus | 5 | 55 | all-enemies | 1 | 18 | splash:1.0 |

## 3.3 HEALING (17)

| Name | Elem | Cost | Power | Targets | Unlock | Special |
|------|------|------|-------|---------|--------|---------|
| Heal | — | 2 | 40 | single-ally | 1 | — |
| Aqua Heal | Mercury | 3 | 50 | single-ally | 1 | — |
| Party Heal | — | 4 | 25 | all-allies | 2 | — |
| Cure | Mercury | 4 | 40 | single-ally | 2 | — |
| Purify | Mercury | 2 | 0 | all-allies | 6 | cleanse:byType |
| Cleanse | Mercury | 2 | 0 | single-ally | 7 | cleanse:negative |
| Regen | Mercury | 2 | 20 | single-ally | 9 | HoT:12/3t |
| Renewal | Mercury | 3 | 25 | all-allies | 10 | HoT:10/3t |
| Mass Regen | Mercury | 4 | 0 | all-allies | 11 | HoT:10/3t |
| Phoenix Flames | Mars | 3 | 40 | single-ally | 14 | cleanse:byType |
| Mass Restoration | Mercury | 5 | 60 | all-allies | 14 | cleanse:negative |
| Restoration | Mercury | 4 | 70 | single-ally | 16 | cleanse:all |
| Divine Renewal | Mercury | 5 | 80 | all-allies | 18 | HoT:15/3t, cleanse:all |
| Earth's Blessing | Venus | 4 | 60 | all-allies | 19 | — |
| Gaia Rebirth | Venus | 5 | 80 | all-allies | 20 | revive:0.5, cleanse:negative |
| Leviathan's Grace | Mercury | 5 | 100 | all-allies | 20 | revive:0.75, HoT:20/3t, cleanse:all |
| Aqua Resurrection | Mercury | 5 | 100 | all-allies | 20 | revive:0.8, HoT:20/4t, cleanse:all |

## 3.4 BUFF (69)

| Name | Elem | Cost | Targets | Unlock | Special |
|------|------|------|---------|--------|---------|
| Boost Attack | — | 2 | single-ally | 1 | atk:+8 |
| Boost Defense | — | 2 | single-ally | 1 | def:+8 |
| Iron Bulwark | — | 1 | self | 1 | def:+6 |
| Steel Focus | — | 1 | self | 1 | atk:+5 |
| Steel Ward | — | 2 | self | 1 | DR:0.15 |
| Iron Mind | — | 1 | self | 1 | immunity |
| Silver Shield | — | 2 | self | 1 | shield:2 |
| Mythril Wisdom | — | 2 | self | 1 | mag:+8 |
| Hyper Speed | — | 1 | self | 1 | spd:+8 |
| Dragon Ward | — | 3 | self | 1 | — |
| Oracle Vision | — | 2 | all-allies | 1 | mag:+6 |
| Storm Mastery | Jupiter | 2 | self | 1 | atk:+5, spd:+8 |
| Frost Mastery | Mercury | 2 | self | 1 | def:+5, mag:+8 |
| Earth Wall | Venus | 2 | all-allies | 1 | DR:0.2 |
| Gaia Fortitude | Venus | 2 | self | 1 | DR:0.15, def:+12 |
| Storm Focus | Jupiter | 2 | self | 1 | atk:+6, mag:+6, spd:+4 |
| Stone Skin | Venus | 2 | single-ally | 1 | def:+10 |
| Fire Ward | Mars | 2 | single-ally | 1 | atk:+10 |
| Wind Barrier | Jupiter | 2 | single-ally | 1 | spd:+8 |
| Guard | — | 3 | single-ally | 2 | def:+8 |
| Battle Cry | — | 2 | self | 5 | atk:+10 |
| Frost Shield | Mercury | 2 | single-ally | 5 | shield:2 |
| Iron Wall | Venus | 2 | self | 6 | def:+12 |
| Static Charge | Jupiter | 2 | self | 6 | mag:+10 |
| Warrior's Resolve | — | 2 | self | 6 | atk:+10, def:+8 |
| Fortify | Venus | 2 | self | 7 | shield:2, def:+5 |
| Counter Stance | — | 2 | self | 7 | DR:0.2 |
| Flame Wall | Mars | 2 | self | 8 | — |
| Warrior's Flame | Mars | 2 | self | 8 | atk:+8, mag:+8 |
| Scholar's Wisdom | — | 2 | self | 8 | mag:+12 |
| Combat Focus | — | 2 | self | 8 | atk:+12, spd:+8 |
| Water Shroud | Mercury | 3 | single-ally | 8 | DR:0.2 |
| Radiant Blessing | Neutral | 2 | single-ally | 8 | — |
| Guardian Stance | — | 2 | self | 9 | DR:0.25 |
| Fortified Guard | — | 3 | self | 9 | shield:3, def:+8 |
| Wind Mastery | Jupiter | 3 | all-allies | 9 | mag:+5, spd:+8 |
| Bulwark | Venus | 3 | all-allies | 10 | def:+6 |
| Battle Master | — | 3 | self | 10 | atk:+12, def:+10, spd:+8 |
| Stone Wall | Venus | 3 | all-allies | 12 | def:+6 |
| Fire Aura | Mars | 3 | all-allies | 12 | atk:+7, mag:+7 |
| Glacial Shield | Mercury | 3 | all-allies | 12 | shield:2 |
| Earthen Armor | Venus | 3 | self | 12 | DR:0.35 |
| Storm Shield | Jupiter | 3 | all-allies | 12 | shield:2, spd:+6 |
| Crystal Barrier | Mercury | 3 | all-allies | 12 | shield:2, def:+6 |
| Blood Rush | — | 3 | self | 12 | atk:+15, spd:+12 |
| Mountain's Strength | Venus | 3 | self | 12 | DR:0.2, atk:+15, def:+12 |
| Unbreakable | — | 3 | self | 13 | shield:3, DR:0.4 |
| Berserker Rage | — | 3 | self | 13 | DR:0.15, atk:+15, spd:+10 |
| Sanctuary | — | 4 | all-allies | 14 | DR:0.2, immunity |
| Storm Blessing | Jupiter | 3 | all-allies | 14 | spd:+10 |
| Guardian's Resolve | — | 4 | all-allies | 14 | def:+7 |
| Terra Shield | Venus | 4 | all-allies | 14 | shield:3, def:+10 |
| Gaia Shield | Venus | 4 | all-allies | 15 | shield:3 |
| Perfect Form | — | 4 | self | 15 | atk:+18, spd:+15 |
| Flame Shield | Mars | 3 | all-allies | 16 | immunity |
| Stone Fortress | Venus | 4 | all-allies | 16 | shield:3, DR:0.25 |
| Lightning Sanctuary | Jupiter | 4 | all-allies | 16 | immunity |
| Phoenix Aura | Mars | 4 | self | 16 | atk:+12, def:+8, mag:+12 |
| Scholar's Sanctuary | Mercury | 4 | all-allies | 16 | def:+8, mag:+8, immunity |
| Legendary Warrior | — | 5 | self | 16 | DR:0.3, atk:+18, def:+15 |
| Mountain's Endurance | — | 3 | self | 17 | immunity |
| Wind Walker | — | 3 | self | 17 | DR:0.25, atk:+10, spd:+15 |
| Infernal Rage | — | 4 | self | 18 | atk:+15, mag:+15 |
| Aqua Barrier | Mercury | 5 | all-allies | 18 | shield:4 |
| Immortal Bulwark | — | 5 | self | 18 | shield:5, DR:0.5, immunity |
| Ultimate Warrior | — | 5 | self | 18 | DR:0.25, atk:+18, def:+10, spd:+12 |
| Supreme Focus | — | 5 | self | 18 | DR:0.2, atk:+20, mag:+10, spd:+18 |
| Master's Aura | — | 5 | all-allies | 19 | atk:+12, def:+12, spd:+10 |
| Atlas's Stand | Venus | 5 | all-allies | 20 | shield:6, DR:0.4, def:+15, immunity |

## 3.5 DEBUFF (4)

| Name | Elem | Cost | Targets | Unlock | Special |
|------|------|------|---------|--------|---------|
| Weaken Defense | — | 2 | single-enemy | 2 | def:-6 |
| Blind | — | 2 | single-enemy | 2 | spd:-3 |
| Taunt | — | 1 | single-enemy | 4 | atk:-5 |
| Umbra Curse | Neutral | 2 | all-enemies | 12 | mag:-8, spd:-5 |

---

# 4. ENEMIES (137)

## 4.1 Venus (28)

| Name | Lv | HP | ATK | DEF | MAG | SPD | XP | Gold |
|------|----|----|-----|-----|-----|-----|----|------|
| Earthbound Wolf | 1 | 55 | 11 | 7 | 3 | 11 | 16 | 8 |
| Earthbound Wolf (Mini-Boss) | 1 | 110 | 11 | 7 | 3 | 11 | 24 | 12 |
| Earth Scout | 1 | 50 | 9 | 8 | 5 | 8 | 15 | 10 |
| Wild Boar | 1 | 36 | 10 | 5 | 2 | 6 | 8 | 5 |
| Lumen Fawn | 1 | 45 | 14 | 8 | 6 | 12 | 12 | 6 |
| Stone Beetle | 2 | 80 | 8 | 15 | 3 | 6 | 22 | 12 |
| Scavenger | 2 | 38 | 8 | 6 | 3 | 7 | 10 | 6 |
| Merchant Guard | 2 | 60 | 10 | 9 | 3 | 8 | 22 | 14 |
| Crystal Bat | 2 | 48 | 8 | 6 | 12 | 18 | 20 | 10 |
| Stone Guardian (Mini-Boss) | 3 | 700 | 12 | 20 | 5 | 6 | 45 | 24 |
| Stone Guardian | 3 | 350 | 12 | 20 | 5 | 6 | 30 | 16 |
| Terra Soldier | 3 | 85 | 14 | 13 | 7 | 9 | 28 | 16 |
| Stone Sprite | 3 | 55 | 6 | 8 | 13 | 14 | 20 | 12 |
| Sentinel | 3 | 154 | 16 | 28 | 13 | 11 | 80 | 25 |
| Earth Shaman | 4 | 220 | 10 | 14 | 16 | 9 | 45 | 22 |
| Mountain Bear | 4 | 110 | 14 | 18 | 6 | 8 | 35 | 18 |
| Stone Captain | 5 | 130 | 18 | 18 | 10 | 10 | 50 | 28 |
| Skeleton Warrior | 5 | 45 | 18 | 12 | 5 | 10 | 42 | 22 |
| Terra Warden | 6 | 260 | 16 | 16 | 14 | 9 | 58 | 28 |
| Rock Elemental | 6 | 140 | 16 | 22 | 12 | 8 | 45 | 24 |
| Mountain Commander | 7 | 180 | 22 | 24 | 14 | 11 | 75 | 40 |
| Basilisk | 8 | 200 | 24 | 22 | 16 | 14 | 90 | 50 |
| Clay Golem | 8 | 80 | 22 | 20 | 8 | 6 | 85 | 45 |
| Granite Warlord | 9 | 250 | 28 | 30 | 18 | 12 | 120 | 60 |
| Stone Roc | 9 | 90 | 28 | 18 | 14 | 12 | 110 | 56 |
| Earth Wyrm | 10 | 180 | 26 | 22 | 26 | 13 | 125 | 65 |
| Elder Basilisk | 11 | 320 | 34 | 30 | 22 | 12 | 185 | 98 |
| Terra Guardian | 12 | 250 | 32 | 36 | 18 | 9 | 165 | 86 |

## 4.2 Mars (28)

| Name | Lv | HP | ATK | DEF | MAG | SPD | XP | Gold |
|------|----|----|-----|-----|-----|-----|----|------|
| Flame Scout | 1 | 45 | 10 | 6 | 8 | 10 | 15 | 10 |
| Flame Bandit (Mini-Boss) | 2 | 120 | 13 | 6 | 8 | 10 | 30 | 18 |
| Flame Bandit | 2 | 60 | 13 | 6 | 8 | 10 | 20 | 12 |
| Flame Wolf | 2 | 58 | 12 | 6 | 5 | 13 | 18 | 9 |
| War Mage | 2 | 135 | 12 | 10 | 23 | 14 | 60 | 19 |
| Ember Cleric | 3 | 190 | 9 | 8 | 11 | 10 | 26 | 14 |
| Blaze Soldier | 3 | 75 | 15 | 10 | 12 | 11 | 28 | 16 |
| Flame Sprite | 3 | 48 | 6 | 6 | 15 | 16 | 20 | 12 |
| Bandit | 3 | 48 | 13 | 8 | 5 | 9 | 18 | 15 |
| Road Bandit | 3 | 50 | 12 | 8 | 4 | 9 | 20 | 12 |
| Inferno Bear | 4 | 105 | 16 | 16 | 8 | 9 | 35 | 18 |
| Bandit Captain | 4 | 90 | 16 | 10 | 6 | 10 | 30 | 25 |
| Lava Salamander | 4 | 130 | 20 | 14 | 18 | 12 | 70 | 35 |
| Inferno Captain | 5 | 115 | 20 | 14 | 16 | 12 | 50 | 28 |
| Flame Elemental | 6 | 180 | 14 | 14 | 24 | 14 | 60 | 30 |
| Zombie Hound | 6 | 50 | 20 | 10 | 6 | 15 | 48 | 25 |
| Fire Eagle | 6 | 45 | 22 | 10 | 18 | 17 | 50 | 26 |
| Fire Elemental | 6 | 180 | 14 | 14 | 24 | 14 | 60 | 30 |
| Flame Herald | 7 | 220 | 18 | 14 | 20 | 13 | 70 | 35 |
| Fire Commander | 7 | 160 | 24 | 18 | 22 | 13 | 75 | 40 |
| Dark Mage | 7 | 200 | 12 | 10 | 48 | 10 | 200 | 120 |
| Phoenix | 8 | 240 | 22 | 18 | 28 | 18 | 110 | 60 |
| Volcano Warlord | 9 | 220 | 30 | 22 | 28 | 14 | 120 | 60 |
| Iron Golem | 9 | 70 | 25 | 25 | 10 | 7 | 105 | 55 |
| Chimera | 10 | 320 | 32 | 28 | 30 | 16 | 200 | 100 |
| Alpha Phoenix | 11 | 280 | 32 | 24 | 36 | 20 | 180 | 95 |
| Magma Colossus | 12 | 220 | 38 | 32 | 20 | 8 | 160 | 84 |
| Titanium Colossus | 25 | 3000 | 120 | 150 | 40 | 30 | 1800 | 900 |

## 4.3 Mercury (40)

| Name | Lv | HP | ATK | DEF | MAG | SPD | XP | Gold |
|------|----|----|-----|-----|-----|-----|----|------|
| Mercury Slime | 1 | 40 | 4 | 5 | 6 | 5 | 12 | 6 |
| Frost Scout | 1 | 48 | 8 | 7 | 7 | 9 | 15 | 10 |
| Frost Wolf | 2 | 56 | 10 | 7 | 6 | 14 | 18 | 9 |
| Frost Mystic | 2 | 200 | 10 | 8 | 12 | 11 | 22 | 12 |
| Aquifer Imp | 2 | 46 | 7 | 6 | 14 | 12 | 20 | 10 |
| Mire Toad | 2 | 60 | 18 | 12 | 4 | 10 | 15 | 8 |
| Tide Soldier | 3 | 80 | 12 | 12 | 10 | 10 | 28 | 16 |
| Frost Sprite | 3 | 50 | 5 | 7 | 14 | 15 | 20 | 12 |
| Mistling | 3 | 72 | 10 | 8 | 18 | 13 | 34 | 16 |
| Poison Toad | 3 | 90 | 12 | 10 | 8 | 9 | 30 | 12 |
| Tide Enchanter | 4 | 240 | 11 | 13 | 18 | 10 | 50 | 24 |
| Glacier Bear | 4 | 115 | 13 | 19 | 7 | 7 | 35 | 18 |
| Glacial Sprite | 4 | 88 | 9 | 10 | 22 | 15 | 42 | 22 |
| Frost Oracle | 5 | 200 | 10 | 12 | 20 | 11 | 55 | 26 |
| Glacier Captain | 5 | 125 | 16 | 16 | 14 | 11 | 50 | 28 |
| Frost Hound | 5 | 110 | 18 | 12 | 16 | 17 | 56 | 28 |
| Ice Elemental | 6 | 130 | 12 | 18 | 18 | 10 | 45 | 24 |
| Warder of the Tides | 6 | 150 | 20 | 18 | 24 | 11 | 80 | 40 |
| Storm Commander | 7 | 170 | 20 | 20 | 20 | 12 | 75 | 40 |
| Bone Mage | 7 | 40 | 15 | 10 | 20 | 11 | 60 | 32 |
| Ice Owl | 7 | 40 | 18 | 12 | 20 | 15 | 62 | 34 |
| Frost Serpent | 7 | 95 | 18 | 14 | 20 | 16 | 70 | 38 |
| Leviathan | 8 | 220 | 20 | 24 | 22 | 12 | 90 | 50 |
| Aqua Drake | 8 | 135 | 22 | 18 | 24 | 14 | 88 | 46 |
| Blizzard Warlord | 9 | 240 | 24 | 26 | 24 | 13 | 120 | 60 |
| Hydra | 9 | 280 | 26 | 26 | 20 | 13 | 110 | 55 |
| Tidal Wraith | 9 | 120 | 18 | 16 | 28 | 18 | 95 | 50 |
| Crystal Golem | 10 | 65 | 20 | 18 | 24 | 8 | 115 | 58 |
| Glacier Wyrm | 10 | 180 | 26 | 22 | 26 | 13 | 125 | 65 |
| Kraken | 11 | 300 | 30 | 28 | 28 | 14 | 175 | 92 |
| Frost Lich | 11 | 155 | 22 | 20 | 36 | 15 | 150 | 78 |
| Tundra Serpent | 12 | 185 | 26 | 22 | 30 | 17 | 155 | 82 |
| Arctic Sovereign | 13 | 380 | 32 | 34 | 40 | 18 | 220 | 115 |
| Permafrost Golem | 13 | 280 | 28 | 38 | 26 | 10 | 170 | 88 |
| Polar Guardian | 13 | 260 | 30 | 32 | 28 | 14 | 165 | 86 |
| Ice Golem | 13 | 280 | 28 | 38 | 26 | 10 | 170 | 88 |
| Neptune Warden | 14 | 410 | 34 | 36 | 42 | 16 | 240 | 125 |
| Maelstrom Beast | 14 | 330 | 35 | 30 | 38 | 19 | 195 | 102 |
| Abyssal Emperor | 15 | 450 | 36 | 40 | 44 | 17 | 260 | 135 |
| Arctic Lich | 25 | 2200 | 40 | 60 | 140 | 60 | 1500 | 600 |

## 4.4 Jupiter (41)

| Name | Lv | HP | ATK | DEF | MAG | SPD | XP | Gold |
|------|----|----|-----|-----|-----|-----|----|------|
| Gale Scout | 1 | 42 | 9 | 6 | 9 | 12 | 15 | 10 |
| Carrion Bird | 1 | 28 | 6 | 4 | 2 | 12 | 9 | 4 |
| Wind Sprite | 2 | 45 | 5 | 5 | 14 | 17 | 18 | 10 |
| Storm Wolf | 2 | 52 | 11 | 6 | 7 | 16 | 18 | 9 |
| Gale Priest | 2 | 180 | 8 | 7 | 14 | 13 | 24 | 14 |
| Zephyr Imp | 2 | 44 | 8 | 6 | 13 | 16 | 18 | 9 |
| Wind Soldier | 3 | 70 | 13 | 9 | 13 | 14 | 28 | 16 |
| Stormcaller | 3 | 103 | 11 | 9 | 32 | 21 | 80 | 25 |
| Gale Moth | 3 | 60 | 10 | 8 | 18 | 20 | 32 | 16 |
| Thunder Bear | 4 | 100 | 15 | 15 | 10 | 12 | 35 | 18 |
| Wind Hawk | 4 | 30 | 16 | 6 | 12 | 20 | 32 | 18 |
| Lightning Hopper | 4 | 78 | 14 | 10 | 20 | 24 | 46 | 24 |
| Thunder Captain | 5 | 110 | 17 | 13 | 18 | 15 | 50 | 28 |
| Ghost Wisp | 5 | 35 | 12 | 8 | 16 | 18 | 40 | 20 |
| Stormling | 5 | 95 | 16 | 12 | 26 | 22 | 72 | 36 |
| Shadow Wisp | 5 | 35 | 12 | 8 | 16 | 18 | 40 | 20 |
| Storm Elemental | 6 | 115 | 13 | 13 | 22 | 16 | 45 | 24 |
| Vortex Sentry | 6 | 140 | 22 | 18 | 30 | 18 | 94 | 48 |
| Storm Knight | 6 | 180 | 28 | 20 | 22 | 14 | 120 | 60 |
| Lightning Commander | 7 | 150 | 21 | 16 | 24 | 18 | 75 | 40 |
| Storm Raven | 7 | 75 | 16 | 12 | 22 | 20 | 68 | 36 |
| Thunderbird | 8 | 170 | 21 | 16 | 28 | 22 | 90 | 50 |
| Lightning Lynx | 8 | 90 | 24 | 14 | 18 | 24 | 85 | 44 |
| Thunder Hawk | 8 | 170 | 21 | 16 | 28 | 22 | 90 | 50 |
| Tempest Warlord | 9 | 210 | 26 | 20 | 30 | 20 | 120 | 60 |
| Cyclone Djinni | 9 | 110 | 20 | 15 | 30 | 19 | 100 | 52 |
| The Overseer | 10 | 400 | 30 | 30 | 35 | 20 | 500 | 250 |
| Storm Golem | 10 | 70 | 24 | 16 | 22 | 9 | 115 | 58 |
| Tempest Dragon | 10 | 165 | 28 | 20 | 32 | 17 | 130 | 68 |
| Void Specter | 11 | 140 | 24 | 18 | 34 | 22 | 145 | 76 |
| Storm Titan | 12 | 350 | 36 | 26 | 38 | 16 | 200 | 105 |
| Monsoon Drake | 12 | 190 | 30 | 18 | 36 | 23 | 158 | 83 |
| Stratosphere Lord | 13 | 340 | 38 | 24 | 42 | 22 | 225 | 118 |
| Voltage Chimera | 13 | 200 | 32 | 20 | 38 | 25 | 175 | 90 |
| Vortex Sentinel | 13 | 210 | 34 | 24 | 40 | 21 | 168 | 87 |
| Zeus Avatar | 14 | 370 | 40 | 26 | 46 | 24 | 245 | 128 |
| Aurora Elemental | 14 | 170 | 26 | 22 | 44 | 27 | 180 | 94 |
| Celestial Fury | 15 | 400 | 42 | 28 | 50 | 26 | 270 | 140 |
| Thunderstorm Colossus | 15 | 310 | 40 | 28 | 46 | 20 | 210 | 110 |
| The Golden Sun | 20 | 5000 | 150 | 120 | 200 | 80 | 50000 | 99999 |
| Void Tempest | 25 | 2400 | 60 | 50 | 120 | 90 | 1600 | 650 |

---

# 5. EQUIPMENT (109)

## 5.1 WEAPON (43)

| Name | Tier | Cost | ATK | DEF | MAG | SPD | HP | Elements | Unlocks Ability |
|------|------|------|-----|-----|-----|-----|----|----------|----------------|
| Eclipse Blade | artifact | 0 | 80 | 0 | 15 | 5 | 0 | Venus, Mars | mythril-edge |
| Storm Cleaver | legendary | 0 | 55 | 0 | 20 | 8 | 0 | Jupiter | storm-slash |
| Frost Reaver | legendary | 0 | 52 | 5 | 25 | 0 | 0 | Mercury | frost-strike |
| Volcanic Hammer | legendary | 0 | 68 | 15 | 0 | -3 | 0 | Mars | volcanic-smash |
| Astral Blade | artifact | 0 | 70 | 0 | 20 | 8 | 0 | Venus, Jupiter | astral-strike |
| Shadowflame Staff | artifact | 0 | 35 | 0 | 55 | 0 | 0 | Mars, Mercury | shadowflame |
| Void Scythe | legendary | 0 | 60 | 0 | 25 | 0 | 0 | Mars, Mercury | void-swipe |
| Wooden Staff | basic | 40 | 3 | 0 | 4 | 0 | 0 | Mercury, Jupiter | — |
| Wooden Sword | basic | 50 | 5 | 0 | 0 | 0 | 0 | Venus, Jupiter | wooden-strike |
| Wooden Axe | basic | 60 | 7 | 0 | 0 | -1 | 0 | Mars | — |
| Bronze Sword | bronze | 120 | 9 | 0 | 0 | 0 | 0 | Venus | bronze-slash |
| Mace | bronze | 150 | 11 | 2 | 0 | 0 | 0 | Mars | — |
| Magic Rod | bronze | 180 | 6 | 0 | 8 | 0 | 0 | Mercury, Jupiter | arcane-bolt |
| Iron Sword | iron | 200 | 14 | 0 | 0 | 0 | 0 | Venus | iron-thrust |
| Battle Axe | iron | 280 | 18 | 0 | 0 | -2 | 0 | Mars | axe-cleave |
| Mystic Dagger | bronze | 300 | 12 | 0 | 8 | 3 | 0 | Mercury, Jupiter | mystic-stab |
| Shaman Rod | iron | 400 | 10 | 0 | 14 | 0 | 0 | Mercury, Jupiter | — |
| Steel Sword | steel | 500 | 22 | 0 | 0 | 0 | 0 | Venus | steel-slash |
| Heavy Mace | steel | 650 | 26 | 5 | 0 | 0 | 0 | Mars | — |
| Great Axe | steel | 800 | 30 | 0 | 0 | -3 | 0 | Mars | great-cleave |
| Heavy Halberd | steel | 900 | 28 | 6 | 0 | -2 | 0 | Venus, Mars | halberd-sweep |
| Silver Blade | silver | 1200 | 32 | 0 | 0 | 0 | 0 | Venus | silver-strike |
| Crystal Rod | silver | 1500 | 18 | 0 | 24 | 0 | 0 | Mercury, Jupiter | crystal-blast |
| Frost Scepter | silver | 2100 | 20 | 0 | 28 | 0 | 0 | Mercury | frost-nova |
| Gleaming Javelin | silver | 2100 | 36 | 0 | 0 | 4 | 0 | Jupiter | javelin-throw |
| Flame Branded Axe | silver | 2200 | 38 | 0 | 8 | -1 | 0 | Mars | flame-burst |
| Thunderbolt Bow | silver | 2400 | 35 | 0 | 5 | 6 | 0 | Jupiter | lightning-shot |
| Night Dagger | silver | 2600 | 34 | 0 | 0 | 10 | 0 | Venus, Jupiter | shadow-step |
| Mythril Blade | mythril | 3000 | 45 | 0 | 0 | 0 | 0 | Venus | mythril-edge |
| Mythril Lance | mythril | 3100 | 42 | 0 | 0 | 3 | 0 | Jupiter | mythril-pierce |
| Mythril Axe | mythril | 3200 | 46 | 0 | 0 | -2 | 0 | Mars | mythril-cleave |
| Demon Mace | mythril | 3500 | 48 | 8 | 0 | 0 | 0 | Mars | — |
| Ether Bow | mythril | 3500 | 38 | 0 | 12 | 8 | 0 | Jupiter | ether-shot |
| Mythril Staff | mythril | 3800 | 26 | 0 | 36 | 0 | 0 | Mercury | mythril-surge |
| Storm Lance | mythril | 3900 | 44 | 0 | 10 | 6 | 0 | Jupiter | storm-pierce |
| Zodiac Wand | mythril | 4000 | 28 | 0 | 38 | 0 | 0 | Mercury, Jupiter | zodiac-bolt |
| Radiant Rapier | mythril | 4200 | 40 | 0 | 0 | 12 | 0 | Venus, Jupiter | radiant-stab |
| Dawn Katana | legendary | 7000 | 58 | 0 | 0 | 14 | 0 | Venus, Jupiter | dawn-slash |
| Gaia Blade | legendary | 7500 | 58 | 0 | 0 | 0 | 0 | Venus | mythril-edge |
| Seraphic Staff | legendary | 8000 | 30 | 0 | 50 | 0 | 0 | Mercury | seraphic-beam |
| Titan's Axe | legendary | 9000 | 65 | 10 | 0 | -2 | 0 | Mars | great-cleave |
| Sol Blade | artifact | 15000 | 72 | 0 | 0 | 0 | 0 | Venus | mythril-edge |
| Staff of Ages | artifact | 18000 | 42 | 0 | 58 | 0 | 0 | Mercury, Jupiter | zodiac-bolt |

## 5.2 ARMOR (24)

| Name | Tier | Cost | ATK | DEF | MAG | SPD | HP | Elements | Unlocks Ability |
|------|------|------|-----|-----|-----|-----|----|----------|----------------|
| Tempest Armor | legendary | 0 | 0 | 55 | 10 | 12 | 0 | Jupiter | — |
| Glacier Mail | legendary | 0 | 0 | 58 | 8 | 0 | 75 | Mercury | — |
| Inferno Plate | legendary | 0 | 15 | 52 | 0 | 0 | 50 | Mars | — |
| Aetheric Mantle | artifact | 0 | 0 | 70 | 25 | 0 | 120 | Mercury, Jupiter | — |
| Stormplate Armor | legendary | 0 | 0 | 60 | 10 | 8 | 0 | Jupiter | — |
| Thunderplate Armor | legendary | 0 | 8 | 62 | 0 | 6 | 0 | Jupiter | — |
| Frostplate Armor | legendary | 0 | 0 | 64 | 12 | 0 | 90 | Mercury | — |
| Volcanic Plate | legendary | 0 | 14 | 66 | 0 | 0 | 60 | Mars | — |
| Terra Guard Armor | mythril | 0 | 0 | 50 | 0 | 0 | 80 | Venus | — |
| Skywarden Mail | mythril | 0 | 0 | 48 | 8 | 10 | 0 | Jupiter | — |
| Oceanic Mail | mythril | 0 | 0 | 52 | 14 | 0 | 70 | Mercury | — |
| Pyro Mail | mythril | 0 | 16 | 54 | 0 | 0 | 0 | Mars | — |
| Lunar Armor | legendary | 0 | 0 | 58 | 18 | 0 | 0 | Mercury, Jupiter | — |
| Solar Armor | legendary | 0 | 18 | 70 | 0 | 0 | 120 | Venus | — |
| Cotton Shirt | basic | 30 | 0 | 3 | 0 | 0 | 5 | Mercury, Jupiter | — |
| Leather Vest | basic | 80 | 0 | 6 | 0 | 0 | 10 | Venus, Mars, Jupiter | — |
| Bronze Armor | bronze | 200 | 0 | 10 | 0 | 0 | 15 | Venus | — |
| Iron Armor | iron | 350 | 0 | 15 | 0 | 0 | 25 | Venus, Mars | iron-bulwark |
| Steel Armor | steel | 800 | 0 | 24 | 0 | 0 | 40 | Venus | steel-ward |
| Silver Armor | silver | 2000 | 0 | 35 | 0 | 0 | 60 | Venus | silver-shield |
| Glacial Robes | mythril | 4800 | 0 | 40 | 15 | 0 | 60 | Mercury | — |
| Mythril Armor | mythril | 5000 | 0 | 48 | 0 | 0 | 85 | Venus | steel-ward |
| Dragon Scales | legendary | 10000 | 0 | 62 | 0 | 0 | 110 | Venus | dragon-ward |
| Valkyrie Mail | artifact | 20000 | 0 | 78 | 0 | 0 | 140 | Venus | — |

## 5.3 HELM (14)

| Name | Tier | Cost | ATK | DEF | MAG | SPD | HP | Elements | Unlocks Ability |
|------|------|------|-----|-----|-----|-----|----|----------|----------------|
| Stormking Crown | legendary | 0 | 0 | 42 | 20 | 0 | 0 | Jupiter | storm-mastery |
| Frostqueen Tiara | legendary | 0 | 0 | 38 | 22 | 0 | 0 | Mercury | frost-mastery |
| Volcanic Visor | legendary | 0 | 12 | 45 | 0 | 0 | 30 | Mars | — |
| Leather Cap | basic | 25 | 0 | 2 | 0 | 0 | 0 | Venus, Mars, Jupiter | — |
| Cloth Cap | basic | 60 | 0 | 4 | 0 | 0 | 0 | Mercury, Jupiter | — |
| Bronze Helm | bronze | 140 | 0 | 6 | 0 | 0 | 0 | Venus, Mars | — |
| Iron Helm | iron | 220 | 0 | 9 | 0 | 0 | 0 | Venus | iron-mind |
| Steel Helm | steel | 500 | 0 | 14 | 0 | 0 | 0 | Venus | steel-focus |
| Silver Circlet | silver | 1300 | 0 | 20 | 5 | 0 | 0 | Mercury, Jupiter | — |
| Mythril Crown | mythril | 3200 | 0 | 28 | 8 | 0 | 0 | Mercury, Jupiter | mythril-wisdom |
| Gaia Helm | mythril | 3400 | 0 | 30 | 0 | 0 | 25 | Venus | gaia-fortitude |
| Storm Circlet | mythril | 3600 | 0 | 26 | 12 | 4 | 0 | Jupiter | storm-focus |
| Oracle's Crown | legendary | 8000 | 0 | 38 | 14 | 0 | 0 | Jupiter | oracle-vision |
| Glory Helm | artifact | 16000 | 0 | 50 | 18 | 0 | 0 | Venus | — |

## 5.4 BOOTS (10)

| Name | Tier | Cost | ATK | DEF | MAG | SPD | HP | Elements | Unlocks Ability |
|------|------|------|-----|-----|-----|-----|----|----------|----------------|
| Windstrider Boots | legendary | 0 | 0 | 15 | 0 | 18 | 0 | Jupiter | — |
| Tidal Treads | legendary | 0 | 0 | 18 | 8 | 12 | 0 | Mercury | — |
| Leather Boots | basic | 70 | 0 | 0 | 0 | 2 | 0 | Venus, Mars, Jupiter | — |
| Iron Boots | iron | 150 | 0 | 2 | 0 | 3 | 0 | Venus | — |
| Steel Greaves | steel | 400 | 0 | 4 | 0 | 5 | 0 | Venus | — |
| Silver Greaves | silver | 1100 | 0 | 6 | 0 | 7 | 0 | Venus | — |
| Mythril Greaves | mythril | 2600 | 0 | 10 | 0 | 9 | 0 | Venus, Mars | — |
| Hyper Boots | mythril | 2800 | 0 | 8 | 0 | 10 | 0 | Jupiter | hyper-speed |
| Quick Boots | legendary | 6500 | 0 | 10 | 0 | 14 | 0 | Jupiter | — |
| Hermes' Sandals | artifact | 14000 | 0 | 12 | 0 | 20 | 0 | Jupiter | — |

## 5.5 ACCESSORY (18)

| Name | Tier | Cost | ATK | DEF | MAG | SPD | HP | Elements | Unlocks Ability |
|------|------|------|-----|-----|-----|-----|----|----------|----------------|
| Gaia Greatshield | legendary | 0 | 0 | 45 | 0 | 0 | 80 | Venus | — |
| Tower Champion's Ring | artifact | 0 | 20 | 20 | 20 | 10 | 0 | Venus, Mars, Mercury, Jupiter | — |
| Tower Master's Medallion | artifact | 0 | 25 | 25 | 25 | 15 | 100 | Venus, Mars, Mercury, Jupiter | — |
| Power Ring | basic | 100 | 5 | 0 | 0 | 0 | 0 | Venus, Mars, Jupiter | — |
| Guardian Ring | basic | 120 | 0 | 5 | 0 | 0 | 0 | Venus, Jupiter | — |
| Jester's Armlet | bronze | 240 | 0 | 0 | 4 | 4 | 0 | Jupiter, Mercury | — |
| Adept's Ring | bronze | 250 | 0 | 0 | 6 | 0 | 0 | Venus | — |
| War Gloves | iron | 400 | 10 | 3 | 0 | 0 | 0 | Mars | — |
| Spirit Gloves | steel | 900 | 0 | 0 | 12 | 0 | 0 | Mercury, Jupiter | — |
| Lucky Medal | silver | 1800 | 0 | 0 | 0 | 5 | 0 | Jupiter | — |
| Earth Warden Shield | silver | 1900 | 0 | 18 | 0 | 0 | 40 | Venus | earth-wall |
| Inferno Gauntlets | mythril | 4000 | 18 | 6 | 10 | 0 | 0 | Mars | inferno-fist |
| Mythril Gauntlets | mythril | 4200 | 16 | 12 | 0 | 0 | 0 | Venus, Mars | mythril-strike |
| Elemental Star | mythril | 4500 | 0 | 0 | 18 | 0 | 0 | Mercury, Jupiter | mythril-wisdom |
| Dragon's Eye | legendary | 8500 | 15 | 10 | 15 | 0 | 0 | Venus, Jupiter | — |
| Cleric Ring | legendary | 9000 | 0 | 0 | 12 | 0 | 0 | Mercury, Jupiter | — |
| Iris Robe | artifact | 12000 | 0 | 20 | 20 | 0 | 0 | Mercury, Jupiter | — |
| Cosmos Shield | artifact | 17000 | 0 | 30 | 0 | 0 | 50 | Venus | silver-shield |

---

# 6. DJINN (23)

## 6.1 Stats & Summons

| Name | Element | Tier | ATK | DEF | MAG | SPD | HP | Summon Dmg |
|------|---------|------|-----|-----|-----|-----|----|------------|
| Breeze | Jupiter | 1 | 0 | 0 | 0 | 0 | 0 | 110 |
| Gust | Jupiter | 1 | 0 | 0 | 0 | 8 | 0 | 0 |
| Squall | Jupiter | 2 | 0 | 0 | 0 | 0 | 0 | 160 |
| Bolt | Jupiter | 2 | 0 | 0 | 0 | 0 | 0 | 200 |
| Storm | Jupiter | 3 | 0 | 0 | 0 | 0 | 0 | 0 |
| Tempest | Jupiter | 3 | 0 | 0 | 0 | 0 | 0 | 350 |
| Eclipse | Jupiter | 3 | 0 | 0 | 0 | 0 | 0 | 0 |
| Forge | Mars | 1 | 0 | 0 | 0 | 0 | 0 | 120 |
| Ember | Mars | 1 | 0 | 0 | 0 | 0 | 0 | 46 |
| Corona | Mars | 2 | 0 | 0 | 8 | 0 | 0 | 0 |
| Fury | Mars | 3 | 0 | 0 | 0 | 0 | 0 | 220 |
| Nova | Mars | 3 | 0 | 0 | 0 | 0 | 0 | 320 |
| Scorch | Mars | 3 | 0 | 0 | 0 | 0 | 0 | 280 |
| Fizz | Mercury | 1 | 0 | 0 | 0 | 0 | 0 | 100 |
| Tonic | Mercury | 2 | 0 | 0 | 0 | 0 | 0 | 0 |
| Surge | Mercury | 2 | 0 | 0 | 0 | 0 | 0 | 180 |
| Crystal | Mercury | 3 | 0 | 0 | 12 | 0 | 0 | 0 |
| Chill | Mercury | 3 | 0 | 0 | 0 | 0 | 0 | 0 |
| Flint | Venus | 1 | 0 | 0 | 0 | 0 | 0 | 80 |
| Rockling | Venus | 1 | 0 | 0 | 0 | 0 | 0 | 46 |
| Granite | Venus | 2 | 0 | 10 | 0 | 0 | 0 | 0 |
| Bane | Venus | 3 | 0 | 0 | 0 | 0 | 0 | 300 |
| Serac | Venus | 3 | 0 | 15 | 0 | 0 | 0 | 0 |

## 6.2 Djinn Ability Pairs (SET vs STANDBY by compatibility)

### Bolt (Jupiter)
- **same**: SET=[bolt-thunder-slash, bolt-lightning-strike] / STANDBY=[—]
- **neutral**: SET=[bolt-spark-pulse] / STANDBY=[—]
- **counter**: SET=[—] / STANDBY=[bolt-plasma-strike, bolt-fire-lightning]

### Breeze (Jupiter)
- **same**: SET=[breeze-gale-force, breeze-wind-veil] / STANDBY=[—]
- **neutral**: SET=[breeze-eddy-pulse] / STANDBY=[—]
- **counter**: SET=[—] / STANDBY=[breeze-ice-burst, breeze-aero-shield]

### Eclipse (Jupiter)
- **same**: SET=[eclipse-wind-slash, eclipse-storm-strike] / STANDBY=[—]
- **neutral**: SET=[eclipse-shadow-pulse] / STANDBY=[—]

### Gust (Jupiter)
- **same**: SET=[gust-swift-strike, gust-wind-slash] / STANDBY=[—]
- **neutral**: SET=[gust-breeze-pulse] / STANDBY=[—]
- **counter**: SET=[—] / STANDBY=[gust-flame-wind, gust-ember-gale]

### Squall (Jupiter)
- **same**: SET=[squall-storm-burst, squall-hurricane-guard] / STANDBY=[—]
- **neutral**: SET=[squall-cloud-strike] / STANDBY=[—]
- **counter**: SET=[—] / STANDBY=[squall-wave-blast, squall-frost-surge]

### Storm (Jupiter)
- **same**: SET=[storm-chain-gale, storm-tempest-shield] / STANDBY=[—]
- **neutral**: SET=[storm-breeze-pulse] / STANDBY=[—]
- **counter**: SET=[—] / STANDBY=[storm-cold-hail, storm-icy-barrier]

### Tempest (Jupiter)
- **same**: SET=[tempest-hurricane-blade, tempest-cyclone-slash] / STANDBY=[—]
- **neutral**: SET=[tempest-gale-pulse] / STANDBY=[—]
- **counter**: SET=[—] / STANDBY=[tempest-fire-tornado, tempest-inferno-cyclone]

### Corona (Mars)
- **same**: SET=[corona-scorch, corona-ember-veil] / STANDBY=[—]
- **neutral**: SET=[corona-solar-spin] / STANDBY=[—]
- **counter**: SET=[—] / STANDBY=[corona-earthbreaker, corona-mountain-wall]

### Ember (Mars)
- **same**: SET=[fire-burst] / STANDBY=[—]

### Forge (Mars)
- **same**: SET=[forge-flame-strike, forge-inferno-blaze] / STANDBY=[—]
- **neutral**: SET=[forge-ember-wave] / STANDBY=[—]
- **counter**: SET=[—] / STANDBY=[forge-stone-slam, forge-granite-shield]

### Fury (Mars)
- **same**: SET=[fury-heat-rush, fury-flare-guard] / STANDBY=[—]
- **neutral**: SET=[fury-wind-flare] / STANDBY=[—]
- **counter**: SET=[—] / STANDBY=[fury-stone-barrage, fury-terra-gate]

### Nova (Mars)
- **same**: SET=[flame-wall, inferno-slash] / STANDBY=[—]
- **neutral**: SET=[flint-ground-shield] / STANDBY=[—]
- **counter**: SET=[—] / STANDBY=[granite-magma-barrier, granite-volcanic-spike]

### Scorch (Mars)
- **same**: SET=[scorch-inferno, scorch-lava-burst] / STANDBY=[—]
- **neutral**: SET=[scorch-heat-pulse] / STANDBY=[—]

### Chill (Mercury)
- **same**: SET=[chill-absolute-heal, chill-frost-revival] / STANDBY=[—]
- **neutral**: SET=[chill-frost-pulse] / STANDBY=[—]
- **counter**: SET=[—] / STANDBY=[chill-frozen-flame, chill-glacial-fire]

### Crystal (Mercury)
- **same**: SET=[crystal-crystal-ray, crystal-aqua-armor] / STANDBY=[—]
- **neutral**: SET=[crystal-mist-veil] / STANDBY=[—]
- **counter**: SET=[—] / STANDBY=[crystal-storm-crystal, crystal-gale-glaze]

### Fizz (Mercury)
- **same**: SET=[fizz-healing-wave, fizz-wave-shield] / STANDBY=[—]
- **neutral**: SET=[fizz-aqua-pulse] / STANDBY=[—]
- **counter**: SET=[—] / STANDBY=[fizz-gale-freeze, fizz-storm-chill]

### Surge (Mercury)
- **same**: SET=[surge-healing-tide, surge-aqua-barrier] / STANDBY=[—]
- **neutral**: SET=[surge-aqua-pulse] / STANDBY=[—]
- **counter**: SET=[—] / STANDBY=[surge-steam-blast, surge-boiling-wave]

### Tonic (Mercury)
- **same**: SET=[tonic-frost-bite, tonic-aqua-veil] / STANDBY=[—]
- **neutral**: SET=[tonic-soothing-wave] / STANDBY=[—]
- **counter**: SET=[—] / STANDBY=[tonic-wind-freeze, tonic-gale-barrier]

### Bane (Venus)
- **same**: SET=[bane-earthquake, bane-terra-guard] / STANDBY=[—]
- **neutral**: SET=[bane-rock-pulse] / STANDBY=[—]
- **counter**: SET=[—] / STANDBY=[bane-lava-fist, bane-molten-armor]

### Flint (Venus)
- **same**: SET=[flint-stone-fist, flint-granite-guard] / STANDBY=[—]
- **neutral**: SET=[flint-earth-pulse] / STANDBY=[—]
- **counter**: SET=[—] / STANDBY=[flint-lava-stone, flint-magma-shield]

### Granite (Venus)
- **same**: SET=[granite-earth-wall, granite-terra-break] / STANDBY=[—]
- **neutral**: SET=[granite-ground-pulse] / STANDBY=[—]
- **counter**: SET=[—] / STANDBY=[granite-magma-barrier, granite-volcanic-spike]

### Rockling (Venus)
- **same**: SET=[earth-spike] / STANDBY=[—]
- **neutral**: SET=[earth-spike] / STANDBY=[—]

### Serac (Venus)
- **same**: SET=[serac-stone-wall, serac-terra-guard] / STANDBY=[—]
- **neutral**: SET=[serac-earth-pulse] / STANDBY=[—]

---

# 7. ENCOUNTERS (55)

| ID | Name | Difficulty | Enemies | XP | Gold | Recruit | Djinn | Equipment |
|----|------|-----------|---------|----|----- |---------|-------|-----------|
| house-01 | House 1: Garet's Liberation | None | garet-enemy | None | None | — | — | — |
| vs1-garet | VS1: Garet's Liberation | None | garet-enemy | None | None | — | — | — |
| house-02 | House 2: The Bronze Trial | easy | earth-scout, venus-wolf | None | None | — | — | — |
| house-03 | House 3: Iron Bonds | easy | flame-scout, mars-wolf | None | None | — | — | — |
| house-04 | House 4: Arcane Power | easy | frost-scout, frost-mystic | None | None | — | — | — |
| house-05 | House 5: The Blazing Warrior | easy | gale-scout, gale-priest | None | None | — | — | — |
| house-06 | House 6: The Steel Guardian | medium | stone-guardian, ember-cleric, flame-s... | 120 | 32 | — | — | steel-helm |
| house-07 | House 7: Winds of Liberation | medium | terra-soldier, venus-bear, earth-shaman | 150 | 40 | — | breeze | — |
| house-08 | House 8: The Frozen Sentinel | medium | jupiter-bear, wind-soldier, tide-ench... | 200 | 55 | — | fizz | steel-armor |
| house-09 | House 9: Inferno's Rage | medium | mercury-bear, frost-oracle, ice-eleme... | 215 | 58 | — | — | battle-axe |
| house-10 | House 10: The Burning Gauntlet | medium | blaze-soldier, mars-bear, flame-eleme... | 235 | 62 | — | — | silver-circlet |
| house-11 | House 11: The Scholar's Trial | hard | stone-captain, rock-elemental, terra-... | 255 | 68 | karis | — | silver-armor |
| house-12 | House 12: The Granite Fortress | hard | inferno-captain, phoenix, flame-herald | 275 | 72 | — | granite | valkyrie-mail |
| house-13 | House 13: The Silver Strike | hard | glacier-captain, leviathan | 295 | 76 | — | — | — |
| house-14 | House 14: The Speed Demon | hard | thunder-captain, thunderbird | 320 | 82 | tyrell | — | hyper-boots |
| house-15 | House 15: The Storm Unleashed | hard | terra-soldier, blaze-soldier, wind-so... | 400 | 110 | — | squall | — |
| house-16 | House 16: The Mythril Edge | boss | lightning-commander, storm-elemental,... | 450 | 120 | — | — | mythril-blade |
| house-17 | House 17: The Master's Arrival | boss | mountain-commander, basilisk, rock-el... | 500 | 130 | felix | — | dragon-scales |
| house-18 | House 18: The Earth's Bane | boss | fire-commander, volcano-warlord | 550 | 140 | — | bane | oracles-crown |
| house-19 | House 19: The Final Armament | boss | storm-commander, hydra | 600 | 150 | — | — | — |
| house-20 | House 20: The Overseer Falls | boss | overseer, chimera, tempest-warlord | 1500 | 300 | — | storm | — |
| house-21 | House 21: The Risen Dead | medium | skeleton-warrior, ghost-wisp, zombie-... | 650 | 160 | — | — | — |
| house-22 | House 22: Wings of Fury | medium | wind-hawk, fire-eagle, storm-raven | 700 | 170 | — | — | hyper-boots |
| house-23 | House 23: The Earthen Guardians | hard | clay-golem, iron-golem | 750 | 185 | — | corona | — |
| house-24 | House 24: Frozen Depths | hard | frost-serpent, aqua-drake, ice-owl | 800 | 195 | — | — | — |
| house-25 | House 25: Storm's Wrath | hard | lightning-lynx, cyclone-djinni, thund... | 850 | 205 | — | tonic | — |
| house-26 | House 26: Necromantic Rites | boss | bone-mage, skeleton-warrior, ghost-wi... | 900 | 220 | — | — | — |
| house-27 | House 27: Crystal Convergence | boss | crystal-golem, storm-golem, iron-golem | 950 | 235 | — | — | gaia-blade |
| house-28 | House 28: Draconic Convergence | boss | glacier-wyrm, tempest-dragon, hydra | 1000 | 250 | — | fury | — |
| house-29 | House 29: Abyssal Depths | boss | tidal-wraith, neptune-warden, frost-s... | 1100 | 270 | — | — | — |
| house-30 | House 30: Volcanic Summit | boss | magma-colossus, flame-elemental, fire... | 1200 | 290 | — | scorch | — |
| house-31 | House 31: Frozen Citadel | boss | permafrost-golem, polar-guardian, arc... | 1300 | 310 | — | — | — |
| house-32 | House 32: Stratosphere Keep | boss | stratosphere-lord, thunderstorm-colos... | 1400 | 330 | — | crystal | — |
| house-33 | House 33: Chimera's Lair | boss | voltage-chimera, chimera, elder-basilisk | 1500 | 350 | — | — | — |
| house-34 | House 34: Spectral Void | boss | frost-lich, void-specter, bone-mage, ... | 1600 | 370 | — | — | oracles-crown |
| house-35 | House 35: Elemental Convergence | boss | aurora-elemental, storm-elemental, fl... | 1800 | 400 | — | serac | — |
| house-36 | House 36: Divine Judgment | boss | zeus-avatar, celestial-fury, vortex-s... | 2500 | 500 | — | eclipse | — |
| training-dummy | Training Arena | easy | mercury-slime | 10 | 0 | — | — | — |
| roadside-bandits | Roadside Bandits | easy | bandit, scavenger | 25 | 12 | — | — | — |
| merchant-guard | Merchant Guard | easy | merchant-guard | 40 | 20 | — | — | sol-blade |
| abandoned-farm | Abandoned Farm | easy | wild-boar, carrion-bird | 30 | 15 | — | — | — |
| house-37 | House 37: Granite Guard | hard | granite-warlord, granite-warlord | 3000 | 600 | — | — | — |
| house-38 | House 38: Magma Twins | hard | volcano-warlord, volcano-warlord | 3200 | 650 | — | — | — |
| house-39 | House 39: Frozen Duo | hard | blizzard-warlord, blizzard-warlord | 3400 | 700 | — | — | — |
| house-40 | House 40: Storm Gate | boss | tempest-warlord, tempest-dragon | 4000 | 1000 | — | — | — |
| house-41 | House 41: Sky Breach | hard | stratosphere-lord | 4200 | 800 | — | — | — |
| house-42 | House 42: Cloud Walker | hard | storm-titan | 4400 | 850 | — | — | — |
| house-43 | House 43: Thunder Peak | hard | tempest-dragon, storm-titan | 4600 | 900 | — | — | — |
| house-44 | House 44: Gale Force | hard | stratosphere-lord, jupiter-vortex-sentry | 4800 | 950 | — | — | — |
| house-45 | House 45: The Eye | boss | zeus-avatar | 5000 | 2000 | — | — | mythril-crown |
| house-46 | House 46: Chaos 1 | hard | granite-warlord, tempest-warlord | 5500 | 1100 | — | — | — |
| house-47 | House 47: Chaos 2 | hard | volcano-warlord, blizzard-warlord | 6000 | 1200 | — | — | — |
| house-48 | House 48: Chaos 3 | hard | tempest-dragon, zeus-avatar | 7000 | 1500 | — | — | — |
| house-49 | House 49: The Gatekeeper | boss | celestial-fury, vortex-sentinel | 8000 | 2000 | — | — | — |
| house-50 | House 50: Golden Sun | boss | the-golden-sun | 99999 | 99999 | — | — | sol-blade |

---

# 8. DATA TOTALS

| Category | Count | Source |
|----------|-------|--------|
| Units | 11 | data_unit_abilities.json |
| Abilities | 241 | data_abilities.json |
| Enemies | 137 | data_enemies.json |
| Equipment | 109 | data_equipment.json |
| Djinn | 23 | data_djinn.json |
| Djinn Ability Pairs | 23 | data_djinn_abilities.json |
| Encounters | 55 | data_encounters.json |
| Battle Backgrounds | 72 | sprites/backgrounds/ |
| Sprite Files | 2,890 | vale-village (original repo) |
