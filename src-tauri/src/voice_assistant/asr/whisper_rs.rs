use std::io::Cursor;
use std::path::Path;
use whisper_rs::{WhisperContext, FullParams, SamplingStrategy, WhisperContextParameters};
use crate::voice_assistant::{AsrProcessor, Mode, VoiceError};
use std::time::Instant;

#[derive(Debug, Clone)]
pub struct WhisperRSConfig {
    pub model_path: String,
    pub sampling_strategy: SamplingStrategyConfig,
    pub language: Option<String>,
    pub translate: bool,
}

#[derive(Debug, Clone)]
pub enum SamplingStrategyConfig {
    Greedy { best_of: u32 },
    Beam { best_of: u32, patience: f32 },
}

pub struct WhisperRSProcessor {
    ctx: WhisperContext,
    config: WhisperRSConfig,
}

impl WhisperRSProcessor {
    pub fn new(config: WhisperRSConfig) -> Result<Self, VoiceError> {
        // Check if model file exists
        if !Path::new(&config.model_path).exists() {
            return Err(VoiceError::Other(format!(
                "Whisper model file not found: {}",
                config.model_path
            )));
        }

        // Create whisper context
        let ctx = WhisperContext::new_with_params(
            &config.model_path,
            WhisperContextParameters::default(),
        ).map_err(|e| {
            VoiceError::Other(format!("Failed to load whisper model: {}", e))
        })?;

        Ok(Self { ctx, config })
    }

    pub fn from_env() -> Result<Self, VoiceError> {
        let model_path = std::env::var("WHISPER_MODEL_PATH")
            .unwrap_or_else(|_| {
                // Default model path - user should set this environment variable
                "./models/ggml-base.bin".to_string()
            });

        let config = WhisperRSConfig {
            model_path,
            sampling_strategy: SamplingStrategyConfig::Greedy { best_of: 1 },
            language: None, // Auto-detect
            translate: false,
        };

        Self::new(config)
    }

    fn create_params(&self, mode: Mode) -> FullParams<'_, '_> {
        let sampling_strategy = match &self.config.sampling_strategy {
            SamplingStrategyConfig::Greedy { best_of } => {
                SamplingStrategy::Greedy { best_of: *best_of as i32 }
            }
            SamplingStrategyConfig::Beam { best_of, patience: _ } => {
                // whisper-rs doesn't have Beam strategy in the version we're using
                // Fall back to Greedy strategy
                println!("⚠️ Beam search not available, falling back to greedy search");
                SamplingStrategy::Greedy { best_of: *best_of as i32 }
            }
        };

        let mut params = FullParams::new(sampling_strategy);
        
        // Set language
        match &self.config.language {
            Some(lang) => {
                if lang == "auto" {
                    params.set_language(None);
                } else {
                    params.set_language(Some(lang));
                }
            }
            None => {
                // Auto-detect for transcriptions, force English for translations
                if matches!(mode, Mode::Translations) {
                    params.set_language(Some("en"));
                } else {
                    params.set_language(None);
                }
            }
        }

        // Set translation flag
        params.set_translate(matches!(mode, Mode::Translations) || self.config.translate);

        // Enable timestamps
        params.set_print_timestamps(true);
        params.set_print_special(false);
        params.set_print_progress(false);

        // Performance settings
        params.set_temperature(0.0f32);
        params.set_max_initial_ts(1_000_000.0); // Set to large value to disable

        params
    }

