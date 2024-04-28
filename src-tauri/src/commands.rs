use providers::{base::Provider, ollama::OllamaProvider};
use crate::providers;

const DEFAULT_TRANSLATION_PROMPT: &str = "
    You're a good translator. 
    Only answer the translated text with the following rules and do not showing your thought: 
    - put your answer in <ans></ans> block and do not include anything else because only the text inside <ans></ans> block will be extracted.
    Example:
        Input: I'm running
        Your answer: <ans>Tôi đang chạy bộ</ans>
    Please translate the text in the the text block below to Vietnamese: 
";
const DEFAULT_CORRECTION_PROMPT: &str = "
    You're a teacher and you're correcting a student's homework. 
    Only answer the correct text without explanation, 
    put your answer in <ans></ans> block and do not include anything else because only the text inside <ans></ans> block will be extracted.
    Example:
        Input: I running
        Your answer: <ans>I'm running</ans>
    Please check grammar correct it in the text block below: ";
const DEFAULT_REFINE_PROMPT: &str = "
    You're a good editor. 
    Only answer the translated text without explanation, 
    put your answer in <ans></ans> block and do not include anything else because only the text inside <ans></ans> block will be extracted.
    Example:
        Input: Hello, how are you?
        Your answer: <ans>What's up!</ans>
    Please refine the text in text block below with conversational style: 
"
;


#[tauri::command]
pub async fn translate(provider: &str, model: &str, text: &str, prompt: Option<&str>) -> Result<String, String> {
    let prompt = prompt.unwrap_or(DEFAULT_TRANSLATION_PROMPT);
    let provider = match provider {
        "ollama" => OllamaProvider::new(None, None, Some(model.to_string())),
        _ => panic!("Invalid provider"),
    };
    let res = provider.completion(format!("{}<text>{}<text>. Your answer: ", prompt, text).as_str()).await;
    // Retrieve the answer from the response
    let ans = res.split("<ans>").collect::<Vec<&str>>()[1].split("</ans>").collect::<Vec<&str>>()[0].replace("<text>", "").replace("</text>", "");
    return Ok(ans);
}

#[tauri::command]
pub async fn correct(provider: &str, model: &str, text: &str, prompt: Option<&str>) -> Result<String, String> {
    let prompt = prompt.unwrap_or(DEFAULT_CORRECTION_PROMPT);
    let provider = match provider {
        "ollama" => OllamaProvider::new(None, None, Some(model.to_string())),
        _ => panic!("Invalid provider"),
    };
    let res = provider.completion(format!("{}<text>{}<text>. Your answer: ", prompt, text).as_str()).await;
    // Retrieve the answer from the response
    let ans = res.split("<ans>").collect::<Vec<&str>>()[1].split("</ans>").collect::<Vec<&str>>()[0].replace("<text>", "").replace("</text>", "");
    return Ok(ans);
}

#[tauri::command]
pub async fn refine(provider: &str, model: &str, text: &str, prompt: Option<&str>) -> Result<String, String> {
    let prompt = prompt.unwrap_or(DEFAULT_REFINE_PROMPT);
    let provider = match provider {
        "ollama" => OllamaProvider::new(None, None, Some(model.to_string())),
        _ => panic!("Invalid provider"),
    };
    let res = provider.completion(format!("{}<text>{}<text>. Your answer: ", prompt, text).as_str()).await;
    // Retrieve the answer from the response and remove all other xml tag in ans
    let ans = res.split("<ans>").collect::<Vec<&str>>()[1].split("</ans>").collect::<Vec<&str>>()[0].replace("<text>", "").replace("</text>", "");
    return Ok(ans);
}