use std::path::{Path, PathBuf};
use std::fs;
use std::process::Command;
use serde::{Serialize, Deserialize};
use tauri::{AppHandle, Emitter, Manager};
use crate::voice_assistant::VoiceError;

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
            download_url: format!("https://huggingface.co/ggerganov/whisper.cpp/resolve/main/{}", file_name),
            is_downloaded: false,
            file_path: None,
            download_progress: 0.0,
            is_downloading: false,
        }
    }
}

pub struct ModelManager {
    models_dir: PathBuf,
    models: Vec<WhisperModel>,
    app_handle: AppHandle,
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
        };

        manager.initialize_models();
        Ok(manager)
    }

    fn initialize_models(&mut self) {
        // Define available models
        self.models = vec![
            WhisperModel::new(
                "tiny",
                "Tiny",
                "ggml-tiny.bin",
                39.0,
                "最快的模型，准确性较低，适合快速测试"
            ),
            WhisperModel::new(
                "base",
                "Base",
                "ggml-base.bin",
                142.0,
                "平衡速度和准确性，推荐日常使用"
            ),
            WhisperModel::new(
                "small",
                "Small",
                "ggml-small.bin",
                466.0,
                "更好的准确性，适合高质量转录"
            ),
            WhisperModel::new(
                "medium",
                "Medium",
                "ggml-medium.bin",
                1530.0,
                "高准确性，适合专业应用"
            ),
            WhisperModel::new(
                "large-v3",
                "Large v3",
                "ggml-large-v3.bin",
                2900.0,
                "最高准确性，需要较强硬件配置"
            ),
        ];

        // Check which models are already downloaded
        self.check_downloaded_models();
    }

    fn check_downloaded_models(&mut self) {
        for model in &mut self.models {
            let model_path = self.models_dir.join(&model.file_name);
            if model_path.exists() {
                model.is_downloaded = true;
                model.file_path = Some(model_path.to_string_lossy().to_string());
                model.download_progress = 100.0;
            }
        }
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

    pub async fn download_model(&mut self, model_name: String) -> Result<(), VoiceError> {
        let model_index = self.models
            .iter()
            .position(|m| m.name == model_name)
            .ok_or_else(|| VoiceError::Other(format!("Model '{}' not found", model_name)))?;

        let model_name_owned = model_name.clone(); // Create owned String
        let model_name_str = model_name_owned.as_str(); // Convert to &str
        
        // Mark as downloading before spawning task
        {
            let model = &mut self.models[model_index];
            if model.is_downloaded {
                return Err(VoiceError::Other("Model already downloaded".to_string()));
            }
            
            if model.is_downloading {
                return Err(VoiceError::Other("Model already downloading".to_string()));
            }
            
            model.is_downloading = true;
            model.download_progress = 0.0;
        }

        // Emit download start event
        self.emit_download_progress(model_name_str, 0.0);

        // Find the model and clone data for the async task
        let model_clone = self.models[model_index].clone();
        let models_dir_clone = self.models_dir.clone();
        let app_handle_clone = self.app_handle.clone();

        // Start download in background task
        tokio::spawn(async move {
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

        Ok(())
    }

    async fn download_model_internal(
        model: &WhisperModel,
        models_dir: &Path,
        app_handle: &AppHandle,
    ) -> Result<(), VoiceError> {
        let model_path = models_dir.join(&model.file_name);
        
        // Create a temporary file for download
        let temp_path = models_dir.join(format!("{}.tmp", model.file_name));

        // Use curl for download (more reliable than reqwest for large files)
        let output = Command::new("curl")
            .args([
                "-L", // Follow redirects
                "--progress-bar",
                "-o",
                &temp_path.to_string_lossy(),
                &model.download_url,
            ])
            .output()
            .map_err(|e| VoiceError::Other(format!("Failed to start curl: {}", e)))?;

        if !output.status.success() {
            let error_msg = String::from_utf8_lossy(&output.stderr);
            return Err(VoiceError::Other(format!("Download failed: {}", error_msg)));
        }

        // Verify the downloaded file
        if !temp_path.exists() {
            return Err(VoiceError::Other("Downloaded file not found".to_string()));
        }

        let file_size = fs::metadata(&temp_path)
            .map_err(|e| VoiceError::Other(format!("Failed to read file metadata: {}", e)))?
            .len();

        if file_size == 0 {
            fs::remove_file(&temp_path).ok();
            return Err(VoiceError::Other("Downloaded file is empty".to_string()));
        }

        // Move temp file to final location
        fs::rename(&temp_path, &model_path)
            .map_err(|e| VoiceError::Other(format!("Failed to save model file: {}", e)))?;

        // Emit completion event
        let _ = app_handle.emit("model-download-complete", 
            serde_json::json!({
                "model": model.name,
                "path": model_path.to_string_lossy()
            })
        );

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

        // Emit active model change event
        self.emit_active_model_changed(model_name);

        Ok(())
    }

    fn emit_download_progress(&self, model_name: &str, progress: f64) {
        let _ = self.app_handle.emit("model-download-progress", 
            serde_json::json!({
                "model": model_name,
                "progress": progress
            })
        );
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
    let manager = ModelManager::new(app_handle)
        .map_err(|e| e.to_string())?;
    Ok(manager.get_models())
}

#[tauri::command]
pub async fn download_model(app_handle: AppHandle, model_name: String) -> Result<String, String> {
    let mut manager = ModelManager::new(app_handle)
        .map_err(|e| e.to_string())?;
    
    manager.download_model(model_name)
        .await
        .map(|_| format!("Started downloading model: {}", model_name))
        .map_err(|e| e.to_string())
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