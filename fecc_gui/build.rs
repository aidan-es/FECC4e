// Copyright (C) 2025 aidan-es. Licensed under the GNU AGPLv3.
use std::env;
use std::fs::{self, File};
use std::io::Write as _;
use std::path::Path;

#[path = "build/fonts.rs"]
mod fonts;
#[path = "build/library_symbols.rs"]
mod library_symbols;

fn main() {
    println!("cargo:rerun-if-changed=../art");
    println!("cargo:rerun-if-changed=build");

    let out_dir = env::var("OUT_DIR").expect("OUT_DIR environment variable not set.");

    fonts::write_subset_fonts(Path::new(&out_dir));

    let art_files_path = Path::new(&out_dir).join("art_files.rs");

    let mut files = Vec::new();
    let art_dir = Path::new("../art");

    if art_dir.is_dir() {
        for entry in fs::read_dir(art_dir).expect("Failed to read art directory") {
            let entry = entry.expect("Failed to read directory entry");
            let path = entry.path();
            if path.is_file()
                && path.extension().and_then(|s| s.to_str()) == Some("png")
                && let Some(file_name) = path.file_name().and_then(|s| s.to_str())
            {
                assert!(
                    !file_name.contains(['#', '?', '%']),
                    "art/{file_name} contains '#', '?' or '%', which can break loading it on the web"
                );
                files.push(file_name.to_owned());
            }
        }
    }

    files.sort();

    // Baked into the web build, so the art list always matches the code that reads it
    let mut file = File::create(&art_files_path).expect("Failed to create art file list");
    writeln!(file, "pub const ART_FILES: &[&str] = &[").expect("Failed to write art file list");
    for file_name in &files {
        writeln!(file, "    {file_name:?},").expect("Failed to write art file list");
    }
    writeln!(file, "];").expect("Failed to write art file list");
}
