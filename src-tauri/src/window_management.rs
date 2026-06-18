use tauri::{Manager, Position, WebviewUrl, WebviewWindowBuilder};
use device_query::{DeviceQuery, DeviceState};

const POPUP_WIDTH: f64 = 300.0;
const POPUP_HEIGHT: f64 = 200.0;
const CURSOR_OFFSET: f64 = 6.0;

pub async fn create_or_focus_settings_window(app: &tauri::AppHandle) -> Result<tauri::WebviewWindow, String> {
    if let Some(window) = app.get_webview_window("settings") {
        window.show().map_err(|e| e.to_string())?;
        window.set_focus().map_err(|e| e.to_string())?;
        Ok(window)
    } else {
        match WebviewWindowBuilder::new(
            app,
            "settings",
            WebviewUrl::App("settings".into()),
        )
        .title("Settings")
        .inner_size(480.0, 520.0)
        .resizable(false)
        .build()
        {
            Ok(window) => Ok(window),
            Err(e) => Err(format!("Failed to create settings window: {}", e)),
        }
    }
}

fn get_mouse_position() -> (i32, i32) {
    let device_state = DeviceState::new();
    let (x, y) = device_state.get_mouse().coords;
    (x, y)
}

/// Returns the best top-left position for the compact popup so it stays fully
/// on screen.  On macOS all coordinates are logical pixels; on Windows they are
/// physical pixels (window size is converted with scale_factor accordingly).
fn get_smart_position(app: &tauri::AppHandle, mouse_x: i32, mouse_y: i32) -> (f64, f64) {
    let monitors = match app.available_monitors() {
        Ok(m) => m,
        Err(_) => return (mouse_x as f64, mouse_y as f64),
    };

    if monitors.is_empty() {
        return (mouse_x as f64, mouse_y as f64);
    }

    #[cfg(target_os = "macos")]
    {
        // device_query returns logical coordinates on macOS.
        // Find the monitor whose logical bounds contain the cursor.
        let monitor = monitors
            .iter()
            .find(|m| {
                let pos = m.position();
                let size = m.size();
                let sf = m.scale_factor();
                let lx = pos.x as f64 / sf;
                let ly = pos.y as f64 / sf;
                let lw = size.width as f64 / sf;
                let lh = size.height as f64 / sf;
                (mouse_x as f64) >= lx
                    && (mouse_x as f64) < lx + lw
                    && (mouse_y as f64) >= ly
                    && (mouse_y as f64) < ly + lh
            })
            .or_else(|| monitors.first());

        if let Some(m) = monitor {
            let pos = m.position();
            let size = m.size();
            let sf = m.scale_factor();
            let mon_lx = pos.x as f64 / sf;
            let mon_ly = pos.y as f64 / sf;
            let mon_lw = size.width as f64 / sf;
            let mon_lh = size.height as f64 / sf;

            let mx = mouse_x as f64;
            let my = mouse_y as f64;

            // Prefer below-right of cursor; flip if that would overflow.
            let x = if mx + CURSOR_OFFSET + POPUP_WIDTH > mon_lx + mon_lw {
                (mx - POPUP_WIDTH - CURSOR_OFFSET).max(mon_lx)
            } else {
                mx + CURSOR_OFFSET
            };

            let y = if my + CURSOR_OFFSET + POPUP_HEIGHT > mon_ly + mon_lh {
                (my - POPUP_HEIGHT - CURSOR_OFFSET).max(mon_ly)
            } else {
                my + CURSOR_OFFSET
            };

            return (x, y);
        }
    }

    #[cfg(target_os = "windows")]
    {
        // device_query returns physical coordinates on Windows.
        let monitor = monitors
            .iter()
            .find(|m| {
                let pos = m.position();
                let size = m.size();
                mouse_x >= pos.x
                    && mouse_x < pos.x + size.width as i32
                    && mouse_y >= pos.y
                    && mouse_y < pos.y + size.height as i32
            })
            .or_else(|| monitors.first());

        if let Some(m) = monitor {
            let pos = m.position();
            let size = m.size();
            let sf = m.scale_factor();

            // Window builder takes logical pixels; convert to physical for
            // boundary math so everything is in the same unit.
            let phys_w = (POPUP_WIDTH * sf) as i32;
            let phys_h = (POPUP_HEIGHT * sf) as i32;
            let offset = CURSOR_OFFSET as i32;

            let mon_right = pos.x + size.width as i32;
            let mon_bottom = pos.y + size.height as i32;

            let x = if mouse_x + offset + phys_w > mon_right {
                (mouse_x - phys_w - offset).max(pos.x)
            } else {
                mouse_x + offset
            };

            let y = if mouse_y + offset + phys_h > mon_bottom {
                (mouse_y - phys_h - offset).max(pos.y)
            } else {
                mouse_y + offset
            };

            return (x as f64, y as f64);
        }
    }

    (mouse_x as f64, mouse_y as f64)
}

pub async fn create_or_focus_compact_window(app: &tauri::AppHandle) -> Result<tauri::WebviewWindow, String> {
    let mouse = get_mouse_position();
    let (px, py) = get_smart_position(app, mouse.0, mouse.1);

    if let Some(window) = app.get_webview_window("compact-popup") {
        #[cfg(target_os = "windows")]
        {
            use tauri::PhysicalPosition;
            window
                .set_position(Position::Physical(PhysicalPosition::new(px as i32, py as i32)))
                .unwrap();
        }

        #[cfg(target_os = "macos")]
        {
            use tauri::LogicalPosition;
            window
                .set_position(Position::Logical(LogicalPosition::new(px, py)))
                .unwrap();
        }
        Ok(window)
    } else {
        match WebviewWindowBuilder::new(
            app,
            "compact-popup",
            WebviewUrl::App("compact-popup".into()),
        )
        .title("Quick Translate")
        .transparent(true)
        .decorations(false)
        .always_on_top(true)
        .inner_size(POPUP_WIDTH, POPUP_HEIGHT)
        .skip_taskbar(true)
        .visible(false)
        .position(px, py)
        .build()
        {
            Ok(window) => Ok(window),
            Err(e) => Err(format!("Failed to create compact-popup window: {}", e)),
        }
    }
}
