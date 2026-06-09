use crate::providers::{self, base::get_provider};
use crate::window_management::create_or_focus_settings_window;
use providers::base::Provider;
use tauri_plugin_store::StoreExt;

const DEFAULT_TRANSLATION_PROMPT: &str =
    "You are an expert translator. Translate the following text from {original_lang} to {target_lang}. Provide only the translated text, without any additional explanations or formatting.";

const DEFAULT_CORRECTION_PROMPT: &str =
    "You are an expert in grammar. Correct the grammar of the following text in {target_lang}. Provide only the corrected text, without any additional explanations or formatting.";

const DEFAULT_REFINE_PROMPT: &str =
    "You are an expert editor. Rewrite the following text in a more conversational style, in {target_lang}. Provide only the rewritten text, without any additional explanations or formatting.";

// Helper function to trim thinking blocks from model responses
fn trim_thinking_blocks(response: &str) -> String {
    let mut result = response.to_string();
    
    // Remove <think>...</think> blocks
    while let Some(start) = result.find("<think>") {
        if let Some(end) = result[start..].find("</think>") {
            let end_pos = start + end + 7; // 7 is length of "</think>"
            result.replace_range(start..end_pos, "");
        } else {
            // If no closing tag found, remove everything from <think> to the end
            result.truncate(start);
        }
    }
    
    // Remove <thinking>...</thinking> blocks (alternative format)
    while let Some(start) = result.find("<thinking>") {
        if let Some(end) = result[start..].find("</thinking>") {
            let end_pos = start + end + 10; // 10 is length of "</thinking>"
            result.replace_range(start..end_pos, "");
        } else {
            // If no closing tag found, remove everything from <thinking> to the end
            result.truncate(start);
        }
    }
    
    // Remove <reasoning>...</reasoning> blocks (another alternative format)
    while let Some(start) = result.find("<reasoning>") {
        if let Some(end) = result[start..].find("</reasoning>") {
            let end_pos = start + end + 11; // 11 is length of "</reasoning>"
            result.replace_range(start..end_pos, "");
        } else {
            // If no closing tag found, remove everything from <reasoning> to the end
            result.truncate(start);
        }
    }
    
    // Trim whitespace from the result
    result.trim().to_string()
}

// Helper function to get default settings from store
async fn get_default_settings(app_handle: &tauri::AppHandle) -> Result<(String, String), String> {
    let store = app_handle.store("store.bin").map_err(|e| format!("Failed to get store: {}", e))?;

    let default_provider = store.get("PROVIDER")
        .and_then(|v| v.as_str().map(|s| s.to_string()))
        .unwrap_or_else(|| "ollama".to_string());

    let default_model = store.get("MODEL")
        .and_then(|v| v.as_str().map(|s| s.to_string()))
        .unwrap_or_else(|| "gemma3".to_string());

    Ok((default_provider, default_model))
}

#[derive(serde::Serialize)]
pub struct AllSettings {
    pub provider: Option<String>,
    pub model: Option<String>,
    pub api_key: Option<String>,
    pub shortcut_window_type: Option<String>,
    pub ollama_endpoint: Option<String>,
    pub ollama_thinking: Option<bool>,
    pub prompt_translate: Option<String>,
    pub prompt_correct: Option<String>,
    pub prompt_refine: Option<String>,
    pub preferred_lang: Option<String>,
    pub text_size: Option<String>,
}

