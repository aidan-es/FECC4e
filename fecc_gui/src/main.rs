// Copyright (C) 2025 aidan-es. Licensed under the GNU AGPLv3.
//! Fire Emblem Character Creator application. It handles the initialisation of the
//! application window using `eframe` and sets up logging.
//! The application supports both native and WebAssembly (WASM) builds, with//! conditional compilation (`#[cfg]`) used to manage platform-specific code.

#![warn(clippy::all, rust_2018_idioms)]
// Hide the console window on Windows in release
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod app;
pub(crate) mod extensions;

pub use app::FECharacterCreator;

/// The main entry point for the native application.
#[cfg(not(target_arch = "wasm32"))]
fn main() -> eframe::Result {
    use std::io::Write as _;

    if let Err(e) = setup_logging() {
        // NOTE: Opted against `eprintln!()` to avoid panicking.
        #[expect(clippy::let_underscore_must_use)]
        let _ = writeln!(std::io::stderr(), "Error setting up logging: {e}");
    }

    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([700.0, 700.0])
            .with_min_inner_size([600.0, 400.0])
            .with_icon(
                eframe::icon_data::from_png_bytes(&include_bytes!("../../assets/icon-256.png")[..])
                    .expect("Failed to load icon"),
            ),
        persist_window: false,
        ..Default::default()
    };
    eframe::run_native(
        "FE Character Creator",
        native_options,
        Box::new(|cc| Ok(Box::new(FECharacterCreator::new(cc)))),
    )
}

#[cfg(target_arch = "wasm32")]
fn main() {
    use eframe::wasm_bindgen::JsCast as _;

    #[cfg(debug_assertions)]
    let log_level = log::LevelFilter::Debug;
    #[cfg(not(debug_assertions))]
    let log_level = log::LevelFilter::Info;

    eframe::WebLogger::init(log_level).ok();

    let web_options = eframe::WebOptions::default();

    wasm_bindgen_futures::spawn_local(async {
        let document = web_sys::window()
            .expect("No window")
            .document()
            .expect("No document");

        let canvas = document
            .get_element_by_id("the_canvas_id")
            .expect("Failed to find the_canvas_id")
            .dyn_into::<web_sys::HtmlCanvasElement>()
            .expect("the_canvas_id was not a HtmlCanvasElement");

        let start_result = eframe::WebRunner::new()
            .start(
                canvas,
                web_options,
                Box::new(|cc| Ok(Box::new(FECharacterCreator::new(cc)))),
            )
            .await;

        if let Some(loading_text) = document.get_element_by_id("loading_text") {
            match start_result {
                Ok(_) => {
                    loading_text.remove();
                }
                Err(e) => {
                    loading_text.set_inner_html("<p> The app has crashed :( </p>");
                    panic!("Failed to start eframe: {e:?}");
                }
            }
        }
    });
}

#[cfg(not(target_arch = "wasm32"))]
fn setup_logging() -> Result<(), Box<dyn std::error::Error>> {
    let mut config = fern::Dispatch::new()
        .format(|out, message, record| {
            out.finish(format_args!(
                "[{}][{}] {}",
                chrono::Local::now().format("%Y-%m-%d %H:%M:%S"),
                record.level(),
                message
            ));
        })
        .level(log::LevelFilter::Info);

    #[cfg(debug_assertions)]
    {
        // Debug build: Log to the console (stdout)
        config = config
            .level(log::LevelFilter::Debug)
            .chain(std::io::stdout());
    }

    #[cfg(not(debug_assertions))]
    {
        use std::path::PathBuf;
        // Release build: Log to a file in the system's config directory

        let config_dir: PathBuf =
            dirs_next::config_dir().ok_or("Failed to find a config directory")?;

        let app_log_dir = config_dir.join("FE Character Creator");

        std::fs::create_dir_all(&app_log_dir)?;

        let log_file = app_log_dir.join("app.log");

        config = config.chain(fern::log_file(log_file)?);
    }

    config.apply()?;

    Ok(())
}
