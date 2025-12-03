// Copyright (C) 2025 aidan-es. Licensed under the GNU AGPLv3.
use crate::asset::AssetType;
use crate::character::Colourable::{
    Accessory, Cloth, EyeAndBeard, Hair, Leather, Metal, Skin, Trim,
};
use crate::character::{CharacterPartColours, Colourable, Outlines};
use crate::types::Rgba;
use image::RgbaImage;
use std::collections::HashMap;

const OUTLINE_INDEX: usize = 0;
// Used for either Eye / Beard shades or Hair shades depending on asset type.
const MULTI_LIGHTER_SHADE_INDEX: usize = 1;
const MULTI_NEUTRAL_SHADE_INDEX: usize = 2;
const MULTI_DARKER_SHADE_INDEX: usize = 3;
// Skin shades
const SKIN_LIGHTER_SHADE_INDEX: usize = 4;
const SKIN_NEUTRAL_SHADE_INDEX: usize = 5;
const SKIN_DARKER_SHADE_INDEX: usize = 6;
const SKIN_DARKER_DARKER_SHADE_INDEX: usize = 7;
const SKIN_DARKER_DARKER_DARKER_SHADE_INDEX: usize = 8;
// Accessory / Metal shades
const ACC_METAL_LIGHTER_SHADE_INDEX: usize = 9;
const ACC_METAL_NEUTRAL_SHADE_INDEX: usize = 10;
const ACC_METAL_DARKER_SHADE_INDEX: usize = 11;
// Trim shades
const TRIM_LIGHTER_SHADE_INDEX: usize = 12;
const TRIM_NEUTRAL_SHADE_INDEX: usize = 13;
const TRIM_DARKER_SHADE_INDEX: usize = 14;
// Cloth shades
const CLOTH_LIGHTER_SHADE_INDEX: usize = 15;
const CLOTH_NEUTRAL_SHADE_INDEX: usize = 16;
const CLOTH_DARKER_SHADE_INDEX: usize = 17;
// Leather shades
const LEATHER_LIGHTER_SHADE_INDEX: usize = 18;
const LEATHER_NEUTRAL_SHADE_INDEX: usize = 19;
const LEATHER_DARKER_SHADE_INDEX: usize = 20;

