use std::io::Cursor;
use reqwest::multipart;
use crate::voice_assistant::{AsrProcessor, Mode, VoiceError};
use serde_json::Value;
use std::time::Duration;

pub struct WhisperProcessor {
    client: reqwest::Client,
    api_key: String,
    base_url: String,
    convert_to_simplified: bool,
    add_symbol: bool,
    optimize_result: bool,
}

impl WhisperProcessor {
    pub fn new() -> Result<Self, VoiceError> {
        let api_key = std::env::var("GROQ_API_KEY")
            .map_err(|_| VoiceError::Other("GROQ_API_KEY environment variable not set".to_string()))?;
        
        let base_url = std::env::var("GROQ_BASE_URL")
            .unwrap_or_else(|_| "https://api.groq.com".to_string());

        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(30))
            .build()
            .map_err(|e| VoiceError::Network(e))?;

        Ok(Self {
            client,
            api_key,
            base_url,
            convert_to_simplified: std::env::var("CONVERT_TO_SIMPLIFIED")
                .unwrap_or_else(|_| "true".to_string())
                .parse()
                .unwrap_or(true),
            add_symbol: std::env::var("ADD_SYMBOL")
                .unwrap_or_else(|_| "true".to_string())
                .parse()
                .unwrap_or(true),
            optimize_result: std::env::var("OPTIMIZE_RESULT")
                .unwrap_or_else(|_| "true".to_string())
                .parse()
                .unwrap_or(true),
        })
    }

    async fn call_whisper_api(
        &self,
        mode: Mode,
        audio_data: &[u8],
        prompt: &str,
    ) -> Result<String, VoiceError> {
        let model = match mode {
            Mode::Transcriptions => "whisper-large-v3-turbo",
            Mode::Translations => "whisper-large-v3",
        };

        let form = multipart::Form::new()
            .part("file", multipart::Part::bytes(audio_data.to_vec())
                .file_name("audio.wav")
                .mime_str("audio/wav")?)
            .text("model", model)
            .text("response_format", "json");

        let form = if !prompt.is_empty() {
            form.text("prompt", prompt.to_string())
        } else {
            form
        };

        let request = self.client
            .post(&format!("{}/openai/v1/audio/{}", self.base_url, 
                if mode == Mode::Translations { "translations" } else { "transcriptions" }))
            .header("Authorization", format!("Bearer {}", self.api_key))
            .multipart(form);

        let response = request
            .send()
            .await
            .map_err(|e| VoiceError::Network(e))?;

        let status = response.status();
        if !status.is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(VoiceError::Other(format!("Whisper API error: {} - {}", status, error_text)));
        }

        let result: Value = response.json().await
            .map_err(|e| VoiceError::Network(e))?;

        if let Some(text) = result.get("text").and_then(|v| v.as_str()) {
            let mut processed_text = text.to_string();
            
            if self.add_symbol {
                processed_text = self.add_punctuation(&processed_text);
            }
            
            if self.optimize_result {
                processed_text = self.optimize_text(&processed_text);
            }
            
            if self.convert_to_simplified {
                processed_text = self.convert_traditional_to_simplified(&processed_text);
            }
            
            Ok(processed_text)
        } else {
            Err(VoiceError::Other("No text in Whisper response".to_string()))
        }
    }

    fn add_punctuation(&self, text: &str) -> String {
        // Simple punctuation addition - in a real implementation,
        // this could use more sophisticated NLP
        text.to_string()
    }

    fn optimize_text(&self, text: &str) -> String {
        // Basic text optimization - remove excessive whitespace
        text.split_whitespace().collect::<Vec<_>>().join(" ")
    }

    fn convert_traditional_to_simplified(&self, text: &str) -> String {
        // Placeholder for Chinese text conversion
        // In a real implementation, you'd use a library like chinese-conversion
        text.to_string()
    }
}

impl AsrProcessor for WhisperProcessor {
    fn process_audio(
        &self,
        audio_buffer: Cursor<Vec<u8>>,
        mode: Mode,
        prompt: &str,
    ) -> Result<String, VoiceError> {
        // Since we need async, but the trait is sync, we'll use a blocking runtime
        let rt = tokio::runtime::Runtime::new()
            .map_err(|e| VoiceError::Other(format!("Failed to create runtime: {}", e)))?;

        rt.block_on(async {
            let audio_data = audio_buffer.into_inner();
            self.call_whisper_api(mode, &audio_data, prompt).await
        })
    }
}