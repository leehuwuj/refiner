// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

pub mod providers;
use providers::{base::Provider, ollama::OllamaProvider};
// use tauri::menu::MenuBuilder;
// use tauri::menu::MenuItemBuilder;
use tauri::tray::ClickType;
use tauri::tray::TrayIconBuilder;
use tauri::Manager;

fn main() {
  tauri::Builder::default()
    .plugin(tauri_plugin_positioner::init())
    .setup(|app| {
      // let toggle = MenuItemBuilder::with_id("toggle", "Toggle").build(app)?;
      // let menu = MenuBuilder::new(app).items(&[&toggle]).build()?;
      let _tray = TrayIconBuilder::new()
        .on_tray_icon_event(|tray, event| {
            let app = tray.app_handle();
            tray.app_handle().show().unwrap();
            tauri_plugin_positioner::on_tray_event(app, &event);
            if event.click_type == ClickType::Left {
                if let Some(webview_window) = app.get_webview_window("main") {
                  let _ = webview_window.show();
                  let _ = webview_window.set_focus();
                }
            }
        })
        .build(app);
      Ok(())
    })
    // .system_tray(SystemTray::new().with_menu(tray_menu))
    // .on_system_tray_event(|app, event| {
    //   tauri_plugin_positioner::on_tray_event(app, &event);
    //         match event {
    //             SystemTrayEvent::LeftClick {
    //                 position: _,
    //                 size: _,
    //                 ..
    //             } => {
    //                 let window = app.get_window("main").unwrap();
    //                 let _ = window.move_window(Position::TrayCenter);

    //                 if window.is_visible().unwrap() {
    //                     window.hide().unwrap();
    //                 } else {
    //                     window.show().unwrap();
    //                     window.set_focus().unwrap();
    //                 }
    //             }
    //             SystemTrayEvent::RightClick {
    //                 position: _,
    //                 size: _,
    //                 ..
    //             } => {
    //                 println!("system tray received a right click");
    //             }
    //             SystemTrayEvent::DoubleClick {
    //                 position: _,
    //                 size: _,
    //                 ..
    //             } => {
    //                 println!("system tray received a double click");
    //             }
    //             SystemTrayEvent::MenuItemClick { id, .. } => match id.as_str() {
    //                 "quit" => {
    //                     std::process::exit(0);
    //                 }
    //                 "hide" => {
    //                     let window = app.get_window("main").unwrap();
    //                     window.hide().unwrap();
    //                 }
    //                 _ => {}
    //             },
    //             _ => {}
    //         }
    // })
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
    .invoke_handler(tauri::generate_handler![translate])
    .run(tauri::generate_context!())
    .expect("error while running tauri application");
}

#[tauri::command]
async fn translate(provider: &str, model: &str, text: &str) -> Result<String, String> {
  let provider = match provider {
    "ollama" => OllamaProvider::new(None, None, Some(model.to_string())),
    _ => panic!("Invalid provider"),
  };
  let res = provider.completion(text).await;
  return Ok(res);
}