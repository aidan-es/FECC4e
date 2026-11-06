// Copyright (C) 2025-2026 aidan-es. Licensed under the GNU AGPLv3.
//! Generates the 404 page's random characters, as an art bundle.
//! Effectively a proof of concept API style usage of the FECC core library.
//!
//! Run from the repository root, where it writes `404/404-characters.gz`.
use fecc_core::asset::AssetType;
use fecc_core::asset_tags::AssetFilter;
use fecc_core::character::{Character, Colourable};
use fecc_core::export::export_character;
use fecc_core::file_io::{load_asset_libraries, load_colours_from_csv, load_image_bytes};
use fecc_core::random::{randomize_assets, randomize_colours};
use fecc_core::types::Point;
use image::RgbaImage;
use std::collections::HashMap;
use std::sync::Arc;
use strum::IntoEnumIterator;

/// How many characters the 404 page shows.
const TOTAL_CHARACTERS: u32 = 404;

/// The width and height of each character.
const TILE_SIZE: u32 = 96;

/// Where the bundle is written.
const OUTPUT_PATH: &str = "404/404-characters.gz";

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    simple_logger::init_with_level(log::Level::Info)?;

    log::info!("This software comes with ABSOLUTELY NO WARRANTY.");
    log::info!("Licensed under the GNU AGPLv3 - excluding art assets.");
    log::info!("Source Code: https://github.com/aidan-es/FECC4e");
    log::info!("Full License: https://www.gnu.org/licenses/agpl-3.0.html");
    log::info!("Starting character generation...");

    let asset_libraries = load_asset_libraries()
        .await
        .expect("Failed to load asset libraries");

    let mut colour_palettes = std::collections::HashMap::new();
    for colourable in Colourable::iter().filter(|&c| c != Colourable::Outline) {
        let filename = colourable.to_string() + "_colour_palette.csv";
        match load_colours_from_csv(&filename).await {
            Ok(colours) => {
                if !colours.is_empty() {
                    colour_palettes.insert(
                        colourable,
                        fecc_core::character::ColourPalette::new(colours),
                    );
                } else {
                    log::warn!("Loaded empty colour palette for {colourable}");
                }
            }
            Err(e) => {
                log::error!("Failed to load colour palette for {colourable}: {e}");
            }
        }
    }

    let types_to_randomize: Vec<AssetType> = AssetType::get_selectable_part_types()
        .filter(|&t| t != AssetType::Accessory)
        .collect();

    let output_size = (TILE_SIZE, TILE_SIZE);
    let ui_canvas_size = Point::new(TILE_SIZE as f32, TILE_SIZE as f32);
    let parts_to_draw = &[
        AssetType::HairBack,
        AssetType::Armour,
        AssetType::Face,
        AssetType::Hair,
    ];

    log::info!("Generating {TOTAL_CHARACTERS} characters...");

    let mut characters = Vec::new();
    for i in 0..TOTAL_CHARACTERS {
        let mut character = Character::default();

        randomize_assets(
            &mut character,
            &asset_libraries,
            &types_to_randomize,
            ui_canvas_size,
            &AssetFilter::default(),
        );

        randomize_colours(&mut character, &colour_palettes);

        for part_type in parts_to_draw {
            if let Some(mut part) = character.get_character_part(part_type)
                && part.asset.image_data.is_none()
            {
                let bytes = load_image_bytes(&part.asset.path)
                    .await
                    .map_err(|e| e as Box<dyn std::error::Error>)?;
                let image = image::load_from_memory(&bytes)?.to_rgba8();
                part.asset.image_data = Some(Arc::new(image));
                character.set_character_part(part_type, part);
            }
        }

        let char_image = export_character(&character, parts_to_draw, output_size, ui_canvas_size)
            .expect("Failed to export character image");
        characters.push((format!("{i:03}.png"), encode_png(&char_image)?));

        if (i + 1) % 50 == 0 {
            log::info!("Generated {}/{}", i + 1, TOTAL_CHARACTERS);
        }
    }

    let bundle = fecc_art_bundle::write_bundle(
        characters
            .iter()
            .map(|(name, data)| (name.as_str(), data.as_slice())),
    );
    let compressed = fecc_art_bundle::gzip(&bundle);
    std::fs::write(OUTPUT_PATH, &compressed)?;

    log::info!(
        "Characters saved to {OUTPUT_PATH}, {} KB",
        compressed.len() / 1024
    );

    Ok(())
}

