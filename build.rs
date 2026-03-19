use std::fs;
use std::path::Path;

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=status/draft-enemy-manifest.toml");
    println!("cargo:rerun-if-changed=status/draft-djinn-manifest.toml");

    // Count expected vs found sprites
    let mut expected = 0u32;
    let mut found = 0u32;
    let mut missing = Vec::new();

    // Check enemy sprites from manifest
    if let Ok(content) = fs::read_to_string("status/draft-enemy-manifest.toml") {
        for line in content.lines() {
            let line = line.trim();
            if line.starts_with("sprite_idle") || line.starts_with("sprite_attack") || line.starts_with("sprite_hit") || line.starts_with("sprite =") {
                if let Some(path) = line.split('"').nth(1) {
                    expected += 1;
                    if Path::new(path).exists() {
                        found += 1;
                    } else {
                        missing.push(path.to_string());
                    }
                }
            }
        }
    }

    // Check djinn sprites
    if let Ok(content) = fs::read_to_string("status/draft-djinn-manifest.toml") {
        for line in content.lines() {
            let line = line.trim();
            if line.starts_with("sprite =") {
                if let Some(path) = line.split('"').nth(1) {
                    expected += 1;
                    if Path::new(path).exists() {
                        found += 1;
                    } else {
                        missing.push(path.to_string());
                    }
                }
            }
        }
    }

    if expected > 0 {
        println!("cargo:warning=Sprites: {found}/{expected} present ({} missing)", missing.len());
        if missing.len() <= 10 {
            for m in &missing {
                println!("cargo:warning=  Missing: {m}");
            }
        }
    }
}
