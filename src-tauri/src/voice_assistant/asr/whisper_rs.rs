use std::io::Cursor;
use std::path::Path;
use std::sync::{Arc, Mutex};
use whisper_rs::{WhisperContext, FullParams, SamplingStrategy, WhisperContextParameters};
use crate::voice_assistant::{AsrProcessor, Mode, VoiceError};
use std::time::Instant;
use serde_json;

#[derive(Debug, Clone)]
pub enum SamplingStrategyConfig {
    Greedy { best_of: u32 },
    Beam { beam_size: u32, patience: f32 },
}

#[derive(Debug, Clone)]
pub enum OutputFormat {
    Text,    // 纯文本
    Json,    // JSON格式
    Srt,     // SRT字幕
    Vtt,     // VTT字幕
    Csv,     // CSV格式
}

/// 段落数据结构
#[derive(Debug, Clone)]
pub struct SegmentData {
    pub text: String,
    pub start_ms: u64,
    pub end_ms: u64,
    pub index: i32,
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
    pub output_format: OutputFormat, // 🔥 NEW: 输出格式控制
}

pub struct WhisperRSProcessor {
    ctx: Option<Arc<WhisperContext>>,
    config: WhisperRSConfig,
    // VAD flag for basic energy-based VAD (thread-safe alternative)
    enable_basic_vad: bool,
    // For thread-safe access if needed
    _state_guard: Mutex<()>,
}

impl WhisperRSProcessor {
    pub fn new(config: WhisperRSConfig) -> Result<Self, VoiceError> {
        println!("📍 [DEBUG] Step A: new() called with model: {}", config.model_path);

        // Check if model file exists
        println!("📍 [DEBUG] Step B: Checking if model file exists...");
        if !Path::new(&config.model_path).exists() {
            return Err(VoiceError::Other(format!(
                "Whisper model file not found: {}",
                config.model_path
            )));
        }
        println!("📍 [DEBUG] Step C: Model file exists");

        // 设置GPU后端参数
        println!("🔧 Initializing Whisper with backend: {:?}", config.backend);

        println!("📍 [DEBUG] Step D: Creating WhisperContextParameters...");
        let params = WhisperContextParameters::default();
        println!("📍 [DEBUG] Step E: Parameters created");

        // 根据配置的后端设置参数
        match config.backend {
            WhisperBackend::CUDA => {
                println!("🚀 Initializing CUDA backend for GPU acceleration");

                // 设置CUDA设备ID（如果指定）
                if let Some(device_id) = config.gpu_device_id {
                    // whisper-rs通过环境变量设置CUDA设备
                    std::env::set_var("CUDA_VISIBLE_DEVICES", device_id.to_string());
                    println!("📱 Using CUDA device ID: {}", device_id);
                }

                // 注意：当前版本使用CPU后端，CUDA支持需要重新编译
                println!("⚠️ CUDA backend requested but running in CPU mode");
                println!("💡 To enable CUDA, recompile with: cargo build --features cuda");
            }
            WhisperBackend::Vulkan => {
                println!("⚠️ Vulkan backend requested but running in CPU mode");
                println!("💡 To enable Vulkan, recompile with: cargo build --features vulkan");
            }
            WhisperBackend::Metal => {
                println!("⚠️ Metal backend requested but running in CPU mode");
                println!("💡 To enable Metal, recompile with: cargo build --features metal");
            }
            WhisperBackend::OpenCL => {
                println!("⚠️ OpenCL backend requested but running in CPU mode");
                println!("💡 OpenCL support not available in current build");
            }
            WhisperBackend::CPU => {
                println!("💻 Using CPU backend");
            }
        }

        // Create whisper context
        println!("📍 [DEBUG] Step F: About to call WhisperContext::new_with_params...");
        println!("📍 [DEBUG] Step F-1: Model path: {}", config.model_path);
        println!("📍 [DEBUG] Step F-2: This is where it likely hangs...");

        let ctx = WhisperContext::new_with_params(
            &config.model_path,
            params,
        ).map_err(|e| {
            VoiceError::Other(format!("Failed to load whisper model: {}", e))
        })?;

        println!("📍 [DEBUG] Step G: WhisperContext created successfully");

        // 验证实际使用的后端
        println!("✅ Whisper context created successfully");

        // 如果GPU后端初始化失败但请求了GPU，提供fallback建议
        if config.use_gpu_if_available && config.backend != WhisperBackend::CPU {
            println!("⚠️ Requested GPU backend but currently using CPU backend");
            println!("💡 To enable GPU acceleration:");
            println!("   1. Install NVIDIA GPU drivers");
            println!("   2. Install CUDA Toolkit (for CUDA support)");
            println!("   3. Recompile with GPU features:");
            println!("      cargo build --release --features cuda");
            println!("   4. Check CUDA installation guide");
        }

        // Initialize VAD functionality
        println!("📍 [DEBUG] Step H: Initializing VAD...");
        let enable_basic_vad = if config.enable_vad {
            println!("🎯 Enabling basic energy-based VAD (thread-safe alternative)");
            true
        } else {
            false
        };

        println!("📍 [DEBUG] Step I: Creating processor struct...");
        Ok(Self {
            ctx: Some(Arc::new(ctx)),
            config,
            enable_basic_vad,
            _state_guard: Mutex::new(()),
        })
    }

