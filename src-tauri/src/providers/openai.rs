use crate::providers::base::Provider;
use async_openai::{
    config::OpenAIConfig,
    types::{ChatCompletionRequestUserMessageArgs, CreateChatCompletionRequestArgs},
    Client,
};

pub struct OpenAIProvider {
    client: Client<OpenAIConfig>,
    model: String,
}

impl OpenAIProvider {
    pub fn new(api_key: Option<&str>, model: Option<&str>) -> Self {
        let config = OpenAIConfig::new().with_api_key(api_key.unwrap());

        // Init client
        let client = Client::with_config(config);

        Self {
            client,
            model: model.unwrap_or("gpt-4o").to_string(),
        }
    }
}

impl Provider for OpenAIProvider {
    async fn completion(&self, prompt: &str) -> Result<String, String> {
        println!("Prompt: {}", prompt);
        let client = self.client.clone();
        let request = CreateChatCompletionRequestArgs::default()
            .model(self.model.clone())
            .messages([ChatCompletionRequestUserMessageArgs::default()
                .content(prompt)
                .build()
                .unwrap()
                .into()])
            .build()
            .unwrap();
        let res = client.chat().create(request).await;
        match res {
            Ok(res) => {
                let response = res.choices[0].message.content.clone().unwrap();
                Ok(response)
            }
            Err(err) => Err(err.to_string()),
        }
    }
}
