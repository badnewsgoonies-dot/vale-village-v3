---
name: locked-design-rules
description: All locked combat/design decisions that must never be contradicted during implementation
type: project
---

These rules are LOCKED (confirmed by Geni, March 17, 2026). Violating any is a bug.

**Why:** These represent months of design iteration and are the contract for implementation.

**How to apply:** Check any implementation against these rules before committing.

**Core combat:**
- Exactly 2 actions: ATTACK (0 cost, +1 mana/hit, +1 crit/hit) and ABILITY (costs mana, generates nothing)
- No Defend, no Items-in-combat, no Summon button
- Planning order = execution order (select Unit 1-4 sequentially, execute in same order)
- NO randomness anywhere - fully deterministic
- Damage: Physical = basePower + ATK - (DEF * 0.5), floor 1; Psynergy = basePower + MAG - (DEF * 0.3), floor 1
- No element damage modifiers (elements affect ONLY djinn compatibility)

**Mana:**
- Team-wide shared pool = sum of living units' manaContribution + item manaBonus
- Resets to max each round (Slay the Spire model)
- Auto-attack hits generate +1 mana mid-execution (available to later SPD units)
- Visual: solid circles = guaranteed mana, different-colored = projected from planned autos

**Crit:**
- Every 10th auto-attack HIT = guaranteed crit (per-unit counter, X/10 on portrait)
- 2x multiplier, resets per battle
- Only auto-attacks advance counter, not abilities

**Status effects (6 total):**
- Stun: cannot act, X turns
- Null: can only auto-attack, X turns
- Incapacitate: can only use abilities, X turns
- Burn: % maxHP/turn
- Poison: % missing HP/turn
- Freeze: skip turn, breaks after N cumulative damage (threshold per ability)
- All deterministic - always applies on hit, no chance field

**Djinn (2 states: GOOD/RECOVERY):**
- GOOD: stat bonus + Set A abilities, can activate
- RECOVERY: no stat bonus + Set B abilities, cannot activate
- Same-element: 2 GOOD abilities + 2 RECOVERY abilities
- Counter-element: 2 GOOD + 2 RECOVERY (RECOVERY ones are stronger)
- Neutral: 1 ability always-on regardless of state
- Recovery: 1 djinn/turn, starting turn after next, in activation order
- Summons via djinn menu (click sprite), execute before SPD order

**Barriers & HoT:**
- Barriers: per-instance damage blockers, stackable, duration-based, don't block status damage
- HoT: fixed HP/turn for N turns
- Three defensive layers: HP, Barriers, HoT

**Equipment:** 5 slots (weapon, helm, armor, boots, accessory). Equipment abilities are core (51% of items unlock abilities).
