// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

pub mod providers;
mod commands;
use commands::{correct, refine, translate};
use tauri::{menu::{Menu, MenuItem}, tray::ClickType, App};
use tauri_plugin_positioner::{WindowExt, Position};
use tauri::Manager;

fn setup_tray(app: &App) -> Result<(), String> {
  let app_handler = app.app_handle();
  let tray = app_handler.tray_by_id("refiner").unwrap();
  let menu = Menu::with_items(app_handler, &[
    &MenuItem::with_id(app_handler, "quit", "Quit", true, None::<String>).unwrap()
  ]).unwrap();
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
    .plugin(tauri_plugin_positioner::init())
    .setup(move |app| Ok({
      setup_tray(app).unwrap();
    }))
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