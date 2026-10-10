// Copyright (C) 2025 aidan-es. Licensed under the GNU AGPLv3.
//! Subsets egui's default fonts for the app.
//!
//! The full fonts add roughly 670 KB (compressed) to the download, but the app only draws Latin
//! text and a handful of symbols. Subsetting keeps just those characters.

use read_fonts::collections::IntSet;
use read_fonts::collections::int_set::Domain;
use read_fonts::{FontRef, TableProvider as _};
use skera::{DEFAULT_DROP_TABLES, DEFAULT_LAYOUT_FEATURES, Plan, SubsetFlags, subset_font};
use std::fs;
use std::path::Path;
use write_fonts::from_obj::ToOwnedTable as _;
use write_fonts::tables::name::Name;
use write_fonts::{FontBuilder, OffsetMarker};

use crate::library_symbols::LIBRARY_SYMBOLS;

/// Source folders scanned for the symbols the app draws itself, such as ◀ and ▶.
const APP_SOURCE_DIRS: [&str; 2] = ["src", "../fecc_core/src"];

/// Writes the subset fonts to `out_dir/fonts`, for `src/fonts.rs` to include.
pub fn write_subset_fonts(out_dir: &Path) {
    let mut symbols: IntSet<u32> = LIBRARY_SYMBOLS.iter().copied().collect();
    for dir in APP_SOURCE_DIRS {
        println!("cargo:rerun-if-changed={dir}");
        collect_non_ascii(Path::new(dir), &mut symbols);
    }

    // Latin with accents, general punctuation such as quotes and dashes, € and ™
    let mut text = symbols.clone();
    text.insert_range(0x20..=0x7E);
    text.insert_range(0xA0..=0x17F);
    text.insert_range(0x2000..=0x206F);
    text.extend([0x20AC, 0x2122]);

    let mut monospace = symbols.clone();
    monospace.insert_range(0x20..=0x7E);

    let fonts_dir = out_dir.join("fonts");
    fs::create_dir_all(&fonts_dir).expect("Failed to create the fonts directory");
    let write = |name: &str, font: Vec<u8>| {
        fs::write(fonts_dir.join(format!("{name}.ttf")), font)
            .expect("Failed to write a subset font");
    };

    // The Ubuntu Font Licence requires a modified font to be renamed in this way
    let ubuntu = subset(epaint_default_fonts::UBUNTU_LIGHT, &text);
    write("Ubuntu-Light", add_to_name(&ubuntu, "derivative FECC"));
    write(
        "Hack-Regular",
        subset(epaint_default_fonts::HACK_REGULAR, &monospace),
    );
    write(
        "NotoEmoji-Regular",
        subset(epaint_default_fonts::NOTO_EMOJI_REGULAR, &symbols),
    );
    write(
        "emoji-icon-font",
        subset(epaint_default_fonts::EMOJI_ICON, &symbols),
    );
}

/// Adds every non-ASCII character in the Rust files under `dir` to `characters`.
fn collect_non_ascii(dir: &Path, characters: &mut IntSet<u32>) {
    for entry in fs::read_dir(dir).expect("Failed to read a source directory") {
        let path = entry.expect("Failed to read a directory entry").path();
        if path.is_dir() {
            collect_non_ascii(&path, characters);
        } else if path.extension().is_some_and(|extension| extension == "rs") {
            let source = fs::read_to_string(&path).expect("Failed to read a source file");
            characters.extend(source.chars().filter(|c| !c.is_ascii()).map(u32::from));
        }
    }
}

/// Returns a copy of the font with only the given characters.
fn subset(data: &[u8], characters: &IntSet<u32>) -> Vec<u8> {
    let font = FontRef::new(data).expect("Failed to read a default font");

    // Keeps every name record, so the copyright notice and licence details survive
    let plan = Plan::new(
        &IntSet::empty(),
        characters,
        &font,
        SubsetFlags::SUBSET_FLAGS_DEFAULT,
        &DEFAULT_DROP_TABLES.iter().copied().collect(),
        &all(),
        &DEFAULT_LAYOUT_FEATURES.iter().copied().collect(),
        &all(),
        &all(),
    );
    subset_font(&font, &plan).expect("Failed to subset a default font")
}

/// Returns a set containing every value.
fn all<T: Domain>() -> IntSet<T> {
    let mut set = IntSet::empty();
    set.invert();
    set
}

/// Returns a copy of the font with `suffix` added to its family, full and PostScript names.
fn add_to_name(data: &[u8], suffix: &str) -> Vec<u8> {
    let font = FontRef::new(data).expect("Failed to read a subset font");
    let mut name: Name = font
        .name()
        .expect("Font has no name table")
        .to_owned_table();

    for record in &mut name.name_record {
        let old = record.string.as_str();
        let new = match record.name_id.to_u16() {
            // Family, unique ID, full name and typographic family
            1 | 3 | 4 | 16 => format!("{old} {suffix}"),
            // PostScript name, which can't contain spaces
            6 => format!("{old}-{}", suffix.replace(' ', "-")),
            _ => continue,
        };
        record.string = OffsetMarker::new(new);
    }

    let mut builder = FontBuilder::new();
    builder
        .add_table(&name)
        .expect("Failed to write the name table");
    builder.copy_missing_tables(font);
    builder.build()
}
