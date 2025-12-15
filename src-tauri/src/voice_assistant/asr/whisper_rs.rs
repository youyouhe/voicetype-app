use std::io::Cursor;
use std::path::Path;
use std::sync::{Arc, Mutex};
use whisper_rs::{WhisperContext, FullParams, SamplingStrategy, WhisperContextParameters};
use crate::voice_assistant::{AsrProcessor, Mode, VoiceError};
use std::time::Instant;

#[derive(Debug, Clone)]
pub enum SamplingStrategyConfig {
    Greedy { best_of: u32 },
    Beam { beam_size: u32, patience: f32 },
}

#[derive(Debug, Clone, PartialEq)]
pub enum WhisperBackend {
    CPU,
    CUDA,
    Vulkan,
    Metal,     // Apple Silicon
    OpenCL,    // Fallback for older GPUs
}

impl Default for WhisperBackend {
    fn default() -> Self {
        Self::CPU
    }
}

impl std::fmt::Display for WhisperBackend {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            WhisperBackend::CPU => write!(f, "CPU"),
            WhisperBackend::CUDA => write!(f, "CUDA"),
            WhisperBackend::Vulkan => write!(f, "Vulkan"),
            WhisperBackend::Metal => write!(f, "Metal"),
            WhisperBackend::OpenCL => write!(f, "OpenCL"),
        }
    }
}

#[derive(Debug, Clone)]
pub struct WhisperRSConfig {
    pub model_path: String,
    pub sampling_strategy: SamplingStrategyConfig,
    pub language: Option<String>,
    pub translate: bool,
    pub enable_vad: bool,
    pub backend: WhisperBackend,
    pub use_gpu_if_available: bool,
    pub gpu_device_id: Option<u32>,
}

pub struct WhisperRSProcessor {
    ctx: Arc<WhisperContext>,
    config: WhisperRSConfig,
    // VAD flag for basic energy-based VAD (thread-safe alternative)
    enable_basic_vad: bool,
    // For thread-safe access if needed
    _state_guard: Mutex<()>,
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

        // Initialize VAD functionality
        let enable_basic_vad = if config.enable_vad {
            println!("🎯 Enabling basic energy-based VAD (thread-safe alternative)");
            true
        } else {
            false
        };

