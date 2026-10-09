// Copyright (C) 2025 aidan-es. Licensed under the GNU AGPLv3.
pub mod legacy;

use crate::asset::{Asset, AssetType};
use crate::character::Character;
use crate::types::Rgba;
use indexmap::IndexMap;
#[cfg(target_arch = "wasm32")]
use js_sys;
#[cfg(target_arch = "wasm32")]
use serde_wasm_bindgen;
use std::collections::HashMap;
use std::error::Error;
use std::path::{Path, PathBuf};

/// Asynchronously loads all character assets from the `art` directory into libraries. (Be it local or remote)
///
/// Handles asset loading for both native and WebAssembly (WASM) builds.
/// For native builds, it scans the `art` directory directly. For WASM, it fetches a
/// manifest file and then loads the assets listed within it.
pub async fn load_asset_libraries()
-> Result<HashMap<AssetType, IndexMap<String, Asset>>, Box<dyn Error + Send + Sync>> {
    let mut asset_libraries: HashMap<AssetType, IndexMap<String, Asset>> = [
        (AssetType::Armour, IndexMap::new()),
        (AssetType::Face, IndexMap::new()),
        (AssetType::Hair, IndexMap::new()),
        (AssetType::HairBack, IndexMap::new()),
        (AssetType::Accessory, IndexMap::new()),
        (AssetType::Token, IndexMap::new()),
    ]
    .into_iter()
    .collect();

    #[cfg(not(target_arch = "wasm32"))]
    {
        let path_pattern = "art/*.png";
        for path in glob::glob(path_pattern)
            .expect("Failed to read glob pattern")
            .flatten()
        {
            add_asset_to_library(&mut asset_libraries, &path);
        }
    }

    #[cfg(target_arch = "wasm32")]
    {
        let asset_list_val = wasm::fetch_asset_list("assets/asset_manifest.json")
            .await
            .map_err(|e| e.as_string().unwrap_or_else(|| "JS error".to_string()))?;

        let files: Vec<String> =
            serde_wasm_bindgen::from_value(asset_list_val).map_err(|e| e.to_string())?;

        for filename in files {
            let path = PathBuf::from(format!("art/{}", filename));
            add_asset_to_library(&mut asset_libraries, &path);
        }
    }

    Ok(asset_libraries)
}

/// Parses an asset from a path and adds it to the appropriate library.
fn add_asset_to_library(
    asset_libraries: &mut HashMap<AssetType, IndexMap<String, Asset>>,
    path: &PathBuf,
) {
    match Asset::try_from(path.as_path()) {
        Ok(asset) => {
            if let Some(library) = asset_libraries.get_mut(&asset.asset_type) {
                if let Some(existing) = library.get(&asset.id) {
                    log::warn!(
                        "{path:?} has the same name as {:?} once tags are removed, so it replaces it",
                        existing.path
                    );
                }
                library.insert(asset.id.clone(), asset);
            }
        }
        Err(e) => {
            log::warn!("Skipping file {path:?}: {e}");
        }
    }
}

/// The libraries the native app loads from the `art` directory.
#[cfg(test)]
pub(crate) fn repository_art_libraries() -> HashMap<AssetType, IndexMap<String, Asset>> {
    use strum::IntoEnumIterator as _;

    let art_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("../art");
    let mut paths: Vec<PathBuf> = std::fs::read_dir(&art_dir)
        .expect("Failed to read the art directory")
        .map(|entry| entry.expect("Failed to read the art directory").path())
        .filter(|path| path.extension().is_some_and(|extension| extension == "png"))
        .collect();
    paths.sort();

    let mut asset_libraries = AssetType::iter()
        .map(|asset_type| (asset_type, IndexMap::new()))
        .collect();
    for path in &paths {
        add_asset_to_library(&mut asset_libraries, path);
    }
    asset_libraries
}

