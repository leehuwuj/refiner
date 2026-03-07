use crate::providers::base::Provider;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::error::Error as StdError;

#[derive(Serialize)]
struct OllamaRequest<'a> {
    model: &'a str,
    prompt: &'a str,
    stream: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    think: Option<bool>,
}

#[derive(Deserialize)]
struct OllamaResponse {
    response: String,
}

pub struct OllamaProvider {
    client: Client,
    model: String,
    host: String,
    thinking: Option<bool>,
}

impl OllamaProvider {
    pub fn new(endpoint: Option<String>, model: Option<String>, thinking: Option<bool>) -> Self {
        let host = endpoint.unwrap_or_else(|| String::from("http://localhost:11434"));
        let default_model = String::from("gemma3");

        Self {
            client: Client::new(),
            model: model.unwrap_or(default_model),
            host,
            thinking,
        }
    }
}

impl Provider for OllamaProvider {
    async fn completion(&self, text: &str) -> Result<String, String> {
        let url = format!("{}/api/generate", self.host);
        // Only send think:false to disable it; omit the field for models that don't support it
        let think = match self.thinking {
            Some(false) => Some(false),
            _ => None,
        };
        let request_body = OllamaRequest {
            model: &self.model,
            prompt: text,
            stream: false,
            think,
        };

        let res = self.client.post(&url).json(&request_body).send().await;

        match res {
            Ok(res) => {
                if res.status().is_success() {
                    let ollama_response = res.json::<OllamaResponse>().await;
                    match ollama_response {
                        Ok(data) => Ok(data.response),
                        Err(err) => Err(format!("Failed to deserialize Ollama response: {}", err)),
                    }
                } else {
                    let status = res.status();
                    let error_text = res.text().await.unwrap_or_default();
                    // Try to extract the "error" field from Ollama's JSON error response
                    let msg = serde_json::from_str::<serde_json::Value>(&error_text)
                        .ok()
                        .and_then(|v| v["error"].as_str().map(|s| s.to_string()))
                        .unwrap_or(error_text);
                    Err(format!("Ollama error (HTTP {}): {}", status.as_u16(), msg))
                }
            }
            Err(err) => {
                let root = StdError::source(&err)
                    .map(|s| s.to_string())
                    .unwrap_or_else(|| err.to_string());
                let msg = if err.is_connect() {
                    format!("Cannot connect to Ollama at {} — {}", self.host, root)
                } else if err.is_timeout() {
                    format!("Ollama at {} timed out — {}", self.host, root)
                } else {
                    format!("Ollama request failed — {}", root)
                };
                Err(msg)
            }
        }
    }
}
