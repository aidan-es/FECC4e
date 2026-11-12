// Copyright (C) 2026 aidan-es. Licensed under the GNU AGPLv3.
//! The folders the native app keeps its log and settings in.

use std::path::{Path, PathBuf};

/// The app's name, which eframe also uses to name the settings folder.
pub(crate) const APP_NAME: &str = "FE Character Creator";

/// The folder eframe saves the app's settings to.
pub(crate) fn settings_dir() -> Option<PathBuf> {
    eframe::storage_dir(APP_NAME)
}

/// The folder the release build writes its log to.
///
/// Uses each platform's folder for logs, and names it the way eframe names the settings folder.
pub(crate) fn log_dir() -> Option<PathBuf> {
    if cfg!(target_os = "windows") {
        // Local rather than Roaming, as the log belongs to this machine
        dirs::data_local_dir().map(|dir| dir.join(APP_NAME))
    } else if cfg!(target_os = "macos") {
        dirs::home_dir().map(|dir| {
            dir.join("Library")
                .join("Logs")
                .join(APP_NAME.replace(' ', "-"))
        })
    } else {
        dirs::state_dir().map(|dir| dir.join(APP_NAME.to_lowercase().replace(' ', "")))
    }
}

/// Opens a folder in the file manager, creating it first if needed.
pub(crate) fn open_folder(dir: &Path) -> Result<(), String> {
    let result = std::fs::create_dir_all(dir)
        .map_err(|e| e.to_string())
        .and_then(|()| opener::open(dir).map_err(|e| e.to_string()));
    result.map_err(|e| format!("{}: {e}", dir.display()))
}
