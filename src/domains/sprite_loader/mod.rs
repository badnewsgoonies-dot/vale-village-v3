//! Sprite loader domain.
//! Reads draft sprite manifests and builds a registry of image handles.

use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

use bevy::asset::RenderAssetUsages;
use bevy::prelude::*;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};

const ENEMY_MANIFEST_PATH: &str = "status/draft-enemy-manifest.toml";
const DJINN_MANIFEST_PATH: &str = "status/draft-djinn-manifest.toml";
const ASSET_ROOT: &str = "assets";

#[derive(Resource, Default)]
pub struct SpriteRegistry {
    pub enemy_idle: HashMap<String, Handle<Image>>,
    pub enemy_attack: HashMap<String, Handle<Image>>,
    pub djinn: HashMap<String, Handle<Image>>,
    pub unit_portraits: HashMap<String, Handle<Image>>,
    pub fallback: Handle<Image>,
}

impl SpriteRegistry {
    pub fn get_unit_portrait(&self, id: &str) -> Handle<Image> {
        self.unit_portraits
            .get(id)
            .cloned()
            .unwrap_or_else(|| self.fallback.clone())
    }

    pub fn get_enemy_idle(&self, id: &str) -> Handle<Image> {
        self.enemy_idle
            .get(id)
            .cloned()
            .unwrap_or_else(|| self.fallback.clone())
    }

    pub fn get_enemy_attack(&self, id: &str) -> Handle<Image> {
        self.enemy_attack
            .get(id)
            .cloned()
            .unwrap_or_else(|| self.fallback.clone())
    }

    pub fn get_djinn(&self, id: &str) -> Handle<Image> {
        self.djinn
            .get(id)
            .cloned()
            .unwrap_or_else(|| self.fallback.clone())
    }
}

pub struct SpriteLoaderPlugin;

impl Plugin for SpriteLoaderPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<SpriteRegistry>()
            .add_systems(Startup, load_sprites);
    }
}

pub fn load_sprites(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut images: ResMut<Assets<Image>>,
) {
    let fallback = images.add(Image::new_fill(
        Extent3d {
            width: 1,
            height: 1,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        &[255, 0, 255, 255],
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::default(),
    ));

    let mut registry = SpriteRegistry {
        fallback: fallback.clone(),
        ..Default::default()
    };

    if let Ok(content) = fs::read_to_string(ENEMY_MANIFEST_PATH) {
        for entry in parse_enemy_manifest(&content) {
            let idle = load_or_fallback(entry.sprite_idle.as_deref(), &asset_server, &fallback);
            let attack = load_or_fallback(entry.sprite_attack.as_deref(), &asset_server, &fallback);
            registry.enemy_idle.insert(entry.id.clone(), idle);
            registry.enemy_attack.insert(entry.id, attack);
        }
    }

    if let Ok(content) = fs::read_to_string(DJINN_MANIFEST_PATH) {
        for entry in parse_djinn_manifest(&content) {
            let sprite = load_or_fallback(entry.sprite.as_deref(), &asset_server, &fallback);
            registry.djinn.insert(entry.id, sprite);
        }
    }

    // Scan assets/sprites/units/ for player unit portraits.
    // Convention: <unit_id>_portrait.png → key is the unit_id (e.g. "adept").
    let unit_sprite_dir = Path::new(ASSET_ROOT).join("sprites/units");
    if let Ok(entries) = fs::read_dir(&unit_sprite_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().and_then(|e| e.to_str()) == Some("png") {
                if let Some(stem) = path.file_stem().and_then(|s| s.to_str()) {
                    let unit_id = stem.trim_end_matches("_portrait").to_string();
                    let asset_path = normalize_asset_path(&path);
                    let handle = asset_server.load(asset_path);
                    registry.unit_portraits.insert(unit_id, handle);
                }
            }
        }
    }

    commands.insert_resource(registry);
}

#[derive(Default)]
struct EnemyManifestEntry {
    id: String,
    sprite_idle: Option<String>,
    sprite_attack: Option<String>,
}

#[derive(Default)]
struct DjinnManifestEntry {
    id: String,
    sprite: Option<String>,
}

fn parse_enemy_manifest(content: &str) -> Vec<EnemyManifestEntry> {
    let mut entries = Vec::new();
    let mut current: Option<EnemyManifestEntry> = None;

    for raw_line in content.lines() {
        let line = raw_line.trim();
        if line.is_empty() {
            continue;
        }

        if line.starts_with("[entities.") && line.ends_with(']') {
            if let Some(entry) = current.take() {
                entries.push(entry);
            }

            let id = line
                .trim_start_matches("[entities.")
                .trim_end_matches(']')
                .to_string();
            current = Some(EnemyManifestEntry {
                id,
                ..Default::default()
            });
            continue;
        }

        let Some((key, value)) = line.split_once('=') else {
            continue;
        };
        let value = parse_string_value(value);

        if let Some(entry) = current.as_mut() {
            match key.trim() {
                "sprite_idle" => entry.sprite_idle = Some(value),
                "sprite_attack" => entry.sprite_attack = Some(value),
                _ => {}
            }
        }
    }

    if let Some(entry) = current {
        entries.push(entry);
    }

    entries
}

fn parse_djinn_manifest(content: &str) -> Vec<DjinnManifestEntry> {
    let mut entries = Vec::new();
    let mut current: Option<DjinnManifestEntry> = None;

    for raw_line in content.lines() {
        let line = raw_line.trim();
        if line.is_empty() {
            continue;
        }

        if line == "[[djinn]]" {
            if let Some(entry) = current.take() {
                entries.push(entry);
            }
            current = Some(DjinnManifestEntry::default());
            continue;
        }

        let Some((key, value)) = line.split_once('=') else {
            continue;
        };
        let value = parse_string_value(value);

        if let Some(entry) = current.as_mut() {
            match key.trim() {
                "id" => entry.id = value,
                "sprite" => entry.sprite = Some(value),
                _ => {}
            }
        }
    }

    if let Some(entry) = current {
        entries.push(entry);
    }

    entries
}

fn parse_string_value(raw: &str) -> String {
    raw.trim().trim_matches('"').to_string()
}

fn load_or_fallback(
    manifest_path: Option<&str>,
    asset_server: &AssetServer,
    fallback: &Handle<Image>,
) -> Handle<Image> {
    let Some(manifest_path) = manifest_path else {
        return fallback.clone();
    };

    let disk_path = Path::new(manifest_path);
    if !disk_path.exists() {
        return fallback.clone();
    }

    let asset_path = normalize_asset_path(disk_path);
    asset_server.load(asset_path)
}

fn normalize_asset_path(path: &Path) -> PathBuf {
    path.strip_prefix(ASSET_ROOT)
        .map(Path::to_path_buf)
        .unwrap_or_else(|_| path.to_path_buf())
}
