use crate::providers::base::Provider;
use reqwest::Client;
use serde::{Deserialize, Serialize};

#[derive(Serialize)]
struct GroqChatCompletionRequest<'a> {
    model: &'a str,
    messages: Vec<GroqMessage<'a>>,
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
}

impl GroqProvider {
    pub fn new(api_key: Option<&str>, model: Option<&str>) -> Self {
        Self {
            client: Client::new(),
            api_key: api_key.unwrap_or("").to_string(),
            model: model.unwrap_or("llama-3.1-8b-instant").to_string(),
        }
    }
}

impl Provider for GroqProvider {
    async fn completion(&self, prompt: &str) -> Result<String, String> {
        let url = "https://api.groq.com/openai/v1/chat/completions";
        
        let messages = vec![GroqMessage {
            role: "user",
            content: prompt,
        }];

        let body = GroqChatCompletionRequest {
            model: &self.model,
            messages,
        };

        let res = self
            .client
            .post(url)
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