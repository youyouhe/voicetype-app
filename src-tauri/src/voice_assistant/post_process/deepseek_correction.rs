use crate::voice_assistant::VoiceError;
use serde_json::{json, Value};
use std::time::Duration;

pub struct DeepSeekPostProcessor {
    client: reqwest::Client,
    url: String,
    api_key: String,
    model: String,
    system_prompt: String,
    timeout: Duration,
}

impl DeepSeekPostProcessor {
    pub fn new(
        endpoint: String,
        api_key: String,
        model: String,
        system_prompt: String,
        timeout_seconds: i64,
    ) -> Result<Self, VoiceError> {
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(timeout_seconds as u64))
            .default_headers({
                let mut headers = reqwest::header::HeaderMap::new();
                headers.insert(
                    reqwest::header::CONTENT_TYPE,
                    reqwest::header::HeaderValue::from_static("application/json")
                );
                headers.insert(
                    reqwest::header::AUTHORIZATION,
                    reqwest::header::HeaderValue::from_str(&format!("Bearer {}", api_key))
                        .map_err(|e| VoiceError::Other(format!("Invalid API key: {}", e)))?
                );
                headers
            })
            .build()
            .map_err(|e| VoiceError::Network(e))?;

        Ok(Self {
            client,
            url: endpoint,
            api_key,
            model,
            system_prompt,
            timeout: Duration::from_secs(timeout_seconds as u64),
        })
    }

    /// Correct text using DeepSeek API (OpenAI-compatible format)
    pub fn correct_text(&self, text: &str) -> Result<String, VoiceError> {
        if text.trim().is_empty() {
            return Ok(String::new());
        }

        // Try to get current runtime handle, otherwise create a new one
        match tokio::runtime::Handle::try_current() {
            Ok(handle) => {
                // Already in async context, use current runtime
                tokio::task::block_in_place(move || {
                    handle.block_on(async move {
                        self.call_api(text).await
                    })
                })
            }
            Err(_) => {
                // Not in async context, create new runtime
                let rt = tokio::runtime::Runtime::new()
                    .map_err(|e| VoiceError::Other(format!("Failed to create runtime: {}", e)))?;
                rt.block_on(async {
                    self.call_api(text).await
                })
            }
        }
    }

    async fn call_api(&self, text: &str) -> Result<String, VoiceError> {
        // DeepSeek uses OpenAI-compatible format
        let payload = json!({
            "model": self.model,
            "messages": [
                {
                    "role": "system",
                    "content": self.system_prompt
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
            return Err(VoiceError::Other(format!("DeepSeek API error: {} - {}", status, error_text)));
        }

        let result: Value = response.json().await
            .map_err(|e| VoiceError::Network(e))?;

        // DeepSeek returns OpenAI-compatible format: choices[0].message.content
        if let Some(content) = result
            .get("choices")
            .and_then(|c| c.get(0))
            .and_then(|m| m.get("message"))
            .and_then(|msg| msg.get("content"))
            .and_then(|v| v.as_str())
        {
            Ok(content.trim().to_string())
        } else {
            Err(VoiceError::Other("No correction content in DeepSeek response".to_string()))
        }
    }
}
