# Player Trace — Wave 11 (Adventure Mode Integration)

From boot to first meaningful interaction via `cargo run -- --adventure`:

1. [Observed] Player launches with --adventure flag and sees the title screen with ASCII banner and ENTER prompt. (game_loop.rs:174-181, verified in test_starter_data_loads)

2. [Observed] Pressing ENTER transitions to the world map showing 3 accessible locations: Vale Village (Town), Mercury Lighthouse (Dungeon), Kolima Forest (Dungeon). (game_loop.rs:run_world_map, verified in test_world_map_setup)

3. [Observed] Selecting Vale Village enters the town and shows NPC list with numbered talk options, shop access, and leave. Selecting NPC #0 triggers the Elder's dialogue tree with branching responses and quest advancement side effects. (game_loop.rs:run_town → run_dialogue, verified in test_dialogue_tree_traversal)

4. [Observed] Selecting Mercury Lighthouse enters a 3-room dungeon. Player navigates rooms via directional commands, picks up items (one-time enforced), and reaches the boss room. Boss fight advances the Mercury Lighthouse quest and unlocks Imil on the world map. (game_loop.rs:run_dungeon, verified in test_dungeon_traversal)

5. [Inferred] After boss victory, returning to world map shows Imil as newly accessible. Entering Imil shows the Fizz djinn discovery point and a different shop. [Not traced end-to-end in a single automated run — each segment verified independently.]

Verification debt: Full end-to-end automated playthrough from boot through boss defeat to Imil. Currently each segment is tested independently but the full chain is [Inferred].
