// Copyright (C) 2025-2026 aidan-es. Licensed under the GNU AGPLv3.
use std::env;
use std::path::Path;

#[path = "build/fonts.rs"]
mod fonts;
#[path = "build/library_symbols.rs"]
mod library_symbols;

fn main() {
    println!("cargo:rerun-if-changed=build");

    let out_dir = env::var("OUT_DIR").expect("OUT_DIR environment variable not set.");

    fonts::write_subset_fonts(Path::new(&out_dir));
}
