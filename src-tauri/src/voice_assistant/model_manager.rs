use std::path::{Path, PathBuf};
use std::fs;
use std::process::Command;
use serde::{Serialize, Deserialize};
use tauri::{AppHandle, Emitter, Manager};
use crate::voice_assistant::VoiceError;

/// Download site configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DownloadSite {
    pub id: String,
    pub name: String,
    pub base_url: String,
}

impl DownloadSite {
    pub fn new(id: &str, name: &str, base_url: &str) -> Self {
        Self {
            id: id.to_string(),
            name: name.to_string(),
            base_url: base_url.to_string(),
        }
    }

    /// Get all available download sites
    pub fn get_all_sites() -> Vec<Self> {
        vec![
            Self::new(
                "huggingface",
                "Hugging Face (Official)",
                "https://huggingface.co/ggerganov/whisper.cpp/resolve/main"
            ),
            Self::new(
                "hf-mirror",
                "HF-Mirror (China)",
                "https://hf-mirror.com/ggerganov/whisper.cpp/resolve/main"
            ),
        ]
    }

    /// Test if this site is accessible by making a HEAD request
    pub fn test_connectivity(&self) -> bool {
        println!("🔍 Testing connectivity to: {} ({})", self.name, self.base_url);

        // Use curl to test connectivity with a timeout
        let test_url = format!("{}/ggml-tiny.bin", self.base_url); // Test with smallest file

        let result = Command::new("curl")
            .args([
                "-I",              // HEAD request only
                "-s",              // Silent mode
                "--connect-timeout", "5", // 5 second timeout
                "--max-time", "10",       // 10 second max time
                &test_url,
            ])
            .output();

        match result {
            Ok(output) => {
                let is_accessible = output.status.success() &&
                    String::from_utf8_lossy(&output.stdout).contains("HTTP");
                println!("{} Connectivity test for {}: {}",
                    if is_accessible { "✅" } else { "❌" },
                    self.name,
                    if is_accessible { "SUCCESS" } else { "FAILED" }
                );
                is_accessible
            }
            Err(e) => {
                println!("❌ Connectivity test for {} failed: {}", self.name, e);
                false
            }
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WhisperModel {
    pub name: String,
    pub display_name: String,
    pub file_name: String,
    pub size_mb: f64,
    pub description: String,
    pub download_url: String,
    pub is_downloaded: bool,
    pub file_path: Option<String>,
    pub download_progress: f64,
    pub is_downloading: bool,
}

impl WhisperModel {
    pub fn new(name: &str, display_name: &str, file_name: &str, size_mb: f64, description: &str) -> Self {
        Self {
            name: name.to_string(),
            display_name: display_name.to_string(),
            file_name: file_name.to_string(),
            size_mb,
            description: description.to_string(),
            download_url: String::new(), // Will be set later based on available site
            is_downloaded: false,
            file_path: None,
            download_progress: 0.0,
            is_downloading: false,
        }
    }

    /// Set download URL based on base site
    pub fn set_download_url(&mut self, base_url: &str) {
        self.download_url = format!("{}/{}", base_url.trim_end_matches('/'), self.file_name);
    }
}

pub struct ModelManager {
    models_dir: PathBuf,
    models: Vec<WhisperModel>,
    #[allow(dead_code)]
    app_handle: AppHandle,
    preferred_site: Option<String>, // Store last successful site ID
}

impl ModelManager {
    pub fn new(app_handle: AppHandle) -> Result<Self, VoiceError> {
        let models_dir = app_handle
            .path()
            .app_data_dir()
            .unwrap_or_else(|_| std::env::current_dir().unwrap().join("data"))
            .join("models");

        // Create models directory if it doesn't exist
        fs::create_dir_all(&models_dir)
            .map_err(|e| VoiceError::Other(format!("Failed to create models directory: {}", e)))?;

        let mut manager = Self {
            models_dir,
            models: Vec::new(),
            app_handle,
            preferred_site: None,
        };

        manager.initialize_models();
        Ok(manager)
    }

    fn initialize_models(&mut self) {
        // Define available models - turbo, v2, and quantized versions
        // size_mb will be updated to actual file size if downloaded, otherwise use estimate
        self.models = vec![
            WhisperModel::new(
                "large-v3-turbo",
                "Turbo",
                "ggml-large-v3-turbo.bin",
                0.0, // Will be updated from actual file or estimate
                "最新的高效模型，在保持高准确性的同时显著提升推理速度，适合生产环境使用"
            ),
            WhisperModel::new(
                "large-v3-turbo-q5_0",
                "Turbo Q5_0",
                "ggml-large-v3-turbo-q5_0.bin",
                0.0, // Will be updated from actual file or estimate
                "Turbo模型的Q5_0量化版本，体积更小但保持高准确性，推荐用于存储空间有限的设备"
            ),
            WhisperModel::new(
                "large-v2",
                "V2",
                "ggml-large-v2.bin",
                0.0, // Will be updated from actual file or estimate
                "成熟稳定的模型，具有良好的准确性和兼容性"
            ),
        ];

        // Check which models are already downloaded and get actual sizes
        self.check_downloaded_models();
    }

    fn check_downloaded_models(&mut self) {
        for model in &mut self.models {
            let model_path = self.models_dir.join(&model.file_name);
            if model_path.exists() {
                model.is_downloaded = true;
                model.file_path = Some(model_path.to_string_lossy().to_string());
                model.download_progress = 100.0;

                // Get actual file size in MB
                if let Ok(metadata) = fs::metadata(&model_path) {
                    let file_size_bytes = metadata.len();
                    model.size_mb = file_size_bytes as f64 / (1024.0 * 1024.0);
                    println!("✅ Actual file size for {}: {:.2} MB", model.name, model.size_mb);
                }
            } else {
                // Use estimated size for non-downloaded models
                model.size_mb = match model.name.as_str() {
                    "large-v3-turbo" => 1570.0,
                    "large-v3-turbo-q5_0" => 990.0, // Q5_0 quantized version is ~1GB
                    "large-v2" => 1550.0,
                    _ => 0.0,
                };
                println!("ℹ️ Using estimated size for {}: {:.2} MB", model.name, model.size_mb);
            }
        }
    }

    /// Automatically select the best available download site
    fn select_best_site(&mut self) -> Result<DownloadSite, VoiceError> {
        let sites = DownloadSite::get_all_sites();

        println!("🔍 Starting automatic site detection...");

        // If we have a preferred site, try it first
        if let Some(ref preferred_id) = self.preferred_site {
            if let Some(preferred_site) = sites.iter().find(|s| &s.id == preferred_id) {
                println!("🔄 Testing preferred site: {}", preferred_site.name);
                if preferred_site.test_connectivity() {
                    println!("✅ Preferred site is accessible: {}", preferred_site.name);
                    return Ok(preferred_site.clone());
                } else {
                    println!("⚠️ Preferred site is not accessible, trying others...");
                    self.preferred_site = None; // Reset if not accessible
                }
            }
        }

        // Try all sites in order
        for site in &sites {
            if site.test_connectivity() {
                println!("✅ Found accessible site: {}", site.name);
                self.preferred_site = Some(site.id.clone());
                return Ok(site.clone());
            }
        }

        Err(VoiceError::Other("No accessible download site found. Please check your internet connection.".to_string()))
    }

    /// Get current preferred site (for UI display)
    pub fn get_preferred_site(&self) -> Option<String> {
        self.preferred_site.clone()
    }

    pub fn get_models(&self) -> Vec<WhisperModel> {
        self.models.clone()
    }

    pub fn get_downloaded_models(&self) -> Vec<WhisperModel> {
        self.models
            .iter()
            .filter(|m| m.is_downloaded)
            .cloned()
            .collect()
    }

    pub async fn download_model(&mut self, model_name: &str) -> Result<(), VoiceError> {
        println!("🚀 Starting download for model: {}", model_name);

        let model_index = self.models
            .iter()
            .position(|m| m.name == model_name)
            .ok_or_else(|| VoiceError::Other(format!("Model '{}' not found", model_name)))?;

        let model_name_owned = model_name.to_string(); // Create owned String
        let model_name_str = model_name; // Use the original &str

        // Auto-select best available download site
        println!("🌐 Detecting best download site...");
        let download_site = self.select_best_site()?;

        // Mark as downloading and set download URL
        {
            let model = &mut self.models[model_index];

            if model.is_downloaded {
                println!("⚠️ Model '{}' already downloaded", model_name);
                return Err(VoiceError::Other("Model already downloaded".to_string()));
            }

            if model.is_downloading {
                println!("⚠️ Model '{}' already downloading", model_name);
                return Err(VoiceError::Other("Model already downloading".to_string()));
            }

            // Set download URL based on selected site
            model.set_download_url(&download_site.base_url);

            println!("📋 Model info: {} ({} MB)", model.display_name, model.size_mb);
            println!("🌐 Download site: {}", download_site.name);
            println!("🔗 Download URL: {}", model.download_url);

            model.is_downloading = true;
            model.download_progress = 0.0;
            println!("✅ Model marked as downloading, progress set to 0%");
        }

        println!("📂 Models directory: {}", self.models_dir.display());

        // Emit download start event
        println!("📡 Emitting download start event");
        self.emit_download_progress(model_name_str, 0.0);

        // Find the model and clone data for the async task
        let model_clone = self.models[model_index].clone();
        let models_dir_clone = self.models_dir.clone();
        let app_handle_clone = self.app_handle.clone();

        println!("🔄 Spawning async download task");
        // Start download in background task
        tokio::spawn(async move {
            println!("📥 Async download task started for model: {}", model_name_owned);
            match Self::download_model_internal(&model_clone, &models_dir_clone, &app_handle_clone).await {
                Ok(_) => {
                    println!("✅ Model download completed: {}", model_name_owned);
                }
                Err(e) => {
                    eprintln!("❌ Model download failed: {} - {}", model_name_owned, e);
                    // Emit error event
                    let _ = app_handle_clone.emit("model-download-error",
                        serde_json::json!({
                            "model": model_name_owned,
                            "error": e.to_string()
                        })
                    );
                }
            }
        });

        println!("🎯 Download function returned successfully for model: {}", model_name);
        Ok(())
    }

    async fn download_model_internal(
        model: &WhisperModel,
        models_dir: &Path,
        app_handle: &AppHandle,
    ) -> Result<(), VoiceError> {
        println!("📥 Starting internal download for model: {}", model.name);

        let model_path = models_dir.join(&model.file_name);
        let temp_path = models_dir.join(format!("{}.tmp", model.file_name));

        println!("📂 Target path: {}", model_path.display());
        println!("📂 Temp path: {}", temp_path.display());

        // Check if curl is available
        println!("🔍 Checking if curl is available...");
        if let Err(e) = Command::new("curl").arg("--version").output() {
            return Err(VoiceError::Other(format!("curl not available: {}", e)));
        }
        println!("✅ curl is available");

        println!("🌐 Downloading from URL: {}", model.download_url);

        // Use curl for download (more reliable than reqwest for large files)
        let mut curl_cmd = Command::new("curl");
        curl_cmd.args([
            "-L", // Follow redirects
            "--progress-bar",
            "-v", // Verbose output for debugging
            "-o",
            &temp_path.to_string_lossy(),
            &model.download_url,
        ]);

        println!("🔧 Running curl command: {:?}", curl_cmd);

        let output = curl_cmd
            .output()
            .map_err(|e| VoiceError::Other(format!("Failed to start curl: {}", e)))?;

        println!("📊 curl exit status: {}", output.status);
        println!("📤 curl stdout length: {} bytes", output.stdout.len());
        println!("📤 curl stderr length: {} bytes", output.stderr.len());

        if !output.stderr.is_empty() {
            let stderr_output = String::from_utf8_lossy(&output.stderr);
            println!("📤 curl stderr: {}", stderr_output);
        }

        if !output.status.success() {
            let error_msg = String::from_utf8_lossy(&output.stderr);
            return Err(VoiceError::Other(format!("Download failed: {}", error_msg)));
        }

        // Verify the downloaded file
        println!("🔍 Verifying downloaded file...");
        if !temp_path.exists() {
            return Err(VoiceError::Other("Downloaded file not found".to_string()));
        }

        let file_size = fs::metadata(&temp_path)
            .map_err(|e| VoiceError::Other(format!("Failed to read file metadata: {}", e)))?
            .len();

        println!("📊 Downloaded file size: {} bytes ({} MB)", file_size, file_size / 1024 / 1024);

        if file_size == 0 {
            fs::remove_file(&temp_path).ok();
            return Err(VoiceError::Other("Downloaded file is empty".to_string()));
        }

        // Move temp file to final location
        println!("📁 Moving temp file to final location...");
        fs::rename(&temp_path, &model_path)
            .map_err(|e| VoiceError::Other(format!("Failed to save model file: {}", e)))?;

        println!("✅ File successfully moved to: {}", model_path.display());

        // Emit completion event
        println!("📡 Emitting download completion event");
        let completion_data = serde_json::json!({
            "model": model.name,
            "path": model_path.to_string_lossy()
        });
        println!("📦 Completion event data: {}", completion_data);

        match app_handle.emit("model-download-complete", completion_data) {
            Ok(_) => println!("✅ Download completion event emitted successfully"),
            Err(e) => println!("❌ Failed to emit download completion event: {}", e),
        }

        Ok(())
    }

    pub fn delete_model(&mut self, model_name: &str) -> Result<(), VoiceError> {
        let model_index = self.models
            .iter()
            .position(|m| m.name == model_name)
            .ok_or_else(|| VoiceError::Other(format!("Model '{}' not found", model_name)))?;

        let model = &mut self.models[model_index];
        
        if !model.is_downloaded {
            return Err(VoiceError::Other("Model not downloaded".to_string()));
        }

        if let Some(file_path) = &model.file_path {
            fs::remove_file(file_path)
                .map_err(|e| VoiceError::Other(format!("Failed to delete model file: {}", e)))?;
        }

        model.is_downloaded = false;
        model.file_path = None;
        model.download_progress = 0.0;

        // Emit deletion event
        self.emit_model_deleted(model_name);

        Ok(())
    }

    pub fn set_active_model(&mut self, model_name: &str) -> Result<(), VoiceError> {
        let model = self.models
            .iter()
            .find(|m| m.name == model_name && m.is_downloaded)
            .ok_or_else(|| VoiceError::Other(format!("Downloaded model '{}' not found", model_name)))?;

        // Set environment variable
        std::env::set_var("WHISPER_MODEL_PATH", &model.file_path.as_ref().unwrap());

        // 🔥 NEW: 预加载模型到GPU
        println!("🚀 Pre-loading model '{}' to GPU...", model_name);
        let model_path = model.file_path.as_ref().unwrap();

        // 启动异步任务预加载模型
        let app_handle = self.app_handle.clone();
        let model_name_clone = model_name.to_string();
        let model_path_clone = model_path.to_string();

        tokio::spawn(async move {
            match crate::voice_assistant::global_whisper::get_or_create_whisper_processor(&model_path_clone).await {
                Ok(_) => {
                    println!("✅ Model '{}' pre-loaded to GPU successfully", model_name_clone);
                    // 发送预加载成功事件
                    let _ = app_handle.emit("model-preloaded",
                        serde_json::json!({
                            "model": model_name_clone,
                            "status": "success"
                        })
                    );
                }
                Err(e) => {
                    println!("❌ Failed to pre-load model '{}' to GPU: {}", model_name_clone, e);
                    // 发送预加载失败事件
                    let _ = app_handle.emit("model-preloaded",
                        serde_json::json!({
                            "model": model_name_clone,
                            "status": "error",
                            "error": e.to_string()
                        })
                    );
                }
            }
        });

        // Emit active model change event
        self.emit_active_model_changed(model_name);

        Ok(())
    }

    fn emit_download_progress(&self, model_name: &str, progress: f64) {
        println!("📡 Emitting download progress event: {} = {}%", model_name, progress);
        let event_data = serde_json::json!({
            "model": model_name,
            "progress": progress
        });
        println!("📦 Event data: {}", event_data);

        match self.app_handle.emit("model-download-progress", event_data) {
            Ok(_) => println!("✅ Download progress event emitted successfully"),
            Err(e) => println!("❌ Failed to emit download progress event: {}", e),
        }
    }

    fn emit_model_deleted(&self, model_name: &str) {
        let _ = self.app_handle.emit("model-deleted", 
            serde_json::json!({
                "model": model_name
            })
        );
    }

    fn emit_active_model_changed(&self, model_name: &str) {
        let _ = self.app_handle.emit("active-model-changed", 
            serde_json::json!({
                "model": model_name
            })
        );
    }

    pub fn get_active_model(&self) -> Option<String> {
        std::env::var("WHISPER_MODEL_PATH")
            .ok()
            .and_then(|path| {
                self.models
                    .iter()
                    .find(|m| m.file_path.as_ref() == Some(&path))
                    .map(|m| m.name.clone())
            })
    }

    pub fn get_model_stats(&self) -> serde_json::Value {
        let total_models = self.models.len();
        let downloaded_models = self.models.iter().filter(|m| m.is_downloaded).count();
        let total_size_mb: f64 = self.models.iter().map(|m| m.size_mb).sum();
        let downloaded_size_mb: f64 = self.models
            .iter()
            .filter(|m| m.is_downloaded)
            .map(|m| m.size_mb)
            .sum();

        serde_json::json!({
            "total_models": total_models,
            "downloaded_models": downloaded_models,
            "total_size_mb": total_size_mb,
            "downloaded_size_mb": downloaded_size_mb,
            "models_dir": self.models_dir.to_string_lossy()
        })
    }
}

// Tauri commands
#[tauri::command]
pub async fn get_available_models(app_handle: AppHandle) -> Result<Vec<WhisperModel>, String> {
    println!("🎯 Tauri command get_available_models called");

    let manager = ModelManager::new(app_handle)
        .map_err(|e| {
            println!("❌ Failed to create ModelManager: {}", e);
            e.to_string()
        })?;

    let models = manager.get_models();

    println!("📋 Available models count: {}", models.len());
    for model in &models {
        println!("  - {}: {} ({} MB) - Downloaded: {}",
                model.name, model.display_name, model.size_mb, model.is_downloaded);
    }

    Ok(models)
}

#[tauri::command]
pub async fn download_model(app_handle: AppHandle, model_name: String) -> Result<String, String> {
    println!("🎯 Tauri command download_model called with model: {}", model_name);

    let mut manager = ModelManager::new(app_handle)
        .map_err(|e| {
            println!("❌ Failed to create ModelManager: {}", e);
            e.to_string()
        })?;

    println!("📝 ModelManager created successfully, calling download_model...");
    manager.download_model(&model_name)
        .await
        .map(|_| {
            println!("✅ download_model command completed successfully for: {}", model_name);
            format!("Started downloading model: {}", model_name)
        })
        .map_err(|e| {
            println!("❌ download_model command failed: {} - Error: {}", model_name, e);
            e.to_string()
        })
}

#[tauri::command]
pub async fn delete_model(app_handle: AppHandle, model_name: String) -> Result<String, String> {
    let mut manager = ModelManager::new(app_handle)
        .map_err(|e| e.to_string())?;
    
    manager.delete_model(&model_name)
        .map(|_| format!("Model deleted: {}", model_name))
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn set_active_model(app_handle: AppHandle, model_name: String) -> Result<String, String> {
    let mut manager = ModelManager::new(app_handle)
        .map_err(|e| e.to_string())?;
    
    manager.set_active_model(&model_name)
        .map(|_| format!("Active model set: {}", model_name))
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_active_model_info() -> Result<Option<String>, String> {
    Ok(std::env::var("WHISPER_MODEL_PATH").ok())
}

#[tauri::command]
pub async fn get_model_stats(app_handle: AppHandle) -> Result<serde_json::Value, String> {
    let manager = ModelManager::new(app_handle)
        .map_err(|e| e.to_string())?;
    Ok(manager.get_model_stats())
}

/// 🔥 NEW: 检查指定模型是否已预加载到GPU
#[tauri::command]
pub async fn check_model_loaded(model_name: String) -> Result<bool, String> {
    // 从ASR配置中获取当前活动模型
    let db = crate::database::Database::new()
        .await
        .map_err(|e| format!("Failed to initialize database: {}", e))?;

    let asr_config = db.get_asr_config()
        .await
        .map_err(|e| format!("Failed to get ASR config: {}", e))?;

    if let Some(config) = asr_config {
        if config.whisper_model.as_ref() == Some(&model_name) {
            // 检查全局WhisperRS管理器中的模型状态
            let status = crate::voice_assistant::global_whisper::get_global_whisper_status().await;
            let has_processor = status.get("has_processor").and_then(|v| v.as_bool()).unwrap_or(false);
            let current_model_path = status.get("current_model_path").and_then(|v| v.as_str());

            // 检查是否匹配对应的模型文件路径
            if let Some(path) = current_model_path {
                let is_matching = path.contains(&format!("ggml-{}.bin", model_name));
                return Ok(has_processor && is_matching);
            }

            return Ok(has_processor);
        }
    }

    Ok(false)
}

/// Get all available download sites
#[tauri::command]
pub async fn get_download_sites() -> Result<Vec<DownloadSite>, String> {
    Ok(DownloadSite::get_all_sites())
}

/// Test connectivity to all download sites
#[tauri::command]
pub async fn test_download_sites() -> Result<Vec<DownloadSite>, String> {
    let sites = DownloadSite::get_all_sites();

    // Test all sites in parallel using tokio
    let test_results = tokio::task::spawn_blocking(move || {
        sites.iter().map(|site| {
            let is_accessible = site.test_connectivity();
            (site.clone(), is_accessible)
        }).collect::<Vec<_>>()
    }).await.map_err(|e| e.to_string())?;

    // Return all sites with their accessibility status
    let sites_with_status: Vec<DownloadSite> = test_results.iter()
        .filter(|(_, accessible)| *accessible)
        .map(|(site, _)| site.clone())
        .collect();

    Ok(sites_with_status)
}