# VALE VILLAGE — Complete Mechanics Session Notes
## Everything the game IS, from the v2 canon

---

## THE GAME IN ONE SENTENCE

A deterministic, queue-based tactical RPG where the entire strategic depth comes from how you assign and cycle djinn across your party — not from which spell you cast on which enemy.

---

## CORE DESIGN PILLARS (from balance-spec.md)

1. **Deterministic combat — NO randomness.** No random crits, no misses, no damage variance. Uncertainty comes from enemy AI behavior and decision complexity, not dice rolls. Player skill determines outcomes.
2. **Low numbers for clarity.** Single/double digit stats. Every point matters (~10% of a stat). Mental math should be instant.
3. **Meaningful choices.** No dominant strategies. Multiple viable builds. Trade-offs for every decision.

---

## THE MANA SYSTEM (team-wide shared pool)

This is NOT per-unit PP like Golden Sun. It's a shared team resource.

**How the pool is calculated:**
```
maxMana = sum of all active units' manaContribution
```

**Unit mana contributions:**
| Unit | Mana | Role reasoning |
|------|------|---------------|
| Adept | 1 | Tank — hits, doesn't cast |
| War Mage | 2 | Primary caster |
| Mystic | 2 | Healer/caster |
| Ranger | 1 | Physical DPS |
| Sentinel | 1 | Defensive support |
| Stormcaller | 3 | AoE mage powerhouse |
| Blaze | 2 | Balanced warrior-mage |
| Karis | 2 | Scholar mage |
| Tyrell | 1 | Pure physical DPS |
| Felix | 1 | Master warrior |

**Example:** Party of Adept(1) + War Mage(2) + Mystic(2) + Ranger(1) = **6 mana pool.**
Swap Ranger for Stormcaller(3) = **8 mana pool.** Party comp directly determines your casting budget.

**Mana rules (Slay the Spire model):**
- Pool **RESETS to max** at start of each round — NOT persistent
- Abilities cost mana from the shared pool (1-5 mana each)
- Auto-attacks cost 0 mana
- **Auto-attacks GENERATE +1 mana PER HIT during execution** — this is the mid-round bonus
- Mana generated mid-round is available to LATER units in SPD order
- All actions are planned/allocated before the round executes, but the execution order matters

**The SPD-order mana trick:**
Actions execute in speed order. If Unit 2 (fast, SPD 18) auto-attacks first, they generate +1 mana mid-execution. That mana becomes available for Unit 4 (slow, SPD 10) whose spell costs more than the base pool could afford. You plan this BEFORE the round — you know the speed order, so you can deliberately assign fast units to auto-attack to fund slow units' expensive spells.

**Example round:**
```
Base pool: 6 mana
Planning: Unit 1 (SPD 18) → auto-attack (0 cost)
          Unit 2 (SPD 15) → auto-attack (0 cost)  
          Unit 3 (SPD 12) → Heal (3 mana) → pool: 3 remaining
          Unit 4 (SPD 10) → Earthquake (5 mana) → would need 5, only 3 left...
          BUT: Units 1+2 auto-attack first (higher SPD), generating +2 mana
          → pool during execution: 3 + 2 = 5 → Unit 4's spell goes through
```

**The strategic tension:** Every unit you assign to auto-attack is one not casting a spell, but auto-attackers actively GROW the pool mid-round for your casters. Party composition determines base budget. Speed determines who generates vs who spends. Multi-hit auto-attacks multiply the generation.

---

## THE AUTO-ATTACK MECHANIC (the resource engine)

**ONLY auto-attacks generate mana and advance the crit counter.** Not abilities. Not psynergy. Not healing. Only auto-attacks. This is the core design decision that creates the tension.

Auto-attacks are the 0-cost physical actions (Strike, Heavy Strike, Guard Break, Precise Jab). The Q key is a shortcut to assign one quickly.

**What auto-attacks do that nothing else does:**
- Cost 0 mana
- Generate +1 mana PER HIT during execution (multi-hit auto-attacks = multi-mana)
- Each hit advances that unit's crit counter by 1 toward the 10-hit threshold
- The ONLY way to build crit and generate bonus mana

