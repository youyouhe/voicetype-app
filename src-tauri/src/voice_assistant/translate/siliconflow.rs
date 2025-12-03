use crate::voice_assistant::{TranslateProcessor, VoiceError};
use serde_json::{json, Value};
use std::time::Duration;

pub struct SiliconFlowTranslateProcessor {
    client: reqwest::Client,
    #[allow(dead_code)] // Kept for potential future use
    api_key: String,
    model: String,
    base_url: String,
}

impl SiliconFlowTranslateProcessor {
    pub fn new() -> Result<Self, VoiceError> {
        let api_key = std::env::var("SILICONFLOW_API_KEY")
            .map_err(|_| VoiceError::Other("SILICONFLOW_API_KEY environment variable not set".to_string()))?;

        let model = std::env::var("SILICONFLOW_TRANSLATE_MODEL")
            .unwrap_or_else(|_| "THUDM/glm-4-9b-chat".to_string());

        let base_url = std::env::var("SILICONFLOW_BASE_URL")
            .unwrap_or_else(|_| "https://api.siliconflow.cn".to_string());

        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(30))
            .default_headers({
                let mut headers = reqwest::header::HeaderMap::new();
                headers.insert(
                    reqwest::header::CONTENT_TYPE,
                    reqwest::header::HeaderValue::from_static("application/json")
                );
                headers.insert(
                    reqwest::header::AUTHORIZATION,
                    reqwest::header::HeaderValue::from_str(&format!("Bearer {}", api_key))
                        .map_err(|e| VoiceError::Other(format!("Invalid auth header: {}", e)))?
                );
                headers
            })
            .build()
            .map_err(|e| VoiceError::Network(e))?;

        Ok(Self {
            client,
            api_key,
            model,
            base_url,
        })
    }

    pub fn with_config(api_key: String, model: String, base_url: String) -> Result<Self, VoiceError> {
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(30))
            .default_headers({
                let mut headers = reqwest::header::HeaderMap::new();
                headers.insert(
                    reqwest::header::CONTENT_TYPE,
                    reqwest::header::HeaderValue::from_static("application/json")
                );
                headers.insert(
                    reqwest::header::AUTHORIZATION,
                    reqwest::header::HeaderValue::from_str(&format!("Bearer {}", api_key))
                        .map_err(|e| VoiceError::Other(format!("Invalid auth header: {}", e)))?
                );
                headers
            })
            .build()
            .map_err(|e| VoiceError::Network(e))?;

        Ok(Self {
            client,
            api_key,
            model,
            base_url,
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
            ]
        });

        let response = self.client
            .post(&format!("{}/v1/chat/completions", self.base_url))
            .json(&payload)
            .send()
            .await
            .map_err(|e| VoiceError::Network(e))?;

        let status = response.status();
        if !status.is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(VoiceError::Other(format!("SiliconFlow API error: {} - {}", status, error_text)));
        }

        let result: Value = response.json().await
            .map_err(|e| VoiceError::Network(e))?;

        if let Some(content) = result
            .get("choices")
            .and_then(|v| v.as_array())
            .and_then(|arr| arr.first())
            .and_then(|choice| choice.get("message"))
            .and_then(|msg| msg.get("content"))
            .and_then(|v| v.as_str())
        {
            Ok(content.trim().to_string())
        } else {
            Err(VoiceError::Other("No translation content in SiliconFlow response".to_string()))
        }
    }
}

impl TranslateProcessor for SiliconFlowTranslateProcessor {
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