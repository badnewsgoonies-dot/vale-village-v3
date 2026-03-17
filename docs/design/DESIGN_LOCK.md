# VALE VILLAGE — Design Decisions Lock (v1)
## Post-Assessment Corrections — March 17, 2026

Everything below is LOCKED (confirmed by Geni, March 17, 2026).

---

# LOCKED DECISIONS

## 1. Actions: Exactly Two Verbs

- **ATTACK**: Auto-attack. 0 cost. Generates +1 mana per hit. Advances crit +1 per hit.
- **ABILITY**: Costs mana. Does NOT generate mana. Does NOT advance crit.

There is no "Psynergy" category. Abilities just cost mana. That's the only resource.
There is no Defend. There is no Items-in-combat. There is no Summon button.

Summons are accessed through the **Djinn menu** (see §5).

## 2. Mana System — Planning = Execution Order

**The unit selection order during planning IS the execution order.** Same sequence.

### How planning works:
1. Select Unit 1's action. If ABILITY → mana drains from pool. If ATTACK → mana circle fills (+1 per expected hit, shown in **different color** = "projected mana from auto").
2. Select Unit 2's action. Pool updates. Projected mana from Unit 1's auto is now available in a visually distinct color.
3. Select Unit 3's action. Same.
4. Select Unit 4's action. Same.
5. Review. **Backtrack and adjust as many times as you want.** Nothing is committed until you confirm all 4.
6. Confirm → Execute. Actions resolve in the same order you selected them.

### Key mana rule:
If Unit 2 is set to ATTACK (generating +1 projected mana) and Unit 3's ability depends on that mana — **if Unit 2 dies before their turn, Unit 3's move fails.** The projected mana was conditional on Unit 2 surviving.

### Visual language:
- Solid filled circles = guaranteed mana (from pool reset)
- Different colored circles = projected mana (from planned auto-attacks, conditional on survival)

## 3. Status Effects — Fully Deterministic

**No percentages. No chances. No randomness anywhere.**

Every status effect either applies on hit (always) or doesn't exist. The `chance` field in v2 data must be converted:

### Status Types (LOCKED):

| Status | Effect | Duration | Restriction |
|--------|--------|----------|-------------|
| **Stun** | Cannot act at all | X turns | No abilities, no auto-attacks |
| **Null** | Can only auto-attack | X turns | Abilities disabled, auto-attacks work |
| **Incapacitate** | Can only use abilities | X turns | Auto-attacks disabled, abilities work |
| **Burn** | HP damage per turn, scales with max HP | X turns | e.g., 10% maxHP/turn |
| **Poison** | HP damage per turn, scales with MISSING HP | X turns | Gets worse as unit gets weaker |
| **Freeze** | Skip turn, breaks after N cumulative damage | Until broken | Deterministic: threshold set per ability |

**This is the full list. 6 statuses. Not a centerpiece feature — just enough to add tactical texture.**

### Conversion rule for v2 data:
- Any ability with `statusEffect.chance < 1.0` → becomes `always: true` (applies on hit, always)
- OR gets cut entirely and replaced with a different effect
- The `chance` field is DELETED from the schema

### Freeze mechanic (LOCKED):
- Freeze = skip turn until cumulative damage received exceeds a threshold
- Threshold set per ability (e.g., "Freeze Blast: freeze, breaks after 40 damage")
- Light freeze = low threshold (breaks easily), Heavy freeze = high threshold
- Deterministic: player can calculate exactly how many hits to free an ally or keep an enemy locked

### Burn vs Poison (LOCKED):
- **Burn** = flat % of max HP per turn (e.g., 10% maxHP). Hurts tanks and squishies equally in proportion.
- **Poison** = scales with MISSING HP (e.g., 5% of missing HP per turn). Gets worse as unit weakens. Finishing pressure.
- Both are deterministic (% of a known value = known damage). No randomness.

## 4. Djinn — Two States, Ability Oscillation via Recovery

**Two states: GOOD and RECOVERY. Abilities SWAP between states.**

| State | Stat Bonus | Same/Counter Abilities | Can Activate |
|-------|-----------|----------------------|-------------|
| **GOOD** | Yes | **Ability Set A** (GOOD-mode abilities) | Yes → fires immediate effect + enters RECOVERY |
| **RECOVERY** | No | **Ability Set B** (RECOVERY-mode abilities) | No (recovering) |

### How activation works:
1. Djinn is GOOD → player clicks it via djinn menu
2. Djinn fires an **immediate effect** (damage, buff, heal, etc.)
3. Djinn enters RECOVERY → stat bonus gone, **abilities SWAP to Set B**
4. Recovery: **1 djinn recovers per turn, starting the turn after next**
5. Recovery order = activation order (first spent = first recovered)
6. When recovered → returns to GOOD (stats return, **abilities swap back to Set A**)

### The oscillation:
Activating a djinn doesn't just remove abilities — it **replaces them**. Your kit changes shape.

