use crate::providers::base::Provider;
use reqwest::Client;
use serde::{Deserialize, Serialize};

#[derive(Serialize)]
struct OpenAIChatCompletionRequest<'a> {
    model: &'a str,
    messages: Vec<OpenAIMessage<'a>>,
}

#[derive(Serialize)]
struct OpenAIMessage<'a> {
    role: &'a str,
    content: &'a str,
}

#[derive(Deserialize, Debug)]
struct OpenAIChatCompletionResponse {
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

pub struct OpenAIProvider {
    client: Client,
    api_key: String,
    model: String,
}

impl OpenAIProvider {
    pub fn new(api_key: Option<&str>, model: Option<&str>) -> Self {
        Self {
            client: Client::new(),
            api_key: api_key.unwrap_or("").to_string(),
            model: model.unwrap_or("gpt-4.1-nano").to_string(),
        }
    }
}

impl Provider for OpenAIProvider {
    async fn completion(&self, prompt: &str) -> Result<String, String> {
        let url = "https://api.openai.com/v1/chat/completions";
        
        let messages = vec![OpenAIMessage {
            role: "user",
            content: prompt,
        }];

        let body = OpenAIChatCompletionRequest {
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
                    let response_body = res.json::<OpenAIChatCompletionResponse>().await;
                    match response_body {
                        Ok(data) => {
                            if let Some(choice) = data.choices.get(0) {
                                Ok(choice.message.content.clone())
                            } else {
                                Err("No content in OpenAI response".to_string())
                            }
                        }
                        Err(err) => Err(format!("Failed to deserialize OpenAI response: {}", err)),
                    }
                } else {
                    let error_text = res.text().await.unwrap_or_else(|e| e.to_string());
                    Err(format!("OpenAI API request failed: {}", error_text))
                }
            }
            Err(err) => Err(err.to_string()),
        }
    }
}
