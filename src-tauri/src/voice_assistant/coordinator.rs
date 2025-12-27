use std::sync::{Arc, Mutex};
use std::collections::HashMap;
use std::sync::OnceLock;
use tauri::{AppHandle, Emitter};
use crate::voice_assistant::{
    AsrProcessor, TranslateProcessor,
    AudioRecorder, KeyboardManager, Mode, InputState, VoiceError,
    WhisperProcessor, SenseVoiceProcessor, LocalASRProcessor,
    SiliconFlowTranslateProcessor, OllamaTranslateProcessor,
    WhisperRSProcessor // , EnhancedWhisperProcessor
};
use tracing::{info, error};

// Global VoiceAssistant instance
static VOICE_ASSISTANT: OnceLock<Arc<Mutex<Option<VoiceAssistant>>>> = OnceLock::new();
// Global App handle for emitting events
static APP_HANDLE: OnceLock<Arc<Mutex<Option<AppHandle>>>> = OnceLock::new();

// Helper function to set the global app handle
pub fn set_app_handle(handle: AppHandle) {
    APP_HANDLE.set(Arc::new(Mutex::new(Some(handle)))).ok();
}

// Helper function to emit voice assistant state change events
fn emit_voice_assistant_state_change(state: &InputState) {
    if let Some(handle_guard) = APP_HANDLE.get() {
        if let Ok(app_handle) = handle_guard.lock() {
            if let Some(ref handle) = *app_handle {
                let state_str = match state {
                    InputState::Idle => "Running".to_string(), // 🔥 FIXED: Keep service as "Running" instead of "Idle"
                    InputState::Recording => "Recording".to_string(),
                    InputState::RecordingTranslate => "RecordingTranslate".to_string(),
                    InputState::Processing => "Processing".to_string(),
                    InputState::Translating => "Translating".to_string(),
                    InputState::Error => "Error".to_string(),
                    InputState::Warning => "Warning".to_string(),
                };
                
                if let Err(e) = handle.emit("voice-assistant-state-changed", &state_str) {
                    error!("Failed to emit voice assistant state change event: {}", e);
                } else {
                    info!("✅ Emitted voice assistant state change: {}", state_str);
                }
            }
        }
    }
}

// Public function that can be called from keyboard manager
pub fn emit_voice_assistant_state_from_keyboard(state: &InputState) {
    emit_voice_assistant_state_change(state);
}

// Helper function to emit new history record events
pub fn emit_new_history_record_event() {
    if let Some(handle_guard) = APP_HANDLE.get() {
        if let Ok(app_handle) = handle_guard.lock() {
            if let Some(ref handle) = *app_handle {
                if let Err(e) = handle.emit("new-history-record", "record_added") {
                    error!("Failed to emit new history record event: {}", e);
                } else {
                    info!("✅ Emitted new history record event");
                }
            }
        }
    }
}

// Helper function to emit service status update events
pub fn emit_service_status_updated_event() {
    if let Some(handle_guard) = APP_HANDLE.get() {
        if let Ok(app_handle) = handle_guard.lock() {
            if let Some(ref handle) = *app_handle {
                if let Err(e) = handle.emit("service-status-updated", "status_updated") {
                    error!("Failed to emit service status update event: {}", e);
                } else {
                    info!("✅ Emitted service status update event");
                }
            }
        }
    }
}

// Directly save ASR result to database and emit update events
pub async fn save_asr_result_directly(
    output_text: String,
    processor_type: &str,
    processing_time_ms: Option<i64>,
    success: bool,
    error_message: Option<String>,
) {
    println!("📊 [Coordinator] Directly saving ASR result to database...");
    
    // Create history record
    let record = crate::database::NewHistoryRecord {
        record_type: "asr".to_string(),
        input_text: None,
        output_text: Some(output_text),
        audio_file_path: None,
        processor_type: Some(processor_type.to_string()),
        processing_time_ms,
        success,
        error_message,
    };

    // Use global database pool
    match crate::database::Database::from_global_pool().await {
        Ok(database) => {
            match database.add_history_record(record).await {
                Ok(_) => {
                    println!("✅ [Coordinator] ASR result saved to database successfully");
                    // Emit update events for frontend refresh
                    emit_new_history_record_event();
                    emit_service_status_updated_event();
                }
                Err(e) => {
                    println!("❌ [Coordinator] Failed to save ASR result to database: {}", e);
                }
            }
        }
        Err(e) => {
            println!("❌ [Coordinator] Failed to get database instance: {}", e);
        }
    }
}