/// Asynchronously loads a list of colours from a CSV file.
///
/// On native builds, it reads from the local filesystem.
/// On WASM, it fetches the file via a JavaScript call.
#[cfg(not(target_arch = "wasm32"))]
pub async fn load_colours_from_csv(path: &str) -> Result<Vec<Rgba>, Box<dyn Error + Send + Sync>> {
    let full_path = format!("assets/csv/{path}");
    let content = tokio::fs::read_to_string(full_path).await?;
    let mut reader = csv::ReaderBuilder::new()
        .has_headers(false)
        .trim(csv::Trim::All)
        .from_reader(content.as_bytes());
    parse_colours(&mut reader)
}

/// JavaScript bindings for file operations in a WASM environment.
#[cfg(target_arch = "wasm32")]
mod wasm {
    use wasm_bindgen::prelude::*;

    #[wasm_bindgen(module = "/src/file_io.js")]
    extern "C" {
        #[wasm_bindgen(js_name = fetch_text, catch)]
        pub async fn fetch_text(url: &str) -> Result<JsValue, JsValue>;

        #[wasm_bindgen(js_name = fetch_image_bytes, catch)]
        pub async fn fetch_image_bytes(url: &str) -> Result<JsValue, JsValue>;

        #[wasm_bindgen(js_name = fetch_asset_list, catch)]
        pub async fn fetch_asset_list(url: &str) -> Result<JsValue, JsValue>;

        #[wasm_bindgen(js_name = trigger_download, catch)]
        pub fn trigger_download(bytes: &[u8], filename: &str) -> Result<JsValue, JsValue>;
    }
}

/// Asynchronously loads a list of colours from a CSV file
#[cfg(target_arch = "wasm32")]
pub async fn load_colours_from_csv(path: &str) -> Result<Vec<Rgba>, Box<dyn Error + Send + Sync>> {
    let url = format!("assets/csv/{}", path);
    let text = wasm::fetch_text(&url)
        .await
        .map_err(|e| e.as_string().unwrap_or_else(|| "JS error".to_string()))?
        .as_string()
        .ok_or("Failed to get file content as string from JS")?;

    let mut reader = csv::ReaderBuilder::new()
        .has_headers(false)
        .trim(csv::Trim::All)
        .from_reader(text.as_bytes());

    parse_colours(&mut reader)
}

/// Asynchronously loads the raw bytes of an image file.
#[cfg(not(target_arch = "wasm32"))]
pub async fn load_image_bytes(path: &Path) -> Result<Vec<u8>, Box<dyn Error + Send + Sync>> {
    Ok(tokio::fs::read(path).await?)
}

/// Asynchronously loads the raw bytes of an image file (WASM version).
#[cfg(target_arch = "wasm32")]
pub async fn load_image_bytes(path: &Path) -> Result<Vec<u8>, Box<dyn Error + Send + Sync>> {
    let path = path.to_str().ok_or("Invalid path")?;
    // Encode each part of the path, as file names may contain tags such as `{Iscaneus}`
    let url = path
        .split('/')
        .map(|part| String::from(js_sys::encode_uri_component(part)))
        .collect::<Vec<_>>()
        .join("/");
    let bytes_val = wasm::fetch_image_bytes(&url)
        .await
        .map_err(|e| e.as_string().unwrap_or_else(|| "JS error".to_string()))?;
    let bytes: Vec<u8> = js_sys::Uint8Array::new(&bytes_val).to_vec();
    Ok(bytes)
}

/// Parses colours from a CSV reader.
fn parse_colours<R: std::io::Read>(
    reader: &mut csv::Reader<R>,
) -> Result<Vec<Rgba>, Box<dyn Error + Send + Sync>> {
    let mut colours = Vec::new();
    for result in reader.records() {
        let record = result?;
        for field in &record {
            match Rgba::from_hex(field) {
                Ok(colour) => colours.push(colour),
                Err(e) => {
                    log::warn!("Record: {record:?}. Failed to parse hex '{field}': {e:?}");
                }
            }
        }
    }
    Ok(colours)
}

