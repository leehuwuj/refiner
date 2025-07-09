use tauri::{App, Manager, Emitter};
use tauri::menu::{Menu, MenuItem};
use tauri_plugin_positioner::{Position, WindowExt};

pub fn setup_tray(app: &App) -> Result<(), String> {
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