        Ok(Self {
            ctx: Arc::new(ctx),
            config,
            enable_basic_vad,
            _state_guard: Mutex::new(()),
        })
    }

    pub fn from_env() -> Result<Self, VoiceError> {
        let model_path = std::env::var("WHISPER_MODEL_PATH")
            .unwrap_or_else(|_| {
                // Default model path - user should set this environment variable
                "./models/ggml-base.bin".to_string()
            });

        // 自动检测最佳GPU后端
        let gpu_detector_mutex = super::gpu_detector::get_gpu_detector();
        let gpu_detector = gpu_detector_mutex.lock().unwrap();
        let backend = gpu_detector.get_preferred_backend().clone();

        let config = WhisperRSConfig {
            model_path,
            sampling_strategy: SamplingStrategyConfig::Greedy { best_of: 1 },
            language: None, // Auto-detect
            translate: false,
            enable_vad: false, // Default VAD disabled
            backend,
            use_gpu_if_available: true,
            gpu_device_id: None,
        };

        Self::new(config)
    }

    fn create_params(&self, mode: Mode) -> FullParams<'_, '_> {
        let sampling_strategy = match &self.config.sampling_strategy {
            SamplingStrategyConfig::Greedy { best_of } => {
                SamplingStrategy::Greedy { best_of: *best_of as i32 }
            }
            SamplingStrategyConfig::Beam { beam_size, patience } => {
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
        params.set_max_initial_ts(1_000_000.0); // Set to large value to disable

        // Enable prompt caching for better performance on subsequent runs
        params.set_no_context(false);

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

        // Apply VAD filtering if enabled
        let final_audio = if self.config.enable_vad {
            println!("🎯 VAD is enabled - processing audio...");
            match self.apply_vad_filtering(&processed_audio) {
                Ok(filtered_audio) => {
                    let original_len = processed_audio.len();
                    let filtered_len = filtered_audio.len();
                    let reduction = (original_len - filtered_len) as f64 / original_len as f64 * 100.0;
                    println!("✅ VAD filtered: {} -> {} samples (reduced {:.1}% audio)", 
                             original_len, filtered_len, reduction);
                    filtered_audio
                }
                Err(e) => {
                    println!("⚠️ VAD filtering failed: {}, using original audio", e);
                    processed_audio.clone()
                }
            }
        } else {
            processed_audio.clone()
        };

        // Check if we have enough audio data (after VAD filtering)
        if final_audio.len() < 1024 {
            return Err(VoiceError::Other("Audio too short for processing after VAD filtering".to_string()));
        }

        // Determine mode based on configuration
        let mode = if self.config.translate {
            Mode::Translations
        } else {
            Mode::Transcriptions
        };

        let params = self.create_params(mode);

        // Run inference
        state.full(params, &final_audio)
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
        let audio_duration = final_audio.len() as f32 / 16000.0;
        let real_time_factor = processing_time.as_secs_f32() / audio_duration;

        println!("🎯 WhisperRS processing completed in {:?}", processing_time);
        println!("⏱️ Audio duration: {:.2}s, Real-time factor: {:.2}x", audio_duration, real_time_factor);

        Ok(result_text)
    }

    fn preprocess_audio(&self, audio_data: &[f32]) -> Vec<f32> {
        // Check if we need to convert stereo to mono
        // If the audio length is even, we assume it might be stereo
        if audio_data.len() % 2 == 0 {
            // Try to convert from stereo to mono by averaging pairs
            let mut mono_audio = Vec::with_capacity(audio_data.len() / 2);
            for chunk in audio_data.chunks_exact(2) {
                let mono_sample = (chunk[0] + chunk[1]) / 2.0;
                mono_audio.push(mono_sample);
            }
            println!("🔄 Converted stereo audio to mono: {} -> {} samples", audio_data.len(), mono_audio.len());
            mono_audio
        } else {
            // Already mono
            println!("📊 Audio is already mono: {} samples", audio_data.len());
            audio_data.to_vec()
        }
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
                            println!("🔄 Converted stereo WAV to mono: {} -> {} samples",
                                    float_samples.len() * 2, float_samples.len());
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
                            println!("🔄 Converted stereo WAV to mono: {} -> {} samples",
                                    float_samples.len() * 2, float_samples.len());
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

    fn apply_vad_filtering(&self, audio_data: &[f32]) -> Result<Vec<f32>, VoiceError> {
        if self.enable_basic_vad {
            println!("🎯 Applying basic energy-based VAD filtering to {} audio samples", audio_data.len());
            
            let filtered_audio = self.basic_energy_vad(audio_data);
            
            println!("✅ Basic VAD filtered: {} -> {} samples (removed {:.1}% non-speech audio)", 
                     audio_data.len(), filtered_audio.len(), 
                     (1.0 - filtered_audio.len() as f64 / audio_data.len() as f64) * 100.0);
            
            Ok(filtered_audio)
        } else {
            println!("⚠️ VAD not enabled, returning original audio");
            Ok(audio_data.to_vec())
        }
    }

    // Basic energy-based VAD implementation (thread-safe)
    fn basic_energy_vad(&self, audio_data: &[f32]) -> Vec<f32> {
        let window_size = 1024; // 64ms windows at 16kHz
        let overlap = 512; // 32ms overlap
        let energy_threshold = 0.01; // Energy threshold for speech detection
        
        if audio_data.len() < window_size {
            return audio_data.to_vec();
        }
        
        let mut speech_segments = Vec::new();
        let mut in_speech = false;
        let mut speech_start = 0;
        
        // Process audio in windows
        for i in (0..audio_data.len() - window_size + 1).step_by(overlap) {
            let window = &audio_data[i..i + window_size];
            
            // Calculate RMS energy
            let energy: f32 = (window.iter().map(|&x| x * x).sum::<f32>() / window_size as f32).sqrt();
            
            if energy > energy_threshold {
                if !in_speech {
                    // Start of speech segment
                    speech_start = i;
                    in_speech = true;
                }
            } else {
                if in_speech {
                    // End of speech segment
                    speech_segments.push((speech_start, i));
                    in_speech = false;
                }
            }
        }
        
        // Handle case where speech extends to end
        if in_speech {
            speech_segments.push((speech_start, audio_data.len()));
        }
        
        // Merge speech segments into continuous audio
        let total_samples: usize = speech_segments.iter()
            .map(|(start, end)| end - start)
            .sum();
        
        let mut filtered_audio = Vec::with_capacity(total_samples);
        for (start, end) in speech_segments {
            filtered_audio.extend_from_slice(&audio_data[start..end]);
        }
        
        filtered_audio
    }
}

// Factory functions for easy creation
impl WhisperRSProcessor {
    pub fn with_model_path(model_path: &str) -> Result<Self, VoiceError> {
        let gpu_detector_mutex = super::gpu_detector::get_gpu_detector();
        let gpu_detector = gpu_detector_mutex.lock().unwrap();
        let backend = gpu_detector.get_preferred_backend().clone();
        
        let config = WhisperRSConfig {
            model_path: model_path.to_string(),
            sampling_strategy: SamplingStrategyConfig::Greedy { best_of: 1 },
            language: None,
            translate: false,
            enable_vad: false,
            backend,
            use_gpu_if_available: true,
            gpu_device_id: None,
        };
        Self::new(config)
    }

    pub fn with_model_path_and_backend(model_path: &str, backend: WhisperBackend) -> Result<Self, VoiceError> {
        let config = WhisperRSConfig {
            model_path: model_path.to_string(),
            sampling_strategy: SamplingStrategyConfig::Greedy { best_of: 1 },
            language: None,
            translate: false,
            enable_vad: false,
            backend,
            use_gpu_if_available: true,
            gpu_device_id: None,
        };
        Self::new(config)
    }

    pub fn with_language(model_path: &str, language: &str) -> Result<Self, VoiceError> {
        let gpu_detector_mutex = super::gpu_detector::get_gpu_detector();
        let gpu_detector = gpu_detector_mutex.lock().unwrap();
        let backend = gpu_detector.get_preferred_backend().clone();
        
        let config = WhisperRSConfig {
            model_path: model_path.to_string(),
            sampling_strategy: SamplingStrategyConfig::Greedy { best_of: 1 },
            language: Some(language.to_string()),
            translate: false,
            enable_vad: false,
            backend,
            use_gpu_if_available: true,
            gpu_device_id: None,
        };
        Self::new(config)
    }

    pub fn with_beam_search(
        model_path: &str,
        beam_size: u32,
        patience: f32,
    ) -> Result<Self, VoiceError> {
        let gpu_detector_mutex = super::gpu_detector::get_gpu_detector();
        let gpu_detector = gpu_detector_mutex.lock().unwrap();
        let backend = gpu_detector.get_preferred_backend().clone();
        
        let config = WhisperRSConfig {
            model_path: model_path.to_string(),
            sampling_strategy: SamplingStrategyConfig::Beam { beam_size, patience },
            language: None,
            translate: false,
            enable_vad: false,
            backend,
            use_gpu_if_available: true,
            gpu_device_id: None,
        };
        Self::new(config)
    }

    // Factory functions with VAD support
    pub fn with_model_path_and_vad(model_path: &str, enable_vad: bool) -> Result<Self, VoiceError> {
        let gpu_detector_mutex = super::gpu_detector::get_gpu_detector();
        let gpu_detector = gpu_detector_mutex.lock().unwrap();
        let backend = gpu_detector.get_preferred_backend().clone();
        
        let config = WhisperRSConfig {
            model_path: model_path.to_string(),
            sampling_strategy: SamplingStrategyConfig::Greedy { best_of: 1 },
            language: None,
            translate: false,
            enable_vad,
            backend,
            use_gpu_if_available: true,
            gpu_device_id: None,
        };
        Self::new(config)
    }

    pub fn with_language_and_vad(model_path: &str, language: &str, enable_vad: bool) -> Result<Self, VoiceError> {
        let gpu_detector_mutex = super::gpu_detector::get_gpu_detector();
        let gpu_detector = gpu_detector_mutex.lock().unwrap();
        let backend = gpu_detector.get_preferred_backend().clone();
        
        let config = WhisperRSConfig {
            model_path: model_path.to_string(),
            sampling_strategy: SamplingStrategyConfig::Greedy { best_of: 1 },
            language: Some(language.to_string()),
            translate: false,
            enable_vad,
            backend,
            use_gpu_if_available: true,
            gpu_device_id: None,
        };
        Self::new(config)
    }

    pub fn with_beam_search_and_vad(
        model_path: &str,
        beam_size: u32,
        patience: f32,
        enable_vad: bool,
    ) -> Result<Self, VoiceError> {
        let gpu_detector_mutex = super::gpu_detector::get_gpu_detector();
        let gpu_detector = gpu_detector_mutex.lock().unwrap();
        let backend = gpu_detector.get_preferred_backend().clone();
        
        let config = WhisperRSConfig {
            model_path: model_path.to_string(),
            sampling_strategy: SamplingStrategyConfig::Beam { beam_size, patience },
            language: None,
            translate: false,
            enable_vad,
            backend,
            use_gpu_if_available: true,
            gpu_device_id: None,
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