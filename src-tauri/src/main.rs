// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod commands;
pub mod providers;
mod selected_text;
use commands::{correct, refine, translate};
use tauri::{ActivationPolicy, App, Emitter, Manager};
use tauri::menu::{Menu, MenuItem};
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

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        // .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .plugin(tauri_plugin_clipboard_manager::init())
        .plugin(tauri_plugin_positioner::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_store::Builder::default().build())
        .setup(move |app| {
            {
                use tauri_plugin_global_shortcut::{Code, Modifiers};
                
                #[cfg(target_os = "macos")] {

                    app.handle().plugin(
                        tauri_plugin_global_shortcut::Builder::new()
                            .with_shortcuts(["super+e"])?
                            .with_handler(|app, shortcut, _| {
                                // Add 'async' here
                                if shortcut.matches(Modifiers::SUPER, Code::KeyE) {
                                    // Trigger get selected text
                                    let selected_text = tauri::async_runtime::block_on(async {
                                        get_selected_text(app).await
                                    });
                                    if let Ok(selected_text) = selected_text {
                                        let window = app.clone().get_webview_window("main").unwrap();
                                        window.emit("shortcut-quickTranslate", selected_text).unwrap();
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
                            .with_handler(|app, shortcut| {
                                if shortcut.matches(Modifiers::CONTROL, Code::KeyE) {
                                    // Get selected text
                                    let selected_text = tauri::async_runtime::block_on(async {
                                        unsafe { get_selected_text(app) }.await
                                    });

                                    match selected_text {
                                        Ok(selected_text) => {
                                            let window = app.clone().get_webview_window("main").unwrap();
                                            window.emit("shortcut-quickTranslate", selected_text).unwrap(); 
                                            window.show().unwrap();
                                            window.set_focus().unwrap();
                                        }
                                        Err(_) => {
                                            let window = app.clone().get_webview_window("main").unwrap();
                                            window.emit("shortcut-quickTranslate", "Failed to read from clipboard!").unwrap(); 
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

            // Hide the app icon from the dock
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
        .invoke_handler(tauri::generate_handler![translate, correct, refine])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
