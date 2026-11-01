//! Optional record of every AI request, used by language analysis.
//! Stored as `history.json` in the app data directory (same format as the
//! Tauri build).

use std::path::PathBuf;
use std::sync::Mutex;

use serde::{Deserialize, Serialize};

use crate::settings::data_dir;

/// Serializes read-modify-write of the history file across background tasks.
static LOCK: Mutex<()> = Mutex::new(());

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

fn history_path() -> PathBuf {
    data_dir().join("history.json")
}

pub fn now_ms() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0)
}

pub fn load() -> Result<HistoryFile, String> {
    let path = history_path();
    if !path.exists() {
        return Ok(HistoryFile::default());
    }
    let content = std::fs::read_to_string(&path).map_err(|e| e.to_string())?;
    serde_json::from_str(&content).map_err(|e| e.to_string())
}

fn save(history: &HistoryFile) -> Result<(), String> {
    let content = serde_json::to_string_pretty(history).map_err(|e| e.to_string())?;
    std::fs::write(history_path(), content).map_err(|e| e.to_string())
}

pub fn append(mode: &str, input_text: &str, source_lang: &str, target_lang: &str) {
    if input_text.trim().is_empty() {
        return;
    }
    let _guard = LOCK.lock();
    let now = now_ms();
    if let Ok(mut history) = load() {
        history.entries.push(HistoryEntry {
            id: now,
            timestamp: now,
            mode: mode.to_string(),
            input_text: input_text.to_string(),
            source_lang: source_lang.to_string(),
            target_lang: target_lang.to_string(),
        });
        let _ = save(&history);
    }
}

pub fn count() -> usize {
    load().map(|h| h.entries.len()).unwrap_or(0)
}

/// Writes all entries to `history_export.json` and returns its path.
pub fn export_json() -> Result<PathBuf, String> {
    let history = load()?;
    let path = data_dir().join("history_export.json");
    let content = serde_json::to_string_pretty(&history.entries).map_err(|e| e.to_string())?;
    std::fs::write(&path, content).map_err(|e| e.to_string())?;
    Ok(path)
}

pub fn clear() -> Result<(), String> {
    let _guard = LOCK.lock();
    let path = history_path();
    if path.exists() {
        std::fs::remove_file(&path).map_err(|e| e.to_string())?;
    }
    Ok(())
}

/// Opens a file or folder with the OS default handler.
pub fn open_path(path: &std::path::Path) {
    #[cfg(target_os = "macos")]
    let _ = std::process::Command::new("open").arg(path).spawn();
    #[cfg(target_os = "windows")]
    let _ = std::process::Command::new("explorer").arg(path).spawn();
    #[cfg(target_os = "linux")]
    let _ = std::process::Command::new("xdg-open").arg(path).spawn();
}
