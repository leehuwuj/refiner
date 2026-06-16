use crate::providers::base::Provider;
use reqwest::Client;
use serde::Deserialize;

#[derive(Deserialize, Debug)]
struct GeminiResponse {
    candidates: Vec<Candidate>,
}

#[derive(Deserialize, Debug)]
struct Candidate {
    content: Content,
}

#[derive(Deserialize, Debug)]
struct Content {
    parts: Vec<Part>,
}

#[derive(Deserialize, Debug)]
struct Part {
    text: String,
}

pub struct GeminiProvider {
    client: Client,
    api_key: String,
    model: String,
    base_url: String,
    thinking: Option<bool>,
}

impl GeminiProvider {
    pub fn new(api_key: Option<&str>, model: Option<&str>, base_url: Option<String>, thinking: Option<bool>) -> Self {
        Self {
            client: Client::new(),
            api_key: api_key.unwrap_or("").to_string(),
            model: model.unwrap_or("gemini-2.0-flash-lite").to_string(),
            base_url: base_url.unwrap_or_else(|| "https://generativelanguage.googleapis.com/v1beta".to_string()),
            thinking,
        }
    }
}

impl Provider for GeminiProvider {
    async fn completion(&self, prompt: &str) -> Result<String, String> {
        let url = format!(
            "{}/models/{}:generateContent?key={}",
            self.base_url, self.model, self.api_key
        );

        let mut body = serde_json::json!({
            "contents": [{
                "parts":[{
                    "text": prompt
                }]
            }]
        });

        // thinkingBudget: 0 disables thinking; omitting lets the model decide
        if self.thinking == Some(false) {
            body["generationConfig"] = serde_json::json!({
                "thinkingConfig": { "thinkingBudget": 0 }
            });
        }

        let body = body;

        let res = self.client.post(&url).json(&body).send().await;

        match res {
            Ok(res) => {
                if res.status().is_success() {
                    let gemini_response = res.json::<GeminiResponse>().await;
                    match gemini_response {
                        Ok(data) => {
                            if let Some(candidate) = data.candidates.get(0) {
                                if let Some(part) = candidate.content.parts.get(0) {
                                    return Ok(part.text.clone());
                                }
                            }
                            Err("Failed to parse response from Gemini API".to_string())
                        }
                        Err(err) => Err(format!("Failed to deserialize Gemini response: {}", err)),
                    }
                } else {
                    let error_text = res.text().await.unwrap_or_else(|e| e.to_string());
                    Err(format!("Gemini API request failed: {}", error_text))
                }
            }
            Err(err) => Err(err.to_string()),
        }
    }
} 