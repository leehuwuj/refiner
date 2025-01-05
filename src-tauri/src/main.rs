// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod commands;
pub mod providers;
mod selected_text;
mod mouse_service;

use commands::{correct, refine, translate};
use mouse_service::{MouseService, MouseEvent};
use tauri::{App, Emitter, Manager, WebviewUrl, WebviewWindowBuilder};
use tauri::menu::{Menu, MenuItem};
use tauri_plugin_positioner::{Position, WindowExt};
use device_query::{DeviceQuery, DeviceState};
use std::sync::Arc;
use tauri::Listener;

use crate::selected_text::get_selected_text;

fn setup_tray(app: &App) -> Result<(), String> {
    let app_handler = app.app_handle();
    let tray = app_handler.tray_by_id("refiner").unwrap();
    let menu = Menu::with_items(
        app_handler,
        &[&MenuItem::with_id(app_handler, "quit", "Quit", true, None::<String>).unwrap()],
    )
    .unwrap();
    tray.set_menu(Some(menu).clone()).unwrap();
    tray.on_menu_event(move |handler, event| match event.id.as_ref() {
        "quit" => {
            handler.exit(0);
        }
        _ => {}
    });
    app.on_tray_icon_event(move |handler, event| {
        // Move the window to the top right corner for macOS
        #[cfg(target_os = "macos")]
        {
            let win = handler.get_webview_window("main").unwrap();
            let _ = win.as_ref().window().move_window(Position::TopRight);
        }

        // Move the window to the bottom right corner for Windows
        #[cfg(target_os = "windows")]
        {
            let win = handler.get_webview_window("main").unwrap();
            let _ = win.as_ref().window().move_window(Position::TopCenter);
        }
        tauri_plugin_positioner::on_tray_event(&handler, &event);
        // if event.click_type == ClickType::Left {
        //     if let Some(webview_window) = handler.get_webview_window("main") {
        //         let _ = webview_window.show();
        //         let _ = webview_window.set_focus();
        //     }
        // }
        if let tauri::tray::TrayIconEvent::Click { .. } = event {
            if let Some(webview_window) = handler.get_webview_window("main") {
                webview_window.emit("tray-click", ()).unwrap();
                webview_window.show().unwrap();
                webview_window.set_focus().unwrap();
            }
        }
    });
    Ok(())
}

#[tauri::command]
fn get_mouse_position() -> (i32, i32) {
    let device_state = DeviceState::new();
    let mouse = device_state.get_mouse();
    mouse.coords
}

async fn create_or_focus_compact_window(app: &tauri::AppHandle, x: i32, y: i32) -> tauri::WebviewWindow {
    if let Some(window) = app.get_webview_window("translate-popup") {
        // Just hide the existing window and reposition it
        let _ = window.hide();
        let _ = window.set_position(tauri::Position::Logical(tauri::LogicalPosition {
            x: x as f64,
            y: y as f64,
        }));
        window
    } else {
        let window = WebviewWindowBuilder::new(
            app,
            "translate-popup", 
            WebviewUrl::App("translate-popup".into())
        )
        .title("Quick Translate")
        .transparent(true)
        .decorations(false)
        .always_on_top(true)
        .inner_size(300.0, 200.0)
        .skip_taskbar(true)
        .visible(false)
        .position(x as f64, y as f64)
        .build()
        .unwrap();

        window
    }
}

