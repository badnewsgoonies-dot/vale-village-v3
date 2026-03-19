//! Data Loader domain — loads all game content from RON files at startup.
//!
//! Every AbilityDef, UnitDef, EnemyDef, EquipmentDef, DjinnDef, and
//! EncounterDef in the game comes through this loader. No hardcoded content.
#![allow(dead_code)]

use std::collections::HashMap;
use std::path::Path;

use crate::shared::{
    AbilityDef, AbilityId, CombatConfig, DjinnDef, DjinnId, EncounterDef, EncounterId, EnemyDef,
    EnemyId, EquipmentDef, EquipmentId, UnitDef, UnitId,
};

// ── GameData resource ───────────────────────────────────────────────

/// Holds all loaded game content, keyed by their respective IDs.
#[derive(Debug)]
pub struct GameData {
    pub abilities: HashMap<AbilityId, AbilityDef>,
    pub units: HashMap<UnitId, UnitDef>,
    pub enemies: HashMap<EnemyId, EnemyDef>,
    pub equipment: HashMap<EquipmentId, EquipmentDef>,
    pub djinn: HashMap<DjinnId, DjinnDef>,
    pub encounters: HashMap<EncounterId, EncounterDef>,
    pub config: CombatConfig,
}

// ── LoadError enum ──────────────────────────────────────────────────

/// Errors that can occur while loading game data from RON files.
#[derive(Debug)]
pub enum LoadError {
    /// A required data file was not found at the expected path.
    FileNotFound(String),
    /// A RON file could not be parsed.
    ParseError(String, String),
    /// Cross-reference validation failed (e.g. an ability ID referenced
    /// by a unit does not exist in the abilities map).
    ValidationError(String),
}

impl std::fmt::Display for LoadError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LoadError::FileNotFound(path) => write!(f, "File not found: {}", path),
            LoadError::ParseError(path, msg) => write!(f, "Parse error in {}: {}", path, msg),
            LoadError::ValidationError(msg) => write!(f, "Validation error: {}", msg),
        }
    }
}

// ── Loader ──────────────────────────────────────────────────────────

/// Load a single RON file, deserializing into a `Vec<T>`.
fn load_ron_file<T: serde::de::DeserializeOwned>(
    path: &Path,
    errors: &mut Vec<LoadError>,
) -> Vec<T> {
    let path_str = path.display().to_string();
    let contents = match std::fs::read_to_string(path) {
        Ok(c) => c,
        Err(_) => {
            errors.push(LoadError::FileNotFound(path_str));
            return Vec::new();
        }
    };
    match ron::from_str::<Vec<T>>(&contents) {
        Ok(items) => items,
        Err(e) => {
            errors.push(LoadError::ParseError(path_str, e.to_string()));
            Vec::new()
        }
    }
}

/// Load a single RON file, deserializing into a single value `T`.
fn load_ron_single<T: serde::de::DeserializeOwned>(
    path: &Path,
    errors: &mut Vec<LoadError>,
) -> Option<T> {
    let path_str = path.display().to_string();
    let contents = match std::fs::read_to_string(path) {
        Ok(c) => c,
        Err(_) => {
            errors.push(LoadError::FileNotFound(path_str));
            return None;
        }
    };
    match ron::from_str::<T>(&contents) {
        Ok(item) => Some(item),
        Err(e) => {
            errors.push(LoadError::ParseError(path_str, e.to_string()));
            None
        }
    }
}

// ── Validation ──────────────────────────────────────────────────────

