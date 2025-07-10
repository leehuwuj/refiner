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
        tauri_plugin_positioner::on_tray_event(&handler, &event);
        
        match event {
            tauri::tray::TrayIconEvent::Click { button, .. } => {
                match button {
                    tauri::tray::MouseButton::Left => {
                        // Move the window to the top right corner for macOS
                        #[cfg(target_os = "macos")]
                        {
                            let win = handler.get_webview_window("main").unwrap();
                            let _ = win.as_ref().window().move_window(Position::TopRight);
                        }

                        // Move the window to the top center for Windows
                        #[cfg(target_os = "windows")]
                        {
                            let win = handler.get_webview_window("main").unwrap();
                            let _ = win.as_ref().window().move_window(Position::TopCenter);
                        }
                        
                        if let Some(webview_window) = handler.get_webview_window("main") {
                            webview_window.emit("tray-click", ()).unwrap();
                            webview_window.show().unwrap();
                            webview_window.set_focus().unwrap();
                        }
                    }
                    tauri::tray::MouseButton::Right => {
                        // Right-click should show the context menu automatically
                        // We don't need to do anything here as the menu is already set
                    }
                    _ => {}
                }
            }
            _ => {}
        }
    });
    Ok(())
} 