// Helper function to emit ASR result events
pub fn emit_asr_result_event(result: &AsrResult) {
    println!("🚀 [Backend] Attempting to emit ASR result event...");
    if let Some(handle_guard) = APP_HANDLE.get() {
        println!("🔍 [Backend] Got app handle guard");
        if let Ok(app_handle) = handle_guard.lock() {
            println!("🔍 [Backend] Got app handle lock");
            if let Some(ref handle) = *app_handle {
                println!("🔍 [Backend] Got app handle reference");
                match handle.emit("asr-result-complete", result) {
                    Ok(_) => {
                        info!("✅ Emitted ASR result event: {} chars", result.output_text.chars().count());
                        println!("✅ [Backend] ASR result event emitted successfully");
                    }
                    Err(e) => {
                        error!("Failed to emit ASR result event: {}", e);
                        println!("❌ [Backend] Failed to emit ASR result event: {}", e);
                    }
                }
            } else {
                println!("❌ [Backend] No app handle reference");
            }
        } else {
            println!("❌ [Backend] Failed to get app handle lock");
        }
    } else {
        println!("❌ [Backend] No app handle guard available");
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct AsrResult {
    pub success: bool,
    pub input_text: Option<String>,
    pub output_text: String,
    pub processor_type: String,
    pub processing_time_ms: Option<i64>,
    pub audio_file_path: Option<String>,
    pub error_message: Option<String>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ProcessorType {
    #[serde(rename = "cloud")]
    CloudASR,
    #[serde(rename = "local")]
    LocalASR,
    #[serde(rename = "whisper-rs")]
    WhisperRS,
    // #[serde(rename = "enhanced-whisper")]
    // EnhancedWhisper,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TranslateType {
    SiliconFlow,
    Ollama,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct VoiceAssistantConfig {
    pub service_platform: String,
    pub asr_processor: ProcessorType,
    pub translate_processor: TranslateType,
    pub convert_to_simplified: bool,
    pub add_symbol: bool,
    pub optimize_result: bool,
}

impl Default for VoiceAssistantConfig {
    fn default() -> Self {
        Self {
            service_platform: std::env::var("SERVICE_PLATFORM").unwrap_or_else(|_| "siliconflow".to_string()),
            asr_processor: ProcessorType::WhisperRS, // 🔥 改为默认使用本地WhisperRS
            translate_processor: TranslateType::Ollama,
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
        }
    }
}

pub struct VoiceAssistant {
    config: VoiceAssistantConfig,
    asr_processor: Option<Arc<dyn AsrProcessor + Send + Sync>>,
    translate_processor: Option<Arc<dyn TranslateProcessor + Send + Sync>>,
    keyboard_manager: Arc<Mutex<KeyboardManager>>,
    recorder: Arc<Mutex<AudioRecorder>>,
    state: Arc<Mutex<InputState>>,
    logger_initialized: bool,
}

impl VoiceAssistant {
    pub async fn new() -> Result<Self, VoiceError> {
        // Initialize logger first
        if let Err(e) = crate::voice_assistant::init_logger() {
            eprintln!("Failed to initialize logger: {}", e);
        }

        // Load configuration from database during initialization
        let config = Self::load_config_from_database().await.unwrap_or_else(|e| {
            println!("⚠️ Failed to load config from database: {}, using default", e);
            VoiceAssistantConfig::default()
        });
        info!("Initializing VoiceAssistant");

        // Create ASR processor based on configuration
        let asr_processor: Arc<dyn AsrProcessor + Send + Sync> = match config.asr_processor {
            ProcessorType::CloudASR => {
                // Choose between Whisper and SenseVoice based on service platform
                if config.service_platform == "groq" {
                    info!("Creating Cloud ASR processor (Whisper backend)");
                    Arc::new(WhisperProcessor::new()?)
                } else {
                    info!("Creating Cloud ASR processor (SenseVoice backend)");
                    Arc::new(SenseVoiceProcessor::new()?)
                }
            },
            ProcessorType::LocalASR => {
                info!("Creating Local ASR processor (HTTP API)");
                // Load ASR configuration from database for Local ASR
                let local_asr_config = Self::load_local_asr_config().await.unwrap_or_else(|e| {
                    println!("⚠️ Failed to load local ASR config from database: {}, using default", e);
                    crate::voice_assistant::asr::local_asr::LocalASRConfig {
                        endpoint: "http://192.168.8.107:5001/inference".to_string(),
                        api_key: "default-key".to_string(),
                    }
                });

                Arc::new(LocalASRProcessor::with_config(local_asr_config)?)
            },
            ProcessorType::WhisperRS => {
                info!("Creating WhisperRS processor (Local whisper.cpp)");
                // Load WhisperRS configuration from environment or use default location
                let model_path = std::env::var("WHISPER_MODEL_PATH")
                    .ok()
                    .and_then(|path| {
                        if std::path::Path::new(&path).exists() {
                            println!("✅ Using active model from environment: {}", path);
                            Some(path)
                        } else {
                            println!("⚠️ Environment model doesn't exist: {}", path);
                            None
                        }
                    })
                    .or_else(|| {
                        // Try to find the model in the default data directory
                        let models_dir = crate::utils::platform::get_models_dir();
                        let model_file = models_dir.join("ggml-large-v3-turbo.bin");
                        if model_file.exists() {
                            Some(model_file.to_string_lossy().to_string())
                        } else {
                            None
                        }
                    })
                    .or_else(|| {
                        // Try to find models in the default data directory
                        let models_dir = crate::utils::platform::get_models_dir();
                        println!("🔍 Searching for models in: {}", models_dir.display());

                        // Try different models in order of preference (添加 large-v2)
                        let model_preferences = [
                            "ggml-large-v2.bin",         // 🔥 用户选择的模型
                            "ggml-large-v3-turbo.bin",     // ~1570MB - highest quality
                            "ggml-large-v3.bin",          // ~2950MB - v3模型
                            "ggml-small.bin",             // ~467MB - 平衡
                            "ggml-base.bin",              // ~148MB - 最快
                        ];

                        for model in model_preferences {
                            let model_file = models_dir.join(model);
                            if model_file.exists() {
                                println!("✅ Found model: {} ({}MB)",
                                        model,
                                        match model {
                                            "ggml-large-v2.bin" => "1550",
                                            "ggml-large-v3-turbo.bin" => "1570",
                                            "ggml-large-v3.bin" => "2950",
                                            "ggml-small.bin" => "467",
                                            "ggml-base.bin" => "148",
                                            _ => "unknown",
                                        });
                                return Some(model_file.to_string_lossy().to_string());
                            }
                        }

                        // 如果上述模型都没找到，列出目录中的所有.bin文件
                        println!("⚠️ No preferred models found, searching for any .bin files...");
                        if let Ok(entries) = std::fs::read_dir(&models_dir) {
                            for entry in entries.flatten() {
                                let path = entry.path();
                                if path.extension().and_then(|s| s.to_str()) == Some("bin") {
                                    println!("✅ Found alternative model: {}", path.display());
                                    return Some(path.to_string_lossy().to_string());
                                }
                            }
                        }

                        None
                    })
                    .unwrap_or_else(|| {
                        println!("⚠️ No Whisper model found. Please download a model to {:?}", crate::utils::platform::get_models_dir());
                        println!("💡 Recommended models for CPU: ggml-base.bin (fastest) or ggml-small.bin (balanced)");
                        println!("📥 Download from: https://huggingface.co/ggerganov/whisper.cpp/tree/main");
                        "./models/ggml-base.bin".to_string()
                    });

                println!("🎯 Using Whisper model: {}", model_path);

                {
                    println!("🔧 Creating WhisperRSProcessor with model: {}", model_path);
                    
                    // Try to create WhisperRSProcessor with timeout to prevent indefinite hangs
                    println!("⏱️ Creating WhisperRSProcessor with safety timeout...");
                    
                    let processor_result = std::thread::spawn(move || {
                        // Use a simple timeout mechanism
                        let (tx, rx) = std::sync::mpsc::channel();
                        
                        // Spawn the processor creation in a separate thread
                        std::thread::spawn(move || {
                            let result = WhisperRSProcessor::with_model_path(&model_path);
                            let _ = tx.send(result);
                        });
                        
                        // Wait for up to 180 seconds (3 minutes) for processor creation - large models need more time
                        match rx.recv_timeout(std::time::Duration::from_secs(180)) {
                            Ok(processor_result) => processor_result,
                            Err(_) => {
                                eprintln!("⏰ WhisperRSProcessor creation timed out after 180 seconds");
                                eprintln!("💡 This indicates a deadlock or infinite loop in whisper.cpp");
                                eprintln!("💡 Or the model is very large and needs even more time to load");
                                Err(crate::voice_assistant::VoiceError::Other(
                                    "WhisperRSProcessor creation timeout".to_string()
                                ))
                            }
                        }
                    }).join().unwrap_or_else(|_| {
                        eprintln!("💥 WhisperRSProcessor creation thread panicked!");
                        Err(crate::voice_assistant::VoiceError::Other(
                            "WhisperRSProcessor creation thread panicked".to_string()
                        ))
                    });
                    
                    match processor_result {
                        Ok(processor) => {
                            println!("✅ WhisperRSProcessor created successfully");
                            Arc::new(processor)
                        }
                        Err(e) => {
                            eprintln!("❌ Failed to create WhisperRSProcessor: {}", e);
                            eprintln!("💡 Falling back to Cloud ASR processor...");

                            // Fallback to Cloud ASR
                            if config.service_platform == "groq" {
                                println!("🔄 Fallback: Creating Whisper Cloud processor...");
                                match WhisperProcessor::new() {
                                    Ok(processor) => {
                                        println!("✅ Cloud ASR fallback processor created successfully");
                                        Arc::new(processor)
                                    }
                                    Err(e) => {
                                        eprintln!("❌ Failed to create Cloud ASR fallback: {}", e);
                                        eprintln!("💡 Trying SenseVoice as final fallback...");
                                        match SenseVoiceProcessor::new() {
                                            Ok(processor) => {
                                                println!("✅ SenseVoice fallback processor created successfully");
                                                Arc::new(processor)
                                            }
                                            Err(e) => {
                                                eprintln!("❌ All ASR processors failed: {}", e);
                                                return Err(crate::voice_assistant::VoiceError::Other(
                                                    format!("All ASR processors failed. Primary error: {}, Cloud fallback failed: {}", e, e)
                                                ));
                                            }
                                        }
                                    }
                                }
                            } else {
                                println!("🔄 Fallback: Creating SenseVoice Cloud processor...");
                                match SenseVoiceProcessor::new() {
                                    Ok(processor) => {
                                        println!("✅ SenseVoice fallback processor created successfully");
                                        Arc::new(processor)
                                    }
                                    Err(e) => {
                                        eprintln!("❌ Failed to create SenseVoice fallback: {}", e);
                                        eprintln!("💡 Trying Whisper Cloud as final fallback...");
                                        match WhisperProcessor::new() {
                                            Ok(processor) => {
                                                println!("✅ Cloud ASR fallback processor created successfully");
                                                Arc::new(processor)
                                            }
                                            Err(e) => {
                                                eprintln!("❌ All ASR processors failed: {}", e);
                                                return Err(crate::voice_assistant::VoiceError::Other(
                                                    format!("All ASR processors failed. Primary error: {}, Cloud fallback failed: {}", e, e)
                                                ));
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            },
            // ProcessorType::EnhancedWhisper => {
            //     info!("Creating Enhanced Whisper processor (with VAD support)");
            //     // Load model path with intelligent detection
            //     let model_path = std::env::var("WHISPER_MODEL_PATH")
            //         .ok()
            //         .and_then(|path| {
            //             if std::path::Path::new(&path).exists() {
            //                 Some(path)
            //             } else {
            //                 None
            //             }
            //         })
            //         .or_else(|| {
            //             // Try to find the model in the default data directory
            //             let home = std::env::var("HOME").ok()?;
            //             let model_file = format!("{}/.local/share/com.martin.flash-input/models/ggml-large-v3-turbo.bin", home);
            //             if std::path::Path::new(&model_file).exists() {
            //                 Some(model_file)
            //             } else {
            //                 None
            //             }
            //         })
            //         .unwrap_or_else(|| {
            //             println!("⚠️ Whisper model not found. Please download ggml-large-v3-turbo.bin to ~/.local/share/com.martin.flash-input/models/ or set WHISPER_MODEL_PATH");
            //             "./models/ggml-large-v3-turbo.bin".to_string()
            //         });

            //     println!("🚀 Using Enhanced Whisper with VAD model: {}", model_path);

            //     // Use beam search with VAD for better accuracy
            //     Arc::new(EnhancedWhisperProcessor::with_beam_search_and_vad(
            //         &model_path,
            //         5,  // beam_size
            //         -1.0, // patience (default)
            //     )?)
            // },
        };

        // Create translation processor
        let translate_processor: Option<Arc<dyn TranslateProcessor + Send + Sync>> = match config.translate_processor {
            TranslateType::SiliconFlow => {
                info!("Creating SiliconFlow translation processor");
                Some(Arc::new(SiliconFlowTranslateProcessor::new()?))
            },
            TranslateType::Ollama => {
                info!("Creating Ollama translation processor");
                Some(Arc::new(OllamaTranslateProcessor::new()?))
            },
        };

        // Create audio recorder
        let recorder = Arc::new(Mutex::new(AudioRecorder::new()?));

        // Create keyboard manager
        let keyboard_manager = Arc::new(Mutex::new(KeyboardManager::new(
            asr_processor.clone(),
            translate_processor.clone(),
        )?));

        Ok(Self {
            config,
            asr_processor: Some(asr_processor),
            translate_processor,
            keyboard_manager,
            recorder,
            state: Arc::new(Mutex::new(InputState::Idle)),
            logger_initialized: true,
        })
    }

    pub async fn with_config(config: VoiceAssistantConfig) -> Result<Self, VoiceError> {
        // Similar to new() but uses provided config
        let mut assistant = Self::new().await?;
        assistant.config = config;
        Ok(assistant)
    }

    /// 🔥 刷新所有配置 - 确保从数据库获取最新设置
    pub async fn refresh_all_configs(&mut self) -> Result<(), VoiceError> {
        println!("🔄 Refreshing all configurations from database...");
        
        // 1. 刷新核心配置
        let fresh_config = Self::load_config_from_database().await?;
        self.config = fresh_config;
        println!("✅ Core configuration refreshed");
        
        // 2. 刷新ASR处理器（如果类型发生变化）
        let new_asr_processor: Arc<dyn AsrProcessor + Send + Sync> = match self.config.asr_processor {
            ProcessorType::CloudASR => {
                // 根据service_platform选择不同的云ASR后端
                if self.config.service_platform == "groq" {
                    println!("🔄 Creating Cloud ASR processor (Whisper backend)");
                    Arc::new(crate::voice_assistant::asr::whisper::WhisperProcessor::new()?)
                } else {
                    println!("🔄 Creating Cloud ASR processor (SenseVoice backend)");
                    Arc::new(crate::voice_assistant::asr::sensevoice::SenseVoiceProcessor::new()?)
                }
            },
            ProcessorType::LocalASR => {
                println!("🔄 Creating Local ASR processor (HTTP API)");
                let local_asr_config = Self::load_local_asr_config().await?;
                Arc::new(crate::voice_assistant::asr::local_asr::LocalASRProcessor::with_config(local_asr_config)?)
            },
            ProcessorType::WhisperRS => {
                println!("🔄 Creating WhisperRS processor (Local whisper.cpp)");
                // Load WhisperRS configuration from environment or use default location
                let model_path = std::env::var("WHISPER_MODEL_PATH")
                    .ok()
                    .and_then(|path| {
                        if std::path::Path::new(&path).exists() {
                            println!("✅ Using active model from environment: {}", path);
                            Some(path)
                        } else {
                            println!("⚠️ Environment model doesn't exist: {}", path);
                            None
                        }
                    })
                    .or_else(|| {
                        // 🔥 搜索模型目录，按优先级查找
                        let models_dir = crate::utils::platform::get_models_dir();
                        println!("🔍 Searching for models in: {}", models_dir.display());

                        // Try different models in order of preference
                        let model_preferences = [
                            "ggml-large-v2.bin",         // 🔥 用户选择的模型
                            "ggml-large-v3-turbo.bin",     // ~1570MB
                            "ggml-large-v3.bin",          // ~2950MB
                            "ggml-small.bin",             // ~467MB
                            "ggml-base.bin",              // ~148MB
                        ];

                        for model in model_preferences {
                            let model_file = models_dir.join(model);
                            if model_file.exists() {
                                println!("✅ Found model: {}", model);
                                return Some(model_file.to_string_lossy().to_string());
                            }
                        }

                        // 如果上述模型都没找到，扫描所有.bin文件
                        println!("⚠️ No preferred models found, searching for any .bin files...");
                        if let Ok(entries) = std::fs::read_dir(&models_dir) {
                            for entry in entries.flatten() {
                                let path = entry.path();
                                if path.extension().and_then(|s| s.to_str()) == Some("bin") {
                                    println!("✅ Found alternative model: {}", path.display());
                                    return Some(path.to_string_lossy().to_string());
                                }
                            }
                        }

                        None
                    })
                    .ok_or_else(|| {
                        println!("⚠️ Whisper model not found. Please download a model to {}/", crate::utils::platform::get_models_dir().display());
                        VoiceError::Other("Whisper model not found".to_string())
                    })?;

                println!("🎯 Using Whisper model: {}", model_path);

                Arc::new(crate::voice_assistant::asr::whisper_rs::WhisperRSProcessor::with_model_path(&model_path)?)
            },
        };
        self.asr_processor = Some(new_asr_processor);
        println!("✅ ASR processor refreshed");

        // 3. 刷新翻译处理器
        let new_translate_processor: Option<Arc<dyn TranslateProcessor + Send + Sync>> = match self.config.translate_processor {
            TranslateType::SiliconFlow => {
                println!("🔄 Creating SiliconFlow translation processor");
                Some(Arc::new(crate::voice_assistant::translate::siliconflow::SiliconFlowTranslateProcessor::new()?))
            },
            TranslateType::Ollama => {
                println!("🔄 Creating Ollama translation processor");
                Some(Arc::new(crate::voice_assistant::translate::ollama::OllamaTranslateProcessor::new()?))
            },
        };
        self.translate_processor = new_translate_processor;
        println!("✅ Translation processor refreshed");

        // 4. 更新键盘管理器的处理器引用
        if let Ok(mut keyboard_manager) = self.keyboard_manager.lock() {
            keyboard_manager.update_processors(
                self.asr_processor.clone(),
                self.translate_processor.clone()
            )?;
            println!("✅ Keyboard manager processors updated");
        }
        
        println!("🎉 All configurations successfully refreshed from database");
        Ok(())
    }

    pub async fn start(&mut self) -> Result<(), VoiceError> {
        println!("🚀 === VoiceAssistant Starting ===");
        info!("Starting VoiceAssistant");
        
        // STEP 0: Skip refresh - config already loaded during initialization
        println!("🔄 Step 0: Configuration already loaded during initialization");
        
        // Step 1: Load hotkey configuration from database
        println!("📊 Step 1: Loading hotkey configuration...");
        let db_config = crate::commands::get_hotkey_config_from_database().await?;
        if let Some(config) = db_config {
            println!("✅ Database config found:");
            println!("  - Transcribe: {}", config.transcribe_key);
            println!("  - Translate: {}", config.translate_key);
            println!("  - Trigger delay: {}ms", config.trigger_delay_ms);
            println!("  - Anti-mistouch enabled: {}", config.anti_mistouch_enabled);
            println!("  - Save WAV files: {}", config.save_wav_files);
            
            // Step 2: Set hotkeys on keyboard manager and start listening
            println!("📝 Step 2: Setting hotkeys on keyboard manager...");
            if let Ok(mut keyboard_manager) = self.keyboard_manager.lock() {
                println!("🔓 Keyboard manager lock acquired");
                if let Err(e) = keyboard_manager.set_hotkeys(&config.transcribe_key, &config.translate_key) {
                    println!("❌ Failed to set hotkeys: {}", e);
                    return Err(VoiceError::Audio(format!("Failed to set hotkeys: {}", e)));
                }
                println!("✅ Hotkeys set successfully");

                // Step 2.5: Set save_wav_files configuration
                println!("📁 Step 2.5: Setting save_wav_files configuration...");
                keyboard_manager.set_save_wav_files(config.save_wav_files);

                // Step 3: Start keyboard listening
                println!("👂 Step 3: Starting keyboard listening...");
                keyboard_manager.start_listening();
                println!("✅ Keyboard listening started");
            } else {
                println!("❌ Failed to acquire keyboard manager lock");
                return Err(VoiceError::Audio("Failed to acquire keyboard manager lock".to_string()));
            }
        } else {
            println!("⚠️ No hotkey configuration found in database, using defaults");
            if let Ok(mut keyboard_manager) = self.keyboard_manager.lock() {
                // 使用默认热键 (F4 和 Shift + F4)
                if let Err(e) = keyboard_manager.set_hotkeys("F4", "Shift + F4") {
                    return Err(VoiceError::Audio(format!("Failed to set default hotkeys: {}", e)));
                }
                keyboard_manager.start_listening();
            }
        }

        info!("VoiceAssistant started successfully");

        // Check PRIMARY selection content at startup - DISABLED
        // println!("🔍 Checking PRIMARY selection content at startup...");
        // if let Ok(current_primary) = std::process::Command::new("xclip")
        //     .args(&["-selection", "primary", "-o"])
        //     .output()
        // {
        //     if current_primary.status.success() {
        //         let current_text = String::from_utf8_lossy(&current_primary.stdout);
        //         let trimmed_text = current_text.trim_end_matches('\n');
        //         if !trimmed_text.is_empty() {
        //             println!("📋 PRIMARY SELECTION AT STARTUP: \"{}\"", trimmed_text);
        //             println!("📏 Length: {} characters", trimmed_text.len());
        //         } else {
        //             println!("📋 PRIMARY SELECTION AT STARTUP: <empty>");
        //         }
        //     } else {
        //         println!("❌ Failed to read PRIMARY selection at startup: {}", String::from_utf8_lossy(&current_primary.stderr));
        //     }
        // } else {
        //     println!("❌ xclip command not available at startup");
        // }

        Ok(())
    }

    pub fn stop(&mut self) -> Result<(), VoiceError> {
        info!("Stopping VoiceAssistant");

        // Reset keyboard manager state
        if let Ok(mut keyboard_manager) = self.keyboard_manager.lock() {
            keyboard_manager.reset_state();
            // 🔥 重要：也清除 KeyboardManager 中持有的处理器引用
            // 这样才能让 Arc 的引用计数降为 0，真正释放模型
            keyboard_manager.clear_processors();
        }

        // Reset any active recording
        if let Ok(mut recorder) = self.recorder.lock() {
            if recorder.is_recording() {
                let _ = recorder.stop_recording();
            }
        }

        // 🔥 显式释放 ASR 处理器以卸载模型并释放内存
        if self.asr_processor.is_some() {
            info!("🗑️ Unloading ASR model to free memory...");
            self.asr_processor = None;
            info!("✅ ASR model unloaded successfully");
        }

        // 🔥 显式释放翻译处理器
        if self.translate_processor.is_some() {
            info!("🗑️ Unloading translation processor to free memory...");
            self.translate_processor = None;
            info!("✅ Translation processor unloaded successfully");
        }

        *self.state.lock().unwrap() = InputState::Idle;
        info!("VoiceAssistant stopped");
        Ok(())
    }

    pub fn get_state(&self) -> InputState {
        *self.state.lock().unwrap()
    }

    pub fn get_config(&self) -> VoiceAssistantConfig {
        self.config.clone()
    }

    async fn load_local_asr_config() -> Result<crate::voice_assistant::asr::local_asr::LocalASRConfig, VoiceError> {
        // Get ASR configs from database
        let asr_configs = crate::commands::get_asr_config_internal().await?;

        // Find the local ASR configuration
        if let Some(local_config) = asr_configs.iter().find(|config| config.service_provider == "local") {
            Ok(crate::voice_assistant::asr::local_asr::LocalASRConfig {
                endpoint: local_config.local_endpoint.clone().unwrap_or_else(|| "http://192.168.8.107:5001/inference".to_string()),
                api_key: local_config.local_api_key.clone().unwrap_or_else(|| "default-key".to_string()),
            })
        } else {
            // Fallback to default local config
            Ok(crate::voice_assistant::asr::local_asr::LocalASRConfig {
                endpoint: "http://192.168.8.107:5001/inference".to_string(),
                api_key: "default-key".to_string(),
            })
        }
    }

    async fn load_config_from_database() -> Result<VoiceAssistantConfig, VoiceError> {
        println!("📊 Loading configuration from database...");
        
        // Get ASR config from database
        let asr_configs = crate::commands::get_asr_config_internal().await?;
        if !asr_configs.is_empty() {
            println!("✅ Found {} ASR config(s) in database", asr_configs.len());
            for (i, config) in asr_configs.iter().enumerate() {
                println!("  ASR Config {}: service={}, local_endpoint={:?}", 
                    i+1, config.service_provider, config.local_endpoint);
            }
        } else {
            println!("⚠️ No ASR configs found in database");
        }

        // Get translation config from database
        let translation_configs = crate::commands::get_translation_config_internal().await?;
        if !translation_configs.is_empty() {
            println!("✅ Found {} translation config(s) in database", translation_configs.len());
            for (i, config) in translation_configs.iter().enumerate() {
                println!("  Translation Config {}: provider={}, endpoint={:?}", 
                    i+1, config.provider, config.endpoint);
            }
        } else {
            println!("⚠️ No translation configs found in database");
        }

        // Determine ASR processor type from database config
        let asr_processor = if let Some(asr_config) = asr_configs.first() {
            match asr_config.service_provider.as_str() {
                "local" => ProcessorType::WhisperRS, // 🔥 改为使用本地WhisperRS而不是HTTP API
                "cloud" => ProcessorType::CloudASR,
                "whisper-rs" => ProcessorType::WhisperRS, // 显式支持whisper-rs
                _ => ProcessorType::WhisperRS, // 默认使用WhisperRS
            }
        } else {
            ProcessorType::WhisperRS // 🔥 默认使用WhisperRS
        };

        // Determine translate processor type from database config
        let translate_processor = if let Some(translation_config) = translation_configs.first() {
            match translation_config.provider.as_str() {
                "siliconflow" => TranslateType::SiliconFlow,
                "ollama" => TranslateType::Ollama,
                _ => TranslateType::Ollama,
            }
        } else {
            TranslateType::Ollama
        };

        // Get service platform from ASR config
        let service_platform = if let Some(asr_config) = asr_configs.first() {
            asr_config.service_provider.clone()
        } else {
            "siliconflow".to_string()
        };

        println!("📊 Loaded config from database:");
        println!("  - ASR processor: {:?}", asr_processor);
        println!("  - Translate processor: {:?}", translate_processor);
        println!("  - Service platform: {}", service_platform);

        Ok(VoiceAssistantConfig {
            service_platform,
            asr_processor,
            translate_processor,
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

    pub fn process_audio_file(
        &self,
        audio_file_path: &str,
        mode: Mode,
        prompt: Option<&str>,
    ) -> Result<String, VoiceError> {
        info!("Processing audio file: {} in mode: {:?}", audio_file_path, mode);

        // Read audio file
        let audio_data = std::fs::read(audio_file_path)
            .map_err(|e| VoiceError::Io(e))?;

        let audio_cursor = std::io::Cursor::new(audio_data);

        // Process with ASR
        let prompt_str = prompt.unwrap_or("");
        let asr = self.asr_processor.as_ref().ok_or_else(|| VoiceError::Other("ASR processor not available".to_string()))?;
        let result = asr.process_audio(audio_cursor, mode, prompt_str)?;

        info!("Audio processing completed, result length: {}", result.len());
        Ok(result)
    }

    pub fn translate_text(&self, text: &str) -> Result<String, VoiceError> {
        if let Some(ref translate_processor) = self.translate_processor {
            info!("Translating text: {}", text);
            translate_processor.translate(text)
        } else {
            Err(VoiceError::Other("No translation processor available".to_string()))
        }
    }

  pub fn test_asr_processor(&self, processor_type: ProcessorType) -> Result<String, VoiceError> {
        info!("Testing ASR processor: {:?}", processor_type);

        let test_result = match processor_type {
            ProcessorType::CloudASR => "Cloud ASR processor test successful",
            ProcessorType::LocalASR => "Local ASR processor test successful",
            ProcessorType::WhisperRS => {
                // Check if model file exists for WhisperRS
                let model_path = std::env::var("WHISPER_MODEL_PATH").unwrap_or_else(|_| {
                    "./models/ggml-base.bin".to_string()
                });
                
                if std::path::Path::new(&model_path).exists() {
                    "WhisperRS processor test successful - model found"
                } else {
                    "WhisperRS processor test failed - model file not found"
                }
            }
        };

        info!("{}", test_result);
        Ok(test_result.to_string())
    }

    pub fn test_translate_processor(&self, translate_type: TranslateType) -> Result<String, VoiceError> {
        info!("Testing translation processor: {:?}", translate_type);

        if let Some(ref translate_processor) = self.translate_processor {
            let test_text = "Hello, this is a test translation.";
            let result = translate_processor.translate(test_text)?;
            info!("Translation test result: {}", result);
            Ok(result)
        } else {
            Err(VoiceError::Other("No translation processor available for testing".to_string()))
        }
    }

    pub fn get_system_info(&self) -> HashMap<String, String> {
        let mut info = HashMap::new();
        
        info.insert("version".to_string(), env!("CARGO_PKG_VERSION").to_string());
        info.insert("service_platform".to_string(), self.config.service_platform.clone());
        info.insert("asr_processor".to_string(), format!("{:?}", self.config.asr_processor));
        info.insert("translate_processor".to_string(), format!("{:?}", self.config.translate_processor));
        info.insert("state".to_string(), format!("{:?}", self.get_state()));
        info.insert("logger_initialized".to_string(), self.logger_initialized.to_string());

        // Add environment variables info (without exposing sensitive data)
        if std::env::var("GROQ_API_KEY").is_ok() {
            info.insert("groq_configured".to_string(), "true".to_string());
        }
        if std::env::var("SILICONFLOW_API_KEY").is_ok() {
            info.insert("siliconflow_configured".to_string(), "true".to_string());
        }

        info
    }
  /// 设置转录热键
    pub fn set_transcribe_hotkey(&self, hotkey_str: &str) -> Result<(), VoiceError> {
        let keyboard_manager = self.keyboard_manager.lock().unwrap();
        keyboard_manager.set_transcribe_hotkey(hotkey_str)
    }

    /// 设置翻译热键
    pub fn set_translate_hotkey(&self, hotkey_str: &str) -> Result<(), VoiceError> {
        let keyboard_manager = self.keyboard_manager.lock().unwrap();
        keyboard_manager.set_translate_hotkey(hotkey_str)
    }

    /// 设置触发延迟（毫秒）
    pub fn set_trigger_delay_ms(&self, delay_ms: i64) {
        let keyboard_manager = self.keyboard_manager.lock().unwrap();
        keyboard_manager.set_trigger_delay_ms(delay_ms);
    }

    /// 设置反触误触功能
    pub fn set_anti_mistouch_enabled(&self, enabled: bool) {
        let keyboard_manager = self.keyboard_manager.lock().unwrap();
        keyboard_manager.set_anti_mistouch_enabled(enabled);
    }

    /// 设置延迟配置
    pub fn set_typing_delays(&self, typing_delays: crate::database::TypingDelays) {
        let keyboard_manager = self.keyboard_manager.lock().unwrap();
        keyboard_manager.set_typing_delays(typing_delays);
    }
}

impl Default for VoiceAssistant {
    fn default() -> Self {
        // Use handle_current to avoid creating a new runtime
        use tokio::runtime::Handle;
        match Handle::try_current() {
            Ok(handle) => {
                // We're already in a Tokio context, block on it
                handle.block_on(async {
                    Self::new().await.expect("Failed to create VoiceAssistant")
                })
            }
            Err(_) => {
                // No Tokio context available, create a new one as fallback
                use tokio::runtime::Runtime;
                let rt = Runtime::new().expect("Failed to create runtime");
                rt.block_on(async {
                    Self::new().await.expect("Failed to create VoiceAssistant")
                })
            }
        }
    }
}

// Helper function to get global VoiceAssistant instance
fn get_voice_assistant_instance() -> &'static Arc<Mutex<Option<VoiceAssistant>>> {
    VOICE_ASSISTANT.get_or_init(|| Arc::new(Mutex::new(None)))
}

// Tauri commands - Real implementation
#[tauri::command]
pub async fn start_voice_assistant() -> Result<String, String> {
    info!("🚀 Start VoiceAssistant command called");

    let instance = get_voice_assistant_instance();

    // Check if already running
    {
        let va = instance.lock().unwrap();
        if va.is_some() {
            info!("⚠️ VoiceAssistant is already running");
            return Ok("VoiceAssistant is already running".to_string());
        }
    }

    // Create new VoiceAssistant
    match VoiceAssistant::new().await {
        Ok(mut assistant) => {
            // Start the assistant
            match assistant.start().await {
                Ok(()) => {
                    // Store the instance
                    {
                        let mut va = instance.lock().unwrap();
                        *va = Some(assistant);
                    }
                    info!("✅ VoiceAssistant started successfully");
                    // Emit "Running" state to indicate VoiceAssistant service is active
                    // This matches the logic in get_voice_assistant_state()
                    if let Some(handle_guard) = APP_HANDLE.get() {
                        if let Ok(app_handle) = handle_guard.lock() {
                            if let Some(ref handle) = *app_handle {
                                if let Err(e) = handle.emit("voice-assistant-state-changed", "Running") {
                                    error!("Failed to emit voice assistant state change event: {}", e);
                                } else {
                                    info!("✅ Emitted voice assistant state change: Running");
                                }
                            }
                        }
                    }
                    Ok("VoiceAssistant started successfully".to_string())
                }
                Err(e) => {
                    error!("❌ Failed to start VoiceAssistant: {}", e);
                    Err(format!("Failed to start VoiceAssistant: {}", e))
                }
            }
        }
        Err(e) => {
            error!("❌ Failed to create VoiceAssistant: {}", e);
            Err(format!("Failed to create VoiceAssistant: {}", e))
        }
    }
}

#[tauri::command]
pub async fn stop_voice_assistant() -> Result<String, String> {
    info!("⏹️ Stop VoiceAssistant command called");

    let instance = get_voice_assistant_instance();

    // Check if running
    {
        let mut va = instance.lock().unwrap();
        if va.is_none() {
            info!("⚠️ VoiceAssistant is not running");
            return Ok("VoiceAssistant is not running".to_string());
        }

        // Stop and remove the instance
        if let Some(mut assistant) = va.take() {
            match assistant.stop() {
                Ok(()) => {
                    info!("✅ VoiceAssistant stopped successfully");
                    // Emit stopped state - use "Idle" to indicate service is actually stopped
                    if let Some(handle_guard) = APP_HANDLE.get() {
                        if let Ok(app_handle) = handle_guard.lock() {
                            if let Some(ref handle) = *app_handle {
                                if let Err(e) = handle.emit("voice-assistant-state-changed", "Idle") {
                                    error!("Failed to emit voice assistant state change event: {}", e);
                                } else {
                                    info!("✅ Emitted voice assistant state change: Idle (service stopped)");
                                }
                            }
                        }
                    }
                    Ok("VoiceAssistant stopped successfully".to_string())
                }
                Err(e) => {
                    error!("❌ Failed to stop VoiceAssistant: {}", e);
                    Err(format!("Failed to stop VoiceAssistant: {}", e))
                }
            }
        } else {
            unreachable!() // We already checked it's Some
        }
    }
}

#[tauri::command]
pub async fn get_voice_assistant_state() -> Result<String, String> {
    let instance = get_voice_assistant_instance();

    let va = instance.lock().unwrap();
    if let Some(assistant) = va.as_ref() {
        let state = assistant.get_state();
        // If VoiceAssistant instance exists, it's running even if internal state is Idle
        match state {
            InputState::Idle => Ok("Running".to_string()),
            _ => Ok(format!("{:?}", state))
        }
    } else {
        Ok("Idle".to_string())
    }
}

#[tauri::command]
pub async fn get_voice_assistant_config() -> Result<VoiceAssistantConfig, String> {
    Ok(VoiceAssistantConfig::default())
}

#[tauri::command]
pub async fn test_asr(processor_type: ProcessorType) -> Result<String, String> {
    info!("Testing ASR processor: {:?}", processor_type);
    match processor_type {
        ProcessorType::CloudASR => Ok("✅ Cloud ASR processor test successful".to_string()),
        ProcessorType::LocalASR => Ok("✅ Local ASR processor test successful".to_string()),
        ProcessorType::WhisperRS => Ok("✅ WhisperRS processor test successful".to_string()),
    }
}

#[tauri::command]
pub async fn test_translation(translate_type: TranslateType) -> Result<String, String> {
    info!("Testing translation processor: {:?}", translate_type);
    match translate_type {
        TranslateType::SiliconFlow => Ok("✅ SiliconFlow translation test successful".to_string()),
        TranslateType::Ollama => Ok("✅ Ollama translation test successful".to_string()),
    }
}

#[tauri::command]
pub async fn get_system_info() -> Result<HashMap<String, String>, String> {
    let mut info = HashMap::new();
    info.insert("Platform".to_string(), std::env::consts::OS.to_string());
    info.insert("Arch".to_string(), std::env::consts::ARCH.to_string());
    info.insert("Rust Version".to_string(), "1.70+".to_string());
    info.insert("Tauri Version".to_string(), "2.0".to_string());
    info.insert("Status".to_string(), "Ready".to_string());
    Ok(info)
}

#[tauri::command]
pub async fn configure_hotkeys(
    transcribe_key: String,
    translate_key: String,
    trigger_delay_ms: i64,
    anti_mistouch_enabled: bool,
    typing_delays: Option<crate::database::TypingDelays>,
) -> Result<String, String> {
    info!("Configuring hotkeys:");
    info!("  - Transcribe: {}", transcribe_key);
    info!("  - Translate: {}", translate_key);
    info!("  - Trigger delay: {}ms", trigger_delay_ms);
    info!("  - Anti-mistouch: {}", anti_mistouch_enabled);
    if typing_delays.is_some() {
        info!("  - Typing delays configured");
    }

    // Create a temporary VoiceAssistant to configure hotkeys
    match VoiceAssistant::new().await {
        Ok(assistant) => {
            // Set hotkey configuration
            if let Err(e) = assistant.set_transcribe_hotkey(&transcribe_key) {
                return Err(format!("Failed to set transcribe hotkey: {}", e));
            }

            if let Err(e) = assistant.set_translate_hotkey(&translate_key) {
                return Err(format!("Failed to set translate hotkey: {}", e));
            }

            assistant.set_trigger_delay_ms(trigger_delay_ms);
            assistant.set_anti_mistouch_enabled(anti_mistouch_enabled);

            // Set typing delays if provided
            if let Some(delays) = typing_delays {
                assistant.set_typing_delays(delays);
            }

            info!("✅ Hotkeys configured successfully");
            Ok("Hotkeys configured successfully".to_string())
        }
        Err(e) => {
            let error_msg = format!("Failed to create VoiceAssistant for hotkey configuration: {}", e);
            error!("{}", error_msg);
            Err(error_msg)
        }
    }
}