fn validate(data: &GameData, errors: &mut Vec<LoadError>) {
    // All AbilityId references in UnitDef.abilities exist in the abilities map
    for unit in data.units.values() {
        for prog in &unit.abilities {
            if !data.abilities.contains_key(&prog.ability_id) {
                errors.push(LoadError::ValidationError(format!(
                    "Unit '{}' references ability '{}' which does not exist",
                    unit.id.0, prog.ability_id.0
                )));
            }
        }
    }

    // All AbilityId references in EnemyDef.abilities exist in the abilities map
    for enemy in data.enemies.values() {
        for ability_id in &enemy.abilities {
            if !data.abilities.contains_key(ability_id) {
                errors.push(LoadError::ValidationError(format!(
                    "Enemy '{}' references ability '{}' which does not exist",
                    enemy.id.0, ability_id.0
                )));
            }
        }
    }

    // All EnemyId references in EncounterDef.enemies exist in the enemies map
    for encounter in data.encounters.values() {
        for enc_enemy in &encounter.enemies {
            if !data.enemies.contains_key(&enc_enemy.enemy_id) {
                errors.push(LoadError::ValidationError(format!(
                    "Encounter '{}' references enemy '{}' which does not exist",
                    encounter.id.0, enc_enemy.enemy_id.0
                )));
            }
        }
    }

    // All AbilityId references in EquipmentDef.unlocks_ability exist
    for equip in data.equipment.values() {
        if let Some(ref ability_id) = equip.unlocks_ability {
            if !data.abilities.contains_key(ability_id) {
                errors.push(LoadError::ValidationError(format!(
                    "Equipment '{}' references ability '{}' which does not exist",
                    equip.id.0, ability_id.0
                )));
            }
        }
    }

    // All DjinnId references in EncounterDef.djinn_reward exist
    for encounter in data.encounters.values() {
        if let Some(ref djinn_id) = encounter.djinn_reward {
            if !data.djinn.contains_key(djinn_id) {
                errors.push(LoadError::ValidationError(format!(
                    "Encounter '{}' references djinn reward '{}' which does not exist",
                    encounter.id.0, djinn_id.0
                )));
            }
        }
    }

    // All AbilityId references in DjinnDef.ability_pairs exist in abilities map
    for djinn in data.djinn.values() {
        for (compat_label, ability_set) in [
            ("same", &djinn.ability_pairs.same),
            ("counter", &djinn.ability_pairs.counter),
            ("neutral", &djinn.ability_pairs.neutral),
        ] {
            for ability_id in &ability_set.good_abilities {
                if !data.abilities.contains_key(ability_id) {
                    errors.push(LoadError::ValidationError(format!(
                        "Djinn '{}' {}.good_abilities references ability '{}' which does not exist",
                        djinn.id.0, compat_label, ability_id.0
                    )));
                }
            }
            for ability_id in &ability_set.recovery_abilities {
                if !data.abilities.contains_key(ability_id) {
                    errors.push(LoadError::ValidationError(format!(
                        "Djinn '{}' {}.recovery_abilities references ability '{}' which does not exist",
                        djinn.id.0, compat_label, ability_id.0
                    )));
                }
            }
        }
    }

    // All EquipmentId references in EncounterDef.equipment_rewards exist
    for encounter in data.encounters.values() {
        for equip_id in &encounter.equipment_rewards {
            if !data.equipment.contains_key(equip_id) {
                errors.push(LoadError::ValidationError(format!(
                    "Encounter '{}' equipment_rewards references equipment '{}' which does not exist",
                    encounter.id.0, equip_id.0
                )));
            }
        }
    }

    // All UnitId references in EncounterDef.recruit exist
    for encounter in data.encounters.values() {
        if let Some(ref unit_id) = encounter.recruit {
            if !data.units.contains_key(unit_id) {
                errors.push(LoadError::ValidationError(format!(
                    "Encounter '{}' recruit references unit '{}' which does not exist",
                    encounter.id.0, unit_id.0
                )));
            }
        }
    }
}

// ── Public API ──────────────────────────────────────────────────────

/// Load all game data from RON files in the given directory.
///
/// Expects the following files inside `data_dir`:
///   abilities.ron, units.ron, enemies.ron, equipment.ron,
///   djinn.ron, encounters.ron, config.ron
///
/// Collects ALL errors (file-not-found, parse, validation) and returns
/// them together rather than stopping at the first.
pub fn load_game_data(data_dir: &Path) -> Result<GameData, Vec<LoadError>> {
    let mut errors = Vec::new();

    // Load each RON file
    let abilities_list: Vec<AbilityDef> =
        load_ron_file(&data_dir.join("abilities.ron"), &mut errors);
    let units_list: Vec<UnitDef> = load_ron_file(&data_dir.join("units.ron"), &mut errors);
    let enemies_list: Vec<EnemyDef> = load_ron_file(&data_dir.join("enemies.ron"), &mut errors);
    let equipment_list: Vec<EquipmentDef> =
        load_ron_file(&data_dir.join("equipment.ron"), &mut errors);
    let djinn_list: Vec<DjinnDef> = load_ron_file(&data_dir.join("djinn.ron"), &mut errors);
    let encounters_list: Vec<EncounterDef> =
        load_ron_file(&data_dir.join("encounters.ron"), &mut errors);
    let config: Option<CombatConfig> = load_ron_single(&data_dir.join("config.ron"), &mut errors);

    // Build HashMaps keyed by ID
    let abilities: HashMap<AbilityId, AbilityDef> = abilities_list
        .into_iter()
        .map(|a| (a.id.clone(), a))
        .collect();
    let units: HashMap<UnitId, UnitDef> =
        units_list.into_iter().map(|u| (u.id.clone(), u)).collect();
    let enemies: HashMap<EnemyId, EnemyDef> = enemies_list
        .into_iter()
        .map(|e| (e.id.clone(), e))
        .collect();
    let equipment: HashMap<EquipmentId, EquipmentDef> = equipment_list
        .into_iter()
        .map(|eq| (eq.id.clone(), eq))
        .collect();
    let djinn: HashMap<DjinnId, DjinnDef> =
        djinn_list.into_iter().map(|d| (d.id.clone(), d)).collect();
    let encounters: HashMap<EncounterId, EncounterDef> = encounters_list
        .into_iter()
        .map(|enc| (enc.id.clone(), enc))
        .collect();

    // Use loaded config or fall back to default
    let config = config.unwrap_or_else(crate::data::default_combat_config);

    let data = GameData {
        abilities,
        units,
        enemies,
        equipment,
        djinn,
        encounters,
        config,
    };

    // Cross-reference validation
    validate(&data, &mut errors);

    if errors.is_empty() {
        Ok(data)
    } else {
        Err(errors)
    }
}

