use crate::providers::{self, base::get_provider};
use providers::base::Provider;
use tauri_plugin_store::StoreExt;

const DEFAULT_TRANSLATION_PROMPT: &str =
    "You are an expert translator. Translate the following text from {original_lang} to {target_lang}. Provide only the translated text, without any additional explanations or formatting.";

const DEFAULT_CORRECTION_PROMPT: &str =
    "You are an expert in grammar. Correct the grammar of the following text in {target_lang}. Provide only the corrected text, without any additional explanations or formatting.";

const DEFAULT_REFINE_PROMPT: &str =
    "You are an expert editor. Rewrite the following text in a more conversational style, in {target_lang}. Provide only the rewritten text, without any additional explanations or formatting.";

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
    if res.is_err() {
        return Ok(
            "Something wrong with LLM provider API. Please check the config and try again!"
                .to_string(),
        );
    } else {
        let res = res.unwrap();
        return Ok(res);
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
    if res.is_err() {
        return Ok(
            "Something wrong with LLM provider API. Please check the config and try again!"
                .to_string(),
        );
    } else {
        let res = res.unwrap();
        return Ok(res);
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
    // Retrieve the answer from the response and remove all other xml tag in ans
    if res.is_err() {
        return Ok(
            "Something wrong with LLM provider API. Please check the config and try again!"
                .to_string(),
        );
    } else {
        let res = res.unwrap();
        return Ok(res);
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
    prompt: Option<String>
) -> Result<(), String> {
    let store = app_handle.store("store.bin").map_err(|e| format!("Failed to get store: {}", e))?;
    
    if let Some(key) = api_key {
        if !key.is_empty() {
            store.set("OPENAI_API_KEY", key);
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
    
    if let Some(prompt_data) = prompt {
        if !prompt_data.is_empty() && prompt_data != "undefined" && prompt_data != "null" {
            store.set("PROMPT", prompt_data);
        }
    }
    
    store.save().map_err(|e| format!("Failed to save store: {}", e))?;
    
    Ok(())
}
