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

    async fn call_api_with_format(&self, audio_data: &[u8], lang: &str, format: String) -> Result<String, VoiceError> {
        let form = multipart::Form::new()
            .part("file", multipart::Part::bytes(audio_data.to_vec())
                .file_name("audio.wav")
                .mime_str("audio/wav")?)
            .text("response_format", format.clone())
            .text("language", lang.to_string());

        println!("🔍 Sending ASR request with format={}, language={}", format, lang);

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
            return Err(VoiceError::Other(format!("Local ASR API error ({} format): {} - {}", format, status, error_text)));
        }

        let response_text = response.text().await
            .map_err(|e| VoiceError::Network(e))?;

        println!("🔍 ASR response ({} format): {} chars", format, response_text.len());
        println!("📄 Response preview: {}", &response_text[..response_text.len().min(200)]);

        Ok(response_text)
    }

    async fn call_api(&self, audio_data: &[u8], lang: &str) -> Result<String, VoiceError> {
        // 🔥 根据API错误信息，优化格式尝试顺序：SRT → JSON → Text
        let formats = vec!["srt", "json", "text"];

        for format in formats {
            println!("🔄 Trying response format: {}", format);

            match self.call_api_with_format(audio_data, lang, format.to_string()).await {
                Ok(response_text) => {
                    // 处理响应
                    let processed_text = self.process_response(&response_text, format)?;

                    if !processed_text.is_empty() {
                        println!("✅ Successfully processed response with {} format", format);
                        return Ok(processed_text);
                    } else {
                        println!("⚠️ Empty result with {} format, trying next...", format);
                    }
                }
                Err(e) => {
                    println!("❌ Failed with {} format: {}, trying next...", format, e);
                }
            }
        }

        Err(VoiceError::Other("All response formats failed".to_string()))
    }

    fn process_response(&self, response_text: &str, format: &str) -> Result<String, VoiceError> {
        match format {
            "json" => self.process_json_response(response_text),
            "text" => Ok(response_text.trim().to_string()),
            "srt" => {
                // 🔥 SRT格式可能包装在JSON中，先尝试解析JSON
                if response_text.trim().starts_with('{') {
                    if let Ok(json_result) = serde_json::from_str::<Value>(response_text) {
                        if let Some(data) = json_result.get("data").and_then(|v| v.as_str()) {
                            println!("✅ Found SRT data in JSON wrapper");
                            return self.process_srt_response(data);
                        }
                    }
                }
                // 如果不是JSON包装，直接处理SRT
                self.process_srt_response(response_text)
            },
            _ => Ok(response_text.trim().to_string()),
        }
    }

    fn process_json_response(&self, response_text: &str) -> Result<String, VoiceError> {
        if !response_text.trim().starts_with('{') {
            return Ok(response_text.trim().to_string());
        }

        if let Ok(json_result) = serde_json::from_str::<Value>(response_text) {
            println!("✅ Successfully parsed JSON response");

            // OpenAI Whisper API格式: {"text": "transcription"}
            if let Some(text) = json_result.get("text").and_then(|v| v.as_str()) {
                println!("✅ Found text in OpenAI format");
                return Ok(text.to_string());
            }

            // Custom format: {"code":0,"msg":"ok","data":"transcription"}
            if let (Some(code), Some(data)) = (
                json_result.get("code").and_then(|v| v.as_i64()),
                json_result.get("data").and_then(|v| v.as_str())
            ) {
                if code == 0 {
                    println!("✅ Found text in custom format");
                    return Ok(data.to_string());
                }
            }

            // Array format: [{"text":"","raw_text":"","clean_text":""}]
            if let Some(result) = json_result.get("result").and_then(|v| v.as_array()) {
                if let Some(first_item) = result.first() {
                    if let Some(text) = first_item.get("text").and_then(|v| v.as_str()) {
                        println!("✅ Found text in array format");
                        return Ok(text.to_string());
                    }
                }
            }

            // 尝试其他可能的字段
            for field in ["transcription", "result", "output", "content"] {
                if let Some(text) = json_result.get(field).and_then(|v| v.as_str()) {
                    println!("✅ Found text in field '{}'", field);
                    return Ok(text.to_string());
                }
            }

            println!("⚠️ JSON parsed but no text field found");
            println!("🔍 Full JSON: {}", serde_json::to_string_pretty(&json_result).unwrap_or_else(|_| "Failed to serialize".to_string()));
        } else {
            println!("❌ Failed to parse JSON response");
        }

        Ok(String::new())
    }

    fn process_srt_response(&self, srt_text: &str) -> Result<String, VoiceError> {
        println!("🔍 Processing SRT response: {} chars", srt_text.len());

        // SRT格式处理：移除时间戳和序号，只保留文本
        let cleaned_text = srt_text
            .lines()
            .filter(|line| {
                !line.contains("-->") &&
                !line.trim().parse::<u32>().is_ok() &&
                !line.trim().is_empty()
            })
            .collect::<Vec<_>>()
            .join(" ");

        println!("✅ Extracted text: {}", cleaned_text);

        if cleaned_text.trim().is_empty() {
            println!("⚠️ SRT processing resulted in empty text");
            return Err(VoiceError::Other("SRT processing resulted in empty text".to_string()));
        }

        Ok(cleaned_text)
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