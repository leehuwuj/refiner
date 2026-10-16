//! Blocking LLM provider clients. Call from a background task.
//!
//! Request shapes (and the "thinking" switches) mirror the Tauri build:
//! - Ollama: `POST {host}/api/generate`, `think: false` to disable reasoning.
//! - OpenAI / OpenAI-compatible: `POST {base}/chat/completions`.
//! - Groq: OpenAI-compatible, `reasoning_effort: "none"` to disable reasoning.
//! - Gemini: `POST {base}/models/{model}:generateContent?key=…`.

use std::time::Duration;

use serde_json::{Value, json};

use crate::settings::Settings;

#[derive(Clone, Debug)]
pub struct LlmConfig {
    pub provider: String,
    pub model: String,
    pub api_key: String,
    /// Empty means the provider default.
    pub base_url: String,
    pub thinking: bool,
}

impl LlmConfig {
    pub fn from_settings(s: &Settings) -> Self {
        Self {
            provider: s.provider.clone(),
            model: s.model.clone(),
            api_key: s.api_key.clone(),
            base_url: s.model_url.trim().trim_end_matches('/').to_string(),
            thinking: s.thinking,
        }
    }

    fn base_or(&self, default: &str) -> String {
        if self.base_url.is_empty() {
            default.to_string()
        } else {
            self.base_url.clone()
        }
    }

    fn model_or(&self, default: &str) -> String {
        let m = self.model.trim();
        if m.is_empty() {
            default.to_string()
        } else {
            m.to_string()
        }
    }
}

fn agent() -> ureq::Agent {
    ureq::Agent::config_builder()
        // Read error bodies ourselves so the user sees the provider's message.
        .http_status_as_error(false)
        // Long analysis prompts on local models can take a few minutes.
        .timeout_global(Some(Duration::from_secs(600)))
        .build()
        .into()
}

/// Sends `prompt` to the configured provider and returns the raw completion.
pub fn complete(cfg: &LlmConfig, prompt: &str) -> Result<String, String> {
    match cfg.provider.as_str() {
        "ollama" => ollama(cfg, prompt),
        "openai" => openai(cfg, prompt),
        "gemini" => gemini(cfg, prompt),
        "groq" => groq(cfg, prompt),
        other => Err(format!("Unknown provider: {other}")),
    }
}

/// POSTs JSON; returns the parsed body on 2xx, or `Err(status, body)`.
fn post_json(
    url: &str,
    bearer: Option<&str>,
    body: &Value,
) -> Result<Value, (Option<u16>, String)> {
    let mut req = agent().post(url).header("Content-Type", "application/json");
    if let Some(key) = bearer {
        req = req.header("Authorization", format!("Bearer {key}"));
    }
    let mut res = req.send_json(body).map_err(|e| (None, e.to_string()))?;
    let status = res.status().as_u16();
    let text = res
        .body_mut()
        .read_to_string()
        .map_err(|e| (Some(status), e.to_string()))?;
    if !(200..300).contains(&status) {
        return Err((Some(status), text));
    }
    serde_json::from_str(&text).map_err(|e| (Some(status), format!("invalid JSON: {e}")))
}

/// Extracts `error.message` / `error` from a JSON error body, else the raw text.
fn error_message(body: &str) -> String {
    serde_json::from_str::<Value>(body)
        .ok()
        .and_then(|v| {
            v["error"]["message"]
                .as_str()
                .or_else(|| v["error"].as_str())
                .map(str::to_string)
        })
        .unwrap_or_else(|| body.trim().to_string())
}

fn ollama(cfg: &LlmConfig, prompt: &str) -> Result<String, String> {
    let host = cfg.base_or("http://localhost:11434");
    let url = format!("{host}/api/generate");
    let mut body = json!({
        "model": cfg.model_or("gemma3"),
        "prompt": prompt,
        "stream": false,
    });
    // Only send think:false; models without reasoning reject the field otherwise.
    if !cfg.thinking {
        body["think"] = json!(false);
    }
    match post_json(&url, None, &body) {
        Ok(v) => v["response"]
            .as_str()
            .map(str::to_string)
            .ok_or_else(|| "Failed to parse Ollama response".to_string()),
        Err((Some(status), body)) => Err(format!(
            "Ollama error (HTTP {status}): {}",
            error_message(&body)
        )),
        Err((None, err)) => Err(format!("Cannot connect to Ollama at {host} — {err}")),
    }
}

fn chat_content(v: &Value) -> Option<String> {
    v["choices"][0]["message"]["content"]
        .as_str()
        .map(str::to_string)
}

