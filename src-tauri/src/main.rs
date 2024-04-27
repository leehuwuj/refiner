// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

pub mod providers;
use providers::{base::Provider, ollama::OllamaProvider};

fn main() {
  tauri::Builder::default()
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