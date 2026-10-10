// Copyright (C) 2025 aidan-es. Licensed under the GNU AGPLv3.
//! Provides the app's fonts, which are egui's defaults subset by `build/fonts.rs`.

use egui::{FontData, FontDefinitions, FontFamily, FontTweak};
use std::sync::Arc;

/// Includes a subset font written by the build script.
macro_rules! subset_font {
    ($name:literal) => {
        include_bytes!(concat!(env!("OUT_DIR"), "/fonts/", $name, ".ttf"))
    };
}

/// Returns egui's default font setup, using the subset fonts.
///
/// Matches `FontDefinitions::default()` in epaint, including its tweaks and fallback order.
pub fn subset_font_definitions() -> FontDefinitions {
    let mut fonts = FontDefinitions::empty();

    fonts.font_data.insert(
        "Hack".to_owned(),
        Arc::new(FontData::from_static(subset_font!("Hack-Regular"))),
    );
    fonts.font_data.insert(
        "NotoEmoji-Regular".to_owned(),
        Arc::new(
            FontData::from_static(subset_font!("NotoEmoji-Regular")).tweak(FontTweak {
                scale: 0.81,
                ..Default::default()
            }),
        ),
    );
    fonts.font_data.insert(
        "Ubuntu-Light".to_owned(),
        Arc::new(FontData::from_static(subset_font!("Ubuntu-Light"))),
    );
    fonts.font_data.insert(
        "emoji-icon-font".to_owned(),
        Arc::new(
            FontData::from_static(subset_font!("emoji-icon-font")).tweak(FontTweak {
                scale: 0.90,
                ..Default::default()
            }),
        ),
    );

    fonts.families.insert(
        FontFamily::Monospace,
        vec![
            "Hack".to_owned(),
            "Ubuntu-Light".to_owned(),
            "NotoEmoji-Regular".to_owned(),
            "emoji-icon-font".to_owned(),
        ],
    );
    fonts.families.insert(
        FontFamily::Proportional,
        vec![
            "Ubuntu-Light".to_owned(),
            "NotoEmoji-Regular".to_owned(),
            "emoji-icon-font".to_owned(),
        ],
    );

    fonts
}
