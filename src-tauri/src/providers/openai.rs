use crate::providers::base::Provider;
use async_openai::{config::OpenAIConfig, Client};


pub struct OpenAIProvider {
    client: Client<OpenAIConfig>,
    model: String,
}

impl OpenAIProvider {
    pub fn new(api_key: Option<String>, model: Option<String>) -> Self {
        let config = if let Some(api_key) = api_key {
            OpenAIConfig::new().with_api_key(api_key)
        } else {
            OpenAIConfig::default()
        };

        // Init client
        let client = Client::with_config(config);

        Self {
            client,
            model: model.unwrap_or_else(|| "gpt-3.5-turbo".to_string()),
        }
    }
}

impl Provider for OpenAIProvider {
    async fn completion(&self, prompt: &str) -> String {
        let client = self.client.clone();
        let request = async_openai::types::CreateCompletionRequestArgs::default()
            .model(self.model.clone())
            .prompt(prompt)
            .max_tokens(40_u16)
            .build()
            .unwrap();

        let response = client
            .completions()
            .create(request)
            .await
            .unwrap();

        return response.choices.first().unwrap().text.clone();
    }
}