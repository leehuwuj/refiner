use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use tauri::Manager;
use tauri_plugin_store::StoreExt;

pub(crate) fn open_path(path: &str) {
    #[cfg(target_os = "macos")]
    let _ = std::process::Command::new("open").arg(path).spawn();
    #[cfg(target_os = "windows")]
    let _ = std::process::Command::new("explorer").arg(path).spawn();
    #[cfg(target_os = "linux")]
    let _ = std::process::Command::new("xdg-open").arg(path).spawn();
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct HistoryEntry {
    pub id: u64,
    pub timestamp: u64,
    pub mode: String,
    pub input_text: String,
    pub source_lang: String,
    pub target_lang: String,
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct HistoryFile {
    pub entries: Vec<HistoryEntry>,
}

fn history_file_path(app: &tauri::AppHandle) -> Result<PathBuf, String> {
    let dir = app.path().app_data_dir().map_err(|e| e.to_string())?;
    fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    Ok(dir.join("history.json"))
}

pub fn load_history_file(app: &tauri::AppHandle) -> Result<HistoryFile, String> {
    let path = history_file_path(app)?;
    if !path.exists() {
        return Ok(HistoryFile::default());
    }
    let content = fs::read_to_string(&path).map_err(|e| e.to_string())?;
    serde_json::from_str(&content).map_err(|e| e.to_string())
}

fn save_history_file(app: &tauri::AppHandle, history: &HistoryFile) -> Result<(), String> {
    let path = history_file_path(app)?;
    let content = serde_json::to_string_pretty(history).map_err(|e| e.to_string())?;
    fs::write(path, content).map_err(|e| e.to_string())
}

pub fn is_history_enabled(app: &tauri::AppHandle) -> bool {
    app.store("store.bin")
        .ok()
        .and_then(|s| s.get("HISTORY_ENABLED"))
        .and_then(|v| v.as_bool())
        .unwrap_or(false)
}

/// Called after every successful AI invocation to optionally record the input.
pub fn append_entry_if_enabled(
    app: &tauri::AppHandle,
    mode: &str,
    input_text: &str,
    source_lang: &str,
    target_lang: &str,
) {
    if !is_history_enabled(app) || input_text.trim().is_empty() {
        return;
    }
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0);

    let entry = HistoryEntry {
        id: now,
        timestamp: now,
        mode: mode.to_string(),
        input_text: input_text.to_string(),
        source_lang: source_lang.to_string(),
        target_lang: target_lang.to_string(),
    };

    if let Ok(mut history) = load_history_file(app) {
        history.entries.push(entry);
        let _ = save_history_file(app, &history);
    }
}

#[tauri::command]
pub async fn get_history_enabled(app_handle: tauri::AppHandle) -> bool {
    is_history_enabled(&app_handle)
}

#[tauri::command]
pub async fn toggle_history(app_handle: tauri::AppHandle, enabled: bool) -> Result<(), String> {
    let store = app_handle
        .store("store.bin")
        .map_err(|e| e.to_string())?;
    store.set("HISTORY_ENABLED", serde_json::Value::Bool(enabled));
    store.save().map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_history_count(app_handle: tauri::AppHandle) -> Result<usize, String> {
    Ok(load_history_file(&app_handle)?.entries.len())
}

#[tauri::command]
pub async fn export_history_json(app_handle: tauri::AppHandle) -> Result<String, String> {
    let history = load_history_file(&app_handle)?;
    let dir = app_handle
        .path()
        .app_data_dir()
        .map_err(|e| e.to_string())?;
    let export_path = dir.join("history_export.json");
    let content =
        serde_json::to_string_pretty(&history.entries).map_err(|e| e.to_string())?;
    fs::write(&export_path, content).map_err(|e| e.to_string())?;

    let path_str = export_path.to_string_lossy().to_string();
    open_path(&path_str);
    Ok(path_str)
}

#[tauri::command]
pub async fn clear_history(app_handle: tauri::AppHandle) -> Result<(), String> {
    let path = history_file_path(&app_handle)?;
    if path.exists() {
        fs::remove_file(&path).map_err(|e| e.to_string())?;
    }
    Ok(())
}
