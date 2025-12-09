use std::io::Cursor;
use std::sync::{Arc, Mutex};
use std::time::Instant;
use whisper_rs::{WhisperContext, FullParams, SamplingStrategy, WhisperContextParameters};
use crate::voice_assistant::{AsrProcessor, Mode, VoiceError};
use super::{WhisperRSConfig, WhisperVadProcessor, VadSegment, filter_silence};

pub struct EnhancedWhisperProcessor {
    whisper_ctx: Arc<WhisperContext>,
    vad_processor: Option<WhisperVadProcessor>,
    config: WhisperRSConfig,
    _state_guard: Mutex<()>,
}

impl EnhancedWhisperProcessor {
    pub fn new(config: WhisperRSConfig, enable_vad: bool) -> Result<Self, VoiceError> {
        let whisper_ctx = WhisperContext::new_with_params(
            &config.model_path,
            WhisperContextParameters::default(),
        ).map_err(|e| {
            VoiceError::Other(format!("Failed to load whisper model: {}", e))
        })?;

        let vad_processor = if enable_vad {
            // Try to initialize VAD if requested
            match super::create_vad_processor() {
                Ok(vad) => {
                    println!("✅ VAD processor initialized successfully");
                    Some(vad)
                }
                Err(e) => {
                    println!("⚠️ VAD initialization failed: {}. Continuing without VAD.", e);
                    None
                }
            }
        } else {
            None
        };

        Ok(Self {
            whisper_ctx: Arc::new(whisper_ctx),
            vad_processor,
            config,
            _state_guard: Mutex::new(()),
        })
    }

    pub fn with_vad(config: WhisperRSConfig) -> Result<Self, VoiceError> {
        Self::new(config, true)
    }

    pub fn without_vad(config: WhisperRSConfig) -> Result<Self, VoiceError> {
        Self::new(config, false)
    }

    fn create_params(&self, mode: Mode) -> FullParams<'_, '_> {
        let sampling_strategy = match &self.config.sampling_strategy {
            super::SamplingStrategyConfig::Greedy { best_of } => {
                SamplingStrategy::Greedy { best_of: *best_of as i32 }
            }
            super::SamplingStrategyConfig::Beam { beam_size, patience } => {
                println!("🎯 Using Beam Search with beam_size: {}, patience: {}", beam_size, patience);
                SamplingStrategy::BeamSearch {
                    beam_size: *beam_size as i32,
                    patience: *patience,
                }
            }
        };

        let mut params = FullParams::new(sampling_strategy);

        // Set number of threads (use all available cores for better performance)
        let num_threads = std::thread::available_parallelism()
            .map(|n| n.get() as i32)
            .unwrap_or(4);
        params.set_n_threads(num_threads);

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

        // Disable printing to reduce noise
        params.set_print_timestamps(false);
        params.set_print_special(false);
        params.set_print_progress(false);
        params.set_print_realtime(false);

        // Performance settings
        params.set_temperature(0.0f32);
        params.set_max_initial_ts(1_000_000.0);

        // Enable prompt caching for better performance on subsequent runs
        params.set_no_context(false);

        params
    }

    fn preprocess_audio(&self, audio_data: &[f32]) -> Vec<f32> {
        // Check if we need to convert stereo to mono
        if audio_data.len() % 2 == 0 {
            // Try to convert from stereo to mono by averaging pairs
            let mut mono_audio = Vec::with_capacity(audio_data.len() / 2);
            for chunk in audio_data.chunks_exact(2) {
                let mono_sample = (chunk[0] + chunk[1]) / 2.0;
                mono_audio.push(mono_sample);
            }
            mono_audio
        } else {
            // Already mono
            audio_data.to_vec()
        }
    }

    fn process_with_vad(&self, audio_data: &[f32], mode: Mode) -> Result<String, VoiceError> {
        if let Some(ref mut vad_processor) = self.vad_processor {
            // Detect speech segments
            let vad_start = Instant::now();
            let segments = vad_processor.detect_speech_segments(audio_data)
                .map_err(|e| VoiceError::Other(format!("VAD failed: {}", e)))?;
            let vad_duration = vad_start.elapsed();

            if segments.is_empty() {
                println!("🔇 VAD: No speech detected in audio");
                return Ok(String::new());
            }

            println!("🎯 VAD detected {} speech segments in {:?}", segments.len(), vad_duration);
            for (i, seg) in segments.iter().enumerate() {
                println!("  Segment {}: {:.2}s - {:.2}s ({:.2}s)",
                        i,
                        seg.start_ms as f32 / 1000.0,
                        seg.end_ms as f32 / 1000.0,
                        seg.duration_seconds());
            }

            // Extract only speech portions
            let speech_audio = vad_processor.extract_speech_audio(audio_data, &segments);

            // Process the speech portions
            self.process_audio_data(&speech_audio, mode)
        } else {
            // Fallback to simple silence filtering
            let filtered_audio = filter_silence(audio_data, 0.01f32, 1024);
            if filtered_audio.len() < 1024 {
                return Err(VoiceError::Other("No significant speech detected after filtering".to_string()));
            }
            self.process_audio_data(&filtered_audio, mode)
        }
    }