**What abilities do that auto-attacks don't:**
- Psynergy, healing, buffs, debuffs — all cost mana, none generate mana, none advance crit
- Higher base damage, AoE, status effects, healing — but at the cost of resources and crit progress

**This creates two distinct unit roles per round:**
- **Auto-attackers:** Generate mana, farm crits, deal physical damage. The engine.
- **Casters:** Spend mana, don't advance crit, deal magical damage / heal / buff. The payload.

Every round you're choosing how many engines vs how many payloads to run.

---

## THE QUEUE BATTLE SYSTEM

**NOT standard turn-by-turn.** You plan ALL actions simultaneously, then they all execute in speed order.

```
PLANNING PHASE:
  → Select action for Unit 1 (ATTACK or PSYNERGY)
  → Select action for Unit 2
  → Select action for Unit 3
  → Select action for Unit 4
  → Mana display updates LIVE as you queue:
    - Select ability → mana goes DOWN
    - Select auto-attack → mana goes UP (pre-accounting the +1 gen)
    - Player always sees correct available mana before committing
  → "Execute Round" when all slots filled

EXECUTION PHASE:
  → ALL queued actions (player + enemy) sorted by effective SPD
  → Execute in order: damage, status, KO checks after each
  → Auto-attacks generate +1 mana PER HIT (available to later units in SPD order)
  → Status effect ticks at end

ROUND END:
  → Djinn recovery checks (2-turn cooldown)
  → Victory/defeat check
  → Mana RESETS to max (Slay the Spire energy model)
  → Return to planning
```

**Only two action types:** ATTACK (auto-attack) and PSYNERGY (abilities). There is NO defend action and NO consumable items. Djinn are managed by clicking the djinn sprites directly on the battlefield, not through a menu option.

---

## BATTLE UI: KEY VISUAL DETAILS (from v2 screenshots)

Three critical UI elements that define the battle screen feel:

### 1. Crit Counter on Portraits
Each party member's portrait in the bottom HUD bar shows a **"X/10" badge** (e.g., "1/10", "0/10"). This IS the crit counter. No separate meter — it's embedded directly on the portrait so you always see it without looking away from your party status. When a unit auto-attacks, their badge increments. At 10/10 → crit fires → resets to 0/10.

### 2. Djinn Sitting on Character Sprites
Djinn are **physically rendered on top of party character sprites** during battle. They're small elemental creatures perched on or hovering near their assigned unit. You click the djinn sprite to interact (activate SET→STANDBY, view status, trigger summon). No menu navigation — direct spatial interaction on the battlefield.

### 3. Live Mana Pre-Tracking During Planning
The mana circle display updates **in real-time as you queue actions**, not after execution:
- Select PSYNERGY for a unit → mana circles drain (cost deducted)
- Select ATTACK for a unit → mana circles **fill** (pre-accounting the +1 per hit generation)
- Change your mind, cancel an action → mana adjusts back
- By the time you hit Execute, the mana display shows **exactly what you'll have** after all auto-attack generation, so you know if your plan is affordable

This means the planning phase IS the mana puzzle — you're watching the circles fill and drain as you assign each unit, and you can see exactly whether your last caster can afford their spell based on how many auto-attackers you've already queued ahead of them.

**Key implication:** You commit ALL actions blind before seeing enemy actions. You can't react mid-round. Your planning is the skill expression.

---

## THE DJINN SYSTEM (the whole game)

### Djinn on the Battlefield

Djinn are **visually present** on the battle screen — they sit on top of your party character sprites. You interact with djinn by **clicking the djinn sprite directly**, not through a menu. This is how you activate, deactivate, and trigger summons. The djinn's visual state on the sprite changes with SET/STANDBY/RECOVERY.

### The Three States

```
SET (equipped) ──activate──→ STANDBY (ready to summon) ──summon──→ RECOVERY (2 turns) ──auto──→ SET
```

