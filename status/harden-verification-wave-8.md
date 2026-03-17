# Harden Verification — Wave 8

## [Assumed] Claims Verified

### 1. Djinn recovery delay → [Observed] CORRECT
- activate_djinn takes recovery_turns: u8 parameter, tests pass 2
- tick_recovery decrements, recovers when remaining == 0
- With 2: decrement twice, recover on 3rd tick = "turn after next"
- source_ref: file:@src/domains/djinn/mod.rs:117-131

### 2. SPD tiebreaker → [Observed] CONFIRMED GAP (P2)
- 2 levels implemented (effective → base). DESIGN_LOCK specifies 4.
- source_ref: file:@src/domains/combat/mod.rs:116-119

### 3. Equipment/Djinn → [Observed] GHOST DOMAINS
- battle_engine applies equipment bonuses at init (line 115-126) — correct
- cli_runner passes default empty loadouts — never exercised
- source_ref: file:@src/domains/cli_runner/mod.rs:43-54
