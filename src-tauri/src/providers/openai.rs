use crate::providers::base::Provider;
use async_openai::{
    config::OpenAIConfig, types::{
        ChatCompletionRequestAssistantMessageArgs, ChatCompletionRequestSystemMessageArgs,
        ChatCompletionRequestUserMessageArgs, CreateChatCompletionRequestArgs,
    }, Client
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
            model: model.unwrap_or("gpt-3.5-turbo").to_string(),
        }
    }
}

impl Provider for OpenAIProvider {
    async fn completion(&self, prompt: &str) -> String {
        println!("Prompt: {}", prompt);
        let client = self.client.clone();
        let request = CreateChatCompletionRequestArgs::default()
            .model("gpt-3.5-turbo-0125")
            .messages([
                ChatCompletionRequestUserMessageArgs::default()
                    .content(prompt)
                    .build()
                    .unwrap()
                    .into()
            ])
            .build().unwrap();
        
        let ans = client.chat().create(request).await.unwrap().choices[0].message.clone().content.unwrap();
        println!("OpenAI response: {}", ans);
        ans
    }
}