async fn create_selection_icon(app: &tauri::AppHandle, x: i32, y: i32) -> tauri::WebviewWindow {
    if let Some(window) = app.get_webview_window("selection-icon") {
        let _ = window.hide();
        let _ = window.set_position(tauri::Position::Logical(tauri::LogicalPosition {
            x: x as f64,
            y: y as f64,
        }));
        window
    } else {
        let window = WebviewWindowBuilder::new(
            app,
            "selection-icon",
            WebviewUrl::App("selection-icon".into())
        )
        .title("Selection Icon")
        .transparent(true)
        .decorations(false)
        .always_on_top(true)
        .inner_size(24.0, 24.0)  // Small icon size
        .skip_taskbar(true)
        .visible(false)
        .position(x as f64, y as f64)
        .build()
        .unwrap();

        window
    }
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
            let app_handle = app.handle();
            
            // Listen for icon click events
            let icon_app_handle = app_handle.clone();
            app_handle.listen("icon-clicked", move |event| {
                if let Some(icon_window) = icon_app_handle.get_webview_window("selection-icon") {
                    // Get the current mouse position for the translate popup
                    let device_state = DeviceState::new();
                    let (x, y) = device_state.get_mouse().coords;
                    
                    // Get the selected text from the event payload
                    let selected_text = event.payload().to_owned();
                    
                    // Hide the icon window first
                    icon_window.hide().unwrap();
                    
                    // Show the translate popup in a separate thread to avoid blocking
                    let app_handle = icon_app_handle.clone();
                    tauri::async_runtime::spawn(async move {
                        let translate_window = create_or_focus_compact_window(&app_handle, x, y).await;
                        translate_window.emit("shortcut-quickTranslate", serde_json::json!({
                            "text": selected_text
                        })).unwrap();
                        translate_window.show().unwrap();
                        translate_window.set_focus().unwrap();
                    });
                }
            });

            let mouse_service = Arc::new(MouseService::new());
            
            // Clone for app state management
            let mouse_service_for_state = mouse_service.clone();
            
            // Start monitoring mouse events
            let app_handle_for_service = app_handle.clone();
            mouse_service.start(app_handle.clone(), move |event| {
                let app_handle = app_handle_for_service.clone();
                match event {
                    MouseEvent::TextSelected(text) => {
                        println!("Text selected: {}", text);
                        let device_state = DeviceState::new();
                        let (x, y) = device_state.get_mouse().coords;
                        
                        // Show the small icon window
                        tauri::async_runtime::block_on(async {
                            let window = create_selection_icon(&app_handle, x + 5, y - 5).await;  // Offset slightly from cursor
                            // Store the selected text in the window's state
                            window.emit("set-selected-text", text).unwrap();
                            window.show().unwrap();
                            window.set_focus().unwrap();
                        });
                    },
                    _ => {}
                }
            });

            // Store the mouse service in the app state for later use
            app.manage(mouse_service_for_state);

            {
                use tauri_plugin_global_shortcut::{Code, Modifiers};
                
                #[cfg(target_os = "macos")] {
                    app.handle().plugin(
                        tauri_plugin_global_shortcut::Builder::new()
                            .with_shortcuts(["super+e"])?
                            // Show popup windows when pressing super+e
                            .with_handler(|app, shortcut, _| {
                                if shortcut.matches(Modifiers::SUPER, Code::KeyE) {
                                    let selected_text = tauri::async_runtime::block_on(async {
                                        get_selected_text(app).await
                                    });
                                    if let Ok(selected_text) = selected_text {
                                        let device_state = DeviceState::new();
                                        let mouse = device_state.get_mouse();
                                        let (x, y) = mouse.coords;
                                        let window = tauri::async_runtime::block_on(async {
                                            create_or_focus_compact_window(app, x as i32, y as i32).await
                                        });
                                        window.emit("shortcut-quickTranslate", serde_json::json!({
                                            "text": selected_text.replace("\"", "")
                                        })).unwrap();
                                        window.show().unwrap();
                                        window.set_focus().unwrap();
                                    }
                                }
                            })
                            .build(),
                    )?;
                }

                #[cfg(target_os = "windows")]
                {
                    app.handle().plugin(
                        tauri_plugin_global_shortcut::Builder::new()
                            .with_shortcuts(["ctrl+e"])?
                            .with_handler(|app, shortcut, _| {
                                if shortcut.matches(Modifiers::CONTROL, Code::KeyE) {
                                    let selected_text = tauri::async_runtime::block_on(async {
                                        get_selected_text(app).await
                                    });
                                    if let Ok(selected_text) = selected_text {
                                        let device_state = DeviceState::new();
                                        let mouse = device_state.get_mouse();
                                        let (x, y) = mouse.coords;
                                        let window = tauri::async_runtime::block_on(async {
                                            create_or_focus_compact_window(app, x as i32, y as i32).await
                                        });
                                        window.emit("shortcut-quickTranslate", serde_json::json!({
                                            "text": selected_text.replace("\"", "")
                                        })).unwrap();
                                        window.show().unwrap();
                                        window.set_focus().unwrap();
                                    }
                                }
                            })
                            .build(),
                    )?;
                }
            }

            setup_tray(app).unwrap();

            // Hide the app icon from the dock
            #[cfg(target_os = "macos")]
            use tauri::ActivationPolicy;
            #[cfg(target_os = "macos")]
            app.set_activation_policy(ActivationPolicy::Accessory);

            Ok(())
        })
        .on_window_event(|window, event| {
            match event {
                tauri::WindowEvent::Focused(is_focused) => {
                    // detect click outside of the focused window and hide the app
                    if !is_focused {
                         window.hide().unwrap();
                    }
                }
                _ => {}
            }
        })
        .invoke_handler(tauri::generate_handler![translate, correct, refine, get_mouse_position])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

fn main() {
    run();
}
