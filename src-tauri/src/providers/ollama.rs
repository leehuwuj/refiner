use crate::providers::base::Provider;
use ollama_rs::{generation::completion::request::GenerationRequest, Ollama};

pub struct OllamaProvider {
    client: Ollama,
    model: String,
}

impl OllamaProvider {
    pub fn new(host: Option<String>, port: Option<u16>, model: Option<String>) -> Self {
        let default_host = String::from("http://localhost");
        let default_port: u16 = 11434;
        let default_model = String::from("llama3");

        // Init client
        let client = Ollama::new(host.unwrap_or(default_host), port.unwrap_or(default_port));

        Self {
            client,
            model: model.unwrap_or(default_model),
        }
    }
}

impl Provider for OllamaProvider {
    async fn completion(&self, text: &str) -> Result<String, String> {
        let request = GenerationRequest::new(self.model.clone(), text.to_string());
        let res = self.client.generate(request).await;
        match res {
            Ok(res) => Ok(res.response),
            Err(err) => Err(err.to_string()),
        }
    }
}
