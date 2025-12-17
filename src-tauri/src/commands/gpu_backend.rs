use crate::voice_assistant::asr::gpu_detector::get_gpu_detector;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct GpuBackendInfo {
    pub backend: String,
    pub available: bool,
    pub priority: i32,
    pub details: Option<String>,
    pub performance_score: Option<f64>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GpuBackendStatus {
    pub available_backends: Vec<GpuBackendInfo>,
    pub preferred_backend: String,
    pub total_detected: usize,
    pub detection_timestamp: String,
}

#[tauri::command]
pub fn get_gpu_backend_status() -> Result<GpuBackendStatus, String> {
    let detector = get_gpu_detector();
    let guard = detector.lock().map_err(|e| format!("Failed to acquire GPU detector lock: {}", e))?;

    let mut backend_infos = Vec::new();

    for backend in guard.get_available_backends() {
        let priority = guard.backend_priority(&backend);
        let info = GpuBackendInfo {
            backend: backend.to_string(),
            available: true,
            priority: priority.into(),
            details: Some(guard.get_backend_info()),
            performance_score: None, // TODO: Add performance scoring
        };
        backend_infos.push(info);
    }

    // Always include CPU as fallback
    backend_infos.push(GpuBackendInfo {
        backend: "CPU".to_string(),
        available: true,
        priority: 0,
        details: Some("Software rendering - always available".to_string()),
        performance_score: None,
    });

    // Sort by priority (descending)
    backend_infos.sort_by(|a, b| b.priority.cmp(&a.priority));

    let status = GpuBackendStatus {
        preferred_backend: guard.get_preferred_backend().to_string(),
        available_backends: backend_infos,
        total_detected: guard.get_available_backends().len(),
        detection_timestamp: chrono::Utc::now().to_rfc3339(),
    };

    Ok(status)
}

#[tauri::command]
pub fn set_preferred_gpu_backend(backend: String) -> Result<String, String> {
    // Validate backend string
    let valid_backends = ["CUDA", "Vulkan", "Metal", "CPU", "OpenCL"];
    if !valid_backends.contains(&backend.as_str()) {
        return Err(format!("Invalid backend '{}'. Valid options: {:?}", backend, valid_backends));
    }

    // TODO: Implement setting preferred backend in GpuDetector
    // For now, return current status
    Ok(format!("Preferred backend set to {} (implementation pending)", backend))
}

#[tauri::command]
pub fn redetect_gpu_backends() -> Result<String, String> {
    // TODO: Implement redetection logic
    // For now, return current status
    Ok("GPU backend redetection triggered (implementation pending)".to_string())
}

#[tauri::command]
pub fn get_backend_details(backend: Option<String>) -> Result<HashMap<String, String>, String> {
    let mut details = HashMap::new();

    details.insert("detection_status".to_string(), "completed".to_string());
    details.insert("last_check".to_string(), chrono::Utc::now().to_rfc3339());

    if let Some(backend_name) = backend {
        // TODO: Add backend-specific details
        details.insert("backend".to_string(), backend_name.clone());
        details.insert("status".to_string(), "available".to_string());
    }

    Ok(details)
}

#[tauri::command]
pub fn test_backend_performance(backend: String) -> Result<HashMap<String, String>, String> {
    let mut results = HashMap::new();

    results.insert("backend".to_string(), backend.clone());
    results.insert("test_status".to_string(), "not_implemented".to_string());
    results.insert("message".to_string(), "Performance testing not yet implemented".to_string());
    results.insert("timestamp".to_string(), chrono::Utc::now().to_rfc3339());

    Ok(results)
}