### Djinn × Unit Element Compatibility

Each djinn has one of 4 elements. Each unit has one of 4 elements. The match determines bonuses AND abilities:

**Same-element djinn** (e.g., Venus djinn on Venus unit):
- Stat boost
- Grants **Ability A** while SET
- When djinn moves to STANDBY → Ability A swaps out, **Ability B** becomes available
- Both abilities are decent, giving purpose to oscillating between SET and STANDBY

**Neutral-element djinn** (neither same nor opposite):
- Lesser stat boost
- Grants 1 ability (always available regardless of state?)
- The neutral abilities can also be significant — not filler

**Opposite-element djinn** (counter pair: Venus↔Jupiter, Mars↔Mercury):
- Stat DECREASE
- Grants **Ability A** while SET (different from same-element abilities)
- When djinn moves to STANDBY → swaps to **Ability B** (potentially stronger, to offset stat penalty)
- This is the high-risk, high-reward path — weaker stats but access to powerful counter-element moves

### Counter Pairs
```
Venus (Earth) ↔ Jupiter (Wind)
Mars (Fire)   ↔ Mercury (Water)
```

### The Oscillation Mechanic (Geni's evolving vision)

The key innovation: **flipping a djinn between SET and STANDBY isn't just a summon setup — it's an ability swap.** Each state gives access to a different ability. This makes djinn state management an active round-to-round decision, not just "save up for summon."

**Why this matters:** In Golden Sun, you activate djinn mainly to summon. Here, activating a djinn ALSO changes your ability loadout. You might activate a djinn not because you want to summon, but because Ability B (STANDBY) is better for this fight than Ability A (SET). Then next round you might want Ability A back, so you let it recover to SET.

This creates a rhythm: SET → activate → STANDBY (different ability + summon available) → summon or let recover → RECOVERY (no ability) → SET (Ability A returns).

### Two Ability Layers (character abilities vs djinn abilities)

**Character abilities:** Each character gains a unique ability every time they level up. These are ALWAYS available regardless of djinn. This is what makes each character truly unique — their permanent kit.

**Djinn abilities:** The swappable layer. Which djinn are equipped and what state they're in determines the extra abilities available. This is the customization layer.

So a character's full ability set in any given moment is:
```
[Character's level-up abilities (permanent)] + [Djinn abilities (state-dependent)]
```

When you swap party members or djinn, the character abilities stay but the djinn layer changes completely.

### The Customization Space

- **4 party slots** from your full roster of 10+ units
- **3 djinn slots** from your full collection of 20 djinn
- Each djinn's abilities change based on which unit they're on (element matching)
- Each djinn's abilities change based on SET vs STANDBY state

**"When you lose once it's just the start."** You lost with Adept/War Mage/Mystic/Ranger + Flint/Forge/Fizz? Try Sentinel/Stormcaller/Blaze/Karis + Bane/Tempest/Tonic. Different character abilities, different djinn abilities, different stat profiles, different mana pool, different speed order — fundamentally different tactical options.

### Summon System

**Any unit can launch a summon** — it's not tied to who "owns" the djinn. The team's standby djinn are a shared resource.

**Summon combos** (spend djinn from standby pool):

| Combo | Djinn spent | Power | Example |
|-------|------------|-------|---------|
| 1 summon | 1 standby | Low-mid | Quick burst |
| 2 summon | 2 standby | Mid-high | Major damage |
| 3 summon | 3 standby | Massive | All-in nuke |
| 1+1+1 | 3 standby (3 separate Tier 1s) | 3× low | Spread damage |
| 2+1 | 3 standby (one Tier 2 + one Tier 1) | Mixed | Burst + utility |

**Summons execute FIRST** — they supersede the normal speed ordering. Before any unit acts in SPD order, all queued summons resolve. This makes them the "instant" option.

