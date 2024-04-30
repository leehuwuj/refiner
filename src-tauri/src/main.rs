// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod commands;
mod selected_text;
pub mod providers;
use commands::{correct, refine, save_api_key, translate};
use tauri::Manager;
use tauri::{
    menu::{Menu, MenuItem},
    tray::ClickType,
    App,
};
use tauri_plugin_positioner::{Position, WindowExt};

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
        // Move the window to the top right corner
        let win = handler.get_webview_window("main").unwrap();
        let _ = win.as_ref().window().move_window(Position::TopRight);
        // Handle the tray icon click event
        handler.show().unwrap();
        tauri_plugin_positioner::on_tray_event(&handler, &event);
        if event.click_type == ClickType::Left {
            if let Some(webview_window) = handler.get_webview_window("main") {
                let _ = webview_window.show();
                let _ = webview_window.set_focus();
            }
        }
    });
    Ok(())
}

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        // .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .plugin(tauri_plugin_clipboard_manager::init())
        .plugin(tauri_plugin_positioner::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_store::Builder::default().build())
        .setup(move |app| {
            #[cfg(desktop)]
            {
                use tauri_plugin_global_shortcut::{Code, Modifiers};

                app.handle()
                    .plugin(
                        tauri_plugin_global_shortcut::Builder::new()
                            .with_shortcuts(["super+e"])?
                            .with_handler(|app, shortcut| { // Add 'async' here
                                if shortcut.matches(Modifiers::SUPER, Code::KeyE) {
                                    // Trigger get selected text
                                    let selected_text = tauri::async_runtime::block_on(async {
                                        get_selected_text(app).await
                                    });
                                    if let Ok(selected_text) = selected_text {
                                        let _ = app.emit("shortcut-quickTranslate", selected_text);
                                    }
                                    let window = app.clone().get_webview_window("main").unwrap();
                                    window.show().unwrap();
                                    window.set_focus().unwrap();
                                    // let _ = app.emit("shortcut-quickTranslate", selected_text);
                                    
                                }
                            })
                            .build(),
                    )?;
            }

            setup_tray(app).unwrap();

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
        .invoke_handler(tauri::generate_handler![
            translate,
            correct,
            refine,
            save_api_key
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
