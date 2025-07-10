use crate::providers::{self, base::get_provider};
use providers::base::Provider;
use tauri_plugin_store::StoreExt;

const DEFAULT_TRANSLATION_PROMPT: &str = "
    You're a good translator.
    Only answer the translated text with the following rules and do not showing your thought:
    - put your answer in <ans></ans> block and do not include anything else because only the text inside <ans></ans> block will be extracted.
    Example 1:
        Original langue: English, Target language: Tiếng Việt
        Input: I'm running
        Your answer: <ans>Tôi đang chạy bộ</ans>
    Example 2:
        Original langue: English, Target language: Español
        Input: I'm running
        Your answer: <ans>Estoy corriendo</ans>
    Please translate the text in original language {original_lang} in the the text block below to {target_lang}:
";
const DEFAULT_CORRECTION_PROMPT: &str = "
    You're a teacher and you're correcting a student's homework.
    Only answer the correct text without explanation,
    put your answer in <ans></ans> block and do not include anything else because only the text inside <ans></ans> block will be extracted.
    Example:
        Input: I running
        Your answer: <ans>I'm running</ans>
    Please check grammar correct it in the text block below and answer in {target_lang} language: ";
const DEFAULT_REFINE_PROMPT: &str = "
    You're a good editor.
    Only answer without explanation,
    put your answer in <ans></ans> block and do not include anything else because only the text inside <ans></ans> block will be extracted.
    Example:
        Input: Hello, how are you?
        Your answer: <ans>What's up!</ans>
    Please rewrite the text in text block below with conversational style in {target_lang} language:
"
;

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
        .completion(format!("{}<text>{}<text>. Your answer: ", new_prompt, text).as_str())
        .await;
    if res.is_err() {
        return Ok(
            "Something wrong with LLM provider API. Please check the config and try again!"
                .to_string(),
        );
    } else {
        let res = res.unwrap();
        let ans = res.split("<ans>").collect::<Vec<&str>>()[1]
            .split("</ans>")
            .collect::<Vec<&str>>()[0]
            .replace("<text>", "")
            .replace("</text>", "");
        return Ok(ans);
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
        .completion(format!("{}<text>{}<text>. Your answer: ", new_prompt, text).as_str())
        .await;
    if res.is_err() {
        return Ok(
            "Something wrong with LLM provider API. Please check the config and try again!"
                .to_string(),
        );
    } else {
        let res = res.unwrap();
        let ans = res.split("<ans>").collect::<Vec<&str>>()[1]
            .split("</ans>")
            .collect::<Vec<&str>>()[0]
            .replace("<text>", "")
            .replace("</text>", "");
        return Ok(ans);
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
        .completion(format!("{}<text>{}<text>. Your answer: ", new_prompt, text).as_str())
        .await;
    // Retrieve the answer from the response and remove all other xml tag in ans
    if res.is_err() {
        return Ok(
            "Something wrong with LLM provider API. Please check the config and try again!"
                .to_string(),
        );
    } else {
        let res = res.unwrap();
        let ans = res.split("<ans>").collect::<Vec<&str>>()[1]
            .split("</ans>")
            .collect::<Vec<&str>>()[0]
            .replace("<text>", "")
            .replace("</text>", "");
        return Ok(ans);
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