**Recovery timing:**
- Djinn used in summon go to RECOVERY starting the turn AFTER the summon
- Recovery is staggered: the first djinn used recovers first, then the next one the following turn, etc.
- So a 3-djinn summon → Djinn A recovers turn N+1, Djinn B recovers turn N+2, Djinn C recovers turn N+3
- This means a full 3-summon isn't "2 turns of nothing" — it's a gradual ramp-back where you regain djinn abilities one at a time

**The summon decision space:**
- Go all-in with 3-summon for massive burst, lose all djinn abilities for 3 staggered turns
- Spread across 1+1+1 for sustained pressure, recover faster
- Mix 2+1 for one big hit plus utility
- Or don't summon at all — keep djinn SET for stat bonuses and abilities

### Djinn Cycling (with corrected recovery)

```
Turn 1: Activate Djinn A → STANDBY (Ability B unlocks, stat bonus lost)
Turn 2: Summon Djinn A (Tier 1) → RECOVERY. Activate Djinn B → STANDBY.
Turn 3: Djinn A recovers → back to SET. Summon Djinn B → RECOVERY. Activate Djinn C.
Turn 4: Djinn B recovers. Summon Djinn C. → continuous 1-summon-per-round rotation
```

With staggered recovery, you can maintain a steady summon cadence while always having 1-2 djinn SET for abilities.

---

## THE CRIT SYSTEM (deterministic, per-battle)

**No random crits.** Every 10th HIT triggers a crit. Per-unit counter, resets per battle.

```
Unit lands an auto-attack hit → critCounter += 1 (PER HIT, not per action)
If critCounter >= 10: CRIT → counter resets to 0
Displayed as "X/10" badge on each unit's portrait during battle
Player always sees exact crit progress for every unit
```

**The critical interaction:** Each individual hit of a multi-hit AUTO-ATTACK advances the crit counter separately. A 4-hit auto-attack advances the counter by 4. Abilities/spells do NOT advance crit. This is the core of the "crit trick."

---

## MULTI-HIT AUTO-ATTACKS (the deep system)

Multi-hit is an AUTO-ATTACK property, not a spell property. Some auto-attacks hit multiple times. Each individual hit:
- Deals damage separately (target can die mid-combo, remaining hits stop)
- **Advances that unit's crit counter by 1** (a 4-hit = +4 toward the 10-hit threshold)
- **Generates +1 mana to the team pool**

**Abilities/spells do NONE of these.** They cost mana, deal their damage, apply their effects — but don't generate mana, don't advance crit.

### The Crit Trick

Because the crit counter advances per auto-attack hit and you can see the "X/10" on each portrait, you can engineer exactly when crits land:

**Example:** Your Ranger's crit counter is at 7/10.
- Option A: Use a 3-hit auto-attack → the 3rd hit is the 10th overall → **that hit crits**
- Option B: Single auto-attack → counter goes to 8/10. Next round, 2-hit auto → 2nd hit crits
- Option C: Single auto → 8/10. Next round single auto → 9/10. Round after that, auto → **the 10th hit crits with full intent**

**The deeper trick:** You want the crit to land on a turn where it matters most. Maybe you spend 2 rounds farming crit counter with auto-attacks, building to 9/10, then the round you KNOW the boss is vulnerable, that auto-attack crit lands.

### The Mana-per-Hit Engine

Multi-hit auto-attacks are exponentially better resource generators:
- Strike (1 hit) → +1 mana, +1 crit
- 2-hit auto-attack → +2 mana, +2 crit — in one action slot
- 4-hit auto-attack → +4 mana, +4 crit — in one action slot

**This is the incentive to auto-attack.** A unit assigned to a 4-hit auto-attack is doing MORE for the team than a unit casting a 2-mana spell in many situations — they're generating 4 mana (net +4 vs net -2) and charging crit by 4 while still dealing damage.

**The resource triangle every round:**
```
              AUTO-ATTACK (multi-hit)
             generates mana + farms crit
                    /          \
                   /            \
         MANA BUDGET ←————→ SPELL CASTING
       (team pool, resets      (costs mana,
        + auto-attack bonus)    no crit, no gen)
```

