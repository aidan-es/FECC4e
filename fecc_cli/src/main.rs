// Copyright (C) 2025 aidan-es. Licensed under the GNU AGPLv3.
//! Generates a sprite sheet of random characters.
//! Effectively a proof of concept API style usage of the FECC core library.
use fecc_core::asset::AssetType;
use fecc_core::character::{Character, Colourable};
use fecc_core::export::export_character;
use fecc_core::file_io::{load_asset_libraries, load_colours_from_csv, load_image_bytes};
use fecc_core::random::{randomize_assets, randomize_colours};
use fecc_core::types::Point;
use std::sync::Arc;
use strum::IntoEnumIterator;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    simple_logger::init_with_level(log::Level::Info)?;

    log::info!("Copyright (C) 2025 aidan-es");
    log::info!("This software comes with ABSOLUTELY NO WARRANTY.");
    log::info!("Licensed under the GNU AGPLv3 - excluding art assets.");
    log::info!("Source Code: https://github.com/aidan-es/FECC4e");
    log::info!("Full License: https://www.gnu.org/licenses/agpl-3.0.html");
    log::info!("Starting character sprite sheet generation...");

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

    // Sprite Sheet Configuration
    const TOTAL_CHARACTERS: u32 = 404;
    const TILE_SIZE: u32 = 96;
    const COLUMNS: u32 = 20;
    let rows = (TOTAL_CHARACTERS as f32 / COLUMNS as f32).ceil() as u32;
    let sheet_width = COLUMNS * TILE_SIZE;
    let sheet_height = rows * TILE_SIZE;

    let mut sprite_sheet = image::RgbaImage::new(sheet_width, sheet_height);

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

    log::info!(
        "Generating {} characters in a {}x{} grid...",
        TOTAL_CHARACTERS,
        COLUMNS,
        rows
    );

    for i in 0..TOTAL_CHARACTERS {
        let mut character = Character::default();

        randomize_assets(
            &mut character,
            &asset_libraries,
            &types_to_randomize,
            ui_canvas_size,
        );

        randomize_colours(&mut character, &colour_palettes);

        for part_type in [
            AssetType::HairBack,
            AssetType::Armour,
            AssetType::Face,
            AssetType::Hair,
        ] {
            if let Some(mut part) = character.get_character_part(&part_type)
                && part.asset.image_data.is_none()
            {
                let bytes = load_image_bytes(&part.asset.path)
                    .await
                    .map_err(|e| e as Box<dyn std::error::Error>)?;
                let image = image::load_from_memory(&bytes)?.to_rgba8();
                part.asset.image_data = Some(Arc::new(image));
                character.set_character_part(&part_type, part);
            }
        }

        let char_image = export_character(&character, parts_to_draw, output_size, ui_canvas_size)
            .expect("Failed to export character image");

        let col = i % COLUMNS;
        let row = i / COLUMNS;
        let x = (col * TILE_SIZE) as i64;
        let y = (row * TILE_SIZE) as i64;

        image::imageops::overlay(&mut sprite_sheet, &char_image, x, y);

        if (i + 1) % 50 == 0 {
            log::info!("Generated {}/{}", i + 1, TOTAL_CHARACTERS);
        }
    }

    let filename = "sprites.png".to_string();
    sprite_sheet.save(&filename)?;

    log::info!("Sprite sheet saved to {filename}");

    Ok(())
}
