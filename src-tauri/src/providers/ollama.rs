use crate::providers::base::Provider;
use reqwest::Client;
use serde::{Deserialize, Serialize};

#[derive(Serialize)]
struct OllamaRequest<'a> {
    model: &'a str,
    prompt: &'a str,
    stream: bool,
}

#[derive(Deserialize)]
struct OllamaResponse {
    response: String,
}

pub struct OllamaProvider {
    client: Client,
    model: String,
    host: String,
}

impl OllamaProvider {
    pub fn new(host: Option<String>, port: Option<u16>, model: Option<String>) -> Self {
        let default_host = String::from("http://localhost");
        let default_port: u16 = 11434;
        let host = format!("{}:{}", host.unwrap_or(default_host), port.unwrap_or(default_port));
        let default_model = String::from("gemma3");

        Self {
            client: Client::new(),
            model: model.unwrap_or(default_model),
            host,
        }
    }
}

impl Provider for OllamaProvider {
    async fn completion(&self, text: &str) -> Result<String, String> {
        let url = format!("{}/api/generate", self.host);
        let request_body = OllamaRequest {
            model: &self.model,
            prompt: text,
            stream: false,
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
                    let error_text = res.text().await.unwrap_or_else(|e| e.to_string());
                    Err(format!("Ollama API request failed: {}", error_text))
                }
            }
            Err(err) => Err(err.to_string()),
        }
    }
}
