// Copyright (C) 2026 aidan-es. Licensed under the GNU AGPLv3.
//! Shared setup for the fuzz targets.
//!
//! On Windows, a panic ends the process through `__fastfail`, which libFuzzer can't catch, so it
//! never writes the crashing input to `fuzz/artifacts`. A panic hook saves it instead. Elsewhere
//! libFuzzer saves it itself, so those functions do nothing.

use std::collections::HashMap;
use std::sync::{Arc, OnceLock};
use std::task::{Context, Poll, Waker};

use fecc_core::asset::{Asset, AssetType};
use fecc_core::file_io::load_asset_libraries;
use indexmap::IndexMap;

/// The art libraries, by asset type.
pub type Libraries = HashMap<AssetType, IndexMap<String, Asset>>;

/// Returns the art libraries, loaded once from the `art` folder.
pub fn libraries() -> &'static Libraries {
    static LIBRARIES: OnceLock<Libraries> = OnceLock::new();

    LIBRARIES.get_or_init(|| {
        // The loader is async but never waits, so one poll finishes it
        let mut future = std::pin::pin!(load_asset_libraries());
        match future
            .as_mut()
            .poll(&mut Context::from_waker(Waker::noop()))
        {
            Poll::Ready(Ok(libraries)) => libraries,
            Poll::Ready(Err(e)) => panic!("Failed to load the art libraries: {e}"),
            Poll::Pending => panic!("Loading the art libraries didn't finish in one poll"),
        }
    })
}

/// Returns the art libraries with every image loaded, for targets that draw.
///
/// The app loads each image when it's first drawn, so [`libraries`] has none.
pub fn libraries_with_images() -> &'static Libraries {
    static LIBRARIES: OnceLock<Libraries> = OnceLock::new();

    LIBRARIES.get_or_init(|| {
        let mut libraries = libraries().clone();
        for asset in libraries.values_mut().flat_map(IndexMap::values_mut) {
            let image = image::open(&asset.path)
                .unwrap_or_else(|e| panic!("Failed to load {}: {e}", asset.path.display()));
            asset.image_data = Some(Arc::new(image.to_rgba8()));
        }
        libraries
    })
}

#[cfg(windows)]
thread_local! {
    static CURRENT_INPUT: std::cell::RefCell<Vec<u8>> = const { std::cell::RefCell::new(Vec::new()) };
}

/// Keeps a copy of this run's input, for the panic hook to save if it crashes.
pub fn remember_input(data: &[u8]) {
    #[cfg(windows)]
    CURRENT_INPUT.with_borrow_mut(|input| {
        input.clear();
        input.extend_from_slice(data);
    });
    #[cfg(not(windows))]
    let _ = data;
}

/// Installs a panic hook that saves the crashing input where libFuzzer would.
///
/// Call it from the fuzz target's `init`, which runs after libfuzzer-sys installs its own hook.
pub fn save_crashing_inputs() {
    #[cfg(windows)]
    {
        use std::hash::{DefaultHasher, Hash as _, Hasher as _};

        // cargo-fuzz passes the folder as `-artifact_prefix=`
        let prefix = std::env::args()
            .find_map(|arg| arg.strip_prefix("-artifact_prefix=").map(str::to_owned))
            .unwrap_or_default();
        let next_hook = std::panic::take_hook();
        std::panic::set_hook(Box::new(move |info| {
            CURRENT_INPUT.with_borrow(|input| {
                let mut hasher = DefaultHasher::new();
                input.hash(&mut hasher);
                let path = format!("{prefix}crash-{:016x}", hasher.finish());
                match std::fs::write(&path, input) {
                    Ok(()) => eprintln!("Saved the crashing input to {path}"),
                    Err(e) => eprintln!("Failed to save the crashing input to {path}: {e}"),
                }
            });
            next_hook(info);
        }));
    }
}
