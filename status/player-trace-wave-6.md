# Player Trace — Wave 6 (Full Data Conversion)

## What the player experiences

1. **[Observed]** `cargo run` prints "Loaded 346 abilities, 11 units, 137 enemies" — full dataset.

2. **[Observed]** Battle still runs correctly with full data loaded. Same demo scenario (Adept+Blaze vs Slime+Scout), 5 rounds to victory.

3. **[Observed]** Data counts match targets: 346 abilities (241 + 105 stubs), 137 enemies, 109 equipment, 23 djinn, 55 encounters, 11 units.

## Harden Assessment

- Reachable? **YES** — cargo run loads full data
- Data complete? **MOSTLY** — 105 stub abilities auto-generated for djinn/equipment ability references (these need real stats eventually)
- Validation passes? **YES** — load_game_data cross-reference validation succeeds

## Non-obvious decision
105 stub abilities were auto-generated because djinn ability pairs and equipment items reference abilities not in the base v2 JSON. These are djinn-specific abilities (flint-stone-fist, forge-flame-strike, etc.) and weapon abilities (shadow-step, seraphic-beam, etc.) that need to be designed. They exist as zero-power placeholders so loading succeeds.

## Graduation

### [Observed] → Named invariant
1. **Full data loads without error** — 346 abilities, 137 enemies, 109 equipment, 23 djinn, 55 encounters, 11 units — needs graduation test

### P0 debt (updated)
- [x] Full data conversion — DONE
- [ ] Enemy actions in battle (enemies still don't attack)
- [ ] 105 stub abilities need real stats/effects designed
- [ ] Demo should use a real encounter, not hardcoded units

### P1 debt
- [ ] Bevy app with visual rendering
- [ ] Player input (interactive planning)
- [ ] Enemy AI
- [ ] Ability usage exercised in demo
