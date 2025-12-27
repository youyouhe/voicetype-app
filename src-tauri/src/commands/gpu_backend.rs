use crate::voice_assistant::asr::gpu_detector::get_gpu_detector;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::process::Command;

#[cfg(target_os = "windows")]
use std::os::windows::process::CommandExt;

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

/// NVIDIA 驱动版本检查结果
#[derive(Debug, Serialize, Deserialize)]
pub struct NvidiaDriverInfo {
    pub installed: bool,
    pub driver_version: Option<String>,
    pub cuda_version: Option<String>,
    pub minimum_required: String,
    pub is_compatible: bool,
    pub gpu_name: Option<String>,
    pub error_message: Option<String>,
}

/// 根据驱动版本推断 CUDA 版本（简化映射）
fn infer_cuda_version(driver_version: &str) -> Option<String> {
    // 解析主版本号
    let major = driver_version
        .split('.')
        .next()
        .and_then(|v| v.parse::<u32>().ok())
        .unwrap_or(0);

    // 驱动版本到 CUDA 版本的映射（基于 NVIDIA 兼容性表）
    let cuda_version = match major {
        570.. => Some("12.8".to_string()),
        560..=569 => Some("12.6".to_string()),
        550..=559 => Some("12.4".to_string()),
        545..=549 => Some("12.3".to_string()),
        535..=544 => Some("12.2".to_string()),
        525..=534 => Some("12.1".to_string()),
        522..=524 => Some("12.0".to_string()),
        515..=521 => Some("11.8".to_string()),
        _ => Some("11.x".to_string()), // 默认值
    };

    cuda_version
}

/// 检查 NVIDIA 驱动版本是否兼容 CUDA 11.8
/// CUDA 11.8 需要驱动版本 >= 522.06 (Tesla) 或 >= 522.25 (GeForce)
#[tauri::command]
pub fn check_nvidia_driver() -> NvidiaDriverInfo {
    const MIN_DRIVER_VERSION: u32 = 522; // CUDA 11.8 最低要求

    // Windows 上检查 nvidia-smi
    let nvidia_smi_path = if cfg!(windows) {
        "C:\\Windows\\System32\\nvidia-smi.exe"
    } else {
        "nvidia-smi"
    };

    // 检查 nvidia-smi 是否存在
    if !std::path::Path::new(nvidia_smi_path).exists() {
        return NvidiaDriverInfo {
            installed: false,
            driver_version: None,
            cuda_version: None,
            minimum_required: format!("{}.xx", MIN_DRIVER_VERSION),
            is_compatible: false,
            gpu_name: None,
            error_message: Some("NVIDIA driver not found. Please install NVIDIA GPU drivers.".to_string()),
        };
    }

    // 执行 nvidia-smi 获取驱动信息，设置超时
    let output = match Command::new(nvidia_smi_path)
        .args(&["--query-gpu=driver_version,name", "--format=csv,noheader,nounits"])
        .creation_flags(0x08000000) // CREATE_NO_WINDOW on Windows
        .output()
    {
        Ok(output) => output,
        Err(e) => {
            return NvidiaDriverInfo {
                installed: true,
                driver_version: None,
                cuda_version: None,
                minimum_required: format!("{}.xx", MIN_DRIVER_VERSION),
                is_compatible: false,
                gpu_name: None,
                error_message: Some(format!("Failed to execute nvidia-smi: {} (driver may be corrupted)", e)),
            };
        }
    };

    if !output.status.success() {
        let _stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let _stderr = String::from_utf8_lossy(&output.stderr).to_string();
        return NvidiaDriverInfo {
            installed: true,
            driver_version: None,
            cuda_version: None,
            minimum_required: format!("{}.xx", MIN_DRIVER_VERSION),
            is_compatible: false,
            gpu_name: None,
            error_message: Some(format!(
                "nvidia-smi exited with error code: {}. This usually means the NVIDIA driver is corrupted or not properly installed. Please reinstall the driver.",
                output.status.code().unwrap_or(-1)
            )),
        };
    }

    // 解析输出：格式为 "560.94, NVIDIA GeForce GTX 1070 Ti"
    let output_str = String::from_utf8_lossy(&output.stdout);
    let parts: Vec<&str> = output_str.trim().split(',').collect();

    if parts.len() < 2 {
        return NvidiaDriverInfo {
            installed: true,
            driver_version: None,
            cuda_version: None,
            minimum_required: format!("{}.xx", MIN_DRIVER_VERSION),
            is_compatible: false,
            gpu_name: None,
            error_message: Some(format!("Failed to parse nvidia-smi output: '{}'", output_str.trim())),
        };
    }

    let driver_version = parts[0].trim().to_string();
    let gpu_name = parts[1].trim().to_string();

    // CUDA 版本从驱动版本推断（简化处理）
    let cuda_version = infer_cuda_version(&driver_version);

    // 解析主版本号 (例如 "522.25" -> 522)
    let version_major = driver_version
        .split('.')
        .next()
        .and_then(|v| v.parse::<u32>().ok())
        .unwrap_or(0);

    let is_compatible = version_major >= MIN_DRIVER_VERSION;

    NvidiaDriverInfo {
        installed: true,
        driver_version: Some(driver_version.clone()),
        cuda_version,
        minimum_required: format!("{}.xx", MIN_DRIVER_VERSION),
        is_compatible,
        gpu_name: Some(gpu_name),
        error_message: if !is_compatible {
            Some(format!(
                "Driver version {} is too old. CUDA 11.8 requires version {}.xx or higher.",
                driver_version, MIN_DRIVER_VERSION
            ))
        } else {
            None
        },
    }
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