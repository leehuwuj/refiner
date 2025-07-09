use tauri::{App, Emitter, Manager};
use tauri_plugin_global_shortcut::{Code, Modifiers};
use crate::selected_text::get_selected_text;
use crate::window_management::create_or_focus_compact_window;
use crate::commands::get_shortcut_window_type;
use std::sync::{Mutex};
use std::time::{Duration, Instant};

// Global debounce state
static LAST_SHORTCUT_TIME: Mutex<Option<Instant>> = Mutex::new(None);
const DEBOUNCE_DURATION: Duration = Duration::from_millis(300);

/// Common shortcut handler that processes the shortcut logic for both platforms
async fn handle_shortcut_common(app_handle: tauri::AppHandle) {
    // Get the window type preference
    let window_type = get_shortcut_window_type(app_handle.clone()).await.unwrap_or("main".to_string());
    
    // Get selected text with platform-specific handling
    let selected_text_result = {
        #[cfg(target_os = "windows")]
        {
            unsafe { get_selected_text(&app_handle).await }
        }
        #[cfg(not(target_os = "windows"))]
        {
            get_selected_text(&app_handle).await
        }
    };
    
    if let Ok(selected_text) = selected_text_result {
        let text = selected_text.trim();
        if !text.is_empty() {
            let window_result = match window_type.as_str() {
                "popup" => create_or_focus_compact_window(&app_handle).await,
                "main" => {
                    if let Some(win) = app_handle.get_webview_window("main") {
                        Ok(win)
                    } else {
                        Err("Main window not found".to_string())
                    }
                },  
                _ => Err("Invalid window type".to_string()),
            };
            
            if let Ok(window) = window_result {
                let _ = window.emit("shortcut-quickTranslate", format!("text:{}", text));
                let _ = window.show();
                let _ = window.set_focus();
            }
        }
    }
}

/// Handles debouncing and executes the shortcut logic
fn handle_shortcut_with_debounce(app: &tauri::AppHandle) {
    let now = Instant::now();
    let mut last_time = LAST_SHORTCUT_TIME.lock().unwrap();
    
    if let Some(last_execution) = *last_time {
        if now.duration_since(last_execution) < DEBOUNCE_DURATION {
            return;
        }
    }
    
    *last_time = Some(now);
    drop(last_time);
    
    let app_handle = app.clone();
    tauri::async_runtime::spawn(async move {
        handle_shortcut_common(app_handle).await;
    });
}

pub fn setup_shortcuts(app: &App) -> Result<(), Box<dyn std::error::Error>> {
    #[cfg(target_os = "macos")] {
        app.handle().plugin(
            tauri_plugin_global_shortcut::Builder::new()
                .with_shortcuts(["super+e"])?
                .with_handler(|app, shortcut, _| {
                    if shortcut.matches(Modifiers::SUPER, Code::KeyE) {
                        handle_shortcut_with_debounce(app);
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
                        handle_shortcut_with_debounce(app);
                    }
                })
                .build(),
        )?;
    }

    Ok(())
} 