#[tauri::command]
pub async fn get_settings(app_handle: tauri::AppHandle) -> Result<AllSettings, String> {
    let store = app_handle.store("store.bin").map_err(|e| format!("Failed to get store: {}", e))?;
    Ok(AllSettings {
        provider: store.get("PROVIDER").and_then(|v| v.as_str().map(|s| s.to_string())),
        model: store.get("MODEL").and_then(|v| v.as_str().map(|s| s.to_string())),
        api_key: store.get("LLM_API_KEY").and_then(|v| v.as_str().map(|s| s.to_string())),
        shortcut_window_type: store.get("SHORTCUT_WINDOW_TYPE").and_then(|v| v.as_str().map(|s| s.to_string())),
        ollama_endpoint: store.get("OLLAMA_ENDPOINT").and_then(|v| v.as_str().map(|s| s.to_string())),
        ollama_thinking: store.get("OLLAMA_THINKING").and_then(|v| v.as_bool()),
        prompt_translate: store.get("PROMPT_TRANSLATE").and_then(|v| v.as_str().map(|s| s.to_string())),
        prompt_correct: store.get("PROMPT_CORRECT").and_then(|v| v.as_str().map(|s| s.to_string())),
        prompt_refine: store.get("PROMPT_REFINE").and_then(|v| v.as_str().map(|s| s.to_string())),
        preferred_lang: store.get("PREFERRED_LANG").and_then(|v| v.as_str().map(|s| s.to_string())),
        text_size: store.get("TEXT_SIZE").and_then(|v| v.as_str().map(|s| s.to_string())),
    })
}

#[tauri::command]
pub async fn open_settings_window(app_handle: tauri::AppHandle) -> Result<(), String> {
    create_or_focus_settings_window(&app_handle).await?;
    Ok(())
}

#[tauri::command]
pub async fn translate(
    app_handle: tauri::AppHandle,
    provider: Option<&str>,
    model: Option<&str>,
    text: &str,
    source_lang: Option<&str>,
    target_lang: Option<&str>,
    prompt: Option<&str>,
) -> Result<String, String> {
    // Get default settings if not provided
    let (default_provider, default_model) = get_default_settings(&app_handle).await?;
    
    let provider = provider.filter(|p| !p.is_empty()).unwrap_or(&default_provider);
    let model = model.filter(|m| !m.is_empty()).unwrap_or(&default_model);
    
    // Format the prompt with the original and target language
    let prompt = prompt.unwrap_or(DEFAULT_TRANSLATION_PROMPT);
    let new_prompt = prompt
        .replace("{original_lang}", source_lang.unwrap_or("English"))
        .replace("{target_lang}", target_lang.unwrap_or("English"));
    
    // Init provider based on the provider name
    let provider_obj = get_provider(app_handle, provider, model);

    let res = provider_obj
        .completion(&format!("<start_of_turn>user\n{}\n\nText to translate: {}\n<end_of_turn>\n<start_of_turn>model", new_prompt, text))
        .await;
    match res {
        Ok(text) => Ok(trim_thinking_blocks(&text)),
        Err(e) => Ok(format!("Error: {}", e)),
    }
}

#[tauri::command]
pub async fn correct(
    app_handle: tauri::AppHandle,
    provider: Option<&str>,
    model: Option<&str>,
    text: &str,
    prompt: Option<&str>,
    source_lang: Option<&str>,
    target_lang: Option<&str>,
) -> Result<String, String> {
    // Get default settings if not provided
    let (default_provider, default_model) = get_default_settings(&app_handle).await?;
    
    let provider = provider.filter(|p| !p.is_empty()).unwrap_or(&default_provider);
    let model = model.filter(|m| !m.is_empty()).unwrap_or(&default_model);
    
    let prompt = prompt.unwrap_or(DEFAULT_CORRECTION_PROMPT);
    // Format the prompt with the original and target language
    let new_prompt = prompt
        .replace("{original_lang}", source_lang.unwrap_or("English"))
        .replace("{target_lang}", target_lang.unwrap_or("English"));
    let provider = get_provider(app_handle, provider, model);
    let res = provider
        .completion(&format!("<start_of_turn>user\n{}\n\nText to correct: {}\n<end_of_turn>\n<start_of_turn>model", new_prompt, text))
        .await;
    match res {
        Ok(text) => Ok(trim_thinking_blocks(&text)),
        Err(e) => Ok(format!("Error: {}", e)),
    }
}

