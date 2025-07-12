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
}

impl GeminiProvider {
    pub fn new(api_key: Option<&str>, model: Option<&str>) -> Self {
        Self {
            client: Client::new(),
            api_key: api_key.unwrap_or("").to_string(),
            model: model.unwrap_or("gemini-2.0-flash-lite").to_string(),
        }
    }
}

impl Provider for GeminiProvider {
    async fn completion(&self, prompt: &str) -> Result<String, String> {
        let url = format!(
            "https://generativelanguage.googleapis.com/v1beta/models/{}:generateContent?key={}",
            self.model, self.api_key
        );

        let body = serde_json::json!({
            "contents": [{
                "parts":[{
                    "text": prompt
                }]
            }]
        });

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