Every round the pool resets to your base max. Auto-attackers can push it ABOVE that base during execution. So the real budget isn't just your manaContribution sum — it's that sum PLUS however many auto-attack hits your fast units land before your slow casters need the mana.

---

## ELEMENT SYSTEM (NO damage modifiers — your decision)

The four elements exist purely for:
1. **Djinn compatibility** — same vs counter vs neutral on a character
2. **Class changes** — djinn element combos determine unit class
3. **Djinn ability access** — which abilities a djinn grants depends on element matching

Elements do NOT affect:
- Spell damage (no 1.5× advantage)
- Physical damage
- Resistance
- Healing effectiveness

This is a deliberate departure from Golden Sun and the v2 code (which has 1.5×/0.67× multipliers implemented). Those multipliers should be removed in the Rust port per your direction.

---

## SPEED SYSTEM & EXECUTION ORDER

Actions execute **fastest to slowest**, inclusive of both player units AND enemies interleaved in one combined order.

**Speed is determined by:** base character stat + level growth + equipment + djinn bonuses (from SET djinn)

**Critical rule:** Any changes to speed that occur during a round (from buffs, debuffs, equipment destruction, djinn state changes) take effect **the following turn**, not immediately. You can't alter turn order mid-round.

**Tiebreaker cascade (when two units have the same effective speed):**
1. Current effective speed
2. Speed at start of battle
3. Unit's base speed (stripped of all items and boosts)
4. Unit's level-base speed (the raw stat)

**Pre-battle team ordering:** When you select your 4 units before battle, they slot into positions 1→4 in the turn order display. That's their formatting for the entire battle — mid-battle speed changes will alter execution position but the visual slot stays.

**Why this matters:** Speed is a buildable stat. You can equip speed boots, assign Jupiter djinn (wind = speed), or pick naturally fast characters to ensure your auto-attackers go FIRST and generate mana before your slow casters need it. The entire mana engine depends on speed ordering.

**Synergy example:** A slow mage (high mana contribution, expensive spells) paired with a fast multi-hit auto-attacker (low mana contribution, high speed). The auto-attacker goes first, generates bonus mana, the mage goes last and has more to spend. This is a deliberate party-building strategy, not an accident.

---

## ITEMS & EQUIPMENT

**Items are element-dependent.** Equipment is tied to elements (Venus swords, Mars axes, Mercury staves, etc.).

**Why this works with the roster system:** Since you're constantly swapping units in and out between battles, investing in one element's equipment isn't wasted — it applies to ANY unit of that element. Buy a powerful Venus sword and it works for Adept, Sentinel, or Felix. Invest in Mars gear and it covers War Mage, Blaze, or Tyrell.