**3-Djinn Summon Example:**
```
Turn 3:  Summon fires. All 3 djinn → RECOVERY.
         You LOSE all 3 "Set A" (GOOD) abilities.
         You GAIN all 3 "Set B" (RECOVERY) abilities.
         Stat bonuses from all 3 djinn are gone.

Turn 4:  Nothing recovers yet. Still running Set B abilities, no stat bonuses.

Turn 5:  Djinn 1 → GOOD. Its abilities flip back to Set A. Stats return.
         Djinn 2 & 3 still RECOVERY (Set B abilities, no stats).

Turn 6:  Djinn 2 → GOOD. Flips back. Only Djinn 3 still in RECOVERY.

Turn 7:  Djinn 3 → GOOD. All restored to Set A. Full stats back.
```

The 4-turn window (turns 3-6) where your kit is partially or fully in "RECOVERY mode" is the real cost of summoning — and it's also an opportunity if the RECOVERY abilities are what you want right now.

### Ability grants by host element compatibility (LOCKED):

| Compatibility | When GOOD (Set A) | When RECOVERY (Set B) |
|--------------|-------------------|----------------------|
| **Same element** | 2 abilities | 2 different abilities |
| **Counter element** | 2 abilities | 2 different abilities |
| **Neutral** | 1 ability | **Same ability (always-on)** |

- **Same-element host**: Best stat bonus. 2 strong GOOD abilities, 2 different RECOVERY abilities. The swap matters — you plan around which set you need when.
- **Counter-element host**: Stat penalty. 2 GOOD abilities (situational), 2 RECOVERY abilities (powerful). The counter-type payoff is that RECOVERY mode gives you the strong abilities — spending djinn is actually beneficial for your kit.
- **Neutral host**: Small stat bonus. 1 ability that **persists through both states**. No oscillation. The safe, consistent choice — you never lose your ability, but you only get one and a smaller stat bonus.

### Design implications:
- Same-element is the "standard" choice — reliable stats, two ability sets to manage
- Counter-element is the "gambler" choice — you WANT to activate because RECOVERY abilities are stronger, but you pay a stat penalty while GOOD
- Neutral is the "rock" choice — one ability that never goes away, small steady bonus, no decision complexity
- Smart players will mix: same-element for sustained value, counter for summon turns, neutral for must-have abilities

## 5. Djinn Menu — Where Summons Live

Summons are NOT in the ability list. Summons are NOT a third action verb.

**When you click a djinn sprite on the battlefield, a DJINN MENU opens showing:**
- All equipped djinn with current state (GOOD / RECOVERY + turns remaining)
- Summon options available (based on GOOD djinn count)
- Tap a summon → that unit's action for this round becomes the summon

So the flow is:
1. It's Unit 3's turn to queue an action
2. Player clicks a djinn sprite on Unit 3
3. Djinn menu opens: shows 3 djinn slots, states, available summons
4. Player selects a summon → Unit 3's action is now "Summon: [name]"
5. The djinn used will enter RECOVERY after execution

**Summons still execute before speed order** (supersede all other actions).

## 6. Equipment Slots — 5 Slots (confirmed)

| Slot | Primary Stats |
|------|--------------|
| Weapon | ATK, hit count, element |
| Helm | DEF, MAG |
| Armor | DEF, HP |
| Boots | SPD, DEF |
| Accessory | Varies (unique effects) |

**Items SHOULD feature many abilities**, especially weapons. Equipment abilities are a core system, not a minority feature. The concern about "muddy action sources" is addressed by clear UI labeling (equipment icon vs. character icon vs. djinn icon on each ability in the list).

Existing v2 data (109 items across 5 slots) carries over as-is.

## 7. Enemy Information — Health Bars Only

**Show enemy health bars.** No deep stat details, no intent telegraphing.

### What IS visible:
- Enemy sprites and names (always)
- Enemy element (via visual design / color)
- **Enemy HP bar** (current / max)
- Damage numbers you deal (floating numbers)

### What is NOT visible:
- Enemy ATK/DEF/MAG/SPD stats
- Enemy ability list
- Enemy intent (what they plan to do this turn)
- Exact HP number (just the bar, not "142/200")

The player learns enemy patterns through experience. First time fighting a new enemy type, you don't know what it does. Second time, you remember.

## 8. UI Screens (Outside Battle)

Confirmed screens:
- **Player UI**: Character details, backstory, stats
- **Shop**: Buy/sell equipment
- **Equipment Page**: Manage loadout (5 slots per unit)
- **Djinn Page**: Manage djinn assignment across party
- **Abilities Page**: View all abilities per unit (character + equipment + djinn-granted)

These are separate from the pre-battle screen, which is the "one-screen" preparation view before entering combat.

## 9. Crit System — LOCKED

