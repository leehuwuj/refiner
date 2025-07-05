// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod commands;
pub mod providers;
mod selected_text;
mod mouse_service;

use commands::{correct, refine, translate, get_double_click_enabled, save_settings};
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

async fn create_or_focus_compact_window(app: &tauri::AppHandle, x: i32, y: i32) -> Result<tauri::WebviewWindow, String> {
    // Properly close existing window to prevent window class conflicts
    if let Some(window) = app.get_webview_window("translate-popup") {
        println!("Closing existing translate-popup window to prevent class conflicts");
        if let Err(e) = window.close() {
            println!("Warning: Failed to close existing translate-popup window: {}", e);
        }
        // Give Windows time to properly unregister the window class
        std::thread::sleep(std::time::Duration::from_millis(100));
    }

    // Create new window with proper error handling
    match WebviewWindowBuilder::new(
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
    {
        Ok(window) => {
            println!("Successfully created translate-popup window");
            Ok(window)
        },
        Err(e) => {
            println!("Failed to create translate-popup window: {}", e);
            Err(format!("Failed to create translate-popup window: {}", e))
        }
    }
}

async fn create_selection_icon(app: &tauri::AppHandle, x: i32, y: i32) -> Result<tauri::WebviewWindow, String> {
    // Properly close existing window to prevent window class conflicts
    if let Some(window) = app.get_webview_window("selection-icon") {
        println!("Closing existing selection-icon window to prevent class conflicts");
        if let Err(e) = window.close() {
            println!("Warning: Failed to close existing selection-icon window: {}", e);
        }
        // Give Windows time to properly unregister the window class
        std::thread::sleep(std::time::Duration::from_millis(100));
    }

    // Create new window with proper error handling
    match WebviewWindowBuilder::new(
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
    {
        Ok(window) => {
            println!("Successfully created selection-icon window");
            Ok(window)
        },
        Err(e) => {
            println!("Failed to create selection-icon window: {}", e);
            Err(format!("Failed to create selection-icon window: {}", e))
        }
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
            // Hide the app icon from the dock
            #[cfg(target_os = "macos")]
            {
                use tauri::ActivationPolicy;
                app.set_activation_policy(ActivationPolicy::Accessory);
            }
            
            let app_handle = app.handle();
            
            // Listen for selection icon creation events from mouse service (thread safety)
            let selection_app_handle = app_handle.clone();
            app_handle.listen("create-selection-icon", move |event| {
                println!("Received create-selection-icon event from mouse service");
                if let Ok(payload) = serde_json::from_str::<serde_json::Value>(event.payload()) {
                    if let (Some(text), Some(x), Some(y)) = (
                        payload.get("text").and_then(|v| v.as_str()).map(|s| s.to_string()),
                        payload.get("x").and_then(|v| v.as_i64()),
                        payload.get("y").and_then(|v| v.as_i64())
                    ) {
                        let app_handle = selection_app_handle.clone();
                        tauri::async_runtime::spawn(async move {
                            match create_selection_icon(&app_handle, x as i32, y as i32).await {
                                Ok(window) => {
                                    // Store the selected text in the window's state
                                    if let Err(e) = window.emit("set-selected-text", &text) {
                                        println!("Failed to emit set-selected-text event: {}", e);
                                    }
                                    if let Err(e) = window.show() {
                                        println!("Failed to show selection icon window: {}", e);
                                    }
                                    if let Err(e) = window.set_focus() {
                                        println!("Failed to set focus on selection icon window: {}", e);
                                    }
                                },
                                Err(e) => {
                                    println!("Failed to create selection icon window: {}", e);
                                }
                            }
                        });
                    } else {
                        println!("Invalid payload for create-selection-icon event");
                    }
                } else {
                    println!("Failed to parse create-selection-icon event payload");
                }
            });

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
                        match create_or_focus_compact_window(&app_handle, x, y).await {
                            Ok(translate_window) => {
                                if let Err(e) = translate_window.emit("shortcut-quickTranslate", serde_json::json!({
                                    "text": selected_text
                                })) {
                                    println!("Failed to emit shortcut-quickTranslate event: {}", e);
                                }
                                if let Err(e) = translate_window.show() {
                                    println!("Failed to show translate window: {}", e);
                                }
                                if let Err(e) = translate_window.set_focus() {
                                    println!("Failed to set focus on translate window: {}", e);
                                }
                            },
                            Err(e) => {
                                println!("Failed to create translate window: {}", e);
                            }
                        }
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
                        let device_state = DeviceState::new();
                        let (x, y) = device_state.get_mouse().coords;
                        
                        // Emit event to main thread to handle window creation (thread safety)
                        println!("Text selected from mouse service, emitting event to main thread");
                        if let Err(e) = app_handle.emit("create-selection-icon", serde_json::json!({
                            "text": text,
                            "x": x + 5,
                            "y": y - 5
                        })) {
                            println!("Failed to emit create-selection-icon event: {}", e);
                        }
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
                            .with_handler(|app, shortcut, _| {
                                if shortcut.matches(Modifiers::SUPER, Code::KeyE) {
                                    if let Some(window) = app.get_webview_window("main") {
                                        // Get selected text and send it to the main window
                                        if let Ok(selected_text) = tauri::async_runtime::block_on(async {
                                            get_selected_text(app).await
                                        }) {
                                            window.emit("shortcut-quickTranslate", format!("text:{}", selected_text)).unwrap();
                                            window.show().unwrap();
                                            window.set_focus().unwrap();
                                        }
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
                                    if let Some(window) = app.get_webview_window("main") {
                                        // Get selected text and send it to the main window
                                        if let Ok(selected_text) = tauri::async_runtime::block_on(async {
                                            unsafe { get_selected_text(app).await }
                                        }) {
                                            window.emit("shortcut-quickTranslate", format!("text:{}", selected_text)).unwrap();
                                            window.show().unwrap();
                                            window.set_focus().unwrap();
                                        }
                                    }
                                }
                            })
                            .build(),
                    )?;
                }
            }

            setup_tray(app).unwrap();

            // Register cleanup handler for proper window management
            let cleanup_handle = app_handle.clone();
            app.listen("app-shutdown", move |_| {
                println!("Application shutting down, cleaning up windows...");
                
                // Close all windows properly to prevent window class issues
                if let Some(window) = cleanup_handle.get_webview_window("translate-popup") {
                    println!("Closing translate-popup window");
                    let _ = window.close();
                }
                
                if let Some(window) = cleanup_handle.get_webview_window("selection-icon") {
                    println!("Closing selection-icon window");
                    let _ = window.close();
                }
                
                if let Some(window) = cleanup_handle.get_webview_window("main") {
                    println!("Closing main window");
                    let _ = window.close();
                }
                
                println!("Window cleanup completed");
            });

            Ok(())
        })
        .on_window_event(|window, event| {
            match event {
                tauri::WindowEvent::Focused(is_focused) => {
                    // detect click outside of the focused window and hide the app
                    if !is_focused {
                        println!("Window lost focus, hiding window: {}", window.label());
                        if let Err(e) = window.hide() {
                            println!("Failed to hide window on focus loss: {}", e);
                        }
                    } else {
                        println!("Window gained focus: {}", window.label());
                    }
                }
                tauri::WindowEvent::Destroyed => {
                    println!("Window destroyed: {}", window.label());
                }
                tauri::WindowEvent::CloseRequested { .. } => {
                    println!("Window close requested: {}", window.label());
                }
                _ => {}
            }
        })
        .invoke_handler(tauri::generate_handler![translate, correct, refine, get_mouse_position, get_double_click_enabled, save_settings])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

fn main() {
    run();
}
