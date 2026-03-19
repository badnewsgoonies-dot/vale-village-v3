# ironclad

Minimal proc-macro crate scaffold for three game-facing macros:

- `#[game_value(min = ..., max = ...)]`: turns a unit struct into an `i64` newtype with a `new()` constructor that panics when the input is outside the declared bounds.
- `#[game_lifecycle]`: placeholder attribute macro that currently returns the annotated item unchanged.
- `#[game_entity]`: placeholder attribute macro that currently returns the annotated item unchanged.

This scaffold is intentionally small and compilable so later waves can tighten parsing, diagnostics, and generated APIs.
