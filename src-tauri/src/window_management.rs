use tauri::{Manager, LogicalPosition, Position, WebviewUrl, WebviewWindowBuilder};
use device_query::{DeviceQuery, DeviceState};

// Window creation functions
pub async fn create_or_focus_compact_window(app: &tauri::AppHandle) -> Result<tauri::WebviewWindow, String> {
    let device_state = DeviceState::new();
    let (x, y) = device_state.get_mouse().coords;

    // Check if windows exist then just open it
    if let Some(window) = app.get_webview_window("translate-popup") {
        // Move the window to the mouse position
        window
            .set_position(Position::Logical(LogicalPosition::new(x as f64, y as f64)))
            .unwrap();
        Ok(window)
    } else {
        match WebviewWindowBuilder::new(
        app,
        "translate-popup", 
        WebviewUrl::App("translate-popup".into())
        ).title("Quick Translate")
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
                Ok(window)
            },
            Err(e) => {
                Err(format!("Failed to create translate-popup window: {}", e))
            }
        }
    }
}
