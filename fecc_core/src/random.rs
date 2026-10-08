// Copyright (C) 2025 aidan-es. Licensed under the GNU AGPLv3.
use crate::asset::{Asset, AssetType};
use crate::asset_tags::AssetFilter;
use crate::character::{Character, CharacterPart, CharacterPartColours, ColourPalette, Colourable};
use crate::types::Point;
use indexmap::IndexMap;
use rand::prelude::*;
use std::collections::HashMap;

/// Randomises the specified parts of the character using the provided asset libraries.
///
/// Only assets that match `filter` are chosen. Parts with no matching assets are left unchanged.
pub fn randomize_assets(
    character: &mut Character,
    asset_libraries: &HashMap<AssetType, IndexMap<String, Asset>>,
    types_to_randomize: &[AssetType],
    canvas_size: Point,
    filter: &AssetFilter,
) {
    let mut rng = rand::rng();
    let center = Point::new(canvas_size.x / 2.0, canvas_size.y / 2.0);

    for &asset_type in types_to_randomize {
        if let Some(library) = asset_libraries.get(&asset_type)
            && let Some((_, random_asset)) = library
                .iter()
                .filter(|(_, asset)| filter.matches(asset))
                .choose(&mut rng)
        {
            let base_height = if asset_type == AssetType::Token {
                64.0
            } else {
                96.0
            };
            let scale = (canvas_size.y / base_height).floor().max(1.0);
            let mut position = center;

            // Special case for Armour positioning (aligned to bottom)
            if asset_type == AssetType::Armour {
                let scaled_asset_height = 96.0 * scale;
                position.y = canvas_size.y - (scaled_asset_height / 2.0);
            }

            let part = CharacterPart {
                position,
                scale,
                rotation: 0.0,
                flipped: false,
                asset: random_asset.clone(),
            };

            if asset_type == AssetType::Hair {
                if let Some(back_part_id) = &random_asset.back_part {
                    if let Some(back_asset) = asset_libraries
                        .get(&AssetType::HairBack)
                        .and_then(|lib| lib.get(back_part_id))
                    {
                        character.set_character_part(
                            &AssetType::HairBack,
                            CharacterPart {
                                asset: back_asset.clone(),
                                flipped: part.flipped,
                                ..part.clone()
                            },
                        );
                    }
                } else {
                    character.remove_character_part(&AssetType::HairBack);
                }
            }

            character.set_character_part(&asset_type, part);
        }
    }
}