#[tauri::command]
pub async fn refine(
    app_handle: tauri::AppHandle,
    provider: Option<&str>,
    model: Option<&str>,
    text: &str,
    prompt: Option<&str>,
    source_lang: Option<&str>,
    target_lang: Option<&str>,
) -> Result<String, String> {
    // Get default settings if not provided
    let (default_provider, default_model) = get_default_settings(&app_handle).await?;
    
    let provider = provider.filter(|p| !p.is_empty()).unwrap_or(&default_provider);
    let model = model.filter(|m| !m.is_empty()).unwrap_or(&default_model);
    
    let prompt = prompt.unwrap_or(DEFAULT_REFINE_PROMPT);
    // Format the prompt with the original and target language
    let new_prompt = prompt
        .replace("{original_lang}", source_lang.unwrap_or("English"))
        .replace("{target_lang}", target_lang.unwrap_or("English"));
    let provider = get_provider(app_handle, provider, model);
    let res = provider
        .completion(&format!("<start_of_turn>user\n{}\n\nText to refine: {}\n<end_of_turn>\n<start_of_turn>model", new_prompt, text))
        .await;
    match res {
        Ok(text) => Ok(trim_thinking_blocks(&text)),
        Err(e) => Ok(format!("Error: {}", e)),
    }
}

#[tauri::command]
pub async fn get_shortcut_window_type(app_handle: tauri::AppHandle) -> Result<String, String> {
    let store = app_handle.store("store.bin").map_err(|e| format!("Failed to get store: {}", e))?;
    match store.get("SHORTCUT_WINDOW_TYPE") {
        Some(value) => {
            if let Some(window_type) = value.as_str() {
                Ok(window_type.to_string())
            } else {
                Ok("main".to_string()) // Default to main if value is not a string
            }
        },
        None => Ok("main".to_string()), // Default to main if key doesn't exist
    }
}

#[tauri::command]
pub async fn save_settings(
    app_handle: tauri::AppHandle,
    api_key: Option<String>,
    shortcut_window_type: Option<String>,
    provider: Option<String>,
    model: Option<String>,
    ollama_endpoint: Option<String>,
    ollama_thinking: Option<bool>,
    prompt_translate: Option<String>,
    prompt_correct: Option<String>,
    prompt_refine: Option<String>,
    preferred_lang: Option<String>,
    text_size: Option<String>,
) -> Result<(), String> {
    let store = app_handle.store("store.bin").map_err(|e| format!("Failed to get store: {}", e))?;

    if let Some(key) = api_key {
        if !key.is_empty() {
            store.set("LLM_API_KEY", key);
        }
    }

    if let Some(window_type) = shortcut_window_type {
        if !window_type.is_empty() {
            store.set("SHORTCUT_WINDOW_TYPE", window_type);
        }
    }

    if let Some(provider_name) = provider {
        if !provider_name.is_empty() {
            store.set("PROVIDER", provider_name);
        }
    }

    if let Some(model_name) = model {
        if !model_name.is_empty() {
            store.set("MODEL", model_name);
        }
    }

    if let Some(endpoint) = ollama_endpoint {
        if !endpoint.is_empty() {
            store.set("OLLAMA_ENDPOINT", endpoint);
        }
    }

    if let Some(thinking) = ollama_thinking {
        store.set("OLLAMA_THINKING", thinking);
    }

    // Per-mode prompts: save if non-empty, delete key to reset to default
    match prompt_translate {
        Some(p) if !p.is_empty() => { store.set("PROMPT_TRANSLATE", p); }
        Some(_) => { store.delete("PROMPT_TRANSLATE"); }
        None => {}
    }
    match prompt_correct {
        Some(p) if !p.is_empty() => { store.set("PROMPT_CORRECT", p); }
        Some(_) => { store.delete("PROMPT_CORRECT"); }
        None => {}
    }
    match prompt_refine {
        Some(p) if !p.is_empty() => { store.set("PROMPT_REFINE", p); }
        Some(_) => { store.delete("PROMPT_REFINE"); }
        None => {}
    }

    if let Some(lang) = preferred_lang {
        if !lang.is_empty() {
            store.set("PREFERRED_LANG", lang);
        }
    }

    if let Some(size) = text_size {
        if !size.is_empty() {
            store.set("TEXT_SIZE", size);
        }
    }

    store.save().map_err(|e| format!("Failed to save store: {}", e))?;

    Ok(())
}
