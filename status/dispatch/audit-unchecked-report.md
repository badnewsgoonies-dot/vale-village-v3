I ran the CSV audit read-only and waited for all 5 workers. No code was changed.

**Consolidated Findings**
- Across all audits: `215` `new_unchecked()` sites total.
- Breakdown: `128` constant, `50` computed, `37` runtime.

- The biggest hotspot is `[src/domains/battle_engine/mod.rs](/home/claude/vale-village-v3/src/domains/battle_engine/mod.rs#L136)`: `30` computed and `7` runtime uses.
- The computed sites there are derived stat/buff calculations that only clamp the low end, so upper-bound validation is still the main gap.
- The runtime sites there are direct caller-supplied values passed into unchecked constructors, including `Hp`, `BaseStat`, `ManaCost`, and `BasePower`.

- `[src/domains/status/mod.rs](/home/claude/vale-village-v3/src/domains/status/mod.rs#L219)` has `5` computed and `12` runtime uses.
- The computed cases are modifier accumulation; the runtime cases are direct handoff from live state or parameters into unchecked constructors.

- `[src/domains/combat/mod.rs](/home/claude/vale-village-v3/src/domains/combat/mod.rs#L286)` has `5` runtime uses, all in test helper paths that accept caller-supplied stats/cost/power.

- The repo-wide pass also found runtime/computed cases in `[src/domains/ai/mod.rs](/home/claude/vale-village-v3/src/domains/ai/mod.rs#L314)`, `[src/domains/damage_mods/mod.rs](/home/claude/vale-village-v3/src/domains/damage_mods/mod.rs#L98)`, `[src/domains/progression/mod.rs](/home/claude/vale-village-v3/src/domains/progression/mod.rs#L79)`, `[src/domains/equipment/mod.rs](/home/claude/vale-village-v3/src/domains/equipment/mod.rs#L133)`, and `[src/domains/djinn/mod.rs](/home/claude/vale-village-v3/src/domains/djinn/mod.rs#L198)`.
- Those are mostly derived from runtime state or test/helper parameters, so they deserve validation review if they are not intentionally trusted paths.

- `[src/data/mod.rs](/home/claude/vale-village-v3/src/data/mod.rs#L11)` and `[src/domains/data_loader/mod.rs](/home/claude/vale-village-v3/src/domains/data_loader/mod.rs#L1)` were clean from a runtime-risk standpoint: all 7 uses there were hardcoded constants, not deserialized input.

The exported audit results are in `[status/tasks/audit-unchecked.results.csv](/home/claude/vale-village-v3/status/tasks/audit-unchecked.results.csv)`. If you want, I can turn this into a prioritized patch plan next.