/// Randomises the colours of the character using the provided palettes.
pub fn randomize_colours(
    character: &mut Character,
    colour_palettes: &HashMap<Colourable, ColourPalette>,
) {
    use strum::IntoEnumIterator;

    let mut rng = rand::rng();

    for colourable in Colourable::iter().filter(|&c| c != Colourable::Outline) {
        if let Some(palette) = colour_palettes.get(&colourable)
            && let Some(random_color) = palette.colours().choose(&mut rng)
        {
            character
                .character_colours
                .insert(colourable, CharacterPartColours::new(random_color));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::asset::{Asset, AssetType};
    use crate::asset_tags::TagChoice;
    use crate::character::{ColourPalette, Colourable};
    use crate::types::Rgba;

    #[test]
    fn test_randomize_colours() {
        let mut character = Character::default();
        let mut palettes = HashMap::new();

        let hair_colour = Rgba::new(255, 0, 0, 255);
        palettes.insert(Colourable::Hair, ColourPalette::new(vec![hair_colour]));

        randomize_colours(&mut character, &palettes);

        let assigned_colour = character.character_colours.get(&Colourable::Hair).unwrap();
        assert_eq!(assigned_colour.base, hair_colour);
    }

    #[test]
    fn test_randomize_assets() {
        let mut character = Character::default();
        let mut libraries = HashMap::new();

        let mut face_assets = IndexMap::new();
        let face_asset = Asset::new(
            "Face1".to_string(),
            std::path::PathBuf::new(),
            None,
            AssetType::Face,
        );
        face_assets.insert(face_asset.id.clone(), face_asset.clone());
        libraries.insert(AssetType::Face, face_assets);

        let types_to_randomize = vec![AssetType::Face];
        let canvas_size = Point::new(100.0, 100.0);

        randomize_assets(
            &mut character,
            &libraries,
            &types_to_randomize,
            canvas_size,
            &AssetFilter::default(),
        );

        assert!(character.face.is_some());
        assert_eq!(character.face.as_ref().unwrap().asset.id, face_asset.id);
    }

    #[test]
    fn test_randomize_assets_follows_filter() {
        let faces: IndexMap<String, Asset> = ["Lyn[FE7]", "Roy[FE6]", "Tiefling"]
            .into_iter()
            .map(|name| {
                let asset = Asset::new(
                    name.to_string(),
                    std::path::PathBuf::new(),
                    None,
                    AssetType::Face,
                );
                (asset.id.clone(), asset)
            })
            .collect();
        let libraries = HashMap::from([(AssetType::Face, faces)]);
        let canvas_size = Point::new(100.0, 100.0);

        let mut filter = AssetFilter::default();
        filter.categories.insert(TagChoice::tag("FE7"));
        for _ in 0..20 {
            let mut character = Character::default();
            randomize_assets(
                &mut character,
                &libraries,
                &[AssetType::Face],
                canvas_size,
                &filter,
            );
            assert_eq!(character.face.unwrap().asset.id, "Lyn_Face");
        }

        // No matches, so the part is left unchanged
        filter.categories = [TagChoice::tag("FE8")].into();
        let mut character = Character::default();
        randomize_assets(
            &mut character,
            &libraries,
            &[AssetType::Face],
            canvas_size,
            &filter,
        );
        assert!(character.face.is_none());
    }

    #[test]
    fn test_randomize_assets_hair_back_link() {
        let mut character = Character::default();
        let mut libraries = HashMap::new();

        let mut hair_assets = IndexMap::new();
        let hair_asset = Asset::new(
            "Hair1".to_string(),
            std::path::PathBuf::new(),
            Some("Hair1_HairBack".to_string()),
            AssetType::Hair,
        );
        hair_assets.insert(hair_asset.id.clone(), hair_asset.clone());
        libraries.insert(AssetType::Hair, hair_assets);

        let mut hair_back_assets = IndexMap::new();
        let hair_back_asset = Asset::new(
            "Hair1".to_string(),
            std::path::PathBuf::new(),
            None,
            AssetType::HairBack,
        );
        hair_back_assets.insert(hair_back_asset.id.clone(), hair_back_asset.clone());
        libraries.insert(AssetType::HairBack, hair_back_assets);

        let types_to_randomize = vec![AssetType::Hair];
        let canvas_size = Point::new(100.0, 100.0);

        randomize_assets(
            &mut character,
            &libraries,
            &types_to_randomize,
            canvas_size,
            &AssetFilter::default(),
        );

        assert!(character.hair.is_some());
        assert!(character.hair_back.is_some());
        assert_eq!(character.hair.as_ref().unwrap().asset.id, hair_asset.id);
        assert_eq!(
            character.hair_back.as_ref().unwrap().asset.id,
            hair_back_asset.id
        );
    }

    #[test]
    fn test_randomize_assets_token_scale() {
        let mut character = Character::default();
        let mut libraries = HashMap::new();

        let mut token_assets = IndexMap::new();
        let token_asset = Asset::new(
            "Token1".to_string(),
            std::path::PathBuf::new(),
            None,
            AssetType::Token,
        );
        token_assets.insert(token_asset.id.clone(), token_asset.clone());
        libraries.insert(AssetType::Token, token_assets);

        let types_to_randomize = vec![AssetType::Token];
        // Canvas height 128: 128 / 64 = 2.0 scale (for 96 it would have been 1.0)
        let canvas_size = Point::new(128.0, 128.0);

        randomize_assets(
            &mut character,
            &libraries,
            &types_to_randomize,
            canvas_size,
            &AssetFilter::default(),
        );

        assert!(character.token.is_some());
        let part = character.token.as_ref().unwrap();
        assert_eq!(part.asset.id, token_asset.id);
        assert_eq!(part.scale, 2.0);
        assert_eq!(part.position, Point::new(64.0, 64.0));
    }
}
