use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::json;

#[derive(Debug, Serialize, Deserialize)]
pub struct TranslateRequest {
    pub text: String,
    pub from_lang: String,
    pub to_lang: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TranslateResponse {
    pub translated_text: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct OllamaResponse {
    response: String,
}

pub struct OllamaClient {
    client: Client,
    base_url: String,
}

impl OllamaClient {
    pub fn new() -> Self {
        Self {
            client: Client::new(),
            base_url: "http://localhost:11434".to_string(),
        }
    }

    pub async fn translate(&self, request: TranslateRequest) -> Result<TranslateResponse, String> {
        let prompt = format!(
            "Translate the following text from {} to {}. Only provide the translation without any explanations or additional text:\n\n{}",
            request.from_lang, request.to_lang, request.text
        );

        let body = json!({
            "model": "qwen2.5:3b",
            "prompt": prompt,
            "stream": false,
            "options": {
                "temperature": 0.3,
                "top_p": 0.9
            }
        });

        let response = self.client
            .post(&format!("{}/api/generate", self.base_url))
            .json(&body)
            .send()
            .await
            .map_err(|e| format!("Failed to send request: {}", e))?;

        if !response.status().is_success() {
            return Err(format!("API error: {}", response.status()));
        }

        let ollama_response: OllamaResponse = response
            .json()
            .await
            .map_err(|e| format!("Failed to parse response: {}", e))?;

        Ok(TranslateResponse {
            translated_text: ollama_response.response.trim().to_string(),
        })
    }

    pub async fn check_health(&self) -> Result<bool, String> {
        let response = self.client
            .get(&format!("{}/api/tags", self.base_url))
            .send()
            .await
            .map_err(|e| format!("Failed to check health: {}", e))?;

        Ok(response.status().is_success())
    }
}