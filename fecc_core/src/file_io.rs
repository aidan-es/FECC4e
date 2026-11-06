// Copyright (C) 2025-2026 aidan-es. Licensed under the GNU AGPLv3.
pub mod legacy;

use crate::asset::{Asset, AssetType};
use crate::character::Character;
use crate::types::Rgba;
use indexmap::IndexMap;
#[cfg(target_arch = "wasm32")]
use js_sys;
use std::collections::HashMap;
use std::error::Error;
use std::path::{Path, PathBuf};
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::JsCast as _;

/// The folder the native app reads `art` and `assets` from.
///
/// Uses the executable's folder, as the release archives ship both beside it.
/// Falls back to the working directory, where `cargo run` finds the repository's copies.
#[cfg(not(target_arch = "wasm32"))]
pub fn resource_dir() -> &'static Path {
    static RESOURCE_DIR: std::sync::OnceLock<PathBuf> = std::sync::OnceLock::new();

    RESOURCE_DIR.get_or_init(|| {
        let exe_dir = std::env::current_exe()
            .ok()
            // Follow a link to the binary, such as one in `~/.local/bin`
            .map(|exe| {
                if exe.is_symlink() {
                    std::fs::canonicalize(&exe).unwrap_or(exe)
                } else {
                    exe
                }
            })
            .and_then(|exe| exe.parent().map(Path::to_path_buf))
            .filter(|dir| dir.join("art").is_dir());

        match exe_dir {
            Some(dir) => {
                log::info!("Loading art from {}", dir.display());
                dir
            }
            None => {
                log::info!("Loading art from the working directory");
                PathBuf::new()
            }
        }
    })
}

/// Asynchronously loads all character assets from the `art` directory into libraries.
///
/// The web build has no directory to scan, so it uses [`baked_asset_libraries`] instead.
#[cfg(not(target_arch = "wasm32"))]
pub async fn load_asset_libraries()
-> Result<HashMap<AssetType, IndexMap<String, Asset>>, Box<dyn Error + Send + Sync>> {
    let art_dir = resource_dir().join("art");
    art_libraries_in(&art_dir).map_err(|e| {
        let shown_dir = std::path::absolute(&art_dir).unwrap_or(art_dir);
        format!("{}: {e}", shown_dir.display()).into()
    })
}

/// Builds the libraries from the PNG files in an `art` directory.
#[cfg(any(test, not(target_arch = "wasm32")))]
fn art_libraries_in(
    art_dir: &Path,
) -> std::io::Result<HashMap<AssetType, IndexMap<String, Asset>>> {
    let mut file_names = Vec::new();
    for entry in std::fs::read_dir(art_dir)? {
        let path = entry?.path();
        if path.extension().is_none_or(|extension| extension != "png") {
            continue;
        }
        match path.file_name().and_then(|name| name.to_str()) {
            Some(name) => file_names.push(name.to_owned()),
            None => log::warn!("Skipping file {path:?}: its name isn't valid UTF-8"),
        }
    }
    // Sort, as the directory order varies by platform and the library order is the display order
    file_names.sort();

    Ok(asset_libraries_from_files(&file_names))
}

/// Builds the libraries from the `art` file list that build.rs bakes into the web build.
///
/// Needs no fetch, so the libraries are ready for the first frame.
#[cfg(target_arch = "wasm32")]
pub fn baked_asset_libraries() -> HashMap<AssetType, IndexMap<String, Asset>> {
    asset_libraries_from_files(web_art::ART_FILES)
}

/// Builds the libraries from a list of `art` file names.
fn asset_libraries_from_files(
    files: &[impl AsRef<str>],
) -> HashMap<AssetType, IndexMap<String, Asset>> {
    let mut asset_libraries = empty_asset_libraries();

    for file_name in files {
        let path = PathBuf::from(format!("art/{}", file_name.as_ref()));
        add_asset_to_library(&mut asset_libraries, &path);
    }

    asset_libraries
}

