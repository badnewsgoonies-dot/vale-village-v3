# Vale Village v3 — Canonical Game Spec

> Single source of truth. When this file contradicts any other document, this file wins.
> Other docs in docs/design/ and docs/session-notes/ are v2 reference material — superseded where they disagree.

---

## 1. One-Sentence Pitch

A deterministic, queue-based tactical RPG where ATTACK prints mana and crit, ABILITY spends them, enemy intent is hidden, and djinn activation temporarily rewires each unit's available kit.

---

## 2. Core Pillars

1. **Deterministic combat** — no randomness. No random crits, misses, or damage variance. Player skill determines outcomes.
2. **Low numbers** — single/double digit stats. Every point matters (~10% of a stat). Mental math instant.
3. **Meaningful choices** — no dominant strategies. Multiple viable builds. Trade-offs for every decision.

---

## 3. Actions

Exactly two verbs:

| Action | Cost | Generates mana? | Advances crit? |
|--------|------|-----------------|----------------|
| **ATTACK** | 0 | +1 per hit | +1 per hit |
| **ABILITY** | 1-5 mana | No | No |

No Defend. No Items-in-combat. No Summon button.
Summons are accessed through the djinn menu (click djinn sprite).
Djinn activation is also through the djinn menu.

---

## 4. Planning & Execution

**Planning order = execution order.** The order you select unit actions is the order they execute. NOT SPD-sorted.

1. Select Unit 1's action → mana display updates
2. Select Unit 2's action → mana display updates
3. Select Unit 3's action → mana display updates
4. Select Unit 4's action → mana display updates
5. Backtrack and adjust freely. Nothing committed until confirm.
6. Confirm → execute in the same order you selected.

**Visual mana:**
- Solid circles = guaranteed mana (from pool reset)
- Different-colored circles = projected mana (from planned auto-attacks, conditional on survival)

**Key rule:** If Unit 2 is set to ATTACK (generating +1 projected mana) and Unit 3's ability depends on that mana — if Unit 2 dies before their turn, Unit 3's ability fails.

**Exception:** Summons execute BEFORE all other actions (supersede planning order).

---

## 5. Mana System

Team-wide shared pool. NOT per-unit PP.

```
max_mana = sum of living units' mana_contribution + equipment mana_bonus
```

- **Resets to max** each round (Slay the Spire model)
- Auto-attack hits generate +1 mana mid-execution (available to later units in planning order)
- Abilities cost mana, generate nothing

| Unit | Mana | Role |
|------|------|------|
| Adept | 1 | Tank |
| War Mage | 2 | Caster |
| Mystic | 2 | Healer |
| Ranger | 1 | Physical DPS |
| Sentinel | 1 | Support |
| Stormcaller | 3 | AoE mage |
| Blaze | 2 | Warrior-mage |
| Karis | 2 | Scholar |
| Tyrell | 1 | Pure DPS |
| Felix | 1 | Master warrior |

---

## 6. Crit System

- Every 10th auto-attack **HIT** (not action) = guaranteed crit
- **2× multiplier**
- Per-unit counter, displayed as X/10 on portrait
- Resets per battle
- Only auto-attacks advance the counter. Abilities do NOT.

---

## 7. Damage Formulas

```
Physical: basePower + ATK - (DEF × 0.5)    floor 1
Psynergy: basePower + MAG - (DEF × 0.3)    floor 1
Healing:  basePower + MAG                   floor 1
```

**No element damage modifiers.** Elements affect ONLY djinn compatibility and visual theming.

---

## 8. Status Effects (6 total, all deterministic)

| Status | Effect | Duration | Notes |
|--------|--------|----------|-------|
| **Stun** | Cannot act at all | X turns | No abilities, no auto-attacks |
| **Null** | Can only auto-attack | X turns | Abilities disabled |
| **Incapacitate** | Can only use abilities | X turns | Auto-attacks disabled |
| **Burn** | % of MAX HP per turn | X turns | e.g. 10% maxHP/turn. Flat scaling. |
| **Poison** | % of MISSING HP per turn | X turns | Gets worse as unit weakens. Finishing pressure. |
| **Freeze** | Skip turn | Until broken | Breaks when cumulative damage received ≥ threshold |

- **No chance field.** Everything applies on hit, always. No randomness.
- "Paralyze" from v2 → convert to Stun.
- "damageReductionPercent" from v2 → REMOVE. Replaced by barriers.

---

## 9. Defensive Layers (3)

### Barriers
- Per-instance damage blockers (not percentage reduction)
- Each charge absorbs one full hit regardless of damage amount
- Multi-hit attacks consume one charge per hit
- Status damage (burn/poison) does NOT consume barrier charges
- Stackable from multiple sources
- Duration-based (expire if not consumed)

### Heal-over-Time (HoT)
- Fixed HP per turn for N turns
- Stacks with barriers

### HP
- Direct healing restores HP

---

## 10. Djinn System

