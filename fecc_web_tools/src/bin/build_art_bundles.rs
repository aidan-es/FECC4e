// Copyright (C) 2026 aidan-es. Licensed under the GNU AGPLv3.
//! Writes the web build's art bundles, one gzip file per part type.
//!
//! Runs as a Trunk hook. The names match the list `fecc_core`'s build.rs bakes into the app,
//! as both come from [`fecc_art_bundle::bundles`].

use std::env;
use std::fs;
use std::path::Path;

fn main() {
    println!("Building art bundles...");

    let staging_dir = env::var("TRUNK_STAGING_DIR").expect("Should be run as a Trunk hook");
    let out_dir = Path::new(&staging_dir).join("art");
    fs::create_dir_all(&out_dir).expect("Failed to create the art folder");

    let bundles = fecc_art_bundle::bundles(Path::new("../art")).expect("Failed to read the art");
    let mut total = 0;
    for bundle in &bundles {
        let compressed = fecc_art_bundle::gzip(&bundle.contents);
        total += compressed.len();
        fs::write(out_dir.join(&bundle.file_name), &compressed)
            .expect("Failed to write an art bundle");
        println!(
            "  {}: {} KB, {} KB compressed",
            bundle.file_name,
            bundle.contents.len() / 1024,
            compressed.len() / 1024
        );
    }

    println!(
        "{} art bundles built, {} KB in all",
        bundles.len(),
        total / 1024
    );
}
