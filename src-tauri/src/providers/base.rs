use std::future::Future;
use tauri::AppHandle;
use tauri_plugin_store::StoreBuilder;

use super::{gemini::GeminiProvider, groq::GroqProvider, ollama::OllamaProvider, openai::OpenAIProvider};

pub trait Provider {
    fn completion(&self, prompt: &str) -> impl Future<Output = Result<String, String>>;
}

pub enum ProviderEnum {
    OllamaProvider(OllamaProvider),
    OpenAIProvider(OpenAIProvider),
    GeminiProvider(GeminiProvider),
    GroqProvider(GroqProvider),
}

impl Provider for ProviderEnum {
    async fn completion(&self, prompt: &str) -> Result<String, String> {
        match self {
            ProviderEnum::OllamaProvider(provider) => provider.completion(prompt).await,
            ProviderEnum::OpenAIProvider(provider) => provider.completion(prompt).await,
            ProviderEnum::GeminiProvider(provider) => provider.completion(prompt).await,
            ProviderEnum::GroqProvider(provider) => provider.completion(prompt).await,
        }
    }
}

pub fn get_provider(app_handler: AppHandle, provider: &str, model: &str) -> ProviderEnum {
    let store = StoreBuilder::new(&app_handler, "store.bin")
        .build()
        .expect("Failed to build store");

    let model_url = store
        .get("MODEL_URL")
        .or_else(|| store.get("OLLAMA_ENDPOINT"))
        .and_then(|v| v.as_str().map(|s| s.to_string()));

    let thinking = store
        .get("THINKING")
        .or_else(|| store.get("OLLAMA_THINKING"))
        .and_then(|v| v.as_bool());

    match provider {
        "ollama" => {
            ProviderEnum::OllamaProvider(OllamaProvider::new(model_url, Some(model.to_string()), thinking))
        }
        "openai" => {
            let api_key_value = store
                .get("LLM_API_KEY")
                .expect("Failed to get API key from store");
            let api_key = api_key_value
                .as_str()
                .expect("API key is not a valid string");
            ProviderEnum::OpenAIProvider(OpenAIProvider::new(Some(api_key), Some(model), model_url, thinking))
        }
        "gemini" => {
            let api_key_value = store
                .get("LLM_API_KEY")
                .expect("Failed to get API key from store");
            let api_key = api_key_value
                .as_str()
                .expect("API key is not a valid string");
            ProviderEnum::GeminiProvider(GeminiProvider::new(Some(api_key), Some(model), model_url, thinking))
        }
        "groq" => {
            let api_key_value = store
                .get("LLM_API_KEY")
                .expect("Failed to get API key from store");
            let api_key = api_key_value
                .as_str()
                .expect("API key is not a valid string");
            ProviderEnum::GroqProvider(GroqProvider::new(Some(api_key), Some(model), model_url, thinking))
        }
        _ => panic!("Invalid provider"),
    }
}
