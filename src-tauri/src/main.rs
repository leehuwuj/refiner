// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod commands;
pub mod providers;
mod selected_text;
mod window_management;
mod tray;
mod shortcuts;

use commands::{correct, refine, translate, save_settings, get_shortcut_window_type};
use device_query::{DeviceQuery, DeviceState};

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
        .setup(move |app| {
            // Setup global shortcuts
            setup_shortcuts(app)?;
            // Setup tray icon
            setup_tray(app).unwrap();

            // Hide the app icon from the dock
            #[cfg(target_os = "macos")]
            {
                use tauri::ActivationPolicy;
                app.set_activation_policy(ActivationPolicy::Accessory);
            }
            Ok(())
        })
        .on_window_event(|window, event| {
            match event {
                tauri::WindowEvent::Focused(is_focused) => {
                    if !is_focused {
                        match window.label() {
                            "compact-popup" => {
                                if let Err(e) = window.hide() {
                                    println!("Failed to hide compact popup on focus loss: {}", e);
                                }
                            }
                            _ => {
                                if let Err(e) = window.hide() {
                                    println!("Failed to hide window on focus loss: {}", e);
                                }
                            }
                        }
                    }
                }
                _ => {}
            }
        })
        .invoke_handler(tauri::generate_handler![translate, correct, refine, get_mouse_position, get_shortcut_window_type, save_settings])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

fn main() {
    run();
}

