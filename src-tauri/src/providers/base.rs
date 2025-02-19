use std::future::Future;
use tauri::AppHandle;
use tauri_plugin_store::StoreBuilder;

use super::{ollama::OllamaProvider, openai::OpenAIProvider};

pub trait Provider {
    fn completion(&self, prompt: &str) -> impl Future<Output = Result<String, String>>;
}

pub enum ProviderEnum {
    OllamaProvider(OllamaProvider),
    OpenAIProvider(OpenAIProvider),
}

impl Provider for ProviderEnum {
    async fn completion(&self, prompt: &str) -> Result<String, String> {
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
            let store = StoreBuilder::new(&app_handler, "store.bin").build();
            // Load store
            store.load().unwrap();

            let api_key_value = store.get("OPENAI_API_KEY").unwrap();
            let api_key = api_key_value.as_str().unwrap();
            ProviderEnum::OpenAIProvider(OpenAIProvider::new(Some(api_key), Some(model)))
        }
        _ => panic!("Invalid provider"),
    }
}
