Scaffolded `status/visual-tests/` as an isolated Rust binary crate for PNG visual diffs.
The binary takes `expected.png` and `actual.png`, compares pixels at zero tolerance, writes a red-highlight diff PNG, and exits 1 with `mismatch_count` when images differ.
Validated with `cargo check --manifest-path status/visual-tests/Cargo.toml`.