- Every 10th auto-attack HIT (not action, HIT) = guaranteed crit
- Per-unit counter, displayed as X/10 on portrait
- **Crit multiplier: 2×** (adopting GPT's recommendation)
- Resets per battle
- Only auto-attacks advance the counter, not abilities

## 10. Barriers & Heal-over-Time (LOCKED)

### Barriers (damage blockers):
- A buff applied by a unit or teammate
- Blocks damage **per instance** (not per point)
- Each barrier charge absorbs one full hit regardless of damage amount
- Multi-hit attacks consume one charge per hit (e.g., double strike vs 1 barrier = first hit blocked, second hits HP)
- **Status damage (burn/poison) does NOT consume barrier charges** — barriers block attack damage only
- Barriers can stack from multiple sources (e.g., 2-charge from ability + 1-charge from equipment = 3 charges)
- Barriers expire if not consumed (duration-based, set per ability)
- Barriers do NOT block status effects applied by the blocked hit — just the damage

### Heal-over-Time (HoT):
- A proper mechanic. Restores X HP per turn for Y turns.
- Stacks with barriers (they serve different roles — HoT recovers, barriers prevent)
- HoT is a fixed amount per tick (e.g., "Regen: 12 HP/turn for 3 turns"), not % based

### The three defensive layers:
1. **HP** — your health pool. Restored by healing abilities.
2. **Barriers** — damage instance blockers. Applied by buff abilities. Consumed on hit.
3. **HoT** — passive recovery over time. Sustain through longer fights.

These create interesting choices: barrier a unit who's about to take a big single hit, HoT a unit who's taking steady small damage, direct heal a unit who's already low.

---

# CONVERSION NEEDED: v2 Data → Bevy Data

## Status effects: 241 abilities need audit
- Every `statusEffect.chance` field → delete, replace with `always: true`
- Burn: keep % maxHP (deterministic). Poison: convert to % missing HP.
- New status types (Null, Incapacitate) need to be designed and assigned to abilities

## Equipment: carry over as-is
- 5 slots confirmed (Weapon, Helm, Armor, Boots, Accessory)
- 109 existing items carry forward unchanged
- No new slots needed

## Djinn: ability pair redesign
- 2 states (GOOD, RECOVERY) with ability SWAP between states
- Same/Counter: 2 abilities in GOOD, 2 different abilities in RECOVERY
- Neutral: 1 ability, always-on regardless of state
- 23 djinn × 3 compatibility types = 69 ability pair assignments to review
- v2 data has `same: [a, b]`, `counter: [c, d]` — map A set to GOOD, B set to RECOVERY

## Element damage: REMOVE
- Delete element advantage/disadvantage multipliers from damage formulas
- Elements affect ONLY djinn compatibility and visual theming

## Barriers: NEW system
- v2 "shieldCharges" maps to barrier charges (per-damage-instance blockers)
- v2 "damageReductionPercent" → REMOVE (replaced by barrier system)
- Abilities that had % DR → convert to barrier charges or cut

---

# SUMMARY: All Decisions Locked

## LOCKED (implement as-is):
- [x] Two actions: ATTACK and ABILITY
- [x] Planning order = execution order
- [x] Mana visual: solid (guaranteed) vs different-colored (projected from autos)
- [x] No randomness anywhere — all status deterministic, always applies on hit
- [x] Djinn: 2 states (GOOD / RECOVERY) with ability SWAP between states
- [x] Same/Counter djinn: 2 GOOD abilities + 2 RECOVERY abilities
- [x] Neutral djinn: 1 ability, always-on through both states
- [x] Summons accessed via djinn menu (click sprite), not action bar
- [x] Recovery: 1/turn, starting turn after next, in activation order
- [x] Crit: 10th hit, 2× multiplier, per-unit, per-battle
- [x] Enemy HP bars visible, but no stats/intent/exact numbers
- [x] Equipment: 5 slots (weapon/helm/armor/boots/accessory)
- [x] Equipment abilities are core (not minority)
- [x] No element damage modifiers
- [x] 6 status effects: Stun, Null, Incapacitate, Burn, Poison, Freeze
- [x] Freeze breaks after N cumulative damage (threshold per ability)
- [x] Burn = % maxHP/turn. Poison = % missing HP/turn.
- [x] Barriers: per-instance damage blockers, stackable, duration-based, don't block status damage
- [x] HoT: proper mechanic, fixed HP/turn for N turns
- [x] Three defensive layers: HP, Barriers, HoT

## REMAINING DESIGN WORK (not open questions — just content authoring):
- [ ] Convert 241 abilities: remove `chance` fields, assign deterministic rules
- [ ] Convert v2 `damageReductionPercent` abilities to barrier-charge abilities
- [ ] Assign Null/Incapacitate status to appropriate abilities (currently all are stun/burn/poison/freeze/paralyze→cut)
- [ ] Design 23 × 3 djinn ability pairs (GOOD set + RECOVERY set per compatibility)
- [ ] Balance burn/poison % values per ability tier
- [ ] Set freeze damage thresholds per ability tier