**Two states: GOOD and RECOVERY.**

| State | Stat Bonus | Abilities | Can Activate |
|-------|-----------|-----------|-------------|
| **GOOD** | Yes | Set A | Yes → fires effect + enters RECOVERY |
| **RECOVERY** | No | Set B | No |

### Activation
1. Djinn is GOOD → player activates via djinn menu
2. Immediate effect fires (damage, buff, heal, etc.)
3. Djinn enters RECOVERY → stat bonus gone, abilities swap to Set B

### Recovery timing
- **1 djinn recovers per turn, starting the turn after next**
- Recovery order = activation order (first activated = first recovered)
- With recovery_turns = 2: activated Turn N → decrement Turn N end → decrement Turn N+1 end → recover Turn N+2 end

### 3-Djinn Summon Example
```
Turn 3: Summon fires. All 3 djinn → RECOVERY.
Turn 4: Nothing recovers yet.
Turn 5: Djinn 1 → GOOD.
Turn 6: Djinn 2 → GOOD.
Turn 7: Djinn 3 → GOOD.
```

### Ability grants by compatibility

| Compatibility | When GOOD (Set A) | When RECOVERY (Set B) |
|--------------|-------------------|----------------------|
| **Same element** | 2 abilities | 2 different abilities |
| **Counter element** | 2 abilities | 2 different abilities (stronger) |
| **Neutral** | 1 ability | Same ability (always-on) |

Counter pairs: Venus ↔ Jupiter, Mars ↔ Mercury.

### Summons
- Accessed via djinn menu, not action bar
- Summons execute BEFORE all other actions
- Tier 1: 1 Good djinn. Tier 2: 2. Tier 3: 3.
- Any unit can summon from the team's Good djinn pool.

---

## 11. Equipment

5 slots:

| Slot | Primary Stats |
|------|--------------|
| Weapon | ATK, hit count, element |
| Helm | DEF, MAG |
| Armor | DEF, HP |
| Boots | SPD, DEF |
| Accessory | Varies |

- Element-gated (Venus items for Venus units, etc.)
- 51% of items unlock abilities — this is a core system
- Equipment can grant: stat bonuses, ability unlocks, mana bonus, hit count bonus, always-first-turn, set bonuses

---

## 12. Enemy Information

**Show:** enemy sprites, names, element (visual), HP bar (proportional, not exact numbers), damage numbers dealt.

**Do NOT show:** ATK/DEF/MAG/SPD stats, ability list, intent, exact HP number.

---

## 13. Buff/Debuff System

- Stat modifiers: ATK, DEF, MAG, SPD (positive or negative)
- **Max 3 stacks per stat per unit** (configurable via CombatConfig.max_buff_stacks)
- Duration-based
- **Must affect combat damage calculations** — effective stats = base + equipment + buff/debuff modifiers

---

## 14. Advanced Damage Modifiers

| System | Description |
|--------|-------------|
| Defense Penetration | Ignore X% of target DEF before damage calc |
| Splash Damage | Single-target ability deals X% to all other enemies |
| Chain Damage | Damage chains to all targets on opposing side, same damage |

---

## 15. Life/Death

- **Revive:** Resurrect KO'd unit at X% of max HP
- **Auto-Revive:** Status effect: on KO, auto-resurrect (uses-based)

---

## 16. Progression

- XP gained per battle from enemy XP values + encounter rewards
- Level up: stats increase by growth rates
- New abilities unlock at specific levels
- Max level: 20

---

## 17. Game Flow

### Story Mode
Linear sequence of encounters (house-01 through house-50). Each victory rewards units, djinn, equipment. Difficulty escalates.

### Battle Tower
Separate multi-floor challenge using current roster. For powering up when stuck in story.

---

## 18. Content Quantities

| Category | Count |
|----------|-------|
| Units | 11 |
| Abilities | 241 base + djinn/equipment stubs |
| Enemies | 137 |
| Equipment | 109 |
| Djinn | 23 |
| Encounters | 55 |

---

## 19. What This Spec Does NOT Cover (reference only)

These topics are in other docs for v2 reference but are NOT authoritative:

- `docs/design/SYSTEMS_FOUNDATION.md` — v2 system mapping. **S11 (Damage Reduction) is REMOVED in v3.** S04 SPD ordering description is WRONG for v3 (planning order = execution order). Poison formula is WRONG (should be missing HP, not max HP).
- `docs/design/DATA_MANIFEST.md` — data file structure proposal. Still useful for architecture reference.
- `docs/session-notes/SESSION_NOTES.md` — v2 deep dive. **3-state djinn model (SET/STANDBY/RECOVERY) is WRONG for v3.** Recovery timing is WRONG. Use this doc for game feel context only, not mechanical rules.
- `docs/data/COMPLETE_DATA.md` — raw v2 data tables. Still has `chance` fields, `damageReductionPercent`, `paralyze`. Data conversion handles these.