/// An empty library for each asset type.
fn empty_asset_libraries() -> HashMap<AssetType, IndexMap<String, Asset>> {
    use strum::IntoEnumIterator as _;

    AssetType::iter()
        .map(|asset_type| (asset_type, IndexMap::new()))
        .collect()
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

/// The libraries the native app loads from the repository's `art` directory.
#[cfg(test)]
pub(crate) fn repository_art_libraries() -> HashMap<AssetType, IndexMap<String, Asset>> {
    art_libraries_in(&Path::new(env!("CARGO_MANIFEST_DIR")).join("../art"))
        .expect("Failed to read the art directory")
}

/// Asynchronously loads a list of colours from a CSV file.
///
/// On native builds, it reads from the local filesystem.
/// On WASM, it fetches the file via a JavaScript call.
#[cfg(not(target_arch = "wasm32"))]
pub async fn load_colours_from_csv(path: &str) -> Result<Vec<Rgba>, Box<dyn Error + Send + Sync>> {
    let full_path = resource_dir().join("assets/csv").join(path);
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

        #[wasm_bindgen(js_name = trigger_download, catch)]
        pub fn trigger_download(bytes: &[u8], filename: &str) -> Result<JsValue, JsValue>;
    }

    // Shared with the 404 page, as index.html has Trunk copy it to the site root
    #[wasm_bindgen(module = "/src/art_bundle.js")]
    extern "C" {
        #[wasm_bindgen(js_name = fetch_gzip_bytes, catch)]
        pub async fn fetch_gzip_bytes(url: &str, priority: &str) -> Result<JsValue, JsValue>;
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
///
/// Reads library paths such as `art/Lyn_Face.png` from the [`resource_dir`].
#[cfg(not(target_arch = "wasm32"))]
pub async fn load_image_bytes(path: &Path) -> Result<Vec<u8>, Box<dyn Error + Send + Sync>> {
    Ok(tokio::fs::read(resource_dir().join(path)).await?)
}

/// Asynchronously loads the raw bytes of an image file for background preloading.
///
/// The same as [`load_image_bytes`] on native, where there is no network to share.
#[cfg(not(target_arch = "wasm32"))]
pub async fn preload_image_bytes(path: &Path) -> Result<Vec<u8>, Box<dyn Error + Send + Sync>> {
    load_image_bytes(path).await
}

/// Asynchronously loads the raw bytes of an `art` image (WASM version).
///
/// Takes it from its part type's bundle, which is fetched the first time it's needed.
#[cfg(target_arch = "wasm32")]
pub async fn load_image_bytes(path: &Path) -> Result<Vec<u8>, Box<dyn Error + Send + Sync>> {
    web_art::load(path, "auto").await
}

/// Asynchronously loads the raw bytes of an `art` image for background preloading (WASM version).
///
/// Starts fetching every bundle at low priority, so the browser and server can put art the user
/// is waiting on first, and the preload doesn't wait on the bundles one at a time.
#[cfg(target_arch = "wasm32")]
pub async fn preload_image_bytes(path: &Path) -> Result<Vec<u8>, Box<dyn Error + Send + Sync>> {
    web_art::load(path, "low").await
}

/// The web build's art, which comes in one bundle per part type.
///
/// A thousand small requests cost more than the art itself, and add a fetch callback for each
/// image. See [`fecc_art_bundle`] for the format.
#[cfg(target_arch = "wasm32")]
mod web_art {
    use super::wasm;
    use futures_util::FutureExt as _;
    use futures_util::future::{LocalBoxFuture, Shared};
    use std::cell::RefCell;
    use std::collections::HashMap;
    use std::error::Error;
    use std::ops::Range;
    use std::path::Path;
    use std::rc::Rc;
    use wasm_bindgen::{JsCast as _, JsValue};

    include!(concat!(env!("OUT_DIR"), "/art.rs"));

    /// A fetched bundle, with where each file sits in it.
    struct Bundle {
        contents: Vec<u8>,
        files: HashMap<String, Range<usize>>,
    }

    /// A bundle fetch that every image in the bundle waits on.
    type BundleFetch = Shared<LocalBoxFuture<'static, Result<Rc<Bundle>, String>>>;

    thread_local! {
        /// Bundles that are fetched or on their way, so each is fetched once.
        static FETCHES: RefCell<HashMap<&'static str, BundleFetch>> = RefCell::default();
    }

    /// Loads an image's bytes from its bundle, fetching the bundle with the given priority hint.
    ///
    /// A low priority load starts fetching every bundle.
    pub(super) async fn load(
        path: &Path,
        priority: &'static str,
    ) -> Result<Vec<u8>, Box<dyn Error + Send + Sync>> {
        let file_name = path
            .file_name()
            .and_then(|name| name.to_str())
            .ok_or("Invalid path")?;
        let bundle_name = fecc_art_bundle::bundle_key(file_name)
            .and_then(|key| {
                ART_BUNDLES
                    .iter()
                    .find(|(bundle_key, _)| *bundle_key == key)
            })
            .map(|(_, bundle_name)| *bundle_name)
            .ok_or_else(|| format!("{file_name} isn't in an art bundle"))?;

        if priority == "low" {
            for (_, other_bundle) in ART_BUNDLES {
                if let (fetch, true) = start_fetch(other_bundle, priority) {
                    wasm_bindgen_futures::spawn_local(fetch.map(|_| ()));
                }
            }
        }
        let (fetch, _) = start_fetch(bundle_name, priority);

        match fetch.clone().await {
            Ok(bundle) => Ok(bundle
                .files
                .get(file_name)
                .map(|range| bundle.contents[range.clone()].to_vec())
                .ok_or_else(|| format!("{file_name} is missing from art/{bundle_name}"))?),
            Err(e) => {
                // Forget the failed fetch, so the next attempt fetches it again
                FETCHES.with_borrow_mut(|fetches| {
                    if fetches
                        .get(bundle_name)
                        .is_some_and(|current| current.ptr_eq(&fetch))
                    {
                        fetches.remove(bundle_name);
                    }
                });
                Err(e.into())
            }
        }
    }

    /// Returns a bundle's fetch, and whether it's new, as it only runs once something awaits it.
    fn start_fetch(bundle_name: &'static str, priority: &'static str) -> (BundleFetch, bool) {
        FETCHES.with_borrow_mut(|fetches| {
            if let Some(fetch) = fetches.get(bundle_name) {
                return (fetch.clone(), false);
            }
            let fetch = fetch_bundle(bundle_name, priority).boxed_local().shared();
            fetches.insert(bundle_name, fetch.clone());
            (fetch, true)
        })
    }

    /// Fetches a bundle and lists its files.
    async fn fetch_bundle(bundle_name: &str, priority: &str) -> Result<Rc<Bundle>, String> {
        let url = format!("art/{bundle_name}");
        let bytes = wasm::fetch_gzip_bytes(&url, priority)
            .await
            .map_err(|e| format!("Failed to fetch {url}: {}", js_error_message(&e)))?;
        let contents = js_sys::Uint8Array::new(&bytes).to_vec();

        let files = fecc_art_bundle::entries(&contents)
            .map_err(|e| format!("{url}: {e}"))?
            .into_iter()
            .map(|(name, range)| (name.to_owned(), range))
            .collect();

        Ok(Rc::new(Bundle { contents, files }))
    }

    /// The message of a thrown JavaScript error.
    fn js_error_message(error: &JsValue) -> String {
        error
            .dyn_ref::<js_sys::Error>()
            .map(|error| String::from(error.message()))
            .or_else(|| error.as_string())
            .unwrap_or_else(|| "unknown error".to_owned())
    }
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
    wasm::trigger_download(bytes, filename).map_err(|e| {
        e.dyn_ref::<js_sys::Error>()
            .map_or_else(|| "JS Error".to_owned(), |e| String::from(e.message()))
    })?;
    Ok(())
}

/// Parses a character save file from text content.
///
/// Automatically detects whether the content is a modern FECC 4e JSON save or
/// a legacy V3 save file. If `file_stem` is provided and the loaded character has
/// an empty name, `file_stem` will be used as the character's name. The parts of a
/// FECC 4e save are relinked to the libraries, following any art renamed since it was saved.
/// A save that uses art missing from the libraries is rejected, as is one with a part that can't
/// be drawn. Colours missing from a save take their defaults.
pub fn parse_character_save(
    content: &str,
    file_stem: Option<&str>,
    asset_libraries: &HashMap<AssetType, IndexMap<String, Asset>>,
) -> Result<Character, String> {
    let mut character = if content.trim_start().starts_with('{') {
        let mut character: Character =
            serde_json::from_str(content).map_err(|e| format!("Invalid JSON save: {e}"))?;
        if character.name.is_empty()
            && let Some(stem) = file_stem
        {
            character.name = stem.to_owned();
        }
        let missing = character.relink_assets(asset_libraries);
        if !missing.is_empty() {
            return Err(format!(
                "Save uses art that isn't in the library: {}",
                missing.join(", ")
            ));
        }
        character
    } else if legacy::is_legacy_save(content) {
        legacy::parse_legacy_save(content, file_stem, asset_libraries)?
    } else {
        return Err(
            "Unrecognized save file format. Expected FECC 4e JSON or legacy V3 save file."
                .to_owned(),
        );
    };

    let unusable = character.repair();
    if !unusable.is_empty() {
        return Err(format!(
            "Save has parts with an unusable position, scale or rotation: {}",
            unusable.join(", ")
        ));
    }
    Ok(character)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::asset::AssetType;
    use crate::character::Colourable;
    use crate::recolour::recolour;
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
    fn test_parse_character_save_ignores_saved_paths() {
        // A save can name any path, such as a network share, so parts take the library's.
        let content = include_str!("../tests/fixtures/4e_save_0-2-0.fecc").replace(
            "art/Legualt_Face.png",
            r"\\\\attacker.example\\share\\x.png",
        );
        let libraries = repository_art_libraries();

        let character = parse_character_save(&content, None, &libraries).unwrap();

        assert_eq!(
            character.face.unwrap().asset.path,
            libraries[&AssetType::Face]["Legault_Face"].path
        );
    }

    #[test]
    fn test_parse_character_save_rejects_art_in_another_slot() {
        // Hair art in the face slot, which would otherwise keep its saved path.
        let content = r#"{"face": {
            "position": {"x": 0.5, "y": 0.5}, "scale": 0.01, "rotation": 0.0,
            "asset": {"id": "Alen_Hair", "name": "Alen", "path": "//attacker.example/x.png",
                      "back_part": null, "asset_type": "Hair"}
        }}"#;
        let libraries = repository_art_libraries();

        let result = parse_character_save(content, None, &libraries);

        assert!(result.is_err_and(|e| e.contains("Alen_Hair")));
    }

    #[test]
    fn test_parse_character_save_fills_missing_colours() {
        // Only one colour saved, which recolouring would otherwise panic on.
        let content =
            r#"{"character_colours": {"Skin": {"base": {"r": 1, "g": 2, "b": 3, "a": 255}}}}"#;

        let character = parse_character_save(content, None, &HashMap::new()).unwrap();

        assert_eq!(character.character_colours[&Colourable::Skin].base.r, 1);
        for asset_type in [AssetType::Face, AssetType::Armour] {
            recolour(
                &mut image::RgbaImage::new(1, 1),
                asset_type,
                &character.character_colours,
                &character.outline_colours,
            );
        }
    }

    #[test]
    fn test_parse_character_save_rejects_unusable_scale() {
        // Too large for an f32, so this reads as infinity.
        let content = r#"{"hair": {
            "position": {"x": 0.5, "y": 0.5}, "scale": 1e39, "rotation": 0.0,
            "asset": {"id": "Alen_Hair", "name": "Alen", "path": "art/Alen_Hair.png",
                      "back_part": null, "asset_type": "Hair"}
        }}"#;
        let libraries = repository_art_libraries();

        let result = parse_character_save(content, None, &libraries);

        assert!(result.is_err_and(|e| e.contains("Alen_Hair")));
    }

    #[test]
    fn test_parse_character_save_rejects_unusable_scale_in_v3_save() {
        let content = "===COLOR_START===\n===COLOR_END===\n===TOOLBOX_START===\n\
            face::>1||0||0||inf||0||000000\n===TOOLBOX_END===\n";
        let libraries = repository_art_libraries();

        let result = parse_character_save(content, None, &libraries);

        assert!(result.is_err_and(|e| e.contains("unusable")));
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
