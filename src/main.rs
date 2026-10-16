//! Refiner: a small AI-powered app for translating, correcting and refining
//! text, living in the menu bar / system tray. Built with GPUI Kit.

// Prevents an extra console window on Windows in release builds.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

/// Prints when `REFINER_DEBUG` is set.
macro_rules! debug_log {
    ($($arg:tt)*) => {
        if $crate::debug_enabled() {
            eprintln!("[refiner] {}", format_args!($($arg)*));
        }
    };
}

pub fn debug_enabled() -> bool {
    static ON: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
    *ON.get_or_init(|| std::env::var_os("REFINER_DEBUG").is_some())
}

mod analysis;
mod assets;
#[cfg(target_os = "macos")]
mod ax;
mod history;
mod hotkey;
mod llm;
mod platform;
mod refine;
mod selection_watch;
mod settings;
#[cfg(feature = "snapshot")]
mod snapshot;
mod state;
mod tray;
mod ui;
mod windows;

use crate::assets::AppAssets;
use crate::state::AppState;

fn main() {
    let app = gpui_kit::application().with_assets(AppAssets);
    app.run(|cx| {
        // Installs themes, key maps and component globals.
        gpui_kit::init(cx);
        AppState::init(cx);
        ui::main_view::bind_keys(cx);
        ui::popup_view::bind_keys(cx);

        platform::set_accessory_app();
        tray::init(cx);
        hotkey::init(cx);
        selection_watch::init(cx);

        windows::show_main(cx);

        #[cfg(feature = "snapshot")]
        snapshot::run(cx);
    });
}
