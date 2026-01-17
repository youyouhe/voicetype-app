use std::io::Cursor;
use std::io::Write;
use std::path::Path;
use std::sync::{Arc, Mutex};
use whisper_rs::{WhisperContext, FullParams, SamplingStrategy, WhisperContextParameters, WhisperState};
use crate::voice_assistant::{AsrProcessor, Mode, VoiceError};
use std::time::{Instant, SystemTime, UNIX_EPOCH};
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

    /// 启动流式会话
    fn start_streaming_session(&self, mode: Mode) -> Result<Box<dyn crate::voice_assistant::StreamingAsrSession>, VoiceError> {
        let ctx = self.ctx.as_ref()
            .ok_or_else(|| VoiceError::Other("WhisperContext not loaded".to_string()))?
            .clone();

        let session = WhisperStreamingSession::new(ctx, mode, &self.config)?;
        Ok(Box::new(session))
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

// ==================== Streaming Session Implementation ====================

/// VAD 段落信息
struct VadSegmentInfo {
    start_sample: usize,
    end_sample: usize,
    start_ms: u64,
    end_ms: u64,
    is_complete: bool,
}

/// 转录结果
struct TranscriptionResult {
    text: String,
    tokens: Vec<i32>,
}

/// Whisper 流式会话
#[allow(dead_code)]
pub struct WhisperStreamingSession {
    ctx: Arc<WhisperContext>,
    state: WhisperState,
    mode: Mode,
    params: FullParams<'static, 'static>,

    // VAD 配置
    vad_threshold: f32,              // 语音能量阈值 (默认 0.5)
    min_speech_duration_ms: u64,     // 最小语音时长 (默认 1000ms)
    min_silence_duration_ms: u64,    // 最小静默时长 (默认 2000ms)
    max_segment_length_ms: u64,      // 最大段落长度 (默认 30000ms)

    // 音频缓冲区
    audio_buffer: Vec<f32>,
    sample_rate: u32,
    last_speech_end: Option<usize>,

    // 状态跟踪
    in_speech: bool,
    speech_start_sample: usize,
    last_transcribed_sample: usize,

    // 上下文管理
    context_tokens: Vec<i32>,
    max_context_tokens: usize,

    // 🔥 方案3：待转录段落队列
    pending_segments: Vec<PendingSegment>,

    // 🔥 DEBUG: 调试计数器
    debug_chunk_counter: u64,
}

/// 🔥 待转录的段落
#[derive(Clone)]
struct PendingSegment {
    audio: Vec<f32>,
    start_ms: u64,
    end_ms: u64,
    start_sample: usize,
    end_sample: usize,
}

impl WhisperStreamingSession {
    pub fn new(ctx: Arc<WhisperContext>, mode: Mode, config: &WhisperRSConfig) -> Result<Self, VoiceError> {
        let state = ctx.create_state()
            .map_err(|e| VoiceError::Other(format!("Failed to create whisper state: {}", e)))?;

        let params = Self::create_streaming_params(mode, config);

        Ok(Self {
            ctx,
            state,
            mode,
            params,
            vad_threshold: 0.02f32,       // 🔥 降低阈值以适应低音量麦克风 (原0.5太高)
            min_speech_duration_ms: 1000,
            min_silence_duration_ms: 2000,
            max_segment_length_ms: 30000,
            audio_buffer: Vec::new(),
            sample_rate: 16000, // Whisper 期望 16kHz
            last_speech_end: None,
            in_speech: false,
            speech_start_sample: 0,
            last_transcribed_sample: 0,
            context_tokens: Vec::new(),
            max_context_tokens: 224, // Whisper 的上下文窗口
            pending_segments: Vec::new(),  // 🔥 初始化待转录队列
            debug_chunk_counter: 0,  // 🔥 调试用：音频块计数器
        })
    }

    fn create_streaming_params(_mode: Mode, _config: &WhisperRSConfig) -> FullParams<'static, 'static> {
        let mut params = FullParams::new(SamplingStrategy::Greedy { best_of: 1 });

        // 流式特定设置
        let num_threads = std::thread::available_parallelism()
            .map(|n| n.get() as i32)
            .unwrap_or(4);
        params.set_n_threads(num_threads);

        params.set_language(None); // 自动检测
        params.set_no_context(false); // 启用上下文连续性
        params.set_single_segment(false); // 允许多个段落

        params.set_print_timestamps(false);
        params.set_print_special(false);
        params.set_print_progress(false);
        params.set_print_realtime(false);

        params
    }

    /// 处理音频块
    pub fn process_audio_chunk(&mut self, audio_samples: &[f32], sample_rate: u32) -> Result<Vec<crate::voice_assistant::StreamingSegment>, VoiceError> {
        // 🔥 调试：保存每个原始音频块
        self.debug_save_audio_chunk(audio_samples, &format!("input_{}hz", sample_rate));

        // 1. 重采样到 16kHz
        let resampled = self.resample_to_16khz(audio_samples, sample_rate);

        // 2. 追加到缓冲区
        let start_idx = self.audio_buffer.len();
        self.audio_buffer.extend_from_slice(&resampled);

        // 3. 运行 VAD 检测语音段落
        let speech_segments = self.detect_vad_segments(start_idx)?;

        // 4. 🔥 方案3：智能转录策略 - 只在没有新语音时才转录
        let mut results = Vec::new();

        for segment in speech_segments {
            if segment.is_complete {
                // 检查是否有新语音正在发生（最后500ms）
                let has_recent_speech = self.check_recent_speech(500);

                if has_recent_speech {
                    // 有新语音，将当前段落加入待转录队列
                    println!("⏸️ VAD: New speech detected, queuing segment {}-{}ms for later transcription",
                        segment.start_ms, segment.end_ms);

                    let segment_audio: Vec<f32> = self.audio_buffer[segment.start_sample..segment.end_sample].to_vec();

                    // 🔥 调试：保存队列中的VAD段落
                    self.debug_save_audio_segment(&segment_audio, segment.start_ms, segment.end_ms, "queued");

                    self.pending_segments.push(PendingSegment {
                        audio: segment_audio,
                        start_ms: segment.start_ms,
                        end_ms: segment.end_ms,
                        start_sample: segment.start_sample,
                        end_sample: segment.end_sample,
                    });
                } else {
                    // 没有新语音，可以安全转录
                    println!("🎯 VAD: No new speech, transcribing segment {}-{}ms",
                        segment.start_ms, segment.end_ms);

                    // Copy audio segment to avoid borrow conflict
                    let segment_audio: Vec<f32> = self.audio_buffer[segment.start_sample..segment.end_sample].to_vec();

                    // 🔥 调试：保存立即转录的VAD段落
                    self.debug_save_audio_segment(&segment_audio, segment.start_ms, segment.end_ms, "immediate");

                    match self.transcribe_segment(&segment_audio) {
                        Ok(transcription) => {
                            if !transcription.text.is_empty() {
                                // 更新上下文
                                self.context_tokens.extend_from_slice(&transcription.tokens);
                                if self.context_tokens.len() > self.max_context_tokens {
                                    let keep = self.max_context_tokens / 2;
                                    self.context_tokens = self.context_tokens[keep..].to_vec();
                                }
                                self.last_transcribed_sample = segment.end_sample;

                                results.push(crate::voice_assistant::StreamingSegment {
                                    text: transcription.text,
                                    start_ms: segment.start_ms,
                                    end_ms: segment.end_ms,
                                    is_final: true,
                                    should_type: true,
                                });
                            }
                        }
                        Err(e) => {
                            eprintln!("Segment transcription failed: {}", e);
                        }
                    }
                }
            }
        }

        // 5. 🔥 尝试处理待转录队列（最多1个，避免阻塞）
        if !self.pending_segments.is_empty() && !self.check_recent_speech(500) {
            // 先取出待处理的段落（避免借用冲突）
            let pending_to_process = self.pending_segments.drain(0..1).next();

            if let Some(pending) = pending_to_process {
                println!("🔄 VAD: Processing pending segment {}-{}ms", pending.start_ms, pending.end_ms);

                // 🔥 调试：保存从队列中取出的VAD段落
                self.debug_save_audio_segment(&pending.audio, pending.start_ms, pending.end_ms, "pending");

                match self.transcribe_segment(&pending.audio) {
                    Ok(transcription) => {
                        if !transcription.text.is_empty() {
                            self.context_tokens.extend_from_slice(&transcription.tokens);
                            if self.context_tokens.len() > self.max_context_tokens {
                                let keep = self.max_context_tokens / 2;
                                self.context_tokens = self.context_tokens[keep..].to_vec();
                            }
                            self.last_transcribed_sample = pending.end_sample;

                            results.push(crate::voice_assistant::StreamingSegment {
                                text: transcription.text,
                                start_ms: pending.start_ms,
                                end_ms: pending.end_ms,
                                is_final: true,
                                should_type: true,
                            });
                        }
                    }
                    Err(e) => {
                        eprintln!("Pending segment transcription failed: {}", e);
                    }
                }
            }
        }

        // 6. 清理旧音频数据（保留最近 60 秒作为上下文）
        if self.audio_buffer.len() > self.sample_rate as usize * 60 {
            let keep_len = self.sample_rate as usize * 60;
            let remove_len = self.audio_buffer.len() - keep_len;
            self.audio_buffer = self.audio_buffer[remove_len..].to_vec();
            self.last_transcribed_sample = self.last_transcribed_sample.saturating_sub(remove_len);
            if let Some(ref mut last_speech_end) = self.last_speech_end {
                *last_speech_end = last_speech_end.saturating_sub(remove_len);
            }
        }

        Ok(results)
    }

    /// 🔥 检查最近N毫秒是否有语音活动
    fn check_recent_speech(&self, recent_ms: usize) -> bool {
        let recent_samples = (self.sample_rate as usize * recent_ms / 1000).min(self.audio_buffer.len());

        if self.audio_buffer.len() < recent_samples {
            return false;
        }

        let start_idx = self.audio_buffer.len() - recent_samples;
        let window_size = (self.sample_rate as f64 * 0.1) as usize; // 100ms窗口

        // 检查最后几个窗口是否有高能量
        let mut high_energy_count = 0;
        let mut total_windows = 0;

        for i in (start_idx..self.audio_buffer.len()).step_by(window_size / 2) {
            if i + window_size > self.audio_buffer.len() {
                break;
            }

            let window = &self.audio_buffer[i..i + window_size];
            let energy = self.calculate_energy(window);
            total_windows += 1;

            if energy > self.vad_threshold {
                high_energy_count += 1;
            }
        }

        // 如果超过30%的窗口有高能量，认为有新语音
        let has_speech = total_windows > 0 && (high_energy_count as f32 / total_windows as f32) > 0.3;

        if has_speech {
            println!("🔊 Recent speech check: {}/{} windows have high energy", high_energy_count, total_windows);
        }

        has_speech
    }

    /// 🔥 调试：保存原始音频块到日志目录
    fn debug_save_audio_chunk(&mut self, audio: &[f32], label: &str) {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis();

        let filename = format!("logs/streaming_debug/chunk_{timestamp}_{label}.wav");
        self.debug_chunk_counter += 1;

        // 创建debug目录
        if let Err(e) = std::fs::create_dir_all("logs/streaming_debug") {
            eprintln!("Failed to create debug directory: {}", e);
            return;
        }

        // 写入WAV文件
        if let Err(e) = self.write_wav_file(&filename, audio, 16000) {
            eprintln!("Failed to save audio chunk: {}", e);
        } else {
            println!("💾 Saved audio chunk: {} ({} samples, label: {})", filename, audio.len(), label);
        }
    }

    /// 🔥 调试：保存VAD检测到的语音段落到日志目录
    fn debug_save_audio_segment(&self, audio: &[f32], start_ms: u64, end_ms: u64, label: &str) {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis();

        let filename = format!("logs/streaming_debug/segment_{timestamp}_{start_ms}-{end_ms}ms_{label}.wav");

        // 创建debug目录
        if let Err(e) = std::fs::create_dir_all("logs/streaming_debug") {
            eprintln!("Failed to create debug directory: {}", e);
            return;
        }

        // 写入WAV文件
        if let Err(e) = self.write_wav_file(&filename, audio, 16000) {
            eprintln!("Failed to save audio segment: {}", e);
        } else {
            println!("💾 Saved VAD segment: {} ({} samples, {}ms)", filename, audio.len(), end_ms - start_ms);
        }
    }

    /// 🔥 辅助：写入WAV文件
    fn write_wav_file(&self, path: &str, audio: &[f32], sample_rate: u32) -> Result<(), std::io::Error> {
        let mut file = std::fs::File::create(path)?;

        // WAV header
        let num_channels = 1u16;
        let bits_per_sample = 16u16;
        let byte_rate = sample_rate * num_channels as u32 * bits_per_sample as u32 / 8;
        let block_align = num_channels * bits_per_sample / 8;
        let data_size = audio.len() * 2;
        let file_size = 36 + data_size as u32;

        // RIFF header
        file.write_all(b"RIFF")?;
        file.write_all(&file_size.to_le_bytes())?;
        file.write_all(b"WAVE")?;

        // fmt chunk
        file.write_all(b"fmt ")?;
        file.write_all(&16u32.to_le_bytes())?; // chunk size
        file.write_all(&1u16.to_le_bytes())?; // audio format (PCM)
        file.write_all(&num_channels.to_le_bytes())?;
        file.write_all(&sample_rate.to_le_bytes())?;
        file.write_all(&byte_rate.to_le_bytes())?;
        file.write_all(&block_align.to_le_bytes())?;
        file.write_all(&bits_per_sample.to_le_bytes())?;

        // data chunk
        file.write_all(b"data")?;
        file.write_all(&data_size.to_le_bytes())?;

        // audio data (convert f32 [-1,1] to i16)
        for &sample in audio {
            let i16_sample = (sample.clamp(-1.0, 1.0) * 32767.0) as i16;
            file.write_all(&i16_sample.to_le_bytes())?;
        }

        Ok(())
    }

    /// VAD 检测语音段落
    fn detect_vad_segments(&mut self, _start_idx: usize) -> Result<Vec<VadSegmentInfo>, VoiceError> {
        let mut segments = Vec::new();

        // 使用 100ms 窗口处理音频
        let window_size = (self.sample_rate as f64 * 0.1) as usize; // 100ms
        let step_size = window_size / 2; // 50% 重叠

        if self.audio_buffer.len() < window_size {
            return Ok(segments);
        }

        let mut i = self.last_transcribed_sample;
        let mut debug_count = 0;

        while i + window_size <= self.audio_buffer.len() {
            let window = &self.audio_buffer[i..i + window_size];
            let energy = self.calculate_energy(window);

            // 每10个窗口打印一次调试信息
            if debug_count % 10 == 0 {
                println!("🎵 VAD: window[{}], energy={:.4}, threshold={:.4}, in_speech={}",
                    debug_count, energy, self.vad_threshold, self.in_speech);
            }
            debug_count += 1;

            if energy > self.vad_threshold {
                // 检测到语音
                if !self.in_speech {
                    println!("✅ VAD: Speech START detected at sample {} (energy={:.4})", i, energy);
                    self.in_speech = true;
                    self.speech_start_sample = i;
                }
            } else {
                // 检测到静默
                if self.in_speech {
                    let silence_samples = (self.sample_rate as f64 * (self.min_silence_duration_ms as f64 / 1000.0)) as usize;

                    // 检查静默持续时间是否足够长
                    if i > self.speech_start_sample + silence_samples {
                        // 计算最小语音时长要求
                        let min_speech_samples = (self.sample_rate as f64 * (self.min_speech_duration_ms as f64 / 1000.0)) as usize;

                        if i - self.speech_start_sample >= min_speech_samples {
                            // 语音段落完成
                            let start_ms = (self.speech_start_sample as f64 / self.sample_rate as f64 * 1000.0) as u64;
                            let end_ms = (i as f64 / self.sample_rate as f64 * 1000.0) as u64;

                            println!("✅ VAD: Segment COMPLETE: {}-{}ms (samples: {}-{})",
                                start_ms, end_ms, self.speech_start_sample, i);

                            segments.push(VadSegmentInfo {
                                start_sample: self.speech_start_sample,
                                end_sample: i,
                                start_ms,
                                end_ms,
                                is_complete: true,
                            });

                            self.last_speech_end = Some(i);
                        }

                        self.in_speech = false;
                    }
                }
            }

            i += step_size;
        }

        Ok(segments)
    }

    fn calculate_energy(&self, samples: &[f32]) -> f32 {
        if samples.is_empty() {
            return 0.0;
        }
        let sum: f32 = samples.iter().map(|&x| x * x).sum();
        (sum / samples.len() as f32).sqrt()
    }

    fn resample_to_16khz(&self, audio: &[f32], original_rate: u32) -> Vec<f32> {
        if original_rate == 16000 {
            return audio.to_vec();
        }

        let ratio = 16000.0 / original_rate as f32;
        let new_len = (audio.len() as f32 * ratio).ceil() as usize;
        let mut resampled = Vec::with_capacity(new_len);

        for i in 0..new_len {
            let src_idx = (i as f32 / ratio) as usize;
            resampled.push(audio[src_idx.min(audio.len() - 1)]);
        }

        resampled
    }

    fn transcribe_segment(&mut self, audio: &[f32]) -> Result<TranscriptionResult, VoiceError> {
        // 运行推理 (WhisperState will be reused, each full() call starts fresh)
        self.state.full(self.params.clone(), audio)
            .map_err(|e| VoiceError::Other(format!("Whisper inference failed: {}", e)))?;

        // 提取文本
        let num_segments = self.state.full_n_segments()
            .map_err(|e| VoiceError::Other(format!("Failed to get number of segments: {}", e)))?;

        let mut text = String::new();
        let tokens = Vec::new();

        for i in 0..num_segments {
            let segment_text = self.state.full_get_segment_text(i)
                .map_err(|e| VoiceError::Other(format!("Failed to get segment text: {}", e)))?;

            text.push_str(&segment_text);
            text.push(' ');

            // TODO: 收集 tokens (需要查看 whisper-rs API)
        }

        Ok(TranscriptionResult {
            text: text.trim().to_string(),
            tokens,
        })
    }

    /// 结束会话（处理剩余音频）
    pub fn finalize(&mut self) -> Result<String, VoiceError> {
        if self.in_speech && self.audio_buffer.len() > self.speech_start_sample {
            // Copy audio to avoid borrow conflict
            let remaining_audio: Vec<f32> = self.audio_buffer[self.speech_start_sample..].to_vec();
            let result = self.transcribe_segment(&remaining_audio)?;
            Ok(result.text)
        } else {
            Ok(String::new())
        }
    }

    /// 获取当前上下文 tokens
    pub fn get_context_tokens(&self) -> Vec<i32> {
        self.context_tokens.clone()
    }
}

// 实现 StreamingAsrSession trait
impl crate::voice_assistant::StreamingAsrSession for WhisperStreamingSession {
    fn process_audio_chunk(&mut self, audio_samples: &[f32], sample_rate: u32) -> Result<Vec<crate::voice_assistant::StreamingSegment>, VoiceError> {
        self.process_audio_chunk(audio_samples, sample_rate)
    }

    fn finalize(&mut self) -> Result<String, VoiceError> {
        self.finalize()
    }

    fn get_context_tokens(&self) -> Vec<i32> {
        self.get_context_tokens()
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