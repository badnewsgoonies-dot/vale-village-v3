# Domain: Data Loader

## Scope
`src/domains/data_loader/`

## Purpose
Load all game content from RON files at startup. Every AbilityDef, UnitDef, EnemyDef, EquipmentDef, DjinnDef, and EncounterDef in the game comes through this loader. No hardcoded content.

## Required imports (from crate::shared)
AbilityDef, UnitDef, EnemyDef, EquipmentDef, DjinnDef, EncounterDef, CombatConfig

## Deliverables

### 1. GameData resource
A single struct holding all loaded content:
```rust
pub struct GameData {
    pub abilities: HashMap<AbilityId, AbilityDef>,
    pub units: HashMap<UnitId, UnitDef>,
    pub enemies: HashMap<EnemyId, EnemyDef>,
    pub equipment: HashMap<EquipmentId, EquipmentDef>,
    pub djinn: HashMap<DjinnId, DjinnDef>,
    pub encounters: HashMap<EncounterId, EncounterDef>,
    pub config: CombatConfig,
}
```

### 2. load_game_data() function
- Reads RON files from a configurable data directory path
- Deserializes into the shared types
- Returns Result<GameData, LoadError>
- LoadError should have variants for: FileNotFound, ParseError(path, message), ValidationError(message)

### 3. Validation on load
- All AbilityId references in UnitDef.abilities exist in the abilities map
- All EnemyId references in EncounterDef.enemies exist in the enemies map
- All AbilityId references in EquipmentDef.unlocks_ability exist
- All DjinnId references in EncounterDef.djinn_reward exist
- Report ALL validation errors, don't stop at first

### 4. Sample RON files
Create `data/sample/` with one small RON file per type containing 2-3 entries each, enough to prove the loader works. Use data from docs/data/COMPLETE_DATA.md.

## Quantitative targets
- 6 RON file types loadable
- 1 GameData struct exported
- 1 load function with validation
- 1 error enum with 3+ variants
- 6 sample RON files

## Does NOT handle
- Bevy asset pipeline integration (future wave)
- JSON→RON conversion scripts (separate task)
- Runtime hot-reloading
- Save/load game state

## Validation
```
cargo check
cargo test -- data_loader
```
