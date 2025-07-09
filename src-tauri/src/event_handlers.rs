use std::sync::Arc;
use tauri::{AppHandle, Emitter, Listener, Manager};
use device_query::{DeviceQuery, DeviceState};
use crate::mouse_service::{MouseService, MouseEvent};
use crate::window_management::{
    create_selection_icon, create_or_focus_compact_window,
    get_current_active_app, restore_focus_to_app
};

#[cfg(target_os = "windows")]
use crate::window_management::{get_current_active_window, restore_focus_to_window};

pub fn setup_selection_icon_events(
    app_handle: &AppHandle,
    mouse_service: &Arc<MouseService>
) {
    let selection_app_handle = app_handle.clone();
    let mouse_service_for_icon = mouse_service.clone();
    
    app_handle.listen("create-selection-icon", move |event| {
        println!("Received create-selection-icon event from mouse service");
        if let Ok(payload) = serde_json::from_str::<serde_json::Value>(event.payload()) {
            if let (Some(text), Some(x), Some(y)) = (
                payload.get("text").and_then(|v| v.as_str()).map(|s| s.to_string()),
                payload.get("x").and_then(|v| v.as_i64()),
                payload.get("y").and_then(|v| v.as_i64())
            ) {
                // Get the current active app/window BEFORE showing the selection icon
                #[cfg(target_os = "macos")]
                let current_app = get_current_active_app();
                
                #[cfg(target_os = "windows")]
                let current_window = get_current_active_window();
                
                #[cfg(not(any(target_os = "macos", target_os = "windows")))]
                let _current_app: Option<String> = None;
                
                let app_handle = selection_app_handle.clone();
                let mouse_service_ref = mouse_service_for_icon.clone();
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
                            
                            // Track the selection icon bounds and visibility
                            let icon_size = 24; // Icon size from create_selection_icon
                            mouse_service_ref.set_selection_icon_bounds(
                                x as i32, 
                                y as i32, 
                                icon_size, 
                                icon_size
                            );
                            mouse_service_ref.set_selection_icon_visible(true);
                            println!("Selection icon bounds set: ({}, {}, {}, {})", x, y, icon_size, icon_size);
                            
                            // After showing the icon, restore focus to the original app/window
                            std::thread::sleep(std::time::Duration::from_millis(100));
                            
                            #[cfg(target_os = "macos")]
                            if let Some(app_name) = current_app {
                                restore_focus_to_app(&app_name);
                                println!("Restored focus to application: {}", app_name);
                            }
                            
                            #[cfg(target_os = "windows")]
                            if let Some(hwnd) = current_window {
                                restore_focus_to_window(hwnd);
                                println!("Restored focus to previous window");
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
}

pub fn setup_icon_click_events(
    app_handle: &AppHandle,
    mouse_service: &Arc<MouseService>
) {
    let icon_app_handle = app_handle.clone();
    let mouse_service_for_click = mouse_service.clone();
    
    app_handle.listen("icon-clicked", move |event| {
        if let Some(_icon_window) = icon_app_handle.get_webview_window("selection-icon") {
            // Get the current mouse position for the translate popup
            let device_state = DeviceState::new();
            let (x, y) = device_state.get_mouse().coords;
            
            // Get the selected text from the event payload
            let selected_text = event.payload().to_owned();
            
            // Show the translate popup in a separate thread to avoid blocking
            let app_handle = icon_app_handle.clone();
            let mouse_service_ref = mouse_service_for_click.clone();
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
                        
                        // Hide the selection icon after showing the translate popup
                        if let Some(icon_window) = app_handle.get_webview_window("selection-icon") {
                            if let Err(e) = icon_window.hide() {
                                println!("Failed to hide selection icon after showing translate popup: {}", e);
                            } else {
                                mouse_service_ref.set_selection_icon_visible(false);
                                println!("Selection icon hidden after showing translate popup");
                            }
                        }
                    },
                    Err(e) => {
                        println!("Failed to create translate window: {}", e);
                    }
                }
            });
        }
    });
}

pub fn setup_mouse_events(
    app_handle: &AppHandle,
    mouse_service: &Arc<MouseService>
) {
    let app_handle_for_service = app_handle.clone();
    let mouse_service_for_events = mouse_service.clone();
    
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
            MouseEvent::ClickOutsideIcon(x, y) => {
                println!("Click outside icon detected at ({}, {})", x, y);
                if let Some(icon_window) = app_handle.get_webview_window("selection-icon") {
                    if let Err(e) = icon_window.hide() {
                        println!("Failed to hide selection icon: {}", e);
                    } else {
                        println!("Selection icon hidden due to outside click");
                        // Update the mouse service state
                        mouse_service_for_events.set_selection_icon_visible(false);
                    }
                }
            },
            _ => {}
        }
    });
}

pub fn setup_cleanup_events(app_handle: &AppHandle) {
    let cleanup_handle = app_handle.clone();
    app_handle.listen("app-shutdown", move |_| {
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
} 