/// Triggers a file download in the browser
#[cfg(target_arch = "wasm32")]
pub fn trigger_download(bytes: &[u8], filename: &str) -> Result<(), Box<dyn Error + Send + Sync>> {
    wasm::trigger_download(bytes, filename)
        .map_err(|e| e.as_string().unwrap_or_else(|| "JS Error".to_string()))?;
    Ok(())
}

/// Parses a character save file from text content.
///
/// Automatically detects whether the content is a modern FECC 4e JSON save or
/// a legacy V3 save file. If `file_stem` is provided and the loaded character has
/// an empty name, `file_stem` will be used as the character's name. The parts of a
/// FECC 4e save are relinked to the libraries, following any art renamed since it was saved.
pub fn parse_character_save(
    content: &str,
    file_stem: Option<&str>,
    asset_libraries: &HashMap<AssetType, IndexMap<String, Asset>>,
) -> Result<Character, String> {
    if content.trim_start().starts_with('{') {
        let mut character: Character =
            serde_json::from_str(content).map_err(|e| format!("Failed to parse JSON save: {e}"))?;
        if character.name.is_empty()
            && let Some(stem) = file_stem
        {
            character.name = stem.to_owned();
        }
        character.relink_assets(asset_libraries);
        Ok(character)
    } else if legacy::is_legacy_save(content) {
        legacy::parse_legacy_save(content, file_stem, asset_libraries)
    } else {
        Err(
            "Unrecognized save file format. Expected FECC 4e JSON or legacy V3 save file."
                .to_owned(),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::asset::AssetType;
    use indexmap::IndexMap;
    use std::collections::HashMap;
    use std::path::PathBuf;

    #[test]
    fn test_add_asset_to_library_success() {
        let mut libraries = HashMap::new();
        libraries.insert(AssetType::Face, IndexMap::new());

        let path = PathBuf::from("assets/Test_Face.png");
        add_asset_to_library(&mut libraries, &path);

        assert!(
            libraries
                .get(&AssetType::Face)
                .unwrap()
                .contains_key("Test_Face")
        );
    }

    #[test]
    fn test_add_asset_to_library_invalid_type() {
        let mut libraries = HashMap::new();
        libraries.insert(AssetType::Face, IndexMap::new());

        let path = PathBuf::from("assets/Test_Unknown.png");
        add_asset_to_library(&mut libraries, &path);

        assert!(libraries.get(&AssetType::Face).unwrap().is_empty());
    }

    #[test]
    fn test_parse_colours_valid() {
        let csv_data = "FF0000\n00FF00\n0000FF";
        let mut reader = csv::ReaderBuilder::new()
            .has_headers(false)
            .trim(csv::Trim::All)
            .from_reader(csv_data.as_bytes());

        let colours = parse_colours(&mut reader).unwrap();
        assert_eq!(colours.len(), 3);
        assert_eq!(colours[0], Rgba::new(255, 0, 0, 255));
        assert_eq!(colours[1], Rgba::new(0, 255, 0, 255));
        assert_eq!(colours[2], Rgba::new(0, 0, 255, 255));
    }

    #[test]
    fn test_parse_colours_with_invalid() {
        let csv_data = "FF0000,InvalidHex";
        let mut reader = csv::ReaderBuilder::new()
            .has_headers(false)
            .trim(csv::Trim::All)
            .from_reader(csv_data.as_bytes());

        let colours = parse_colours(&mut reader).unwrap();
        assert_eq!(colours.len(), 1);
        assert_eq!(colours[0], Rgba::new(255, 0, 0, 255));
    }

    #[test]
    fn test_parse_character_save_json() {
        let libraries = HashMap::new();
        let json_data = r#"{"name":"Hero","parts":{},"colours":{}}"#;
        let character = parse_character_save(json_data, Some("Fallback"), &libraries).unwrap();
        assert_eq!(character.name, "Hero");

        let json_no_name = r#"{"name":"","parts":{},"colours":{}}"#;
        let character_stem =
            parse_character_save(json_no_name, Some("StemName"), &libraries).unwrap();
        assert_eq!(character_stem.name, "StemName");
    }

    #[test]
    fn test_parse_character_save_legacy() {
        let mut libraries = HashMap::new();
        libraries.insert(AssetType::Armour, IndexMap::new());
        libraries.insert(AssetType::Face, IndexMap::new());
        libraries.insert(AssetType::Hair, IndexMap::new());
        libraries.insert(AssetType::HairBack, IndexMap::new());
        libraries.insert(AssetType::Accessory, IndexMap::new());
        libraries.insert(AssetType::Token, IndexMap::new());

        let legacy_content =
            "===COLOR_START===\n===COLOR_END===\n===TOOLBOX_START===\n===TOOLBOX_END===\n";
        let character =
            parse_character_save(legacy_content, Some("LegacyHero"), &libraries).unwrap();
        assert_eq!(character.name, "LegacyHero");
    }

    #[test]
    fn test_parse_character_save_unrecognized() {
        let libraries = HashMap::new();
        let result = parse_character_save("NOT A VALID FORMAT", Some("Test"), &libraries);
        match result {
            Err(e) => assert!(e.contains("Unrecognized save file format")),
            Ok(_) => panic!("Expected error for unrecognized format"),
        }
    }

    /// The asset id of each part, in drawing order, after checking that each part has the
    /// library's copy of its asset.
    fn part_ids<'a>(
        character: &'a Character,
        libraries: &HashMap<AssetType, IndexMap<String, Asset>>,
    ) -> [Option<&'a str>; 6] {
        [
            &character.hair_back,
            &character.armour,
            &character.face,
            &character.hair,
            &character.accessory,
            &character.token,
        ]
        .map(|part| {
            part.as_ref().map(|part| {
                let asset = &part.asset;
                assert_eq!(
                    libraries[&asset.asset_type].get(&asset.id),
                    Some(asset),
                    "{} is not the library's copy",
                    asset.id
                );
                asset.id.as_str()
            })
        })
    }

    #[test]
    fn test_repository_art_names_are_unique_without_tags() {
        let art_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("../art");
        let files = std::fs::read_dir(&art_dir)
            .unwrap()
            .filter(|entry| {
                let path = entry.as_ref().unwrap().path();
                path.extension().is_some_and(|extension| extension == "png")
            })
            .count();
        let assets: usize = repository_art_libraries().values().map(IndexMap::len).sum();
        assert_eq!(
            assets, files,
            "Two art files have the same name once tags are removed."
        );
    }

    #[test]
    fn test_parse_character_save_4e_with_old_names() {
        // A FECC 4e save made before Legualt, EirkOld, Teifling2 and sage_casting were renamed.
        let content = include_str!("../tests/fixtures/4e_save_0-2-0.fecc");
        let libraries = repository_art_libraries();

        let character = parse_character_save(content, None, &libraries).unwrap();

        assert_eq!(
            part_ids(&character, &libraries),
            [
                Some("Tiefling2_HairBack"),
                Some("ErikOld_Armour"),
                Some("Legault_Face"),
                Some("Tiefling2_Hair"),
                None,
                Some("SageCasting_Token"),
            ]
        );
    }

    #[test]
    fn test_parse_character_save_v3_with_old_names() {
        // A V3 save that uses Eirk, Legualt, Teifling2 and sage_casting.
        let content = include_str!("../tests/fixtures/v3_save2.fecc");
        let libraries = repository_art_libraries();

        let character = parse_character_save(content, None, &libraries).unwrap();

        assert_eq!(
            part_ids(&character, &libraries),
            [
                Some("Tiefling2_HairBack"),
                Some("Legault_Armour"),
                Some("ErikOld_Face"),
                Some("Tiefling2_Hair"),
                None,
                Some("SageCasting_Token"),
            ]
        );
    }
}
