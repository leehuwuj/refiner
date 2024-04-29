use crate::providers::{self, base::get_provider};
use providers::base::Provider;

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
    Please check grammar correct it in the text block below and answer in {original_lang} language: ";
const DEFAULT_REFINE_PROMPT: &str = "
    You're a good editor. 
    Only answer the translated text without explanation, 
    put your answer in <ans></ans> block and do not include anything else because only the text inside <ans></ans> block will be extracted.
    Example:
        Input: Hello, how are you?
        Your answer: <ans>What's up!</ans>
    Please refine the text in text block below with conversational style in {original_lang} language: 
"
;

#[tauri::command]
pub async fn translate(
    app_handle: tauri::AppHandle,
    provider: &str,
    model: &str,
    text: &str,
    source_lang: Option<&str>,
    target_lang: Option<&str>,
    prompt: Option<&str>,
) -> Result<String, String> {
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
    let ans = res.split("<ans>").collect::<Vec<&str>>()[1]
        .split("</ans>")
        .collect::<Vec<&str>>()[0]
        .replace("<text>", "")
        .replace("</text>", "");
    return Ok(ans);
}

#[tauri::command]
pub async fn correct(
    app_handle: tauri::AppHandle,
    provider: &str,
    model: &str,
    text: &str,
    prompt: Option<&str>,
    source_lang: Option<&str>,
    target_lang: Option<&str>,
) -> Result<String, String> {
    let prompt = prompt.unwrap_or(DEFAULT_CORRECTION_PROMPT);
    // Format the prompt with the original and target language
    let new_prompt = prompt
        .replace("{original_lang}", source_lang.unwrap_or("English"))
        .replace("{target_lang}", target_lang.unwrap_or("English"));
    let provider = get_provider(app_handle, provider, model);
    let res = provider
        .completion(format!("{}<text>{}<text>. Your answer: ", new_prompt, text).as_str())
        .await;
    // Retrieve the answer from the response
    let ans = res.split("<ans>").collect::<Vec<&str>>()[1]
        .split("</ans>")
        .collect::<Vec<&str>>()[0]
        .replace("<text>", "")
        .replace("</text>", "");
    return Ok(ans);
}

#[tauri::command]
pub async fn refine(
    app_handle: tauri::AppHandle,
    provider: &str,
    model: &str,
    text: &str,
    prompt: Option<&str>,
    source_lang: Option<&str>,
    target_lang: Option<&str>,
) -> Result<String, String> {
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
    let ans = res.split("<ans>").collect::<Vec<&str>>()[1]
        .split("</ans>")
        .collect::<Vec<&str>>()[0]
        .replace("<text>", "")
        .replace("</text>", "");
    return Ok(ans);
}

// Save API key to environment variable
#[tauri::command]
pub fn save_api_key(provider: &str, api_key: &str) -> Result<(), String> {
    std::env::set_var(format!("{}_API_KEY", "OPENAI"), api_key);
    Ok(())
}
