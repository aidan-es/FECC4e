// Copyright (C) 2026 aidan-es. Licensed under the GNU AGPLv3.
//! Checks what loading a save promises, for every input that loads.
//!
//! - Each part uses the library's copy of its art, and can be drawn.
//! - Saving the character and loading the file gives the same character. This covers legacy
//!   saves too, which are saved in the 4e format.
//! - Moving the character onto a canvas and back, as loading and then saving in the app does,
//!   keeps every position and scale.
//!
//! A broken promise panics, so libFuzzer reports it like a crash.

#![no_main]

use fecc_core::asset::AssetType;
use fecc_core::character::{Character, CharacterPart};
use fecc_core::file_io::parse_character_save;
use fecc_core::types::Point;
use libfuzzer_sys::fuzz_target;

/// Portrait and token canvas sizes to check: square, tall and wide.
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

fuzz_target!(init: fecc_fuzz::save_crashing_inputs(), |data: &[u8]| {
    fecc_fuzz::remember_input(data);
    let Ok(content) = std::str::from_utf8(data) else {
        return;
    };
    let Ok(character) = parse_character_save(content, Some("fuzz"), fecc_fuzz::libraries()) else {
        return;
    };

    check_parts(&character);
    check_save_round_trip(&character);
    check_canvas_round_trip(&character);
});

/// Returns each part with the asset type its slot holds.
fn parts(character: &Character) -> [(AssetType, Option<&CharacterPart>); 6] {
    [
        (AssetType::Armour, character.armour.as_ref()),
        (AssetType::Face, character.face.as_ref()),
        (AssetType::Hair, character.hair.as_ref()),
        (AssetType::HairBack, character.hair_back.as_ref()),
        (AssetType::Accessory, character.accessory.as_ref()),
        (AssetType::Token, character.token.as_ref()),
    ]
}

/// Returns the character as JSON, for failure messages.
fn json(character: &Character) -> String {
    serde_json::to_string_pretty(character).unwrap_or_else(|e| format!("<not saveable: {e}>"))
}

/// Checks each part can be drawn and uses the library's copy of art of the right type.
fn check_parts(character: &Character) {
    let mut repaired = character.clone();
    let removed = repaired.repair();
    assert!(
        removed.is_empty(),
        "A loaded character has parts that can't be drawn: {removed:?}\n{}",
        json(character)
    );
    assert!(
        repaired == *character,
        "Repairing a loaded character changed it\n{}",
        json(character)
    );

    for (asset_type, part) in parts(character) {
        let Some(part) = part else {
            continue;
        };
        let library_copy = fecc_fuzz::libraries()
            .get(&asset_type)
            .and_then(|library| library.get(&part.asset.id));
        assert_eq!(
            library_copy,
            Some(&part.asset),
            "The {asset_type} slot doesn't hold the library's copy of its art"
        );
    }
}

/// Checks that saving the character, the way the app does, and loading it gives it back.
fn check_save_round_trip(character: &Character) {
    let saved = serde_json::to_string_pretty(character)
        .unwrap_or_else(|e| panic!("A loaded character can't be saved: {e}"));
    let reloaded = parse_character_save(&saved, Some("fuzz"), fecc_fuzz::libraries())
        .unwrap_or_else(|e| panic!("A saved character won't load: {e}\n{saved}"));
    assert!(
        reloaded == *character,
        "Saving and loading changed the character\nSaved:\n{saved}\nLoaded back:\n{}",
        json(&reloaded)
    );
}

/// Checks that moving onto a canvas and back keeps every transform, give or take float rounding.
fn check_canvas_round_trip(character: &Character) {
    for (portrait, token) in CANVASES {
        let mut moved = character.clone();
        moved.denormalise(portrait, token);
        moved.normalise(portrait, token);

        for ((asset_type, before), (_, after)) in parts(character).into_iter().zip(parts(&moved)) {
            let (Some(before), Some(after)) = (before, after) else {
                continue;
            };
            for (what, old, new) in [
                ("x", before.position.x, after.position.x),
                ("y", before.position.y, after.position.y),
                ("scale", before.scale, after.scale),
                ("rotation", before.rotation, after.rotation),
            ] {
                assert!(
                    close(old, new),
                    "The {asset_type} {what} went from {old} to {new} on a {portrait:?} canvas"
                );
            }
        }
    }
}

/// Returns whether two values match within float rounding, relative to their size.
fn close(a: f32, b: f32) -> bool {
    (a - b).abs() <= 1e-5 * (1.0 + a.abs().max(b.abs()))
}
