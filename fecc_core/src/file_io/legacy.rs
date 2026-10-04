// Copyright (C) 2025 aidan-es. Licensed under the GNU AGPLv3.
pub mod v3_assets;

use crate::asset::{Asset, AssetType};
use crate::asset_aliases;
use crate::character::{Character, CharacterPart, CharacterPartColours, Colourable, Outlines};
use crate::types::{Point, Rgba};
use indexmap::IndexMap;
use std::collections::{HashMap, HashSet};

/// Checks if the file content matches a legacy V3 save file.
pub fn is_legacy_save(content: &str) -> bool {
    let trimmed = content.trim_start();
    !trimmed.starts_with('{')
        && content.contains("===COLOR_START===")
        && content.contains("===TOOLBOX_START===")
}

/// Decodes a hexadecimal colour string into an `Rgba` value.
///
/// Mimics Java's `Color.decode` / `Integer.decode` as used by FECC Version 3,
/// interpreting hex strings into a 24-bit RGB integer `(r << 16) | (g << 8) | b`.
/// This gracefully handles unpadded hex strings produced by V3 (e.g. `#000`, `#ff033`, `#333`).
pub fn decode_hex_colour(hex_str: &str) -> Result<Rgba, String> {
    let clean = hex_str.trim().trim_start_matches('#');
    if clean.is_empty() {
        return Ok(Rgba::BLACK);
    }
    let val = u32::from_str_radix(clean, 16)
        .map_err(|e| format!("Failed to parse hex colour '{hex_str}': {e}"))?;
    let r = ((val >> 16) & 0xFF) as u8;
    let g = ((val >> 8) & 0xFF) as u8;
    let b = (val & 0xFF) as u8;
    Ok(Rgba::new(r, g, b, 255))
}

/// Resolves an index from V3 into an `Asset`.
///
/// Uses V3 asset list, following any art renamed since. A listed asset that is missing from the
/// library gives no part. Fallback to alphabetically sorting our current assets (under the names V3
/// knew them by) provides limited and experimental support for V3 saves that make use of a modded
/// asset where that asset has also been added to 4E.
pub fn get_asset_by_index(
    asset_type: AssetType,
    index: usize,
    asset_libraries: &HashMap<AssetType, IndexMap<String, Asset>>,
) -> Result<Option<Asset>, String> {
    if index == 0 {
        return Ok(None);
    }

    let library = match asset_libraries.get(&asset_type) {
        Some(lib) => lib,
        None => return Ok(None),
    };

    // 1. Try V3 asset listing first
    if let Some(canonical_name) = v3_assets::get_v3_asset_name(asset_type, index) {
        let expected_id = format!("{canonical_name}_{asset_type}");
        if let Some(asset) = asset_aliases::resolve(library, &expected_id) {
            return Ok(Some(asset.clone()));
        }
        if let Some(asset) = library.values().find(|a| a.name == canonical_name) {
            return Ok(Some((*asset).clone()));
        }
        log::warn!("V3 {asset_type} {index} is {canonical_name}, which is not in the art library");
        return Ok(None);
    }

    // 2. Fallback: Case-insensitive sorting for out-of-range indices (e.g. modded V3 saves)
    let sorted_assets = library_in_v3_sort_order(library);
    if index <= sorted_assets.len() {
        Ok(Some(sorted_assets[index - 1].1.clone()))
    } else {
        Err(format!(
            "Asset index {index} out of range for {asset_type:?} (max {})",
            sorted_assets.len()
        ))
    }
}

/// The library in V3's sort order, with renamed art sorted under its old name.
fn library_in_v3_sort_order(library: &IndexMap<String, Asset>) -> Vec<(String, &Asset)> {
    let renamed = asset_aliases::renamed();
    let new_ids: HashSet<&str> = renamed.values().copied().collect();

    let mut assets: Vec<(String, &Asset)> = library
        .values()
        .filter(|asset| !new_ids.contains(asset.id.as_str()))
        .map(|asset| {
            let file_name = asset.path.file_name().and_then(|s| s.to_str());
            (file_name.unwrap_or(&asset.id).to_owned(), asset)
        })
        .collect();
    for (old_id, new_id) in renamed {
        if !library.contains_key(*old_id)
            && let Some(asset) = library.get(*new_id)
        {
            assets.push((format!("{old_id}.png"), asset));
        }
    }

    assets.sort_by_cached_key(|(file_name, _)| file_name.to_lowercase());
    assets
}