/// Encodes an image as a PNG, with a palette if it has 256 colours or fewer.
///
/// A character only has a few dozen colours, so a palette makes it about half the size of RGBA,
/// with no loss. Fully transparent pixels count as one colour.
fn encode_png(image: &RgbaImage) -> Result<Vec<u8>, png::EncodingError> {
    let mut palette: Vec<[u8; 4]> = Vec::new();
    let mut lookup: HashMap<[u8; 4], u8> = HashMap::new();
    let mut indices = Vec::with_capacity(image.as_raw().len() / 4);
    let mut fits_palette = true;

    for pixel in image.pixels() {
        let colour = if pixel.0[3] == 0 { [0; 4] } else { pixel.0 };
        let index = match lookup.get(&colour) {
            Some(&index) => index,
            None => {
                let Ok(index) = u8::try_from(palette.len()) else {
                    fits_palette = false;
                    break;
                };
                palette.push(colour);
                lookup.insert(colour, index);
                index
            }
        };
        indices.push(index);
    }

    let mut bytes = Vec::new();
    let mut encoder = png::Encoder::new(&mut bytes, image.width(), image.height());
    encoder.set_depth(png::BitDepth::Eight);
    encoder.set_compression(png::Compression::High);
    let data = if fits_palette {
        encoder.set_color(png::ColorType::Indexed);
        // Filters predict colour values, which palette indices aren't, so they only add bytes
        encoder.set_filter(png::Filter::NoFilter);
        encoder.set_palette(
            palette
                .iter()
                .flat_map(|c| [c[0], c[1], c[2]])
                .collect::<Vec<_>>(),
        );
        encoder.set_trns(palette.iter().map(|c| c[3]).collect::<Vec<_>>());
        indices.as_slice()
    } else {
        encoder.set_color(png::ColorType::Rgba);
        image.as_raw().as_slice()
    };
    let mut writer = encoder.write_header()?;
    writer.write_image_data(data)?;
    writer.finish()?;

    Ok(bytes)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Decodes a PNG back to RGBA.
    fn decode(bytes: &[u8]) -> RgbaImage {
        image::load_from_memory(bytes)
            .expect("Should be a valid PNG")
            .to_rgba8()
    }

    #[test]
    fn encode_png_is_lossless_with_a_palette() {
        let image = RgbaImage::from_fn(8, 8, |x, y| {
            image::Rgba([x as u8 * 30, y as u8 * 30, 7, if x == 0 { 0 } else { 255 }])
        });
        // Transparent pixels all become the same colour
        let expected = RgbaImage::from_fn(8, 8, |x, y| {
            if x == 0 {
                image::Rgba([0; 4])
            } else {
                *image.get_pixel(x, y)
            }
        });

        let bytes = encode_png(&image).expect("Should encode");
        let png = png::Decoder::new(std::io::Cursor::new(&bytes))
            .read_info()
            .expect("Should be a valid PNG");
        assert_eq!(
            png.info().color_type,
            png::ColorType::Indexed,
            "Should use a palette"
        );
        assert_eq!(decode(&bytes), expected, "Should keep every visible pixel");
    }

    #[test]
    fn encode_png_falls_back_to_rgba_over_256_colours() {
        let image = RgbaImage::from_fn(32, 32, |x, y| image::Rgba([x as u8, y as u8, 0, 255]));

        let bytes = encode_png(&image).expect("Should encode");
        assert_eq!(decode(&bytes), image, "Should keep every pixel");
    }
}
