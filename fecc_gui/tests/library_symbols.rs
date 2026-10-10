// Copyright (C) 2025 aidan-es. Licensed under the GNU AGPLv3.
//! Checks that the subset fonts keep every symbol the egui libraries draw.
//!
//! The build script can't see inside the libraries, so it uses the `LIBRARY_SYMBOLS` list. This
//! test scans the libraries' sources and fails when that list is missing a symbol, for example
//! after an egui upgrade.

#[path = "../build/library_symbols.rs"]
mod library_symbols;

use library_symbols::LIBRARY_SYMBOLS;
use proc_macro2::{TokenStream, TokenTree};
use read_fonts::{FontRef, TableProvider as _};
use std::collections::BTreeSet;
use std::fs;
use std::path::Path;
use std::process::Command;
use syn::Lit;

/// The libraries whose string and character literals are scanned.
const LIBRARIES: [&str; 7] = [
    "eframe",
    "egui",
    "epaint",
    "egui-notify",
    "egui_commonmark",
    "egui_commonmark_backend",
    "egui_extras",
];

#[test]
fn library_symbols_include_every_drawable_symbol() {
    let fonts = [
        epaint_default_fonts::UBUNTU_LIGHT,
        epaint_default_fonts::HACK_REGULAR,
        epaint_default_fonts::NOTO_EMOJI_REGULAR,
        epaint_default_fonts::EMOJI_ICON,
    ]
    .map(|data| FontRef::new(data).expect("Failed to read a default font"));

    // Symbols no default font can draw would be missing either way, so they're skipped
    let drawable = |symbol: char| {
        fonts.iter().any(|font| {
            font.cmap()
                .is_ok_and(|cmap| cmap.map_codepoint(symbol).is_some())
        })
    };

    let mut found = 0;
    let mut missing = BTreeSet::new();
    for package in cargo_metadata_packages() {
        let name = package["name"].as_str().expect("Package has no name");
        if !LIBRARIES.contains(&name) {
            continue;
        }
        found += 1;

        let manifest_path = package["manifest_path"]
            .as_str()
            .expect("Package has no manifest");
        let mut symbols = BTreeSet::new();
        collect_literal_symbols(
            &Path::new(manifest_path).with_file_name("src"),
            &mut symbols,
        );

        for symbol in symbols {
            if drawable(symbol) && !LIBRARY_SYMBOLS.contains(&u32::from(symbol)) {
                missing.insert(format!(
                    "{name} uses U+{:04X} ({symbol})",
                    u32::from(symbol)
                ));
            }
        }
    }

    assert_eq!(
        found,
        LIBRARIES.len(),
        "Failed to find every library's source"
    );
    assert!(
        missing.is_empty(),
        "Add these to LIBRARY_SYMBOLS in fecc_gui/build/library_symbols.rs:\n{}",
        missing.into_iter().collect::<Vec<_>>().join("\n")
    );
}

/// Returns the packages listed by `cargo metadata`, which knows where each one's source is.
fn cargo_metadata_packages() -> Vec<serde_json::Value> {
    let output = Command::new(env!("CARGO"))
        .args(["metadata", "--format-version", "1", "--locked", "--offline"])
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .output()
        .expect("Failed to run cargo metadata");
    assert!(output.status.success(), "cargo metadata failed");

    let mut metadata: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("Failed to parse cargo metadata");
    serde_json::from_value(metadata["packages"].take()).expect("No packages in cargo metadata")
}

/// Adds the non-ASCII characters in the literals of every Rust file under `path` to `symbols`.
fn collect_literal_symbols(path: &Path, symbols: &mut BTreeSet<char>) {
    if path.is_dir() {
        for entry in fs::read_dir(path).expect("Failed to read a source directory") {
            collect_literal_symbols(&entry.expect("Failed to read an entry").path(), symbols);
        }
    } else if path.extension().is_some_and(|extension| extension == "rs") {
        let source = fs::read_to_string(path).expect("Failed to read a source file");
        let tokens = source.parse().expect("Failed to tokenise a source file");
        collect_from_tokens(tokens, symbols);
    }
}

/// Adds the non-ASCII characters in every string and character literal to `symbols`.
///
/// Skips doc comments, which the tokeniser turns into `#[doc = "..."]` or `#![doc = "..."]`.
fn collect_from_tokens(tokens: TokenStream, symbols: &mut BTreeSet<char>) {
    let mut tokens = tokens.into_iter().peekable();
    while let Some(token) = tokens.next() {
        match token {
            TokenTree::Punct(punct) if punct.as_char() == '#' => {
                if matches!(tokens.peek(), Some(TokenTree::Punct(bang)) if bang.as_char() == '!') {
                    tokens.next();
                }
                let is_doc = matches!(tokens.peek(), Some(TokenTree::Group(attribute))
                    if attribute.stream().into_iter().next().is_some_and(|first| first.to_string() == "doc"));
                if is_doc {
                    tokens.next();
                }
            }
            TokenTree::Group(group) => collect_from_tokens(group.stream(), symbols),
            TokenTree::Literal(literal) => {
                let text = match Lit::new(literal) {
                    Lit::Str(string) => string.value(),
                    Lit::Char(character) => character.value().to_string(),
                    _ => continue,
                };
                symbols.extend(text.chars().filter(|c| !c.is_ascii()));
            }
            _ => {}
        }
    }
}
