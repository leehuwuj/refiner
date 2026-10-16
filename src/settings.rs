//! User settings, persisted as JSON in the app data directory.
//!
//! The directory is the same one the Tauri build used
//! (`~/Library/Application Support/dev.leehuwuj.refiner` on macOS,
//! `%APPDATA%\dev.leehuwuj.refiner` on Windows), so history and reports carry
//! over. On first launch the old Tauri `store.bin` is imported.

use std::path::PathBuf;

use serde::{Deserialize, Serialize};
use serde_json::Value;

pub const APP_ID: &str = "dev.leehuwuj.refiner";
const SETTINGS_FILE: &str = "settings.json";
const LEGACY_STORE_FILE: &str = "store.bin";

/// Directory for settings, history and analysis reports.
/// `REFINER_DATA_DIR` overrides it (handy for development).
pub fn data_dir() -> PathBuf {
    let dir = std::env::var_os("REFINER_DATA_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|| {
            dirs::data_dir()
                .unwrap_or_else(std::env::temp_dir)
                .join(APP_ID)
        });
    let _ = std::fs::create_dir_all(&dir);
    dir
}

// ── Catalogs ────────────────────────────────────────────────────────────────

pub struct ProviderInfo {
    pub id: &'static str,
    pub label: &'static str,
    pub models: &'static [&'static str],
    pub needs_api_key: bool,
}

pub const PROVIDERS: &[ProviderInfo] = &[
    ProviderInfo {
        id: "ollama",
        label: "Ollama",
        models: &["qwen3", "gemma3", "phi3", "llama3"],
        needs_api_key: false,
    },
    ProviderInfo {
        id: "openai",
        label: "OpenAI",
        models: &["gpt-4.1-nano", "gpt-4.1-mini", "gpt-4o-mini"],
        needs_api_key: true,
    },
    ProviderInfo {
        id: "gemini",
        label: "Gemini",
        models: &["gemini-2.0-flash-lite-exp", "gemini-2.5-flash"],
        needs_api_key: true,
    },
    ProviderInfo {
        id: "groq",
        label: "Groq",
        models: &["llama-3.1-8b-instant", "gemma2-9b-it"],
        needs_api_key: true,
    },
];

pub fn provider_info(id: &str) -> &'static ProviderInfo {
    PROVIDERS
        .iter()
        .find(|p| p.id == id)
        .unwrap_or(&PROVIDERS[0])
}

/// Source-language choice that lets the model detect the language.
pub const DETECT_LANGUAGE: &str = "Detect Language";

/// Languages offered as translation targets / preferred language.
pub const LANGUAGES: &[&str] = &[
    "English",
    "Tiếng Việt",
    "Français",
    "Deutsch",
    "日本語",
    "中文",
    "한국어",
    "Español",
];

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum ShortcutWindow {
    #[default]
    Main,
    Popup,
}

impl ShortcutWindow {
    pub const ALL: [Self; 2] = [Self::Popup, Self::Main];
    pub fn label(self) -> &'static str {
        match self {
            Self::Popup => "Small Popup",
            Self::Main => "Main Window",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum TextSize {
    Small,
    #[default]
    Medium,
    Large,
}

impl TextSize {
    pub const ALL: [Self; 3] = [Self::Small, Self::Medium, Self::Large];
    pub fn label(self) -> &'static str {
        match self {
            Self::Small => "Small",
            Self::Medium => "Medium",
            Self::Large => "Large",
        }
    }
    pub fn scale(self) -> f32 {
        match self {
            Self::Small => 0.9,
            Self::Medium => 1.0,
            Self::Large => 1.1,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum ThemePref {
    System,
    #[default]
    Dark,
    Light,
}

impl ThemePref {
    pub const ALL: [Self; 3] = [Self::System, Self::Dark, Self::Light];
    pub fn label(self) -> &'static str {
        match self {
            Self::System => "System",
            Self::Dark => "Dark",
            Self::Light => "Light",
        }
    }
}

// ── Settings ────────────────────────────────────────────────────────────────

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Settings {
    pub provider: String,
    pub model: String,
    pub api_key: String,
    /// Base URL override; empty means the provider default.
    pub model_url: String,
    pub thinking: bool,
    pub shortcut_window: ShortcutWindow,
    /// Per-mode prompt overrides; empty means the built-in default.
    pub prompt_translate: String,
    pub prompt_correct: String,
    pub prompt_refine: String,
    /// The user's native language, also the default translation target.
    pub preferred_lang: String,
    pub text_size: TextSize,
    pub theme: ThemePref,
    pub history_enabled: bool,
    /// Show a floating icon next to text selected with the mouse (macOS).
    pub selection_icon: bool,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            provider: "ollama".into(),
            model: "gemma3".into(),
            api_key: String::new(),
            model_url: String::new(),
            thinking: true,
            shortcut_window: ShortcutWindow::Main,
            prompt_translate: String::new(),
            prompt_correct: String::new(),
            prompt_refine: String::new(),
            preferred_lang: "Tiếng Việt".into(),
            text_size: TextSize::Medium,
            theme: ThemePref::Dark,
            history_enabled: false,
            selection_icon: false,
        }
    }
}

impl Settings {
    pub fn load() -> Self {
        let dir = data_dir();
        let path = dir.join(SETTINGS_FILE);
        if let Ok(text) = std::fs::read_to_string(&path) {
            return serde_json::from_str(&text).unwrap_or_else(|err| {
                eprintln!("refiner: ignoring unreadable {}: {err}", path.display());
                Self::default()
            });
        }
        let migrated = Self::from_legacy_store(&dir.join(LEGACY_STORE_FILE)).unwrap_or_default();
        let _ = migrated.save();
        migrated
    }

