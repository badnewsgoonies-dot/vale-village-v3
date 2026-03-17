# Player Trace — Waves 1-3 (Retroactive)

## What the player experiences from boot to first meaningful interaction

1. **[Observed]** `cargo run` prints "Vale Village v3 — contract frozen, awaiting domain builds." and exits. No Bevy window, no game loop, no input. (src/main.rs:6)

2. **[Observed]** 6 domain modules exist with 114 passing tests, but none are called from main.rs. All code is structurally correct and unreachable. (src/domains/mod.rs)

3. **[Observed]** Sample RON data exists in data/sample/ with 2-3 entries per type — not the full 241 abilities / 137 enemies / 109 equipment from the design docs. (data/sample/)

4. **[Assumed]** The battle engine integration worker (in progress) will wire domains together into an executable battle loop.

5. **[Assumed]** A Bevy app with text-based or minimal UI battle will be needed before anything is player-reachable.

## Harden assessment

- Reachable end-to-end? **NO** — nothing runs
- Feedback visible? **NO** — no UI, no output beyond println
- Responsive? **N/A**
- Edge behavior sane? **N/A** (only unit tests)
- Diagnosable when it fails? **YES** — 114 tests with clear names

## Graduation debt

### P0 (stop condition — must exist before next campaign wave)
- [ ] Runnable battle: player can execute at least 1 round of combat
- [ ] Full data: all 241 abilities loaded from RON (not 4)
- [ ] main.rs actually starts a battle loop (not println)

### P1 (before release)
- [ ] All 137 enemies loadable
- [ ] All 109 equipment loadable
- [ ] All 23 djinn loadable
- [ ] All 55 encounters loadable
- [ ] Bevy app with visual battle UI

### P2 (tracked)
- [ ] Sprite pipeline
- [ ] Save/load
- [ ] Enemy AI
- [ ] Shop / equipment management UI
- [ ] Story progression
