// Copyright (C) 2025 aidan-es. Licensed under the GNU AGPLv3.
use crate::asset::{Asset, AssetType};
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
            let path = std::path::PathBuf::from(format!("art/{}", filename));
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
                library.insert(asset.id.clone(), asset);
            }
        }
        Err(e) => {
            log::warn!("Skipping file {path:?}: {e}");
        }
    }
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
    let url = path.to_str().ok_or("Invalid path")?;
    let bytes_val = wasm::fetch_image_bytes(url)
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
}