    /// 显式卸载模型并释放GPU内存
    pub fn unload(&mut self) {
        if self.ctx.is_some() {
            println!("🗑️ WhisperRS: Explicitly unloading model...");
            // Drop the context - this will trigger whisper_free
            self.ctx = None;
            println!("✅ WhisperRS: Model unloaded, GPU memory should be released");
            // 注意：CUDA 运行时可能会缓存内存，内存可能不会立即返回给操作系统
            // 这是 CUDA 的正常行为，内存会在需要时或进程退出时释放
        }
    }

    pub fn from_env() -> Result<Self, VoiceError> {
        let model_path = std::env::var("WHISPER_MODEL_PATH")
            .unwrap_or_else(|_| {
                // Default model path - user should set this environment variable
                "./models/ggml-base.bin".to_string()
            });

        // 🔥 简化：直接使用CPU后端，避免GPU detector死锁
        let config = WhisperRSConfig {
            model_path,
            sampling_strategy: SamplingStrategyConfig::Greedy { best_of: 1 },
            language: None, // Auto-detect
            translate: false,
            enable_vad: false, // Default VAD disabled
            backend: WhisperBackend::CPU,
            use_gpu_if_available: false,
            gpu_device_id: None,
            output_format: OutputFormat::Text,
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
                    println!("🌐 Translation mode: forcing output language to English");
                } else {
                    params.set_language(None);
                    println!("🎤 Transcription mode: auto-detecting language");
                }
            }
        }

        // Set translation flag
        let should_translate = matches!(mode, Mode::Translations) || self.config.translate;
        params.set_translate(should_translate);
        println!("🔄 Translation flag set to: {}", should_translate);
        println!("📋 Mode: {:?}, Config.translate: {}", mode, self.config.translate);

        // Disable printing to reduce noise
        params.set_print_timestamps(false);
        params.set_print_special(false);
        params.set_print_progress(false);
        params.set_print_realtime(false);

        // Performance settings
        // Translation requires higher temperature to avoid repetition loops
        // Transcription uses 0.0 for accuracy, translation uses 0.2 for better results
        let temperature = if matches!(mode, Mode::Translations) {
            0.2f32  // Higher temperature for translation to prevent repetition
        } else {
            0.0f32  // Greedy decoding for transcription accuracy
        };
        params.set_temperature(temperature);
        println!("🌡️ Temperature set to: {} (mode: {:?})", temperature, mode);

        params.set_max_initial_ts(1_000_000.0); // Set to large value to disable

        // Enable prompt caching for better performance on subsequent runs
        params.set_no_context(false);

        params
    }

    #[allow(dead_code)]
    fn process_audio_data(&self, audio_data: &[f32]) -> Result<String, VoiceError> {
        // 🔥 使用配置的翻译模式
        let mode = if self.config.translate {
            Mode::Translations
        } else {
            Mode::Transcriptions
        };
        self.process_audio_data_with_mode(audio_data, mode)
    }

    /// 🔥 使用指定的mode处理音频
    fn process_audio_data_with_mode(&self, audio_data: &[f32], mode: Mode) -> Result<String, VoiceError> {
        let start_time = Instant::now();

        // Create a new state for each processing request
        let ctx = self.ctx.as_ref().ok_or_else(|| VoiceError::Other("WhisperContext not loaded".to_string()))?;
        let mut state = ctx.create_state()
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

        // 🔥 关键：使用传入的mode参数，而不是config.translate
        let params = self.create_params(mode);

        // 🔥 DEBUG: 打印参数设置
        println!("🔍 [DEBUG] About to run whisper inference:");
        println!("   Mode: {:?}", mode);
        println!("   Config.translate: {}", self.config.translate);
        println!("   Audio length: {} samples", final_audio.len());

        // Run inference
        state.full(params, &final_audio)
            .map_err(|e| VoiceError::Other(format!("Whisper inference failed: {}", e)))?;

        // 🔥 根据配置的输出格式处理结果
        let formatted_result = self.format_transcription(&state, &self.config.output_format)?;

        let processing_time = start_time.elapsed();
        let audio_duration = final_audio.len() as f32 / 16000.0;
        let real_time_factor = processing_time.as_secs_f32() / audio_duration;

        println!("🎯 WhisperRS processing completed in {:?}", processing_time);
        println!("⏱️ Audio duration: {:.2}s, Real-time factor: {:.2}x", audio_duration, real_time_factor);
        println!("📄 Output format: {:?}", self.config.output_format);

        Ok(formatted_result)
    }

    /// 🔥 NEW: 根据指定格式格式化转录结果
    fn format_transcription(
        &self,
        state: &whisper_rs::WhisperState,
        output_format: &OutputFormat,
    ) -> Result<String, VoiceError> {
        // 获取所有段落数据
        let num_segments = state
            .full_n_segments()
            .map_err(|e| VoiceError::Other(format!("Failed to get number of segments: {}", e)))?;

        let mut segments = Vec::with_capacity(num_segments as usize);

        // 收集所有段落信息
        for i in 0..num_segments {
            let segment_text = state
                .full_get_segment_text(i)
                .map_err(|e| VoiceError::Other(format!("Failed to get segment text: {}", e)))?;

            let segment_start = state
                .full_get_segment_t0(i)
                .map_err(|e| VoiceError::Other(format!("Failed to get segment start time: {}", e)))?;

            let segment_end = state
                .full_get_segment_t1(i)
                .map_err(|e| VoiceError::Other(format!("Failed to get segment end time: {}", e)))?;

            segments.push(SegmentData {
                text: segment_text.trim().to_string(),
                start_ms: (segment_start as u64) * 10, // whisper uses 100ms units
                end_ms: (segment_end as u64) * 10,
                index: i,
            });
        }

        // 根据格式生成输出
        match output_format {
            OutputFormat::Text => Ok(self.format_as_text(&segments)),
            OutputFormat::Json => Ok(self.format_as_json(&segments)),
            OutputFormat::Srt => Ok(self.format_as_srt(&segments)),
            OutputFormat::Vtt => Ok(self.format_as_vtt(&segments)),
            OutputFormat::Csv => Ok(self.format_as_csv(&segments)),
        }
    }

    
    /// 格式化为纯文本
    fn format_as_text(&self, segments: &[SegmentData]) -> String {
        segments
            .iter()
            .filter(|seg| !seg.text.is_empty())
            .map(|seg| seg.text.trim())
            .collect::<Vec<_>>()
            .join(" ")
    }

    /// 格式化为JSON
    fn format_as_json(&self, segments: &[SegmentData]) -> String {
        let json_segments: Vec<serde_json::Value> = segments
            .iter()
            .filter(|seg| !seg.text.is_empty())
            .map(|seg| serde_json::json!({
                "text": seg.text,
                "start": seg.start_ms,
                "end": seg.end_ms
            }))
            .collect();

        serde_json::json!({
            "text": self.format_as_text(segments),
            "segments": json_segments
        }).to_string()
    }

    /// 格式化为SRT
    fn format_as_srt(&self, segments: &[SegmentData]) -> String {
        let mut srt_content = String::new();
        let mut segment_counter = 1;

        for segment in segments.iter().filter(|seg| !seg.text.is_empty()) {
            let start_time = self.ms_to_srt_time(segment.start_ms);
            let end_time = self.ms_to_srt_time(segment.end_ms);

            srt_content.push_str(&format!(
                "{}\n{} --> {}\n{}\n\n",
                segment_counter,
                start_time,
                end_time,
                segment.text.trim()
            ));

            segment_counter += 1;
        }

        srt_content
    }

    /// 格式化为VTT
    fn format_as_vtt(&self, segments: &[SegmentData]) -> String {
        let mut vtt_content = String::new();
        vtt_content.push_str("WEBVTT\n\n");

        for segment in segments.iter().filter(|seg| !seg.text.is_empty()) {
            let start_time = self.ms_to_vtt_time(segment.start_ms);
            let end_time = self.ms_to_vtt_time(segment.end_ms);

            vtt_content.push_str(&format!(
                "{} --> {}\n{}\n\n",
                start_time,
                end_time,
                segment.text.trim()
            ));
        }

        vtt_content
    }

    /// 格式化为CSV
    fn format_as_csv(&self, segments: &[SegmentData]) -> String {
        let mut csv_content = String::new();
        csv_content.push_str("index,start_ms,end_ms,text\n");

        for segment in segments.iter().filter(|seg| !seg.text.is_empty()) {
            csv_content.push_str(&format!(
                "{},{},{},\"{}\"\n",
                segment.index,
                segment.start_ms,
                segment.end_ms,
                segment.text.trim()
            ));
        }

        csv_content
    }

    /// 转换毫秒为SRT时间格式 (HH:MM:SS,mmm)
    fn ms_to_srt_time(&self, ms: u64) -> String {
        let total_seconds = ms / 1000;
        let hours = total_seconds / 3600;
        let minutes = (total_seconds % 3600) / 60;
        let seconds = total_seconds % 60;
        let milliseconds = ms % 1000;

        format!(
            "{:02}:{:02}:{:02},{:03}",
            hours, minutes, seconds, milliseconds
        )
    }

    /// 转换毫秒为VTT时间格式 (HH:MM:SS.mmm)
    fn ms_to_vtt_time(&self, ms: u64) -> String {
        let total_seconds = ms / 1000;
        let hours = total_seconds / 3600;
        let minutes = (total_seconds % 3600) / 60;
        let seconds = total_seconds % 60;
        let milliseconds = ms % 1000;

        format!(
            "{:02}:{:02}:{:02}.{:03}",
            hours, minutes, seconds, milliseconds
        )
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
        mode: Mode,  // 🔥 使用传入的mode参数
        _prompt: &str,
    ) -> Result<String, VoiceError> {
        // Convert byte buffer to f32 audio samples
        let audio_data = self.convert_bytes_to_f32(audio_buffer.into_inner())?;

        // 🔥 关键修复：使用传入的mode参数，而不是config.translate
        println!("🔍 [ASR] process_audio called with mode: {:?}", mode);
        self.process_audio_data_with_mode(&audio_data, mode)
    }

    fn get_processor_type(&self) -> Option<&str> {
        Some("whisper-rs")
    }

    fn unload(&mut self) {
        self.unload();
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
        println!("📍 [DEBUG] Step 1: with_model_path called with: {}", model_path);

        // 🔥 简化：直接使用CPU后端，避免GPU detector死锁
        println!("📍 [DEBUG] Step 2: Using CPU backend (skipping GPU detection)");
        let backend = WhisperBackend::CPU;
        println!("📍 [DEBUG] Step 3: Backend: {:?}", backend);

        let config = WhisperRSConfig {
            model_path: model_path.to_string(),
            sampling_strategy: SamplingStrategyConfig::Greedy { best_of: 1 },
            language: None,
            translate: false,
            enable_vad: false,
            backend,
            use_gpu_if_available: false,
            gpu_device_id: None,
            output_format: OutputFormat::Text,
        };

        println!("📍 [DEBUG] Step 4: Config created, calling Self::new...");
        let result = Self::new(config);
        println!("📍 [DEBUG] Step 5: Self::new returned: {:?}", result.is_ok());
        result
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
            output_format: OutputFormat::Text,
        };
        Self::new(config)
    }

    pub fn with_language(model_path: &str, language: &str) -> Result<Self, VoiceError> {
        // 🔥 简化：直接使用CPU后端，避免GPU detector死锁
        let config = WhisperRSConfig {
            model_path: model_path.to_string(),
            sampling_strategy: SamplingStrategyConfig::Greedy { best_of: 1 },
            language: Some(language.to_string()),
            translate: false,
            enable_vad: false,
            backend: WhisperBackend::CPU,
            use_gpu_if_available: false,
            gpu_device_id: None,
            output_format: OutputFormat::Text,
        };
        Self::new(config)
    }

    pub fn with_beam_search(
        model_path: &str,
        beam_size: u32,
        patience: f32,
    ) -> Result<Self, VoiceError> {
        // 🔥 简化：直接使用CPU后端，避免GPU detector死锁
        let config = WhisperRSConfig {
            model_path: model_path.to_string(),
            sampling_strategy: SamplingStrategyConfig::Beam { beam_size, patience },
            language: None,
            translate: false,
            enable_vad: false,
            backend: WhisperBackend::CPU,
            use_gpu_if_available: false,
            gpu_device_id: None,
            output_format: OutputFormat::Text,
        };
        Self::new(config)
    }

    // Factory functions with VAD support
    pub fn with_model_path_and_vad(model_path: &str, enable_vad: bool) -> Result<Self, VoiceError> {
        // 🔥 简化：直接使用CPU后端，避免GPU detector死锁
        let config = WhisperRSConfig {
            model_path: model_path.to_string(),
            sampling_strategy: SamplingStrategyConfig::Greedy { best_of: 1 },
            language: None,
            translate: false,
            enable_vad,
            backend: WhisperBackend::CPU,
            use_gpu_if_available: false,
            gpu_device_id: None,
            output_format: OutputFormat::Text,
        };
        Self::new(config)
    }

    pub fn with_language_and_vad(model_path: &str, language: &str, enable_vad: bool) -> Result<Self, VoiceError> {
        // 🔥 简化：直接使用CPU后端，避免GPU detector死锁
        let config = WhisperRSConfig {
            model_path: model_path.to_string(),
            sampling_strategy: SamplingStrategyConfig::Greedy { best_of: 1 },
            language: Some(language.to_string()),
            translate: false,
            enable_vad,
            backend: WhisperBackend::CPU,
            use_gpu_if_available: false,
            gpu_device_id: None,
            output_format: OutputFormat::Text,
        };
        Self::new(config)
    }

    pub fn with_beam_search_and_vad(
        model_path: &str,
        beam_size: u32,
        patience: f32,
        enable_vad: bool,
    ) -> Result<Self, VoiceError> {
        // 🔥 简化：直接使用CPU后端，避免GPU detector死锁
        let config = WhisperRSConfig {
            model_path: model_path.to_string(),
            sampling_strategy: SamplingStrategyConfig::Beam { beam_size, patience },
            language: None,
            translate: false,
            enable_vad,
            backend: WhisperBackend::CPU,
            use_gpu_if_available: false,
            gpu_device_id: None,
            output_format: OutputFormat::Text,
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
            backend: WhisperBackend::CPU,
            use_gpu_if_available: false,
            gpu_device_id: None,
            output_format: OutputFormat::Text,
            enable_vad: false,
        };
        
        assert_eq!(config.model_path, "test.bin");
        assert!(matches!(config.sampling_strategy, SamplingStrategyConfig::Greedy { best_of: 1 }));
        assert_eq!(config.language, Some("en".to_string()));
        assert!(!config.translate);
    }
}