/// Parses a legacy V3 save file and converts it into a V4 `Character`.
pub fn parse_legacy_save(
    content: &str,
    file_stem: Option<&str>,
    asset_libraries: &HashMap<AssetType, IndexMap<String, Asset>>,
) -> Result<Character, String> {
    let mut character = Character::default();
    if let Some(stem) = file_stem {
        character.name = stem.to_owned();
    }

    let colour_start_index = content
        .find("===COLOR_START===")
        .ok_or_else(|| "Missing ===COLOR_START=== header".to_owned())?;
    let colour_end_index = content
        .find("===COLOR_END===")
        .ok_or_else(|| "Missing ===COLOR_END=== header".to_owned())?;
    let toolbox_start_index = content
        .find("===TOOLBOX_START===")
        .ok_or_else(|| "Missing ===TOOLBOX_START=== header".to_owned())?;
    let toolbox_end_index = content
        .find("===TOOLBOX_END===")
        .ok_or_else(|| "Missing ===TOOLBOX_END=== header".to_owned())?;

    if colour_start_index >= colour_end_index || toolbox_start_index >= toolbox_end_index {
        return Err("Invalid section ordering in legacy save file".to_owned());
    }

    // Parse Colours Section
    let colour_section = &content[colour_start_index + "===COLOR_START===".len()..colour_end_index];
    for line in colour_section.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }

        let (key, value) = match line.split_once("::>") {
            Some(pair) => pair,
            None => continue,
        };

        let colourable = match key {
            "hair" => Colourable::Hair,
            "eyes" => Colourable::EyeAndBeard,
            "skin" => Colourable::Skin,
            "metal" => Colourable::Metal,
            "trim" => Colourable::Trim,
            "cloth" => Colourable::Cloth,
            "leather" => Colourable::Leather,
            "accessory" => Colourable::Accessory,
            _ => continue,
        };

        let parts: Vec<&str> = value.split("||").collect();
        if parts.is_empty() {
            continue;
        }

        let base_colour = decode_hex_colour(parts[0])?;
        let mut part_colours = CharacterPartColours::new(&base_colour);

        for shade_entry in &parts[1..] {
            if let Some((index_string, hex_string)) = shade_entry.split_once("->") {
                let index: usize = index_string
                    .trim()
                    .parse()
                    .map_err(|e| format!("Invalid shade index '{index_string}': {e}"))?;
                let colour = decode_hex_colour(hex_string)?;

                match index {
                    1 | 4 | 9 | 12 | 15 | 18 => part_colours.lighter = colour,
                    2 | 5 | 10 | 13 | 16 | 19 => part_colours.neutral = colour,
                    3 | 6 | 11 | 14 | 17 | 20 => part_colours.darker = colour,
                    7 => part_colours.darker_darker = colour,
                    8 => part_colours.darker_darker_darker = colour,
                    _ => {}
                }
            }
        }

        // If the base colour was black but a neutral shade exists, align base with neutral so UI
        // swatches appear correctly.
        if base_colour == Rgba::BLACK && part_colours.neutral != Rgba::BLACK {
            part_colours.base = part_colours.neutral;
        }

        character.character_colours.insert(colourable, part_colours);
    }

    // Parse Toolboxes Section
    let mut outlines = Outlines::new();
    let toolbox_section =
        &content[toolbox_start_index + "===TOOLBOX_START===".len()..toolbox_end_index];

    for line in toolbox_section.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }

        let (key, value) = match line.split_once("::>") {
            Some(pair) => pair,
            None => continue,
        };

        let asset_type = match key {
            "face" => AssetType::Face,
            "armor" => AssetType::Armour,
            "hair" => AssetType::Hair,
            "accessory" => AssetType::Accessory,
            _ => continue,
        };

        let fields: Vec<&str> = value.split("||").collect();
        if fields.len() < 6 {
            return Err(format!(
                "Invalid toolbox line for '{key}': expected at least 6 fields, got {}",
                fields.len()
            ));
        }

        let index: usize = fields[0]
            .trim()
            .parse()
            .map_err(|e| format!("Invalid asset index '{}': {e}", fields[0]))?;
        let x_offset: f32 = fields[1]
            .trim()
            .parse()
            .map_err(|e| format!("Invalid x offset '{}': {e}", fields[1]))?;
        let y_offset: f32 = fields[2]
            .trim()
            .parse()
            .map_err(|e| format!("Invalid y offset '{}': {e}", fields[2]))?;
        let scale_val: f32 = fields[3]
            .trim()
            .parse()
            .map_err(|e| format!("Invalid scale '{}': {e}", fields[3]))?;
        let rotate_val: f32 = fields[4]
            .trim()
            .parse()
            .map_err(|e| format!("Invalid rotate '{}': {e}", fields[4]))?;
        let border_colour = decode_hex_colour(fields[5])?;

        outlines.set_outline_colour(asset_type, &border_colour);

        if let Some(asset) = get_asset_by_index(asset_type, index, asset_libraries)? {
            let position = Point::new(0.5 + (x_offset / 100.0), 0.5 + (y_offset / 100.0));
            // Normalised scale for portrait canvas (96px base)
            let scale = (scale_val / 100.0) / 96.0;
            let rotation = rotate_val.to_radians();

            let part = CharacterPart {
                position,
                scale,
                rotation,
                flipped: false,
                asset: asset.clone(),
            };

            match asset_type {
                AssetType::Face => character.face = Some(part),
                AssetType::Armour => character.armour = Some(part),
                AssetType::Hair => {
                    character.hair = Some(part.clone());

                    // Link HairBack if present
                    if let Some(back_id) = &asset.back_part
                        && let Some(back_lib) = asset_libraries.get(&AssetType::HairBack)
                        && let Some(back_asset) = back_lib.get(back_id)
                    {
                        character.hair_back = Some(CharacterPart {
                            position: part.position,
                            scale: part.scale,
                            rotation: part.rotation,
                            flipped: part.flipped,
                            asset: back_asset.clone(),
                        });
                        outlines.set_outline_colour(AssetType::HairBack, &border_colour);
                    }
                }
                AssetType::Accessory => character.accessory = Some(part),
                _ => {}
            }
        }
    }

    // Parse Token Selection Index
    let after_toolbox = &content[toolbox_end_index + "===TOOLBOX_END===".len()..];
    for line in after_toolbox.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }

        if let Ok(token_index) = line.parse::<usize>() {
            if let Some(token_asset) =
                get_asset_by_index(AssetType::Token, token_index, asset_libraries)?
            {
                character.token = Some(CharacterPart {
                    position: Point::new(0.5, 0.5),
                    scale: 1.0 / 64.0, // Normalised scale for 64px token canvas
                    rotation: 0.0,
                    flipped: false,
                    asset: token_asset,
                });
                outlines.set_outline_colour(AssetType::Token, &Rgba::new(56, 32, 64, 255));
            }
            break;
        }
    }

    character.outline_colours = outlines;
    Ok(character)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_decode_hex_colour() {
        assert_eq!(
            decode_hex_colour("#525273").unwrap(),
            Rgba::new(82, 82, 115, 255)
        );
        assert_eq!(decode_hex_colour("#000").unwrap(), Rgba::new(0, 0, 0, 255));
        assert_eq!(
            decode_hex_colour("#ff033").unwrap(),
            Rgba::new(15, 240, 51, 255)
        );
        assert_eq!(
            decode_hex_colour("#382040").unwrap(),
            Rgba::new(56, 32, 64, 255)
        );
    }

    #[test]
    fn test_is_legacy_save() {
        let legacy_sample = "===COLOR_START===\ncloth::>#525273\n===COLOR_END===\n===TOOLBOX_START===\n===TOOLBOX_END===\n0";
        assert!(is_legacy_save(legacy_sample));

        let modern_json = r#"{"name":"Hero","armour":null}"#;
        assert!(!is_legacy_save(modern_json));

        let random_text = "Hello world";
        assert!(!is_legacy_save(random_text));
    }

    #[test]
    fn test_parse_legacy_save_full() {
        let legacy_text = include_str!("../../tests/fixtures/v3_save.fecc");
        assert!(is_legacy_save(legacy_text));

        let libraries = crate::file_io::repository_art_libraries();
        let character = parse_legacy_save(legacy_text, None, &libraries).unwrap();

        assert_eq!(character.name, "");

        let face = character.face.as_ref().unwrap();
        assert_eq!(face.asset.id, "Amelia_Face");
        assert_eq!(face.position, Point::new(0.5, 0.5));
        assert_eq!(face.scale, 1.0 / 96.0);
        assert_eq!(face.rotation, 0.0);
        assert!(!face.flipped);

        assert_eq!(character.armour.as_ref().unwrap().asset.id, "Alen_Armour");
        assert_eq!(character.hair.as_ref().unwrap().asset.id, "Alen_Hair");
        assert_eq!(
            character.accessory.as_ref().unwrap().asset.id,
            "Wario_Accessory"
        );

        let hair = character.hair.as_ref().unwrap();
        let hair_back = character.hair_back.as_ref().unwrap();
        assert_eq!(hair_back.asset.id, "Alen_HairBack");
        assert_eq!(hair_back.position, hair.position);
        assert_eq!(hair_back.scale, hair.scale);

        let token = character.token.as_ref().unwrap();
        assert_eq!(token.asset.id, "ArcherIscaneus_Token");
        assert_eq!(token.position, Point::new(0.5, 0.5));
        assert_eq!(token.scale, 1.0 / 64.0);

        let cloth = &character.character_colours[&Colourable::Cloth];
        assert_eq!(cloth.lighter, Rgba::new(117, 117, 164, 255));
        assert_eq!(cloth.neutral, Rgba::new(82, 82, 115, 255));
        assert_eq!(cloth.darker, Rgba::new(57, 57, 80, 255));

        let skin = &character.character_colours[&Colourable::Skin];
        assert_eq!(skin.base, skin.neutral);
        assert_eq!(skin.neutral, Rgba::new(248, 208, 112, 255));
        assert_eq!(skin.darker, Rgba::new(232, 152, 80, 255));

        let border = Rgba::new(56, 32, 64, 255);
        for asset_type in [
            AssetType::Face,
            AssetType::Armour,
            AssetType::Hair,
            AssetType::HairBack,
            AssetType::Accessory,
            AssetType::Token,
        ] {
            assert_eq!(
                character.outline_colours.get_outline_colour(asset_type),
                border,
                "{asset_type} outline"
            );
        }
    }

    const V3_LISTS: [(AssetType, &[&str]); 5] = [
        (AssetType::Face, v3_assets::V3_FACES),
        (AssetType::Armour, v3_assets::V3_ARMOUR),
        (AssetType::Hair, v3_assets::V3_HAIR),
        (AssetType::Accessory, v3_assets::V3_ACCESSORIES),
        (AssetType::Token, v3_assets::V3_TOKENS),
    ];

    #[test]
    fn test_every_v3_asset_is_in_the_art_library() {
        let libraries = crate::file_io::repository_art_libraries();

        for (asset_type, names) in V3_LISTS {
            for (position, name) in names.iter().enumerate() {
                let asset = get_asset_by_index(asset_type, position + 1, &libraries).unwrap();
                assert!(
                    asset.is_some(),
                    "V3 {asset_type} {name} is not in the art library"
                );
            }
        }
    }
}
