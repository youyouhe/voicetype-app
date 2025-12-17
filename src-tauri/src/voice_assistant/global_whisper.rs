use std::sync::Arc;
use tokio::sync::RwLock;
use serde::{Serialize, Deserialize};
use std::sync::OnceLock;

use crate::voice_assistant::asr::whisper_rs::{WhisperRSProcessor, WhisperRSConfig};
use crate::voice_assistant::traits::VoiceError;

/// 全局WhisperRS实例管理器
pub struct GlobalWhisperManager {
    processor: Option<Arc<std::sync::Mutex<WhisperRSProcessor>>>,
    current_model_path: Option<String>,
    init_in_progress: bool,
}

impl GlobalWhisperManager {
    /// 创建新的管理器实例
    pub fn new() -> Self {
        Self {
            processor: None,
            current_model_path: None,
            init_in_progress: false,
        }
    }

    /// 获取或创建WhisperRS处理器
    pub async fn get_or_create_processor(&mut self, model_path: &str) -> Result<Arc<std::sync::Mutex<WhisperRSProcessor>>, VoiceError> {
        // 检查是否已经有相同模型的处理器
        if let Some(current_path) = &self.current_model_path {
            if current_path == model_path {
                if let Some(processor) = &self.processor {
                    println!("✅ Reusing existing WhisperRS processor for model: {}", model_path);
                    return Ok(Arc::clone(processor));
                }
            }
        }

        // 如果正在初始化，等待完成
        if self.init_in_progress {
            println!("⏳ WhisperRS processor initialization in progress, waiting...");
            // 这里可以添加等待逻辑，但为简单起见，我们直接返回错误
            return Err(VoiceError::Other("WhisperRS processor initialization in progress".to_string()));
        }

        // 需要创建新的处理器
        println!("🔧 Initializing new WhisperRS processor for model: {}", model_path);
        self.init_in_progress = true;

        // Auto-detect optimal GPU backend
        let gpu_detector = crate::voice_assistant::asr::gpu_detector::GpuDetector::new();
        let optimal_backend = gpu_detector.get_preferred_backend();

        let config = WhisperRSConfig {
            model_path: model_path.to_string(),
            language: None, // Auto-detect
            sampling_strategy: crate::voice_assistant::asr::whisper_rs::SamplingStrategyConfig::Greedy { best_of: 1 },
            translate: false,
            enable_vad: std::env::var("WHISPER_ENABLE_VAD")
                .unwrap_or_else(|_| "false".to_string())
                .parse::<bool>()
                .unwrap_or(false),
            backend: optimal_backend.clone(),
            use_gpu_if_available: std::env::var("WHISPER_USE_GPU")
                .unwrap_or_else(|_| "true".to_string())
                .parse::<bool>()
                .unwrap_or(true),
            gpu_device_id: std::env::var("WHISPER_GPU_DEVICE_ID")
                .ok()
                .and_then(|id| id.parse::<u32>().ok()),
        };

        match WhisperRSProcessor::new(config) {
            Ok(processor) => {
                let arc_processor = Arc::new(std::sync::Mutex::new(processor));
                self.processor = Some(Arc::clone(&arc_processor));
                self.current_model_path = Some(model_path.to_string());
                self.init_in_progress = false;

                // 设置环境变量以保持兼容性
                std::env::set_var("WHISPER_MODEL_PATH", model_path);

                println!("✅ WhisperRS processor initialized successfully for model: {}", model_path);
                Ok(arc_processor)
            }
            Err(e) => {
                self.init_in_progress = false;
                println!("❌ Failed to initialize WhisperRS processor: {}", e);
                Err(e)
            }
        }
    }

    /// 检查是否有可用的处理器
    pub fn has_processor(&self) -> bool {
        self.processor.is_some()
    }

    /// 获取当前模型路径
    pub fn get_current_model_path(&self) -> Option<&str> {
        self.current_model_path.as_deref()
    }

