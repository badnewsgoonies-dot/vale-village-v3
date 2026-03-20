#![allow(dead_code)]
//! Save/Load domain — persist and restore game state via RON files.

use std::fmt;
use std::fs;
use std::io;
use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::shared::{DjinnId, DjinnState, EncounterId, EquipmentId, UnitId};

// ── Constants ───────────────────────────────────────────────────────

const CURRENT_SAVE_VERSION: u32 = 1;

// ── Error Type ──────────────────────────────────────────────────────

#[derive(Debug)]
pub enum SaveError {
    IoError(io::Error),
    SerializeError(String),
    DeserializeError(String),
    VersionMismatch { expected: u32, found: u32 },
}

impl fmt::Display for SaveError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SaveError::IoError(e) => write!(f, "IO error: {}", e),
            SaveError::SerializeError(e) => write!(f, "Serialize error: {}", e),
            SaveError::DeserializeError(e) => write!(f, "Deserialize error: {}", e),
            SaveError::VersionMismatch { expected, found } => {
                write!(
                    f,
                    "Version mismatch: expected {}, found {}",
                    expected, found
                )
            }
        }
    }
}

impl From<io::Error> for SaveError {
    fn from(e: io::Error) -> Self {
        SaveError::IoError(e)
    }
}

// ── Data Structs ────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SaveData {
    pub version: u32,
    pub player_party: Vec<SavedUnit>,
    pub roster: Vec<SavedUnit>,
    pub gold: u32,
    pub xp: u32,
    pub current_encounter_id: Option<EncounterId>,
    pub completed_encounters: Vec<EncounterId>,
    pub inventory: Vec<EquipmentId>,
    #[serde(default)]
    pub team_djinn: Vec<SavedDjinn>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SavedUnit {
    pub unit_id: UnitId,
    pub level: u8,
    pub current_hp: u16,
    pub equipment: SavedEquipment,
    pub djinn: Vec<SavedDjinn>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct SavedEquipment {
    pub weapon: Option<EquipmentId>,
    pub helm: Option<EquipmentId>,
    pub armor: Option<EquipmentId>,
    pub boots: Option<EquipmentId>,
    pub accessory: Option<EquipmentId>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SavedDjinn {
    pub djinn_id: DjinnId,
    pub state: DjinnState,
}

// ── Public Functions ────────────────────────────────────────────────

/// Serialize `SaveData` to RON and write to the given path.
pub fn save_game(data: &SaveData, path: &Path) -> Result<(), SaveError> {
    let ron_string = ron::ser::to_string_pretty(data, ron::ser::PrettyConfig::default())
        .map_err(|e| SaveError::SerializeError(e.to_string()))?;
    fs::write(path, ron_string)?;
    Ok(())
}

/// Read a RON file from disk and deserialize into `SaveData`.
/// Returns `VersionMismatch` if the save version differs from CURRENT_SAVE_VERSION.
pub fn load_game(path: &Path) -> Result<SaveData, SaveError> {
    let contents = fs::read_to_string(path)?;
    let data: SaveData =
        ron::from_str(&contents).map_err(|e| SaveError::DeserializeError(e.to_string()))?;
    if data.version != CURRENT_SAVE_VERSION {
        return Err(SaveError::VersionMismatch {
            expected: CURRENT_SAVE_VERSION,
            found: data.version,
        });
    }
    Ok(data)
}

/// Create a fresh new-game state.
///
/// Starting party: Adept (Venus), War Mage (Mars), Mystic (Mercury), Ranger (Jupiter).
/// All level 1, 100 HP, basic weapon equipped, no djinn, 200 gold, 0 XP.
pub fn create_new_game() -> SaveData {
    let starting_units = vec![
        SavedUnit {
            unit_id: UnitId("adept".into()),
            level: 1,
            current_hp: 100,
            equipment: SavedEquipment {
                weapon: Some(EquipmentId("wooden-sword".into())),
                ..SavedEquipment::default()
            },
            djinn: vec![],
        },
        SavedUnit {
            unit_id: UnitId("war-mage".into()),
            level: 1,
            current_hp: 100,
            equipment: SavedEquipment {
                weapon: Some(EquipmentId("wooden-axe".into())),
                ..SavedEquipment::default()
            },
            djinn: vec![],
        },
        SavedUnit {
            unit_id: UnitId("mystic".into()),
            level: 1,
            current_hp: 100,
            equipment: SavedEquipment {
                weapon: Some(EquipmentId("wooden-staff".into())),
                ..SavedEquipment::default()
            },
            djinn: vec![],
        },
        SavedUnit {
            unit_id: UnitId("ranger".into()),
            level: 1,
            current_hp: 100,
            equipment: SavedEquipment {
                weapon: Some(EquipmentId("wooden-sword".into())),
                ..SavedEquipment::default()
            },
            djinn: vec![],
        },
    ];

    SaveData {
        version: CURRENT_SAVE_VERSION,
        player_party: starting_units,
        roster: vec![],
        gold: 200,
        xp: 0,
        current_encounter_id: None,
        completed_encounters: vec![],
        inventory: vec![],
        team_djinn: vec![],
    }
}

// ── Tests ───────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    /// Helper: create a temp directory for test files, returns path.
    fn test_dir(name: &str) -> std::path::PathBuf {
        let dir = std::env::temp_dir().join(format!("vale_save_test_{}", name));
        let _ = fs::create_dir_all(&dir);
        dir
    }

    // ── 1. Save then load roundtrip ──

    #[test]
    fn save_load_roundtrip_data_matches() {
        let dir = test_dir("roundtrip");
        let path = dir.join("save.ron");

        let data = create_new_game();
        save_game(&data, &path).expect("save should succeed");
        let loaded = load_game(&path).expect("load should succeed");

        assert_eq!(data, loaded);

        let _ = fs::remove_dir_all(&dir);
    }

    // ── 2. create_new_game has correct starting party (4 units) ──

    #[test]
    fn new_game_has_four_party_members() {
        let data = create_new_game();
        assert_eq!(data.player_party.len(), 4);

        let ids: Vec<&str> = data
            .player_party
            .iter()
            .map(|u| u.unit_id.0.as_str())
            .collect();
        assert!(ids.contains(&"adept"));
        assert!(ids.contains(&"war-mage"));
        assert!(ids.contains(&"mystic"));
        assert!(ids.contains(&"ranger"));
    }

    // ── 3. create_new_game starting level is 1 ──

    #[test]
    fn new_game_starting_level_is_one() {
        let data = create_new_game();
        for unit in &data.player_party {
            assert_eq!(unit.level, 1, "unit {} should be level 1", unit.unit_id.0);
        }
    }

    // ── 4. Version mismatch detected on load ──

    #[test]
    fn version_mismatch_detected() {
        let dir = test_dir("version");
        let path = dir.join("old_save.ron");

        let mut data = create_new_game();
        data.version = 999;

        // Write directly with ron (bypassing version check in save_game)
        let ron_string =
            ron::ser::to_string_pretty(&data, ron::ser::PrettyConfig::default()).unwrap();
        fs::write(&path, ron_string).unwrap();

        let result = load_game(&path);
        assert!(result.is_err());
        match result.unwrap_err() {
            SaveError::VersionMismatch { expected, found } => {
                assert_eq!(expected, CURRENT_SAVE_VERSION);
                assert_eq!(found, 999);
            }
            other => panic!("expected VersionMismatch, got: {:?}", other),
        }

        let _ = fs::remove_dir_all(&dir);
    }

    // ── 5. Save to nonexistent directory: IoError ──

    #[test]
    fn save_to_nonexistent_dir_returns_io_error() {
        let path = Path::new("/tmp/vale_no_such_dir_xyz/nested/save.ron");
        let data = create_new_game();
        let result = save_game(&data, path);
        assert!(result.is_err());
        match result.unwrap_err() {
            SaveError::IoError(_) => {} // expected
            other => panic!("expected IoError, got: {:?}", other),
        }
    }

    // ── 6. Load nonexistent file: IoError ──

    #[test]
    fn load_nonexistent_file_returns_io_error() {
        let path = Path::new("/tmp/vale_no_such_file_xyz.ron");
        let result = load_game(path);
        assert!(result.is_err());
        match result.unwrap_err() {
            SaveError::IoError(_) => {} // expected
            other => panic!("expected IoError, got: {:?}", other),
        }
    }

    // ── 7. Completed encounters tracked correctly ──

    #[test]
    fn completed_encounters_roundtrip() {
        let dir = test_dir("encounters");
        let path = dir.join("save.ron");

        let mut data = create_new_game();
        data.completed_encounters = vec![
            EncounterId("forest_wolves".into()),
            EncounterId("bandit_camp".into()),
            EncounterId("cave_boss".into()),
        ];
        data.current_encounter_id = Some(EncounterId("mountain_pass".into()));

        save_game(&data, &path).expect("save should succeed");
        let loaded = load_game(&path).expect("load should succeed");

        assert_eq!(loaded.completed_encounters.len(), 3);
        assert_eq!(
            loaded.completed_encounters[0],
            EncounterId("forest_wolves".into())
        );
        assert_eq!(
            loaded.completed_encounters[1],
            EncounterId("bandit_camp".into())
        );
        assert_eq!(
            loaded.completed_encounters[2],
            EncounterId("cave_boss".into())
        );
        assert_eq!(
            loaded.current_encounter_id,
            Some(EncounterId("mountain_pass".into()))
        );

        let _ = fs::remove_dir_all(&dir);
    }

    // ── 8. Equipment roundtrip (equip, save, load, verify equipped) ──

    #[test]
    fn equipment_roundtrip() {
        let dir = test_dir("equipment");
        let path = dir.join("save.ron");

        let mut data = create_new_game();
        // Fully equip the first unit
        data.player_party[0].equipment = SavedEquipment {
            weapon: Some(EquipmentId("mythril_sword".into())),
            helm: Some(EquipmentId("steel_helm".into())),
            armor: Some(EquipmentId("iron_plate".into())),
            boots: Some(EquipmentId("speed_boots".into())),
            accessory: Some(EquipmentId("power_ring".into())),
        };

        save_game(&data, &path).expect("save should succeed");
        let loaded = load_game(&path).expect("load should succeed");

        let eq = &loaded.player_party[0].equipment;
        assert_eq!(eq.weapon, Some(EquipmentId("mythril_sword".into())));
        assert_eq!(eq.helm, Some(EquipmentId("steel_helm".into())));
        assert_eq!(eq.armor, Some(EquipmentId("iron_plate".into())));
        assert_eq!(eq.boots, Some(EquipmentId("speed_boots".into())));
        assert_eq!(eq.accessory, Some(EquipmentId("power_ring".into())));

        let _ = fs::remove_dir_all(&dir);
    }

    // ── 9. Djinn roundtrip ──

    #[test]
    fn djinn_state_roundtrip() {
        let dir = test_dir("djinn");
        let path = dir.join("save.ron");

        let mut data = create_new_game();
        data.player_party[0].djinn = vec![
            SavedDjinn {
                djinn_id: DjinnId("flint".into()),
                state: DjinnState::Good,
            },
            SavedDjinn {
                djinn_id: DjinnId("granite".into()),
                state: DjinnState::Recovery,
            },
        ];

        save_game(&data, &path).expect("save should succeed");
        let loaded = load_game(&path).expect("load should succeed");

        let djinn = &loaded.player_party[0].djinn;
        assert_eq!(djinn.len(), 2);
        assert_eq!(djinn[0].djinn_id, DjinnId("flint".into()));
        assert_eq!(djinn[0].state, DjinnState::Good);
        assert_eq!(djinn[1].djinn_id, DjinnId("granite".into()));
        assert_eq!(djinn[1].state, DjinnState::Recovery);

        let _ = fs::remove_dir_all(&dir);
    }

    // ── 10. Inventory and gold/xp roundtrip ──

    #[test]
    fn inventory_gold_xp_roundtrip() {
        let dir = test_dir("inventory");
        let path = dir.join("save.ron");

        let mut data = create_new_game();
        data.gold = 5000;
        data.xp = 12345;
        data.inventory = vec![
            EquipmentId("potion".into()),
            EquipmentId("bronze_helm".into()),
        ];

        save_game(&data, &path).expect("save should succeed");
        let loaded = load_game(&path).expect("load should succeed");

        assert_eq!(loaded.gold, 5000);
        assert_eq!(loaded.xp, 12345);
        assert_eq!(loaded.inventory.len(), 2);
        assert_eq!(loaded.inventory[0], EquipmentId("potion".into()));
        assert_eq!(loaded.inventory[1], EquipmentId("bronze_helm".into()));

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn team_djinn_roundtrip() {
        let dir = test_dir("team_djinn");
        let path = dir.join("save.ron");

        let mut data = create_new_game();
        data.team_djinn = vec![
            SavedDjinn {
                djinn_id: DjinnId("flint".into()),
                state: DjinnState::Good,
            },
            SavedDjinn {
                djinn_id: DjinnId("forge".into()),
                state: DjinnState::Recovery,
            },
        ];

        save_game(&data, &path).expect("save should succeed");
        let loaded = load_game(&path).expect("load should succeed");

        assert_eq!(loaded.team_djinn, data.team_djinn);

        let _ = fs::remove_dir_all(&dir);
    }

    // ══════════════════════════════════════════════════════════════
    // P0 Graduation: Full save/load round-trip with rich game state
    // ══════════════════════════════════════════════════════════════

    /// P0 graduation test: create a mid-game save state with every field
    /// populated (party with equipment and djinn, roster, gold, xp,
    /// completed encounters, inventory, team djinn), save to disk, reload,
    /// and verify byte-level identity.
    #[test]
    fn graduation_save_load_full_state_roundtrip() {
        let dir = test_dir("graduation_full");
        let path = dir.join("save.ron");

        let mut data = create_new_game();

        // Simulate mid-game progress
        data.gold = 3750;
        data.xp = 8200;
        data.current_encounter_id = Some(EncounterId("house-12".into()));
        data.completed_encounters = vec![
            EncounterId("house-01".into()),
            EncounterId("house-02".into()),
            EncounterId("house-03".into()),
            EncounterId("house-04".into()),
            EncounterId("house-05".into()),
        ];
        data.inventory = vec![
            EquipmentId("bronze-sword".into()),
            EquipmentId("iron-helm".into()),
            EquipmentId("potion".into()),
        ];

        // Level up party, equip gear, assign per-unit djinn
        data.player_party[0].level = 8;
        data.player_party[0].current_hp = 95;
        data.player_party[0].equipment = SavedEquipment {
            weapon: Some(EquipmentId("iron-sword".into())),
            helm: Some(EquipmentId("leather-cap".into())),
            armor: Some(EquipmentId("chain-mail".into())),
            boots: None,
            accessory: Some(EquipmentId("power-ring".into())),
        };
        data.player_party[0].djinn = vec![SavedDjinn {
            djinn_id: DjinnId("bane".into()),
            state: DjinnState::Good,
        }];

        data.player_party[1].level = 7;
        data.player_party[1].current_hp = 0; // KO'd unit
        data.player_party[1].equipment.weapon =
            Some(EquipmentId("flame-axe".into()));

        // Add roster members (recruited units not in active party)
        data.roster = vec![
            SavedUnit {
                unit_id: UnitId("felix".into()),
                level: 3,
                current_hp: 120,
                equipment: SavedEquipment::default(),
                djinn: vec![],
            },
            SavedUnit {
                unit_id: UnitId("karis".into()),
                level: 5,
                current_hp: 88,
                equipment: SavedEquipment {
                    weapon: Some(EquipmentId("wind-blade".into())),
                    ..SavedEquipment::default()
                },
                djinn: vec![SavedDjinn {
                    djinn_id: DjinnId("breeze".into()),
                    state: DjinnState::Recovery,
                }],
            },
        ];

        // Team-wide djinn pool
        data.team_djinn = vec![
            SavedDjinn {
                djinn_id: DjinnId("flint".into()),
                state: DjinnState::Good,
            },
            SavedDjinn {
                djinn_id: DjinnId("forge".into()),
                state: DjinnState::Good,
            },
            SavedDjinn {
                djinn_id: DjinnId("granite".into()),
                state: DjinnState::Recovery,
            },
        ];

        // Save → Load → Assert identity
        save_game(&data, &path).expect("save should succeed");
        let loaded = load_game(&path).expect("load should succeed");

        assert_eq!(data, loaded, "full state must survive round-trip exactly");

        // Verify specific high-stakes fields individually
        assert_eq!(loaded.gold, 3750);
        assert_eq!(loaded.xp, 8200);
        assert_eq!(loaded.completed_encounters.len(), 5);
        assert_eq!(loaded.player_party[0].level, 8);
        assert_eq!(loaded.player_party[0].current_hp, 95);
        assert_eq!(loaded.player_party[1].current_hp, 0, "KO state must survive");
        assert_eq!(loaded.roster.len(), 2);
        assert_eq!(loaded.roster[1].djinn[0].state, DjinnState::Recovery);
        assert_eq!(loaded.team_djinn.len(), 3);
        assert_eq!(
            loaded.current_encounter_id,
            Some(EncounterId("house-12".into()))
        );

        // Verify the RON file is human-readable (not binary)
        let raw = fs::read_to_string(&path).expect("file should be readable text");
        assert!(raw.contains("house-12"), "RON should contain encounter ID as text");
        assert!(raw.contains("flint"), "RON should contain djinn ID as text");

        let _ = fs::remove_dir_all(&dir);
    }
}
