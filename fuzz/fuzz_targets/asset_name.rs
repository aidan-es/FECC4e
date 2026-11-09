// Copyright (C) 2026 aidan-es. Licensed under the GNU AGPLv3.
//! Fuzzes parsing the file names of custom art, such as `Sword[FE9]{Iscaneus}_Token`.
//!
//! People name their own art, so any name must parse or give an error, never a panic.

#![no_main]

use fecc_core::asset::Asset;
use fecc_core::asset_tags::parse_name;
use libfuzzer_sys::fuzz_target;

fuzz_target!(init: fecc_fuzz::save_crashing_inputs(), |data: &[u8]| {
    fecc_fuzz::remember_input(data);
    if let Ok(name) = std::str::from_utf8(data) {
        let _ = parse_name(name);
        let _ = Asset::parse_filename(name);
    }
});
