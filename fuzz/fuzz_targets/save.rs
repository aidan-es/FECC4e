// Copyright (C) 2026 aidan-es. Licensed under the GNU AGPLv3.
//! Fuzzes loading a character save, in both the FECC 4e JSON and legacy V3 formats.
//!
//! Saves are shared between people, so any input must give a character or an error, never a
//! panic.

#![no_main]

use fecc_core::file_io::parse_character_save;
use libfuzzer_sys::fuzz_target;

fuzz_target!(init: fecc_fuzz::save_crashing_inputs(), |data: &[u8]| {
    fecc_fuzz::remember_input(data);
    // Saves are text, and the app rejects files that aren't UTF-8 before parsing
    if let Ok(content) = std::str::from_utf8(data) {
        let _ = parse_character_save(content, Some("fuzz"), fecc_fuzz::libraries());
    }
});
