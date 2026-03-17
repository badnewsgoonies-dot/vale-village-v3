# Player Trace — Wave 7 (Enemy Actions + Encounters)

## What the player experiences

1. **[Observed]** Battle loads encounter "house-02" from data: Earth Scout + Earthbound Wolf (venus-wolf).

2. **[Observed]** Both sides attack each round. Enemies auto-attack first alive player. SPD ordering: Blaze(13) → venus-wolf(11) → Adept(10) → earth-scout(8).

3. **[Observed]** Adept takes real damage: 120→89 HP over 6 rounds (5/round from wolf + 3/round from scout until scout dies round 3).

4. **[Observed]** Overkill clamped: Blaze deals 11 to wolf's last 11 HP, not the full 12 base damage.

5. **[Observed]** Victory after 6 rounds. Both enemies defeated. Players survive.

## Harden Assessment

- Reachable? **YES**
- Two-sided combat? **YES** — enemies deal damage
- Feedback visible? **YES** — both sides' actions shown
- SPD ordering correct? **YES** — verified against stat values
- Edge behavior? **YES** — overkill clamped, retargeting works, dead enemy skipped

## Graduation

### [Observed] → Named invariant
1. **Enemies attack player units** — verified in output and 6 new tests
2. **SPD ordering interleaves player and enemy** — Blaze > wolf > Adept > scout
3. **Player takes damage** — Adept HP decreases each round

### P0 debt (remaining)
- [ ] 105 stub abilities need real stats
- [ ] Demo should show ability usage, not just auto-attacks

### P1 debt
- [ ] Bevy visual rendering
- [ ] Player input (interactive planning)
- [ ] Smarter enemy AI (not just "attack first alive")
- [ ] Graduation tests: full data load, battle completion, SPD ordering
