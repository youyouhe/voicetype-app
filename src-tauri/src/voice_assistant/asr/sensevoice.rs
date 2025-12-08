use std::io::Cursor;
use reqwest::multipart;
use crate::voice_assistant::{AsrProcessor, Mode, VoiceError, TranslateProcessor};
use serde_json::Value;
use std::time::Duration;
use std::sync::Arc;

pub struct SenseVoiceProcessor {
    client: reqwest::Client,
    api_key: String,
    convert_to_simplified: bool,
    translate_processor: Option<Arc<dyn TranslateProcessor + Send + Sync>>,
}

impl SenseVoiceProcessor {
    pub fn new() -> Result<Self, VoiceError> {
        let api_key = std::env::var("SILICONFLOW_API_KEY")
            .map_err(|_| VoiceError::Other("SILICONFLOW_API_KEY environment variable not set".to_string()))?;

        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(20))
            .build()
            .map_err(|e| VoiceError::Network(e))?;

        Ok(Self {
            client,
            api_key,
            convert_to_simplified: std::env::var("CONVERT_TO_SIMPLIFIED")
                .unwrap_or_else(|_| "true".to_string())
                .parse()
                .unwrap_or(true),
            translate_processor: None,
        })
    }

    pub fn with_translate_processor(mut self, translate_processor: Arc<dyn TranslateProcessor + Send + Sync>) -> Self {
        self.translate_processor = Some(translate_processor);
        self
    }

    async fn call_api(&self, audio_data: &[u8]) -> Result<String, VoiceError> {
        let form = multipart::Form::new()
            .part("file", multipart::Part::bytes(audio_data.to_vec())
                .file_name("audio.wav")
                .mime_str("audio/wav")?)
            .text("model", "FunAudioLLM/SenseVoiceSmall");

        let response = self.client
            .post("https://api.siliconflow.cn/v1/audio/transcriptions")
            .header("Authorization", format!("Bearer {}", self.api_key))
            .multipart(form)
            .send()
            .await
            .map_err(|e| VoiceError::Network(e))?;

        let status = response.status();
        if !status.is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(VoiceError::Other(format!("SenseVoice API error: {} - {}", status, error_text)));
        }

        let result: Value = response.json().await
            .map_err(|e| VoiceError::Network(e))?;

        if let Some(text) = result.get("text").and_then(|v| v.as_str()) {
            let processed_text = if self.convert_to_simplified {
                self.convert_traditional_to_simplified(text)
            } else {
                text.to_string()
            };
            Ok(processed_text)
        } else {
            Err(VoiceError::Other("No text in SenseVoice response".to_string()))
        }
    }

    fn convert_traditional_to_simplified(&self, text: &str) -> String {
        // Placeholder for Chinese text conversion
        text.to_string()
    }
}

impl AsrProcessor for SenseVoiceProcessor {
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
            let transcription = self.call_api(&audio_data).await?;

            match mode {
                Mode::Transcriptions => Ok(transcription),
                Mode::Translations => {
                    if let Some(ref translate_processor) = self.translate_processor {
                        translate_processor.translate(&transcription)
                    } else {
                        Err(VoiceError::Other("No translate processor available for translation mode".to_string()))
                    }
                }
            }
        })
    }
    
    fn get_processor_type(&self) -> Option<&str> {
        Some("sensevoice")
    }
}