fn openai(cfg: &LlmConfig, prompt: &str) -> Result<String, String> {
    let base = cfg.base_or("https://api.openai.com/v1");
    let url = format!("{base}/chat/completions");
    let mut body = json!({
        "model": cfg.model_or("gpt-4.1-nano"),
        "messages": [{ "role": "user", "content": prompt }],
    });
    if !cfg.thinking {
        if base.contains("api.openai.com") {
            body["reasoning"] = json!({ "effort": "none" });
        } else {
            // llama.cpp and other OpenAI-compatible servers.
            body["chat_template_kwargs"] = json!({ "enable_thinking": false });
        }
    }
    match post_json(&url, Some(&cfg.api_key), &body) {
        Ok(v) => chat_content(&v).ok_or_else(|| "No content in OpenAI response".to_string()),
        Err((_, body)) => Err(format!("OpenAI API request failed: {}", error_message(&body))),
    }
}

fn groq(cfg: &LlmConfig, prompt: &str) -> Result<String, String> {
    let base = cfg.base_or("https://api.groq.com/openai/v1");
    let url = format!("{base}/chat/completions");
    let mut body = json!({
        "model": cfg.model_or("llama-3.1-8b-instant"),
        "messages": [{ "role": "user", "content": prompt }],
    });
    if !cfg.thinking {
        body["reasoning_effort"] = json!("none");
    }
    match post_json(&url, Some(&cfg.api_key), &body) {
        Ok(v) => chat_content(&v).ok_or_else(|| "No content in Groq response".to_string()),
        Err((_, body)) => Err(format!("Groq API request failed: {}", error_message(&body))),
    }
}

fn gemini(cfg: &LlmConfig, prompt: &str) -> Result<String, String> {
    let base = cfg.base_or("https://generativelanguage.googleapis.com/v1beta");
    let url = format!(
        "{base}/models/{}:generateContent?key={}",
        cfg.model_or("gemini-2.0-flash-lite"),
        cfg.api_key
    );
    let mut body = json!({ "contents": [{ "parts": [{ "text": prompt }] }] });
    // thinkingBudget 0 disables thinking; omitting it lets the model decide.
    if !cfg.thinking {
        body["generationConfig"] = json!({ "thinkingConfig": { "thinkingBudget": 0 } });
    }
    match post_json(&url, None, &body) {
        Ok(v) => v["candidates"][0]["content"]["parts"][0]["text"]
            .as_str()
            .map(str::to_string)
            .ok_or_else(|| "Failed to parse response from Gemini API".to_string()),
        Err((_, body)) => Err(format!("Gemini API request failed: {}", error_message(&body))),
    }
}

/// Removes `<think>`, `<thinking>` and `<reasoning>` blocks from a completion.
/// An unterminated block drops everything after its opening tag.
pub fn trim_thinking_blocks(response: &str) -> String {
    let mut result = response.to_string();
    for tag in ["think", "thinking", "reasoning"] {
        let open = format!("<{tag}>");
        let close = format!("</{tag}>");
        while let Some(start) = result.find(&open) {
            match result[start..].find(&close) {
                Some(end) => result.replace_range(start..start + end + close.len(), ""),
                None => result.truncate(start),
            }
        }
    }
    result.trim().to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn trims_thinking_blocks() {
        assert_eq!(trim_thinking_blocks("<think>hmm</think>\nHello"), "Hello");
        assert_eq!(
            trim_thinking_blocks("A<thinking>x</thinking>B<reasoning>y</reasoning>C"),
            "ABC"
        );
        assert_eq!(trim_thinking_blocks("Answer <think>never closed"), "Answer");
    }

    #[test]
    fn extracts_error_messages() {
        assert_eq!(error_message(r#"{"error":{"message":"bad key"}}"#), "bad key");
        assert_eq!(error_message(r#"{"error":"model not found"}"#), "model not found");
        assert_eq!(error_message("plain"), "plain");
    }
}

/// Live check against a real server:
/// `REFINER_LIVE_URL=http://localhost:11434 REFINER_LIVE_MODEL=gemma3 cargo test live -- --ignored`
#[cfg(test)]
mod live {
    use super::*;

    #[test]
    #[ignore]
    fn live_ollama_translation() {
        let cfg = LlmConfig {
            provider: "ollama".into(),
            model: std::env::var("REFINER_LIVE_MODEL").unwrap_or_else(|_| "gemma3".into()),
            api_key: String::new(),
            base_url: std::env::var("REFINER_LIVE_URL").unwrap_or_default(),
            thinking: false,
        };
        let mut settings = Settings::default();
        settings.provider = cfg.provider.clone();
        settings.model = cfg.model.clone();
        settings.model_url = cfg.base_url.clone();
        settings.thinking = false;
        let req = crate::refine::Request {
            mode: crate::refine::Mode::Translate,
            text: "Good morning, how are you?".into(),
            source_lang: "English".into(),
            target_lang: "Tiếng Việt".into(),
        };
        let out = crate::refine::run(&settings, &req).expect("request failed");
        eprintln!("live result: {out}");
        assert!(!out.is_empty());
    }
}