    fn process_audio_data(&self, audio_data: &[f32], mode: Mode) -> Result<String, VoiceError> {
        let start_time = Instant::now();

        // Create a new state for each processing request
        let mut state = self.whisper_ctx.create_state()
            .map_err(|e| VoiceError::Other(format!("Failed to create whisper state: {}", e)))?;

        // Resample audio if needed (assuming input is 16kHz mono)
        let processed_audio = self.preprocess_audio(audio_data);

        // Check if we have enough audio data
        if processed_audio.len() < 1024 {
            return Err(VoiceError::Other("Audio too short for processing".to_string()));
        }

        let params = self.create_params(mode);

        // Run inference
        state.full(params, &processed_audio)
            .map_err(|e| VoiceError::Other(format!("Whisper inference failed: {}", e)))?;

        // Use the iterator pattern for cleaner result collection
        let mut result_text = String::new();

        for segment in state.as_iter() {
            let segment_text = segment.to_string();
            let clean_segment = segment_text.trim().to_string();

            if !clean_segment.is_empty() {
                if !result_text.is_empty() {
                    result_text.push(' ');
                }
                result_text.push_str(&clean_segment);
            }
        }

        let processing_time = start_time.elapsed();
        let audio_duration = processed_audio.len() as f32 / 16000.0;
        let real_time_factor = processing_time.as_secs_f32() / audio_duration;

        println!("🎯 Enhanced Whisper processing completed in {:?}", processing_time);
        println!("⏱️ Audio duration: {:.2}s, Real-time factor: {:.2}x", audio_duration, real_time_factor);

        Ok(result_text)
    }

    fn convert_bytes_to_f32(&self, audio_bytes: Vec<u8>) -> Result<Vec<f32>, VoiceError> {
        // Try to parse as WAV file using hound
        let cursor = std::io::Cursor::new(audio_bytes);
        match hound::WavReader::new(cursor) {
            Ok(mut reader) => {
                let spec = reader.spec();

                match spec.sample_format {
                    hound::SampleFormat::Int => {
                        // Convert integer samples to f32
                        let samples: Result<Vec<f32>, _> = reader.samples::<i16>()
                            .map(|s| s.map(|sample| sample as f32 / 32768.0))
                            .collect();

                        let mut float_samples = samples.map_err(|e|
                            VoiceError::Other(format!("Failed to parse WAV samples: {}", e))
                        )?;

                        // Convert stereo to mono if needed
                        if spec.channels == 2 {
                            let mut mono_samples = Vec::with_capacity(float_samples.len() / 2);
                            for chunk in float_samples.chunks_exact(2) {
                                let mono_sample = (chunk[0] + chunk[1]) / 2.0;
                                mono_samples.push(mono_sample);
                            }
                            float_samples = mono_samples;
                        }

                        Ok(float_samples)
                    }
                    hound::SampleFormat::Float => {
                        // Already float samples
                        let samples: Result<Vec<f32>, _> = reader.samples::<f32>()
                            .map(|s| s.map(|sample| sample))
                            .collect();

                        let mut float_samples = samples.map_err(|e|
                            VoiceError::Other(format!("Failed to parse WAV samples: {}", e))
                        )?;

                        // Convert stereo to mono if needed
                        if spec.channels == 2 {
                            let mut mono_samples = Vec::with_capacity(float_samples.len() / 2);
                            for chunk in float_samples.chunks_exact(2) {
                                let mono_sample = (chunk[0] + chunk[1]) / 2.0;
                                mono_samples.push(mono_sample);
                            }
                            float_samples = mono_samples;
                        }

                        Ok(float_samples)
                    }
                }
            }
            Err(e) => {
                // If it's not a valid WAV file, assume raw f32 data
                Err(VoiceError::Other(format!("Failed to parse WAV file: {}. Expected valid WAV format.", e)))
            }
        }
    }
}

impl AsrProcessor for EnhancedWhisperProcessor {
    fn process_audio(
        &self,
        audio_buffer: Cursor<Vec<u8>>,
        _mode: Mode,
        _prompt: &str,
    ) -> Result<String, VoiceError> {
        // Convert byte buffer to f32 audio samples
        let audio_data = self.convert_bytes_to_f32(audio_buffer.into_inner())?;

        // Determine processing mode based on configuration
        let mode = if self.config.translate {
            Mode::Translations
        } else {
            Mode::Transcriptions
        };

        // Process with VAD if available
        if self.vad_processor.is_some() {
            self.process_with_vad(&audio_data, mode)
        } else {
            self.process_audio_data(&audio_data, mode)
        }
    }

    fn get_processor_type(&self) -> Option<&str> {
        Some("enhanced-whisper-rs")
    }
}

// Factory functions for easy creation
impl EnhancedWhisperProcessor {
    pub fn with_model_path(model_path: &str) -> Result<Self, VoiceError> {
        let config = WhisperRSConfig {
            model_path: model_path.to_string(),
            sampling_strategy: super::SamplingStrategyConfig::Greedy { best_of: 1 },
            language: None,
            translate: false,
        };
        Self::new(config, false)
    }

    pub fn with_vad_and_model_path(model_path: &str) -> Result<Self, VoiceError> {
        let config = WhisperRSConfig {
            model_path: model_path.to_string(),
            sampling_strategy: super::SamplingStrategyConfig::Greedy { best_of: 1 },
            language: None,
            translate: false,
        };
        Self::with_vad(config)
    }

    pub fn with_beam_search_and_vad(
        model_path: &str,
        beam_size: u32,
        patience: f32,
    ) -> Result<Self, VoiceError> {
        let config = WhisperRSConfig {
            model_path: model_path.to_string(),
            sampling_strategy: super::SamplingStrategyConfig::Beam { beam_size, patience },
            language: None,
            translate: false,
        };
        Self::with_vad(config)
    }
}