    /// 清除当前处理器（用于错误恢复或模型卸载）
    pub fn clear_processor(&mut self) {
        println!("🗑️ Clearing global WhisperRS processor");
        self.processor = None;
        self.current_model_path = None;
        self.init_in_progress = false;
    }

    /// 强制重新加载处理器
    pub async fn force_reload(&mut self, model_path: &str) -> Result<Arc<std::sync::Mutex<WhisperRSProcessor>>, VoiceError> {
        println!("🔄 Force reloading WhisperRS processor for model: {}", model_path);
        self.clear_processor();
        self.get_or_create_processor(model_path).await
    }
}

/// 全局WhisperRS管理器实例
static GLOBAL_WHISPER_MANAGER: OnceLock<RwLock<GlobalWhisperManager>> = OnceLock::new();

/// 获取全局WhisperRS管理器实例
pub fn get_global_whisper_manager() -> &'static RwLock<GlobalWhisperManager> {
    GLOBAL_WHISPER_MANAGER.get_or_init(|| RwLock::new(GlobalWhisperManager::new()))
}

/// 便利函数：获取或创建WhisperRS处理器
pub async fn get_or_create_whisper_processor(model_path: &str) -> Result<Arc<std::sync::Mutex<WhisperRSProcessor>>, VoiceError> {
    let manager = get_global_whisper_manager();
    let mut manager_guard = manager.write().await;
    manager_guard.get_or_create_processor(model_path).await
}

/// 便利函数：强制重新加载处理器
pub async fn force_reload_whisper_processor(model_path: &str) -> Result<Arc<std::sync::Mutex<WhisperRSProcessor>>, VoiceError> {
    let manager = get_global_whisper_manager();
    let mut manager_guard = manager.write().await;
    manager_guard.force_reload(model_path).await
}

/// 便利函数：清除全局处理器
pub async fn clear_global_whisper_processor() {
    let manager = get_global_whisper_manager();
    let mut manager_guard = manager.write().await;
    manager_guard.clear_processor();
}

/// 检查全局处理器状态
pub async fn get_global_whisper_status() -> serde_json::Value {
    let manager = get_global_whisper_manager();
    let manager_guard = manager.read().await;
    
    serde_json::json!({
        "has_processor": manager_guard.has_processor(),
        "current_model_path": manager_guard.get_current_model_path(),
        "init_in_progress": false // 由于函数作用域限制，这里返回固定值
    })
}

#[derive(Debug, Serialize, Deserialize)]
pub struct WhisperManagerStatus {
    pub has_processor: bool,
    pub current_model_path: Option<String>,
    pub init_in_progress: bool,
}

/// Tauri命令：获取全局WhisperRS状态
#[tauri::command]
pub async fn get_whisper_manager_status() -> Result<WhisperManagerStatus, String> {
    let status = get_global_whisper_status().await;
    serde_json::from_value(status).map_err(|e| format!("Failed to serialize status: {}", e))
}

/// Tauri命令：强制重新加载WhisperRS处理器
#[tauri::command]
pub async fn reload_whisper_processor(model_path: String) -> Result<String, String> {
    match force_reload_whisper_processor(&model_path).await {
        Ok(_) => {
            println!("✅ WhisperRS processor reloaded successfully");
            Ok(format!("Successfully reloaded WhisperRS processor for model: {}", model_path))
        }
        Err(e) => {
            println!("❌ Failed to reload WhisperRS processor: {}", e);
            Err(format!("Failed to reload WhisperRS processor: {}", e))
        }
    }
}

/// Tauri命令：清除全局WhisperRS处理器
#[tauri::command]
pub async fn clear_whisper_processor() -> Result<String, String> {
    clear_global_whisper_processor().await;
    println!("✅ Global WhisperRS processor cleared");
    Ok("Global WhisperRS processor cleared successfully".to_string())
}