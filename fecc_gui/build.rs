// Copyright (C) 2025 aidan-es. Licensed under the GNU AGPLv3.
use std::env;
use std::fs::{self, File};
use std::io::Write as _;
use std::path::Path;

fn main() {
    println!("cargo:rerun-if-changed=../art");

    let out_dir = env::var("OUT_DIR").expect("OUT_DIR environment variable not set.");
    let manifest_path = Path::new(&out_dir).join("asset_manifest.json");

    let mut files = Vec::new();
    let art_dir = Path::new("../art");

    if art_dir.is_dir() {
        for entry in fs::read_dir(art_dir).expect("Failed to read art directory") {
            let entry = entry.expect("Failed to read directory entry");
            let path = entry.path();
            if path.is_file()
                && path.extension().and_then(|s| s.to_str()) == Some("png")
                && let Some(file_name) = path.file_name().and_then(|s| s.to_str())
            {
                files.push(file_name.to_owned());
            }
        }
    }

    files.sort();

    let json = serde_json::json!({ "files": files });
    let mut file = File::create(&manifest_path).expect("Failed to create manifest file");
    file.write_all(json.to_string().as_bytes())
        .expect("Failed to write to manifest file");

    let assets_manifest_path = Path::new("../assets").join("asset_manifest.json");
    fs::copy(&manifest_path, &assets_manifest_path).expect("Failed to copy manifest to assets");
}