/// Recolours an RgbaImage
///
/// The implementation uses a lookup table (LUT) for performance,
/// mapping red channel values to their final colours before iterating over the pixels,
/// avoiding repeated hash map lookups in the inner loop.
pub fn recolour(
    image: &mut RgbaImage,
    asset_type: AssetType,
    character_colours: &HashMap<Colourable, CharacterPartColours>,
    outline_colours: &Outlines,
) {
    let mut recolour_map: [Option<Rgba>; 21] = [None; 21];

    // Map source colour keys (0-20) to target colours.
    // The key is derived from the red channel: (red / 10).
    // Indices correspond to specific shades in the palette.
    if asset_type == AssetType::Face || asset_type == AssetType::Accessory {
        recolour_map[OUTLINE_INDEX] = Some(outline_colours.get_outline_colour(asset_type));
        recolour_map[MULTI_LIGHTER_SHADE_INDEX] = Some(character_colours[&EyeAndBeard].lighter);
        recolour_map[MULTI_NEUTRAL_SHADE_INDEX] = Some(character_colours[&EyeAndBeard].neutral);
        recolour_map[MULTI_DARKER_SHADE_INDEX] = Some(character_colours[&EyeAndBeard].darker);
        recolour_map[SKIN_LIGHTER_SHADE_INDEX] = Some(character_colours[&Skin].lighter);
        recolour_map[SKIN_NEUTRAL_SHADE_INDEX] = Some(character_colours[&Skin].neutral);
        recolour_map[SKIN_DARKER_SHADE_INDEX] = Some(character_colours[&Skin].darker);
        recolour_map[SKIN_DARKER_DARKER_SHADE_INDEX] = Some(character_colours[&Skin].darker_darker);
        recolour_map[SKIN_DARKER_DARKER_DARKER_SHADE_INDEX] =
            Some(character_colours[&Skin].darker_darker_darker);
        recolour_map[ACC_METAL_LIGHTER_SHADE_INDEX] = Some(character_colours[&Accessory].lighter);
        recolour_map[ACC_METAL_NEUTRAL_SHADE_INDEX] = Some(character_colours[&Accessory].neutral);
        recolour_map[ACC_METAL_DARKER_SHADE_INDEX] = Some(character_colours[&Accessory].darker);
    } else {
        recolour_map[OUTLINE_INDEX] = Some(outline_colours.get_outline_colour(asset_type));
        // Hair
        recolour_map[MULTI_LIGHTER_SHADE_INDEX] = Some(character_colours[&Hair].lighter);
        recolour_map[MULTI_NEUTRAL_SHADE_INDEX] = Some(character_colours[&Hair].neutral);
        recolour_map[MULTI_DARKER_SHADE_INDEX] = Some(character_colours[&Hair].darker);
        // Skin
        recolour_map[SKIN_LIGHTER_SHADE_INDEX] = Some(character_colours[&Skin].lighter);
        recolour_map[SKIN_NEUTRAL_SHADE_INDEX] = Some(character_colours[&Skin].neutral);
        recolour_map[SKIN_DARKER_SHADE_INDEX] = Some(character_colours[&Skin].darker);
        recolour_map[SKIN_DARKER_DARKER_SHADE_INDEX] = Some(character_colours[&Skin].darker_darker);
        recolour_map[SKIN_DARKER_DARKER_DARKER_SHADE_INDEX] =
            Some(character_colours[&Skin].darker_darker_darker);
        // Metal
        recolour_map[ACC_METAL_LIGHTER_SHADE_INDEX] = Some(character_colours[&Metal].lighter);
        recolour_map[ACC_METAL_NEUTRAL_SHADE_INDEX] = Some(character_colours[&Metal].neutral);
        recolour_map[ACC_METAL_DARKER_SHADE_INDEX] = Some(character_colours[&Metal].darker);
        // Trim
        recolour_map[TRIM_LIGHTER_SHADE_INDEX] = Some(character_colours[&Trim].lighter);
        recolour_map[TRIM_NEUTRAL_SHADE_INDEX] = Some(character_colours[&Trim].neutral);
        recolour_map[TRIM_DARKER_SHADE_INDEX] = Some(character_colours[&Trim].darker);
        // Cloth
        recolour_map[CLOTH_LIGHTER_SHADE_INDEX] = Some(character_colours[&Cloth].lighter);
        recolour_map[CLOTH_NEUTRAL_SHADE_INDEX] = Some(character_colours[&Cloth].neutral);
        recolour_map[CLOTH_DARKER_SHADE_INDEX] = Some(character_colours[&Cloth].darker);
        // Leather
        recolour_map[LEATHER_LIGHTER_SHADE_INDEX] = Some(character_colours[&Leather].lighter);
        recolour_map[LEATHER_NEUTRAL_SHADE_INDEX] = Some(character_colours[&Leather].neutral);
        recolour_map[LEATHER_DARKER_SHADE_INDEX] = Some(character_colours[&Leather].darker);
    }

    for pixel in image.pixels_mut() {
        let channels = pixel.0;

        // Skip transparent pixels.
        if channels[3] == 0 {
            continue;
        }

        let red_key = (channels[0] / 10) as usize;

        if let Some(Some(new_colour)) = recolour_map.get(red_key) {
            *pixel = image::Rgba([new_colour.r, new_colour.g, new_colour.b, new_colour.a]);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_recolor_face() {
        let mut image = RgbaImage::new(2, 2);
        // Setup pixels with "magic" red values
        // 0 -> Outline
        image.put_pixel(0, 0, image::Rgba([0, 0, 0, 255]));
        // 10 -> Lighter (EyeAndBeard)
        image.put_pixel(1, 0, image::Rgba([10, 0, 0, 255]));
        // 40 -> Skin Lighter
        image.put_pixel(0, 1, image::Rgba([40, 0, 0, 255]));
        // 255 -> Unmapped (should be ignored/unchanged if out of bounds or None,
        // but 255/10 = 25 which is > 20, so it returns None from map)
        image.put_pixel(1, 1, image::Rgba([250, 0, 0, 255]));

        let outline_colour = Rgba::new(10, 10, 10, 255);
        let mut outlines = Outlines::default();
        outlines.set_outline_colour(AssetType::Face, &outline_colour);

        let eye_lighter = Rgba::new(20, 20, 20, 255);
        let skin_lighter = Rgba::new(30, 30, 30, 255);

        let mut char_colours = HashMap::new();
        // Setup EyeAndBeard
        let eye_parts = CharacterPartColours {
            lighter: eye_lighter,
            ..Default::default()
        };
        char_colours.insert(EyeAndBeard, eye_parts);

        // Setup Skin
        let skin_parts = CharacterPartColours {
            lighter: skin_lighter,
            ..Default::default()
        };
        char_colours.insert(Skin, skin_parts);

        // Fill others with default to avoid panic
        char_colours.insert(Accessory, CharacterPartColours::default());

        recolour(&mut image, AssetType::Face, &char_colours, &outlines);

        // Check Outline (Index 0)
        let p00 = image.get_pixel(0, 0);
        assert_eq!(p00[0], outline_colour.r);

        // Check Eye Lighter (Index 1)
        let p10 = image.get_pixel(1, 0);
        assert_eq!(p10[0], eye_lighter.r);

        // Check Skin Lighter (Index 4)
        let p01 = image.get_pixel(0, 1);
        assert_eq!(p01[0], skin_lighter.r);

        // Check Unmapped (Index 25) - should remain unchanged
        let p11 = image.get_pixel(1, 1);
        assert_eq!(p11[0], 250);
    }

    #[test]
    fn test_recolor_armour() {
        let mut image = RgbaImage::new(1, 1);
        // 150 -> Cloth Lighter (15)
        image.put_pixel(0, 0, image::Rgba([150, 0, 0, 255]));

        let outlines = Outlines::default();
        let mut char_colours = HashMap::new();

        let cloth_lighter = Rgba::new(50, 50, 50, 255);
        let cloth_parts = CharacterPartColours {
            lighter: cloth_lighter,
            ..Default::default()
        };
        char_colours.insert(Cloth, cloth_parts);

        // Fill others with default to avoid panic
        for colourable in [Hair, Skin, Metal, Trim, Leather] {
            char_colours.insert(colourable, CharacterPartColours::default());
        }

        recolour(&mut image, AssetType::Armour, &char_colours, &outlines);

        let p00 = image.get_pixel(0, 0);
        assert_eq!(p00[0], cloth_lighter.r);
    }
}