// ── Tests ───────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn sample_dir() -> PathBuf {
        // Navigate from workspace root to data/sample
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("data")
            .join("sample")
    }

    #[test]
    fn test_sample_data_loads_successfully() {
        let dir = sample_dir();
        let result = load_game_data(&dir);
        match &result {
            Ok(data) => {
                assert!(!data.abilities.is_empty(), "abilities should not be empty");
                assert!(!data.units.is_empty(), "units should not be empty");
                assert!(!data.enemies.is_empty(), "enemies should not be empty");
                assert!(!data.equipment.is_empty(), "equipment should not be empty");
                assert!(!data.djinn.is_empty(), "djinn should not be empty");
                assert!(
                    !data.encounters.is_empty(),
                    "encounters should not be empty"
                );
            }
            Err(errors) => {
                for e in errors {
                    eprintln!("  {}", e);
                }
                panic!("Expected sample data to load without errors");
            }
        }
    }

    #[test]
    fn test_config_loads_correctly() {
        let dir = sample_dir();
        let data = load_game_data(&dir).expect("sample data should load");
        assert_eq!(data.config.max_party_size.get(), 4);
        assert_eq!(data.config.crit_threshold, 10);
        assert!((data.config.crit_multiplier - 2.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_validation_catches_missing_ability_in_unit() {
        let dir = sample_dir();
        let mut data = load_game_data(&dir).expect("sample data should load");
        // Remove all abilities so unit refs break
        data.abilities.clear();
        let mut errors = Vec::new();
        validate(&data, &mut errors);
        assert!(
            !errors.is_empty(),
            "Should find validation errors when abilities are missing"
        );
        // Check that at least one error mentions the unit
        let has_unit_err = errors.iter().any(|e| match e {
            LoadError::ValidationError(msg) => msg.contains("Unit"),
            _ => false,
        });
        assert!(has_unit_err, "Should have a unit-related validation error");
    }

    #[test]
    fn test_validation_catches_missing_enemy_in_encounter() {
        let dir = sample_dir();
        let mut data = load_game_data(&dir).expect("sample data should load");
        // Remove all enemies so encounter refs break
        data.enemies.clear();
        let mut errors = Vec::new();
        validate(&data, &mut errors);
        let has_encounter_err = errors.iter().any(|e| match e {
            LoadError::ValidationError(msg) => msg.contains("Encounter"),
            _ => false,
        });
        assert!(
            has_encounter_err,
            "Should have an encounter-related validation error"
        );
    }

    #[test]
    fn test_validation_catches_missing_djinn_reward() {
        let dir = sample_dir();
        let mut data = load_game_data(&dir).expect("sample data should load");
        // Remove all djinn so encounter djinn_reward refs break
        data.djinn.clear();
        let mut errors = Vec::new();
        validate(&data, &mut errors);
        let has_djinn_err = errors.iter().any(|e| match e {
            LoadError::ValidationError(msg) => msg.contains("djinn reward"),
            _ => false,
        });
        assert!(has_djinn_err, "Should have a djinn-reward validation error");
    }

    #[test]
    fn test_validation_catches_missing_ability_in_equipment() {
        let dir = sample_dir();
        let mut data = load_game_data(&dir).expect("sample data should load");
        // Remove abilities so equipment unlocks_ability refs break
        data.abilities.clear();
        let mut errors = Vec::new();
        validate(&data, &mut errors);
        let has_equip_err = errors.iter().any(|e| match e {
            LoadError::ValidationError(msg) => msg.contains("Equipment"),
            _ => false,
        });
        assert!(
            has_equip_err,
            "Should have an equipment-related validation error"
        );
    }

    #[test]
    fn test_parse_error_on_malformed_ron() {
        use std::io::Write;
        let tmp = tempfile::tempdir().expect("create temp dir");
        let dir = tmp.path();

        // Write malformed abilities file
        let mut f = std::fs::File::create(dir.join("abilities.ron")).unwrap();
        writeln!(f, "this is not valid RON {{{{}}}}").unwrap();

        // Write empty lists for the rest so they parse but are empty
        for name in &[
            "units.ron",
            "enemies.ron",
            "equipment.ron",
            "djinn.ron",
            "encounters.ron",
        ] {
            std::fs::write(dir.join(name), "[]").unwrap();
        }
        std::fs::write(
            dir.join("config.ron"),
            r#"(
                physical_def_multiplier: 0.5,
                psynergy_def_multiplier: 0.3,
                crit_threshold: 10,
                crit_multiplier: 2.0,
                mana_gain_per_hit: 1,
                mana_resets_each_round: true,
                max_party_size: 4,
                max_equipped_djinn: 3,
                max_level: 20,
                max_buff_stacks: 3,
                djinn_recovery_start_delay: 1,
                djinn_recovery_per_turn: 1,
            )"#,
        )
        .unwrap();

        let result = load_game_data(dir);
        assert!(result.is_err(), "Should fail on malformed RON");
        let errors = result.unwrap_err();
        let has_parse = errors
            .iter()
            .any(|e| matches!(e, LoadError::ParseError(_, _)));
        assert!(has_parse, "Should contain a ParseError");
    }

    #[test]
    fn test_file_not_found_error() {
        let tmp = tempfile::tempdir().expect("create temp dir");
        let dir = tmp.path();
        // Empty directory — no RON files at all
        let result = load_game_data(dir);
        assert!(result.is_err(), "Should fail when files are missing");
        let errors = result.unwrap_err();
        let has_fnf = errors
            .iter()
            .any(|e| matches!(e, LoadError::FileNotFound(_)));
        assert!(has_fnf, "Should contain a FileNotFound error");
    }

    #[test]
    fn test_validation_catches_missing_ability_in_djinn() {
        let dir = sample_dir();
        let mut data = load_game_data(&dir).expect("sample data should load");
        // Remove all abilities so djinn ability_pairs refs break
        data.abilities.clear();
        let mut errors = Vec::new();
        validate(&data, &mut errors);
        let has_djinn_err = errors.iter().any(|e| match e {
            LoadError::ValidationError(msg) => msg.contains("Djinn"),
            _ => false,
        });
        assert!(
            has_djinn_err,
            "Should have a djinn-related validation error for ability_pairs"
        );
    }

    #[test]
    fn test_validation_catches_missing_equipment_reward() {
        let dir = sample_dir();
        let mut data = load_game_data(&dir).expect("sample data should load");
        // Remove all equipment so encounter equipment_rewards refs break
        data.equipment.clear();
        let mut errors = Vec::new();
        validate(&data, &mut errors);
        let has_equip_err = errors.iter().any(|e| match e {
            LoadError::ValidationError(msg) => msg.contains("equipment_rewards"),
            _ => false,
        });
        assert!(
            has_equip_err,
            "Should have an equipment_rewards validation error"
        );
    }

    #[test]
    fn test_validation_catches_missing_recruit_unit() {
        let dir = sample_dir();
        let mut data = load_game_data(&dir).expect("sample data should load");
        // Insert an encounter with a recruit that doesn't exist
        use crate::shared::{Difficulty, EncounterDef, EncounterId, UnitId};
        data.encounters.insert(
            EncounterId("test-enc".to_string()),
            EncounterDef {
                id: EncounterId("test-enc".to_string()),
                name: "Test".to_string(),
                difficulty: Difficulty::Easy,
                enemies: Vec::new(),
                xp_reward: crate::shared::bounded_types::Xp::new_unchecked(0),
                gold_reward: crate::shared::bounded_types::Gold::new_unchecked(0),
                recruit: Some(UnitId("nonexistent-unit".to_string())),
                djinn_reward: None,
                equipment_rewards: Vec::new(),
            },
        );
        let mut errors = Vec::new();
        validate(&data, &mut errors);
        let has_recruit_err = errors.iter().any(|e| match e {
            LoadError::ValidationError(msg) => msg.contains("recruit"),
            _ => false,
        });
        assert!(has_recruit_err, "Should have a recruit validation error");
    }

    #[test]
    fn test_collects_all_errors_not_just_first() {
        let tmp = tempfile::tempdir().expect("create temp dir");
        let dir = tmp.path();
        // Empty directory — all 7 files missing
        let result = load_game_data(dir);
        assert!(result.is_err());
        let errors = result.unwrap_err();
        // Should have at least 7 FileNotFound errors (one per file)
        let fnf_count = errors
            .iter()
            .filter(|e| matches!(e, LoadError::FileNotFound(_)))
            .count();
        assert!(
            fnf_count >= 7,
            "Should collect errors for all missing files, got {}",
            fnf_count
        );
    }
}
