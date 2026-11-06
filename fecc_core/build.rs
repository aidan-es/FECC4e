// Copyright (C) 2026 aidan-es. Licensed under the GNU AGPLv3.
//! Bakes the `art` file list and bundle names into the web build.
//!
//! The web build has no folder to scan, and the bundle names hold a hash of their contents,
//! so the list always matches the bundles the Trunk hook writes from the same files.

use std::env;
use std::fmt::Write as _;
use std::fs;
use std::io::ErrorKind;
use std::path::Path;

fn main() {
    println!("cargo:rerun-if-changed=../art");

    if env::var("CARGO_CFG_TARGET_ARCH").as_deref() != Ok("wasm32") {
        return;
    }

    let art_dir = Path::new("../art");
    let (files, bundles) = match fecc_art_bundle::art_files(art_dir)
        .and_then(|files| Ok((files, fecc_art_bundle::bundles(art_dir)?)))
    {
        Ok(art) => art,
        Err(e) if e.kind() == ErrorKind::NotFound => {
            println!("cargo:warning=No art folder, so the web build has no art");
            (Vec::new(), Vec::new())
        }
        Err(e) => panic!("Failed to read the art folder: {e}"),
    };

    let mut code =
        String::from("/// The `art` file names, sorted.\npub const ART_FILES: &[&str] = &[\n");
    for file_name in &files {
        writeln!(code, "    {file_name:?},").expect("Writing to a String can't fail");
    }
    code.push_str("];\n\n/// Each part type's bundle file name.\npub const ART_BUNDLES: &[(&str, &str)] = &[\n");
    for bundle in &bundles {
        writeln!(code, "    ({:?}, {:?}),", bundle.key, bundle.file_name)
            .expect("Writing to a String can't fail");
    }
    code.push_str("];\n");

    let out_dir = env::var("OUT_DIR").expect("Cargo sets OUT_DIR");
    fs::write(Path::new(&out_dir).join("art.rs"), code).expect("Failed to write the art list");
}
