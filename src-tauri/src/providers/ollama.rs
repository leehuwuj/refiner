use crate::providers::base::Provider;
use ollama_rs::{generation::completion::request::GenerationRequest, Ollama};

pub(crate) struct OllamaProvider {
    client: Ollama,
    model: String,
}

impl OllamaProvider {
    pub fn new(host: Option<String>, port: Option<u16>, model: Option<String>) -> Self {
        let default_host = String::from("http://localhost");
        let default_port: u16 = 11434;
        let default_model = String::from("gpt-3.5-turbo");

        // Init client
        let client = Ollama::new(host.unwrap_or(default_host), port.unwrap_or(default_port));

        Self {
            client,
            model: model.unwrap_or(default_model),
        }
    }

    pub fn inject_translation_prompt(&self, prompt: &str) -> String {
        let translator_prompt =
            "You're a good translator. Please translate the text above to Vietnamese, only answer the translated text without explanation: ";
        return format!("{}{}", translator_prompt, prompt);
    }
}

impl Provider for OllamaProvider {
    async fn completion(&self, prompt: &str) -> String {
        let request =
            GenerationRequest::new(self.model.clone(), self.inject_translation_prompt(prompt));
        let res = self.client.generate(request).await.unwrap();
        return res.response;
    }
}
