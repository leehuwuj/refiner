use crate::providers::base::Provider;
use reqwest::Client;
use serde::{Deserialize, Serialize};

#[derive(Serialize)]
struct GroqChatCompletionRequest<'a> {
    model: &'a str,
    messages: Vec<GroqMessage<'a>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    reasoning_effort: Option<&'a str>,
}

#[derive(Serialize)]
struct GroqMessage<'a> {
    role: &'a str,
    content: &'a str,
}

#[derive(Deserialize, Debug)]
struct GroqChatCompletionResponse {
    choices: Vec<Choice>,
}

#[derive(Deserialize, Debug)]
struct Choice {
    message: Message,
}

#[derive(Deserialize, Debug)]
struct Message {
    content: String,
}

pub struct GroqProvider {
    client: Client,
    api_key: String,
    model: String,
    base_url: String,
    thinking: Option<bool>,
}

impl GroqProvider {
    pub fn new(api_key: Option<&str>, model: Option<&str>, base_url: Option<String>, thinking: Option<bool>) -> Self {
        Self {
            client: Client::new(),
            api_key: api_key.unwrap_or("").to_string(),
            model: model.unwrap_or("llama-3.1-8b-instant").to_string(),
            base_url: base_url.unwrap_or_else(|| "https://api.groq.com/openai/v1".to_string()),
            thinking,
        }
    }
}

impl Provider for GroqProvider {
    async fn completion(&self, prompt: &str) -> Result<String, String> {
        let url = format!("{}/chat/completions", self.base_url);
        
        let messages = vec![GroqMessage {
            role: "user",
            content: prompt,
        }];

        // "none" disables reasoning for models that support it (e.g. Qwen3);
        // omitting the field uses the model's default (reasoning enabled)
        let reasoning_effort = match self.thinking {
            Some(false) => Some("none"),
            _ => None,
        };

        let body = GroqChatCompletionRequest {
            model: &self.model,
            messages,
            reasoning_effort,
        };

        let res = self
            .client
            .post(&url)
            .bearer_auth(&self.api_key)
            .json(&body)
            .send()
            .await;

        match res {
            Ok(res) => {
                if res.status().is_success() {
                    let response_body = res.json::<GroqChatCompletionResponse>().await;
                    match response_body {
                        Ok(data) => {
                            if let Some(choice) = data.choices.get(0) {
                                Ok(choice.message.content.clone())
                            } else {
                                Err("No content in Groq response".to_string())
                            }
                        }
                        Err(err) => Err(format!("Failed to deserialize Groq response: {}", err)),
                    }
                } else {
                    let error_text = res.text().await.unwrap_or_else(|e| e.to_string());
                    Err(format!("Groq API request failed: {}", error_text))
                }
            }
            Err(err) => Err(err.to_string()),
        }
    }
} 