    fn process_audio_data(&self, audio_data: &[f32]) -> Result<String, VoiceError> {
        let start_time = Instant::now();
        
        // Create a new state for each processing request
        let mut state = self.ctx.create_state()
            .map_err(|e| VoiceError::Other(format!("Failed to create whisper state: {}", e)))?;

        // Resample audio if needed (assuming input is 16kHz mono)
        // whisper.cpp expects 16kHz mono f32 audio
        let processed_audio = self.preprocess_audio(audio_data);

        // Determine mode based on configuration
        let mode = if self.config.translate {
            Mode::Translations
        } else {
            Mode::Transcriptions
        };

        let params = self.create_params(mode);

        // Run inference
        state.full(params, &processed_audio)
            .map_err(|e| VoiceError::Other(format!("Whisper inference failed: {}", e)))?;

        // Collect results
        let num_segments = state.full_n_segments()
            .map_err(|e| VoiceError::Other(format!("Failed to get number of segments: {}", e)))?;

        let mut result_text = String::new();
        
        for i in 0..num_segments {
            let segment_text = state.full_get_segment_text(i)
                .map_err(|e| VoiceError::Other(format!("Failed to get segment text: {}", e)))?;

            // Clean up segment text
            let clean_segment = segment_text.trim().to_string();
            if !clean_segment.is_empty() {
                if !result_text.is_empty() {
                    result_text.push(' ');
                }
                result_text.push_str(&clean_segment);
            }
        }

        let processing_time = start_time.elapsed();
        println!("🎯 WhisperRS processing completed in {:?}", processing_time);

        Ok(result_text)
    }

    fn preprocess_audio(&self, audio_data: &[f32]) -> Vec<f32> {
        // For now, assume input is already in the correct format (16kHz mono f32)
        // In the future, we might need resampling here
        audio_data.to_vec()
    }
}

impl AsrProcessor for WhisperRSProcessor {
    fn process_audio(
        &self,
        audio_buffer: Cursor<Vec<u8>>,
        _mode: Mode,
        _prompt: &str,
    ) -> Result<String, VoiceError> {
        // Convert byte buffer to f32 audio samples
        let audio_data = self.convert_bytes_to_f32(audio_buffer.into_inner())?;
        
        // Process with appropriate mode
        self.process_audio_data(&audio_data)
    }
    
    fn get_processor_type(&self) -> Option<&str> {
        Some("whisper-rs")
    }
}

impl WhisperRSProcessor {
    fn convert_bytes_to_f32(&self, audio_bytes: Vec<u8>) -> Result<Vec<f32>, VoiceError> {
        // Try to parse as WAV file using hound
        let cursor = std::io::Cursor::new(audio_bytes);
        match hound::WavReader::new(cursor) {
            Ok(mut reader) => {
                let samples: Result<Vec<f32>, _> = reader.samples::<i16>()
                    .map(|s| s.map(|sample| sample as f32 / 32768.0))
                    .collect();
                
                samples.map_err(|e| VoiceError::Other(format!("Failed to parse WAV samples: {}", e)))
            }
            Err(e) => {
                // If it's not a valid WAV file, assume raw f32 data
                Err(VoiceError::Other(format!("Failed to parse WAV file: {}. Expected valid WAV format.", e)))
            }
        }
    }
}

// Factory functions for easy creation
impl WhisperRSProcessor {
    pub fn with_model_path(model_path: &str) -> Result<Self, VoiceError> {
        let config = WhisperRSConfig {
            model_path: model_path.to_string(),
            sampling_strategy: SamplingStrategyConfig::Greedy { best_of: 1 },
            language: None,
            translate: false,
        };
        Self::new(config)
    }

    pub fn with_language(model_path: &str, language: &str) -> Result<Self, VoiceError> {
        let config = WhisperRSConfig {
            model_path: model_path.to_string(),
            sampling_strategy: SamplingStrategyConfig::Greedy { best_of: 1 },
            language: Some(language.to_string()),
            translate: false,
        };
        Self::new(config)
    }

    pub fn with_beam_search(
        model_path: &str,
        best_of: u32,
        patience: f32,
    ) -> Result<Self, VoiceError> {
        let config = WhisperRSConfig {
            model_path: model_path.to_string(),
            sampling_strategy: SamplingStrategyConfig::Beam { best_of, patience },
            language: None,
            translate: false,
        };
        Self::new(config)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_creation() {
        let config = WhisperRSConfig {
            model_path: "test.bin".to_string(),
            sampling_strategy: SamplingStrategyConfig::Greedy { best_of: 1 },
            language: Some("en".to_string()),
            translate: false,
        };
        
        assert_eq!(config.model_path, "test.bin");
        assert!(matches!(config.sampling_strategy, SamplingStrategyConfig::Greedy { best_of: 1 }));
        assert_eq!(config.language, Some("en".to_string()));
        assert!(!config.translate);
    }
}