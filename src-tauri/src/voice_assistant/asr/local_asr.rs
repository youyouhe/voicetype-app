use std::io::Cursor;
use reqwest::multipart;
use crate::voice_assistant::{AsrProcessor, Mode, VoiceError};
use serde_json::Value;
use std::time::Duration;

#[derive(Debug, Clone)]
pub struct LocalASRConfig {
    pub endpoint: String,
    pub api_key: String,
}

pub struct LocalASRProcessor {
    client: reqwest::Client,
    api_url: String,
    api_key: String,
}

impl LocalASRProcessor {
    pub fn new() -> Result<Self, VoiceError> {
        let api_url = std::env::var("LOCAL_ASR_URL")
            .unwrap_or_else(|_| "http://192.168.8.107:5001/inference".to_string());

        let api_key = std::env::var("LOCAL_ASR_KEY")
            .unwrap_or_else(|_| "default-key".to_string());

        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(20))
            .build()
            .map_err(|e| VoiceError::Network(e))?;

        Ok(Self {
            client,
            api_url,
            api_key,
        })
    }

    pub fn with_config(config: LocalASRConfig) -> Result<Self, VoiceError> {
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(20))
            .build()
            .map_err(|e| VoiceError::Network(e))?;

        Ok(Self {
            client,
            api_url: config.endpoint,
            api_key: config.api_key,
        })
    }

    async fn call_api(&self, audio_data: &[u8], lang: &str) -> Result<String, VoiceError> {
        let form = multipart::Form::new()
            .part("file", multipart::Part::bytes(audio_data.to_vec())
                .file_name("audio.wav")
                .mime_str("audio/wav")?)
            .text("response_format", "srt")
            .text("language", lang.to_string());

        let response = self.client
            .post(&self.api_url)
            .header("X-API-KEY", &self.api_key)
            .multipart(form)
            .send()
            .await
            .map_err(|e| VoiceError::Network(e))?;

        let status = response.status();
        if !status.is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(VoiceError::Other(format!("Local ASR API error: {} - {}", status, error_text)));
        }

        let response_text = response.text().await
            .map_err(|e| VoiceError::Network(e))?;

        // Parse different response formats
        if response_text.trim().starts_with('{') {
            // JSON response
            if let Ok(json_result) = serde_json::from_str::<Value>(&response_text) {
                // Try new format: {"code":0,"msg":"ok","data":"transcription"}
                if let (Some(code), Some(data)) = (
                    json_result.get("code").and_then(|v| v.as_i64()),
                    json_result.get("data").and_then(|v| v.as_str())
                ) {
                    if code == 0 {
                        return Ok(data.to_string());
                    }
                }

                // Try old format: {"result":[{"text":"","raw_text":"","clean_text":""}]}
                if let Some(result) = json_result.get("result").and_then(|v| v.as_array()) {
                    if let Some(first_item) = result.first() {
                        if let Some(text) = first_item.get("text").and_then(|v| v.as_str()) {
                            return Ok(text.to_string());
                        }
                    }
                }
            }
        }

        // If JSON parsing fails, treat as plain text (SRT format)
        let cleaned_text = self.clean_srt_text(&response_text);
        Ok(cleaned_text)
    }

    fn clean_srt_text(&self, srt_text: &str) -> String {
        // Remove SRT timestamps and formatting, keep only the spoken text
        srt_text
            .lines()
            .filter(|line| {
                !line.contains("-->") &&
                !line.trim().parse::<u32>().is_ok() &&
                !line.trim().is_empty()
            })
            .collect::<Vec<_>>()
            .join(" ")
    }
}

impl AsrProcessor for LocalASRProcessor {
    fn process_audio(
        &self,
        audio_buffer: Cursor<Vec<u8>>,
        mode: Mode,
        _prompt: &str,
    ) -> Result<String, VoiceError> {
        let rt = tokio::runtime::Runtime::new()
            .map_err(|e| VoiceError::Other(format!("Failed to create runtime: {}", e)))?;

        rt.block_on(async {
            let audio_data = audio_buffer.into_inner();
            let lang = match mode {
                Mode::Transcriptions => "auto",
                Mode::Translations => "en",  // Force English for translations
            };

            self.call_api(&audio_data, lang).await
        })
    }

    fn get_processor_type(&self) -> Option<&str> {
        Some("local")
    }
}