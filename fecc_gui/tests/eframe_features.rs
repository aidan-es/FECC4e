// Copyright (C) 2026 aidan-es. Licensed under the GNU AGPLv3.
//! Checks for changes to eframe's default features.
//!
//! Cargo can't turn off a single default feature, so `fecc_gui/Cargo.toml` turns them all off.
//! A new default would be lost without this test to point it out.

use std::process::Command;

/// The eframe defaults that `fecc_gui/Cargo.toml` has been reviewed against.
const REVIEWED_DEFAULTS: [&str; 10] = [
    "accesskit",
    "clipboard",
    "default_fonts",
    "links",
    "system_fonts",
    "wayland",
    "web_screen_reader",
    "wgpu",
    "winit/default",
    "x11",
];

#[test]
fn eframe_defaults_are_reviewed() {
    let output = Command::new(env!("CARGO"))
        .args(["metadata", "--format-version", "1", "--locked", "--offline"])
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .output()
        .expect("Failed to run cargo metadata");
    assert!(output.status.success(), "cargo metadata failed");

    let metadata: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("Failed to parse cargo metadata");
    let eframe = metadata["packages"]
        .as_array()
        .and_then(|packages| packages.iter().find(|package| package["name"] == "eframe"))
        .expect("No eframe package in cargo metadata");

    assert_eq!(
        eframe["features"]["default"],
        serde_json::json!(REVIEWED_DEFAULTS),
        "eframe's default features changed. Update eframe's features in fecc_gui/Cargo.toml if \
         needed, then REVIEWED_DEFAULTS"
    );
}
