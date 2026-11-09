// Copyright (C) 2026 aidan-es. Licensed under the GNU AGPLv3.
//! Fuzzes exporting a loaded character, which the app also does by itself to count its colours.
//!
//! Any character that loads must export on each canvas and at each size, or fail because a part
//! is too large. A successful export is exactly the size asked for.

#![no_main]

use fecc_core::asset::AssetType;
use fecc_core::character::Character;
use fecc_core::export::{export_character, ExportError};
use fecc_core::file_io::parse_character_save;
use fecc_core::types::Point;
use libfuzzer_sys::fuzz_target;

/// The parts of a portrait, in drawing order.
const PORTRAIT_PARTS: [AssetType; 5] = [
    AssetType::HairBack,
    AssetType::Armour,
    AssetType::Face,
    AssetType::Hair,
    AssetType::Accessory,
];

/// Portrait and token canvas sizes to export from: square, tall and wide.
const CANVASES: [(Point, Point); 3] = [
    (Point { x: 96.0, y: 96.0 }, Point { x: 64.0, y: 64.0 }),
    (Point { x: 400.0, y: 600.0 }, Point { x: 150.0, y: 150.0 }),
    (
        Point {
            x: 1600.0,
            y: 300.0,
        },
        Point { x: 37.0, y: 37.0 },
    ),
];

/// Portrait and token export sizes: the app's colour count, and an odd custom size.
const EXPORT_SIZES: [((u32, u32), (u32, u32)); 2] = [((96, 96), (64, 64)), ((301, 120), (37, 50))];

/// The longest side a part is scaled to here.
///
/// Export allows up to 8192, but a part that large takes over a minute to resize under
/// AddressSanitizer, which stops the run as a timeout.
const MAX_FUZZED_SIDE: f32 = 1024.0;

/// The longest side export allows, `MAX_SCALED_SIDE` in export.rs. Larger parts fail quickly.
const MAX_SCALED_SIDE: f32 = 8192.0;

fuzz_target!(init: fecc_fuzz::save_crashing_inputs(), |data: &[u8]| {
    fecc_fuzz::remember_input(data);
    let Ok(content) = std::str::from_utf8(data) else {
        return;
    };
    let Ok(character) = parse_character_save(content, Some("fuzz"), fecc_fuzz::libraries_with_images())
    else {
        return;
    };

    for (portrait_canvas, token_canvas) in CANVASES {
        let mut on_canvas = character.clone();
        on_canvas.denormalise(portrait_canvas, token_canvas);
        for (portrait_size, token_size) in EXPORT_SIZES {
            check_export(&on_canvas, &PORTRAIT_PARTS, portrait_size, portrait_canvas);
            check_export(&on_canvas, &[AssetType::Token], token_size, token_canvas);
        }
    }
});

/// Checks that exporting gives an image of `size`, or says a part is too large.
///
/// Skips exports with a part too slow to scale, see [`MAX_FUZZED_SIDE`].
fn check_export(character: &Character, parts: &[AssetType], size: (u32, u32), canvas: Point) {
    // Same as export_character
    let export_scale = (size.0 as f32 / canvas.x).max(size.1 as f32 / canvas.y);
    let too_slow = parts
        .iter()
        .filter_map(|part_type| character.get_character_part(part_type))
        .filter_map(|part| {
            let image = part.asset.image_data.as_ref()?;
            Some(image.width().max(image.height()) as f32 * part.scale * export_scale)
        })
        .any(|side| side > MAX_FUZZED_SIDE && side <= MAX_SCALED_SIDE);
    if too_slow {
        return;
    }

    match export_character(character, parts, size, canvas) {
        Ok(image) => assert_eq!(
            image.dimensions(),
            size,
            "Exporting from a {canvas:?} canvas gave the wrong size"
        ),
        Err(ExportError::PartTooLarge(_)) => {}
        Err(e) => panic!("Exporting {size:?} from a {canvas:?} canvas failed: {e}"),
    }
}
