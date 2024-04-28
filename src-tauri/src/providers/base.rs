use super::{ollama::OllamaProvider, openai::OpenAIProvider};

pub trait Provider {
    fn completion(&self, prompt: &str) -> impl std::future::Future<Output = String> + Send;
}

pub enum ProviderEnum {
    OllamaProvider(OllamaProvider),
    OpenAIProvider(OpenAIProvider),
}

impl Provider for ProviderEnum {
    async fn completion(&self, prompt: &str) -> String {
        match self {
            ProviderEnum::OllamaProvider(provider) => provider.completion(prompt).await,
            ProviderEnum::OpenAIProvider(provider) => provider.completion(prompt).await,
        }
    }
}

pub fn get_provider(provider: &str, model: &str) -> ProviderEnum {
    match provider {
        "ollama" => ProviderEnum::OllamaProvider(OllamaProvider::new(None, None, Some(model.to_string()))),
        "openai" => ProviderEnum::OpenAIProvider(OpenAIProvider::new(None, Some(model.to_string()))),
        _ => panic!("Invalid provider"),
    }
}