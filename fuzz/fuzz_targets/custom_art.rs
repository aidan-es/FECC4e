// Copyright (C) 2026 aidan-es. Licensed under the GNU AGPLv3.
//! Fuzzes uploading custom art.
//!
//! People share art files, so any file must give an asset or an error, never a panic. Art that
//! loads must export in every slot, or fail because it's too large.

#![no_main]

use fecc_core::asset::{Asset, AssetType};
use fecc_core::character::{Character, CharacterPart};
use fecc_core::export::{export_character, ExportError};
use fecc_core::types::Point;
use libfuzzer_sys::fuzz_target;

/// Every slot, as each recolours with its own palette.
const SLOTS: [AssetType; 6] = [
    AssetType::HairBack,
    AssetType::Armour,
    AssetType::Face,
    AssetType::Hair,
    AssetType::Accessory,
    AssetType::Token,
];

fuzz_target!(init: fecc_fuzz::save_crashing_inputs(), |data: &[u8]| {
    fecc_fuzz::remember_input(data);
    let Ok(asset) = Asset::try_from_bytes("Fuzz_Face.png", data) else {
        return;
    };

    let mut character = Character::default();
    for slot in SLOTS {
        let mut slot_asset = asset.clone();
        slot_asset.asset_type = slot;
        character.set_character_part(
            &slot,
            CharacterPart {
                position: Point::new(48.0, 48.0),
                scale: 1.0,
                rotation: 0.0,
                flipped: false,
                asset: slot_asset,
            },
        );
    }

    let (portrait, token) = SLOTS.split_at(5);
    for (parts, size) in [(portrait, (96, 96)), (token, (64, 64))] {
        match export_character(&character, parts, size, Point::new(96.0, 96.0)) {
            Ok(image) => assert_eq!(image.dimensions(), size, "Exporting gave the wrong size"),
            Err(ExportError::PartTooLarge(_)) => {}
            Err(e) => panic!("Exporting the art failed: {e}"),
        }
    }
});
