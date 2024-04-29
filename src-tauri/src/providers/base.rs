use tauri::AppHandle;
use tauri_plugin_store::StoreBuilder;

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

pub fn get_provider(app_handler: AppHandle, provider: &str, model: &str) -> ProviderEnum {
    match provider {
        "ollama" => {
            ProviderEnum::OllamaProvider(OllamaProvider::new(None, None, Some(model.to_string())))
        }
        "openai" => {
            let mut store = StoreBuilder::new("store.bin").build(app_handler.clone());
            store.load().unwrap();
            let api_key = store.get("OPENAI_API_KEY").unwrap().as_str();
            ProviderEnum::OpenAIProvider(OpenAIProvider::new(api_key, Some(model)))
        }
        _ => panic!("Invalid provider"),
    }
}
