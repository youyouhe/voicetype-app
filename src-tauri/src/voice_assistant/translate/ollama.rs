use crate::voice_assistant::{TranslateProcessor, VoiceError};
use serde_json::{json, Value};
use std::time::Duration;

pub struct OllamaTranslateProcessor {
    client: reqwest::Client,
    url: String,
    model: String,
}

impl OllamaTranslateProcessor {
    pub fn new() -> Result<Self, VoiceError> {
        let url = std::env::var("OLLAMA_URL")
            .unwrap_or_else(|_| "http://192.168.8.107:11434/api/chat".to_string());

        let model = std::env::var("OLLAMA_MODEL")
            .unwrap_or_else(|_| "gpt-oss:latest".to_string());

        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(30))
            .default_headers({
                let mut headers = reqwest::header::HeaderMap::new();
                headers.insert(
                    reqwest::header::CONTENT_TYPE,
                    reqwest::header::HeaderValue::from_static("application/json")
                );
                headers
            })
            .build()
            .map_err(|e| VoiceError::Network(e))?;

        Ok(Self {
            client,
            url,
            model,
        })
    }

    pub fn with_config(url: String, model: String) -> Result<Self, VoiceError> {
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(30))
            .default_headers({
                let mut headers = reqwest::header::HeaderMap::new();
                headers.insert(
                    reqwest::header::CONTENT_TYPE,
                    reqwest::header::HeaderValue::from_static("application/json")
                );
                headers
            })
            .build()
            .map_err(|e| VoiceError::Network(e))?;

        Ok(Self {
            client,
            url,
            model,
        })
    }

    async fn call_api(&self, text: &str) -> Result<String, VoiceError> {
        let system_prompt = "You are a translation assistant. Please translate the user's input into English.";

        let payload = json!({
            "model": self.model,
            "messages": [
                {
                    "role": "system",
                    "content": system_prompt
                },
                {
                    "role": "user", 
                    "content": text
                }
            ],
            "stream": false
        });

        let response = self.client
            .post(&self.url)
            .json(&payload)
            .send()
            .await
            .map_err(|e| VoiceError::Network(e))?;

        let status = response.status();
        if !status.is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(VoiceError::Other(format!("Ollama API error: {} - {}", status, error_text)));
        }

        let result: Value = response.json().await
            .map_err(|e| VoiceError::Network(e))?;

        if let Some(content) = result
            .get("message")
            .and_then(|msg| msg.get("content"))
            .and_then(|v| v.as_str())
        {
            Ok(content.trim().to_string())
        } else {
            Err(VoiceError::Other("No translation content in Ollama response".to_string()))
        }
    }
}

impl TranslateProcessor for OllamaTranslateProcessor {
    fn translate(&self, text: &str) -> Result<String, VoiceError> {
        if text.trim().is_empty() {
            return Ok(String::new());
        }

        let rt = tokio::runtime::Runtime::new()
            .map_err(|e| VoiceError::Other(format!("Failed to create runtime: {}", e)))?;

        rt.block_on(async {
            self.call_api(text).await
        })
    }
}