    pub fn save(&self) -> anyhow::Result<()> {
        let path = data_dir().join(SETTINGS_FILE);
        let json = serde_json::to_string_pretty(self)?;
        let tmp = path.with_extension("json.tmp");
        std::fs::write(&tmp, json)?;
        std::fs::rename(&tmp, &path)?;
        Ok(())
    }

    pub fn provider_info(&self) -> &'static ProviderInfo {
        provider_info(&self.provider)
    }

    /// Imports the key/value JSON written by `tauri-plugin-store`.
    fn from_legacy_store(path: &std::path::Path) -> Option<Self> {
        let text = std::fs::read_to_string(path).ok()?;
        let map: serde_json::Map<String, Value> = serde_json::from_str(&text).ok()?;
        let s = |k: &str| map.get(k).and_then(Value::as_str).map(str::to_string);
        let b = |k: &str| map.get(k).and_then(Value::as_bool);
        let mut out = Self::default();
        if let Some(v) = s("PROVIDER") {
            out.provider = v;
        }
        if let Some(v) = s("MODEL") {
            out.model = v;
        }
        if let Some(v) = s("LLM_API_KEY") {
            out.api_key = v;
        }
        if let Some(v) = s("MODEL_URL").or_else(|| s("OLLAMA_ENDPOINT")) {
            out.model_url = v;
        }
        if let Some(v) = b("THINKING").or_else(|| b("OLLAMA_THINKING")) {
            out.thinking = v;
        }
        if s("SHORTCUT_WINDOW_TYPE").as_deref() == Some("popup") {
            out.shortcut_window = ShortcutWindow::Popup;
        }
        out.prompt_translate = s("PROMPT_TRANSLATE").unwrap_or_default();
        out.prompt_correct = s("PROMPT_CORRECT").unwrap_or_default();
        out.prompt_refine = s("PROMPT_REFINE").unwrap_or_default();
        if let Some(v) = s("PREFERRED_LANG") {
            out.preferred_lang = v;
        }
        out.text_size = match s("TEXT_SIZE").as_deref() {
            Some("small") => TextSize::Small,
            Some("large") => TextSize::Large,
            _ => TextSize::Medium,
        };
        out.history_enabled = b("HISTORY_ENABLED").unwrap_or(false);
        Some(out)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn imports_legacy_tauri_store() {
        let dir = std::env::temp_dir().join(format!("refiner-legacy-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("store.bin");
        std::fs::write(
            &path,
            r#"{"PROVIDER":"openai","MODEL":"gpt-4.1-mini","LLM_API_KEY":"k",
               "OLLAMA_ENDPOINT":"http://x","THINKING":false,"SHORTCUT_WINDOW_TYPE":"popup",
               "PROMPT_CORRECT":"fix it","PREFERRED_LANG":"English","TEXT_SIZE":"large",
               "HISTORY_ENABLED":true}"#,
        )
        .unwrap();
        let s = Settings::from_legacy_store(&path).unwrap();
        assert_eq!(s.provider, "openai");
        assert_eq!(s.model, "gpt-4.1-mini");
        assert_eq!(s.api_key, "k");
        assert_eq!(s.model_url, "http://x");
        assert!(!s.thinking);
        assert_eq!(s.shortcut_window, ShortcutWindow::Popup);
        assert_eq!(s.prompt_correct, "fix it");
        assert_eq!(s.preferred_lang, "English");
        assert_eq!(s.text_size, TextSize::Large);
        assert!(s.history_enabled);
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn partial_json_fills_defaults() {
        let s: Settings = serde_json::from_str(r#"{"model":"qwen3"}"#).unwrap();
        assert_eq!(s.model, "qwen3");
        assert_eq!(s.provider, "ollama");
        assert!(s.thinking);
    }
}
