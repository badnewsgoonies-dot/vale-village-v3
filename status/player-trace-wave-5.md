# Player Trace — Wave 5 (CLI Battle Runner)

## What the player experiences

1. **[Observed]** `cargo run` loads 4 abilities, 2 units, 3 enemies from data/sample/ RON files and prints a load summary.

2. **[Observed]** A battle starts: Adept (120 HP) + Blaze (100 HP) vs Mercury Slime (40 HP) + Earth Scout (50 HP). Party and enemy HP bars displayed.

3. **[Observed]** Each round: both player units auto-attack the first alive enemy. Damage numbers printed. HP bars updated with visual bars [####........].

4. **[Observed]** Mercury Slime defeated in round 2 (HP reaches 0). Player units automatically retarget to Earth Scout.

5. **[Observed]** Earth Scout defeated in round 5. Victory declared with XP and gold rewards.

## Harden Assessment

- Reachable? **YES**
- Feedback visible? **YES** — damage, HP bars, defeat, victory
- Responsive? **YES** — <3 seconds total
- Edge behavior? **Partial** — retargeting works, overkill clamped. No enemy attacks to test defensive paths.
- Diagnosable? **YES**

## Graduation

### [Observed] → Named invariant → Test needed

1. **Battle completes to victory** — cargo run exits 0, prints "VICTORY" — needs graduation test
2. **Damage formula correct** — Blaze ATK 15 vs DEF 5 = 13 damage — already tested in combat domain
3. **Retargeting on death** — when target dies, next unit attacks next alive — needs graduation test

### P0 debt (remaining)
- [ ] Full data conversion (241 abilities in RON)
- [ ] Enemy actions (enemies don't fight back)
- [ ] Ability usage in CLI demo (only auto-attacks)

### P1 debt
- [ ] Bevy app with visual rendering
- [ ] Player input (interactive planning)
- [ ] Enemy AI
