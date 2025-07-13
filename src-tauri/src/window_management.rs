use tauri::{Manager, Position, WebviewUrl, WebviewWindowBuilder};
use device_query::{DeviceQuery, DeviceState};

fn get_mouse_position() -> (i32, i32) {
    let device_state = DeviceState::new();
    let (x, y) = device_state.get_mouse().coords;
    (x, y)
}

pub async fn create_or_focus_compact_window(app: &tauri::AppHandle) -> Result<tauri::WebviewWindow, String> {
    let position = get_mouse_position();

    // Check if windows exist then just open it
    if let Some(window) = app.get_webview_window("compact-popup") {
        // Move the window to the mouse position
        #[cfg(target_os = "windows")]
        {
            use tauri::PhysicalPosition;
            window
            .set_position(Position::Physical(PhysicalPosition::new(position.0, position.1)))
            .unwrap();
        }

        #[cfg(target_os = "macos")]
        {
            use tauri::LogicalPosition;
            window
                .set_position(Position::Logical(LogicalPosition::new(position.0 as f64, position.1 as f64)))
                .unwrap();
        }
        Ok(window)
    } else {
        match WebviewWindowBuilder::new(
        app,
        "compact-popup", 
        WebviewUrl::App("compact-popup".into())
        ).title("Quick Translate")
        .transparent(true)
        .decorations(false)
        .always_on_top(true)
        .inner_size(300.0, 200.0)
        .skip_taskbar(true)
        .visible(false)
        .position(position.0 as f64, position.1 as f64)
        .build()
        {
            Ok(window) => {
                Ok(window)
            },
            Err(e) => {
                Err(format!("Failed to create compact-popup window: {}", e))
            }
        }
    }
}