**Items can affect:**
- Stats (ATK, DEF, SPD, MAG, HP)
- Mana contribution (some items add to a unit's mana pool contribution)
- Multi-hit count (some items/traits grant multi-hit auto-attacks)
- Speed (critical for execution order manipulation)

**Element-locked equipment creates build commitment:** You can't optimize for all 4 elements simultaneously. You invest in 1-2 element loadouts and then pick units that match. This interacts with djinn assignment — if you're invested in Venus gear, you want Venus units, which means same-element Venus djinn for stat bonuses... or you go counter-element Jupiter djinn for the risk/reward play.

---

## MANA POOL — FULL MODEL

**The pool is team-wide and comes from two sources:**

1. **Unit base mana contribution** — each unit brings a fixed amount (1-3 per unit)
2. **Item bonuses** — some equipment adds to mana contribution

**Pool resets each round** to the max (sum of all living units' contributions + item bonuses).

**Mid-round bonus from auto-attacks:** Each auto-attack hit adds +1 mana during execution, available to units that act LATER in SPD order.

**Next-turn mana:** Some characters or items provide mana that's available **next turn** instead of in realtime. This is a different timing — the mana isn't available to same-round casters, but it could push next round's starting pool above the base max.

**Unit archetypes that emerge from the mana system:**

| Archetype | Mana contrib | Speed | Role |
|-----------|-------------|-------|------|
| Mana battery / tank | 3 | Low | Exists to provide huge base pool + soak damage |
| Speed generator | 1 | High | Goes first, auto-attacks with multi-hit, generates bonus mana |
| Expensive caster | 2 | Low | Big spells, goes last, benefits from generated mana |
| Balanced | 2 | Mid | Flexible — can auto-attack or cast depending on the round |

---

## GAMEPLAY LOOP & PROGRESSION

**The game is a series of narrative boss battles.** Not random encounters in the wild — structured story fights with escalating difficulty.

**Each boss battle is a gate.** You either have the team comp, djinn setup, and tactical skill to win, or you don't. When you lose, it's not "grind levels" — it's "rethink your approach."

**Rewards from boss victories:**
- **New units** (expanding your roster and party options)
- **New djinn** (expanding your djinn pool and ability combinations)
- **New items/equipment** (expanding your stat and element options)

**Each reward synergizes with everything else.** Getting a new Venus djinn doesn't just give you one new thing — it creates new combinations with every Venus, neutral, and counter-element unit you already have. It changes which abilities are available, which summon combos work, which stat profiles are possible.

**"Like opening a Christmas present"** — the reward isn't just power, it's possibility. A new unit opens multiple team compositions. A new djinn opens multiple ability loadouts. The combinatorial space grows with each unlock.

**The difficulty + reward loop:**
```
Story battle (hard) → Lose → Rethink team comp / djinn / items / strategy
                    → Win  → New unit/djinn/item → Roster expands
                           → Next story battle (harder, but you have more tools)
                           → The NEW tools are the ones that matter for the NEXT fight
```

**"When you lose once it's just the start."** The customization space is so large that a lost battle opens up experimentation, not frustration. Different 4-unit lineup, different 3-djinn assignment, different equipment, different speed ordering, different mana budget, different crit timing — fundamentally different tactical options each attempt.

---

## GAME FLOW — TWO MODES

### Mode 1: Main Story (narrative boss battles)

The primary progression. A series of scripted boss-level encounters with narrative framing. Each victory rewards units, djinn, or equipment that expand your combinatorial space.

**Structure:** Linear sequence of story battles with escalating difficulty. Not random encounters — each fight is hand-designed with specific enemy compositions, specific rewards, and specific unlock gates.

**When stuck:** You don't grind the same battle. You go to the Battle Tower.

### Mode 2: Battle Tower (roguelite training ground)

A separate multi-floor challenge that uses your **current roster, team setup, djinn, and equipment** from the main game. Not a separate progression — it draws from the same save.

**How it works:**
- Each floor is a battle using your current team
- Beat a floor → next floor unlocks
- Floor rewards: XP for your units, plus equipment/djinn/items
- Purpose: **power up when you're stuck in the main story**
- Uses the v2 30-floor structure with rest stops and boss floors

**The relationship between modes:**
```
Main Story Battle (hard) → Stuck? → Battle Tower (grind floors for XP + rewards)
                                   → Get stronger, unlock gear
                                   → Return to Main Story with upgraded roster
                        → Win!   → New unit/djinn/item
                                 → Next story battle + Tower floors scale up
```

**Tower floor structure (from v2):**
```
Floors 1-4:   Tutorial, easy (Tier 1) — REST at floor 4
Floors 5-8:   First boss zone (Tier 2) — REST at floor 8
Floors 9-12:  Djinn rewards (Tier 3) — REST at floor 12
Floors 13-16: Power spike (Tier 4) — REST at floor 16
Floors 17-20: Chapter 1 finale (Tier 5)
Floors 21-24: Post-Vale (Tier 6) — REST at floor 24
Floors 25-28: Sovereign bosses (Tier 7) — REST at floor 28
Floors 29-30: Final challenge (Tier 8)
```

### Encounter Rewards (scripted per battle)

Every encounter — both story and tower — has fixed rewards. Nothing is random. The player knows what they're working toward.

### Recruitment Timeline (main story)

| Battle | Who joins | Mana change |
|--------|-----------|-------------|
| Start | Adept + War Mage + Mystic + Ranger | Base party |
| Story 5 | Blaze (Mars, mana 2) | Pool grows |
| Story 8 | Sentinel (Venus, mana 1) | +1 |
| Story 11 | Karis (Mercury, mana 2) | +2 |
| Story 14 | Tyrell (Mars, mana 1) | +1 |
| Story 15 | Stormcaller (Jupiter, mana 3) | +3 |
| Story 17 | Felix (Venus, mana 1) | +1 |

As roster grows, you have more mana budget, more party compositions, more djinn combinations, and more speed orderings available. The game gets strategically deeper with each unlock.

---

## CONTENT INVENTORY (v2 canon)

| Category | v2 count | Rust port | Gap |
|----------|---------|-----------|-----|
| Player units | 11 (+Champion) | 10 | 1 missing |
| Enemies | 104+ | 65 | ~40 missing |
| Abilities | 208 | 81 | 127 missing |
| Djinn | 20 | 12 | 8 missing |
| Equipment | 88 | 33 | 55 missing |
| Tower floors | 30 scripted | Partial | Major gap |
| Encounters | 28+ scripted | Basic tiers | Major gap |
| Sprite assets | 1,707 (v2) / 5,874 (Rust) | 17 loaded | 99.7% unwired |

---

## RUST PORT ERRORS (things that MUST change)

1. **Remove element damage modifiers.** Delete the 1.25×/0.75× advantage/disadvantage. Elements affect djinn compatibility only.
2. **Change to queue-based battle.** Current standard turn-by-turn must become plan-all-then-execute.
3. **Change to team mana pool with per-round reset.** Current per-unit PP must become shared pool from manaContribution + item bonuses. Pool RESETS each round (Slay the Spire model). Auto-attack hits generate bonus mana mid-execution in SPD order.
4. **Add deterministic crits (auto-attack only).** Replace ±10% damage variance with counter-based crit system. Every 10th auto-attack hit crits. Per-unit counter, resets per battle. Abilities do NOT advance crit.
5. **Remove all damage randomness.** No ±10% variance. Damage is deterministic.
6. **Remove break gauge.** Not a canon mechanic.
7. **Add djinn ability oscillation.** SET grants Ability A, STANDBY swaps to Ability B. This makes djinn state management an active tactical choice, not just summon prep.
8. **Separate character abilities from djinn abilities.** Character abilities are permanent (gained per level). Djinn abilities are the swappable layer.
9. **Add multi-hit auto-attacks with per-hit mana and crit.** Each hit generates +1 mana and +1 crit counter. Abilities do NOT generate mana or advance crit. Multi-hit can come from character traits or items.
10. **Summons execute first (supersede SPD order).** Any unit can launch summons from the team's standby djinn pool. Recovery is staggered (one djinn back per turn).
11. **Speed changes take effect next turn.** Mid-round speed alterations don't change current round's execution order.
12. **Items are element-dependent.** Equipment is tied to elements, works across any unit of that element.
13. **Add next-turn mana timing.** Some characters/items provide mana available next round instead of in realtime.
14. **Gameplay is narrative boss battles.** Rewards are units/djinn/items that synergize with all existing mechanics. Not random encounters — structured story progression.
15. **Expand content to v2 levels.** 208 abilities, 104 enemies, 88 equipment, 20 djinn, 30 scripted encounters.

---

## OPEN QUESTIONS FOR GENI

1. **Crit multiplier value:** 1.5×? 2×? Something else?

2. **Next-turn mana:** Which specific characters or items have next-turn timing? Is this a per-unit trait, or a per-item trait, or both?

3. **Multi-hit auto-attack sources:** Primarily from weapons/equipment. Possibly also from character archetypes/builds. Exact source list TBD.

4. **Djinn ability oscillation — exact counts:** BRAINSTORM NEEDED. See below.

5. **Djinn stat bonuses:** Still +4/+3 same, +2/+2 neutral, -3/-2 opposite? Or evolved?

6. **Class change system:** Does the synergy table (Venus Warrior, Hybrid, Mystic) still exist alongside oscillation? Or replaced by it?

7. ~~**Summon staggered recovery:** Confirmed. 3-djinn summon = one djinn back per turn over 3 turns.~~

8. **Story structure:** Main story is narrative boss sequence. Battle Tower is roguelite side mode for powering up when stuck. Both use same roster/save.

9. **Total main story battles:** How many? Is it the 28 "houses" from v2, or a different count?

---

## BRAINSTORM: Djinn Ability Oscillation (#4)

The core question: **how many abilities does each djinn provide, and how do they change between SET and STANDBY?**

Here are some options to consider:

### Option A: 1 SET / 1 STANDBY (clean swap)

Each djinn provides exactly 2 abilities total. You always have access to exactly 1 of them depending on state.

```
Flint (Venus, on Venus unit):
  SET:     → "Earth Shield" (defensive)
  STANDBY: → "Stone Fist" (offensive)

Flint (Venus, on Jupiter unit — opposite):
  SET:     → "Tremor" (weaker, stat penalty applies)
  STANDBY: → "Seismic Slam" (stronger, offsets stat penalty)
```

**Pros:** Simple to understand. Clean decision: which ability do I want right now? Djinn have exactly 2 abilities each, manageable content scope (20 djinn × 2 abilities × 3 compatibility types = up to 120 djinn abilities, though many could be shared).

**Cons:** Only 1 ability at a time from each djinn. With 3 djinn, that's only 3 djinn abilities at once.

### Option B: 2 SET / 1 STANDBY (SET is the rich state)

Djinn provide 2 abilities while SET and swap to 1 different ability in STANDBY.

```
SET:     → Ability A + Ability B (the "equipped" loadout)
STANDBY: → Ability C (the "unleashed" move, usually stronger/niche)
```

**Pros:** More ability variety while SET. Incentivizes keeping djinn SET longer. Makes the STANDBY swap feel like a sacrifice (lose 2, gain 1 powerful one).

**Cons:** More content to design. 20 djinn × 3 abilities × compatibility types = more work.

### Option C: 1 SET / 1 STANDBY, but neutral always-on

Same-element and opposite-element djinn swap 1↔1. Neutral-element djinn have 1 ability that's ALWAYS available regardless of state.

```
Same/Opposite: SET → Ability A, STANDBY → Ability B
Neutral:       ALWAYS → Ability C (regardless of state)
```

**Pros:** Gives neutral djinn a unique identity (reliability). Creates three distinct djinn strategies: same (swap for synergy), opposite (swap for power), neutral (consistent).

**Cons:** Neutral might be too safe — no oscillation tension.

### Option D: The "stance" model (0 SET / 2 STANDBY)

Djinn provide NO abilities while SET (only stat bonuses). Activating them opens up 2 abilities in STANDBY. This makes SET purely a stat/passive state and STANDBY the "active" state.

```
SET:     → No djinn abilities (stat bonuses only)
STANDBY: → Ability A + Ability B (the payoff for activating)
```

**Pros:** Cleanest tension — stats OR abilities, never both. Makes every activation meaningful. Recovery is truly painful (no stats AND no abilities for that turn).

**Cons:** Might make SET feel empty. Frontloads all ability value into STANDBY.

### My recommendation

**Option A with one twist:** 1 SET / 1 STANDBY, but the abilities change based on element compatibility:

- **Same-element:** SET ability is utility/defensive, STANDBY ability is offensive. Both solid.
- **Opposite-element:** SET ability is weak/situational, STANDBY ability is powerful. The payoff for the stat penalty.
- **Neutral:** 1 ability always-on (Option C hybrid).

This gives 20 djinn × 2 abilities for same/opposite + 1 for neutral ≈ ~50-60 unique djinn abilities total. Manageable scope, clear identity per compatibility type, and the oscillation has distinct flavor depending on how you build.

**What resonates?**
