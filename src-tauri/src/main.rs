// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod commands;
mod history;
mod language_analysis;
pub mod providers;
mod selected_text;
mod window_management;
mod tray;
mod shortcuts;

use commands::{correct, refine, translate, save_settings, get_settings, get_shortcut_window_type, open_settings_window};
use history::{get_history_enabled, toggle_history, get_history_count, export_history_json, clear_history};
use language_analysis::{get_analysis_status, open_last_report, run_language_analysis, open_reports_folder, list_reports, open_report, AppAnalysisState, AnalysisStatus};
use device_query::{DeviceQuery, DeviceState};
use std::sync::{Arc, Mutex};

use crate::tray::setup_tray;
use crate::shortcuts::setup_shortcuts;

#[tauri::command]
fn get_mouse_position() -> (i32, i32) {
    let device_state = DeviceState::new();
    let mouse = device_state.get_mouse();
    mouse.coords
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_clipboard_manager::init())
        .plugin(tauri_plugin_positioner::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_store::Builder::default().build())
        .manage(AppAnalysisState(Arc::new(Mutex::new(AnalysisStatus::default()))))
        .setup(move |app| {
            setup_shortcuts(app)?;
            setup_tray(app).unwrap();

            #[cfg(target_os = "macos")]
            {
                use tauri::ActivationPolicy;
                app.set_activation_policy(ActivationPolicy::Accessory);
            }
            Ok(())
        })
        .on_window_event(|window, event| {
            if let tauri::WindowEvent::Focused(false) = event {
                match window.label() {
                    // Settings stays open when the user clicks elsewhere
                    "settings" => {}
                    _ => { let _ = window.hide(); }
                }
            }
        })
        .invoke_handler(tauri::generate_handler![
            // core
            translate, correct, refine,
            get_mouse_position,
            get_shortcut_window_type,
            save_settings,
            get_settings,
            open_settings_window,
            // history
            get_history_enabled,
            toggle_history,
            get_history_count,
            export_history_json,
            clear_history,
            // language analysis
            get_analysis_status,
            run_language_analysis,
            open_last_report,
            open_reports_folder,
            list_reports,
            open_report,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

fn main() {
    run();
}
