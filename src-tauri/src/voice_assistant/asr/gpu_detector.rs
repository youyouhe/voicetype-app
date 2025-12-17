use crate::voice_assistant::asr::whisper_rs::WhisperBackend;
use std::sync::{Mutex, OnceLock};

/// GPU后端检测器，用于检测系统中可用的GPU加速后端
#[derive(Clone)]
pub struct GpuDetector {
    available_backends: Vec<WhisperBackend>,
    preferred_backend: WhisperBackend,
}

impl GpuDetector {
    /// 创建新的GPU检测器并自动检测可用后端
    pub fn new() -> Self {
        let mut detector = Self {
            available_backends: Vec::new(),
            preferred_backend: WhisperBackend::CPU,
        };
        
        detector.detect_available_backends();
        detector.select_preferred_backend();
        
        detector
    }
    
    /// 检测系统中可用的GPU后端
    fn detect_available_backends(&mut self) {
        println!("🔍 Starting comprehensive GPU backend detection...");

        // 1. 检测CUDA (NVIDIA GPU)
        println!("   📋 Checking CUDA support (NVIDIA GPUs)...");
        if self.detect_cuda() {
            self.available_backends.push(WhisperBackend::CUDA);
            println!("✅ CUDA backend detected - Highest performance option");
        } else {
            println!("   ❌ CUDA not available");
        }

        // 2. 检测Vulkan (跨厂商GPU)
        println!("   📋 Checking Vulkan support (Cross-vendor GPUs)...");
        if self.detect_vulkan() {
            self.available_backends.push(WhisperBackend::Vulkan);
            println!("✅ Vulkan backend detected - Good performance compatibility");
        } else {
            println!("   ❌ Vulkan not available");
        }

        // 3. 检测Metal (Apple Silicon)
        println!("   📋 Checking Metal support (Apple Silicon)...");
        if self.detect_metal() {
            self.available_backends.push(WhisperBackend::Metal);
            println!("✅ Metal backend detected - Optimized for Apple Silicon");
        } else {
            println!("   ❌ Metal not available");
        }

        // 4. 检测OpenCL (作为fallback)
        println!("   📋 Checking OpenCL support (Legacy GPUs)...");
        if self.detect_opencl() {
            self.available_backends.push(WhisperBackend::OpenCL);
            println!("✅ OpenCL backend detected - Fallback for older GPUs");
        } else {
            println!("   ❌ OpenCL not available");
        }

        // 5. CPU总是可用
        self.available_backends.push(WhisperBackend::CPU);
        println!("✅ CPU backend always available - Baseline performance");

        println!("🎯 GPU backend detection completed. Found {} total backends.", self.available_backends.len());
    }
    
    /// 检测CUDA支持
    fn detect_cuda(&self) -> bool {
        // Simplified CUDA detection for Windows - avoid hanging on external commands
        if crate::utils::platform::is_windows() {
            // Only check file paths on Windows to avoid command hanging
            let cuda_paths = vec![
                "C:\\Program Files\\NVIDIA GPU Computing Toolkit\\CUDA",
                "C:\\Program Files (x86)\\NVIDIA GPU Computing Toolkit\\CUDA",
                "C:\\CUDA",
            ];

            for path in &cuda_paths {
                if std::path::Path::new(path).exists() {
                    println!("🎯 CUDA installation found at: {}", path);
                    return true;
                }
            }

            // Quick check for NVIDIA driver files
            if std::path::Path::new("C:\\Windows\\System32\\nvidia-smi.exe").exists() {
                println!("🚀 NVIDIA GPU detected via nvidia-smi.exe presence");
                return true;
            }
        } else {
            // On Linux/macOS, try nvidia-smi with timeout
            match std::process::Command::new("nvidia-smi")
                .output()
                .ok() {
                Some(output) if output.status.success() => {
                    let output_str = String::from_utf8_lossy(&output.stdout);
                    if output_str.contains("NVIDIA-SMI") && output_str.contains("Driver Version") {
                        println!("🚀 NVIDIA GPU detected via nvidia-smi");
                        return true;
                    }
                }
                _ => {}
            }

            // Check for CUDA paths
            let cuda_paths = vec!["/usr/local/cuda", "/opt/cuda", "/usr/cuda"];
            for path in &cuda_paths {
                if std::path::Path::new(path).exists() {
                    println!("🎯 CUDA installation found at: {}", path);
                    return true;
                }
            }
        }

        false
    }
    
    /// 检测Vulkan支持
    fn detect_vulkan(&self) -> bool {
        // Simplified Vulkan detection - only check for DLL files on Windows to avoid hanging
        let vulkan_libs = if crate::utils::platform::is_windows() {
            vec![
                "C:\\Windows\\System32\\vulkan-1.dll",
                "C:\\Windows\\SysWOW64\\vulkan-1.dll",
            ]
        } else {
            vec![
                "/usr/lib/x86_64-linux-gnu/libvulkan.so.1",
                "/usr/lib/x86_64-linux-gnu/libvulkan.so",
                "/usr/lib/libvulkan.so.1",
                "/usr/lib/libvulkan.so",
            ]
        };

        for lib_path in &vulkan_libs {
            if std::path::Path::new(lib_path).exists() {
                println!("🎮 Vulkan library found at: {}", lib_path);
                return true;
            }
        }

        false
    }
    
    /// 检测Metal支持 (macOS Apple Silicon)
    fn detect_metal(&self) -> bool {
        // Metal只在macOS上可用 - simple check without external commands
        if std::env::consts::OS.contains("macos") {
            // Assume Metal is available on all modern macOS versions
            println!("🍎 Metal assumed available on macOS");
            return true;
        }
        false
    }
    
    /// 检测OpenCL支持
    fn detect_opencl(&self) -> bool {
        // Simplified OpenCL detection - check only common DLL files
        let opencl_libs = if crate::utils::platform::is_windows() {
            vec![
                "C:\\Windows\\System32\\OpenCL.dll",
                "C:\\Windows\\SysWOW64\\OpenCL.dll",
            ]
        } else {
            vec![
                "/usr/lib/x86_64-linux-gnu/libOpenCL.so.1",
                "/usr/lib/x86_64-linux-gnu/libOpenCL.so",
                "/usr/lib/libOpenCL.so.1",
                "/usr/lib/libOpenCL.so",
            ]
        };

        for lib_path in &opencl_libs {
            if std::path::Path::new(lib_path).exists() {
                println!("⚡ OpenCL library found at: {}", lib_path);
                return true;
            }
        }

        false
    }
    
    /// 根据优先级选择最佳后端: CUDA > Vulkan > Metal > OpenCL > CPU
    fn select_preferred_backend(&mut self) {
        self.preferred_backend = self.available_backends
            .iter()
            .cloned()
            .min_by(|a, b| self.backend_priority(a).cmp(&self.backend_priority(b)))
            .unwrap_or(WhisperBackend::CPU);
    }
    
    /// 获取后端优先级 (数字越小优先级越高)
    pub fn backend_priority(&self, backend: &WhisperBackend) -> u8 {
        match backend {
            WhisperBackend::CUDA => 1,      // 最高优先级
            WhisperBackend::Vulkan => 2,    // 第二优先级
            WhisperBackend::Metal => 3,     // Apple Silicon优先级
            WhisperBackend::OpenCL => 4,    // Fallback
            WhisperBackend::CPU => 5,       // 最低优先级
        }
    }
    
    /// 获取首选后端
    pub fn get_preferred_backend(&self) -> &WhisperBackend {
        &self.preferred_backend
    }
    
    /// 获取所有可用后端
    pub fn get_available_backends(&self) -> &[WhisperBackend] {
        &self.available_backends
    }
    
    /// 检查特定后端是否可用
    pub fn is_backend_available(&self, backend: &WhisperBackend) -> bool {
        self.available_backends.contains(backend)
    }
    
    /// 手动设置首选后端
    pub fn set_preferred_backend(&mut self, backend: WhisperBackend) -> Result<(), String> {
        if self.is_backend_available(&backend) {
            self.preferred_backend = backend.clone();
            println!("🎯 Preferred backend manually set to: {}", backend);
            Ok(())
        } else {
            Err(format!("Backend {} is not available", backend))
        }
    }
    
    /// 获取后端信息字符串
    pub fn get_backend_info(&self) -> String {
        format!(
            "Available backends: [{}], Preferred: {}",
            self.available_backends
                .iter()
                .map(|b| b.to_string())
                .collect::<Vec<_>>()
                .join(", "),
            self.preferred_backend
        )
    }
}

/// 全局GPU检测器实例
static GLOBAL_GPU_DETECTOR: OnceLock<Mutex<GpuDetector>> = OnceLock::new();

/// 获取全局GPU检测器
pub fn get_gpu_detector() -> &'static Mutex<GpuDetector> {
    GLOBAL_GPU_DETECTOR.get_or_init(|| Mutex::new(GpuDetector::new()))
}

/// 重新检测GPU后端
pub fn redetect_gpu_backends() -> &'static Mutex<GpuDetector> {
    let new_detector = GpuDetector::new();
    let detector = get_gpu_detector();
    let mut guard = detector.lock().unwrap();
    *guard = new_detector;
    detector
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_gpu_detector_creation() {
        let detector = GpuDetector::new();
        assert!(!detector.get_available_backends().is_empty());
        assert!(detector.is_backend_available(&WhisperBackend::Cpu));
    }
    
    #[test]
    fn test_backend_priority() {
        let detector = GpuDetector::new();
        assert_eq!(detector.backend_priority(&WhisperBackend::CUDA), 1);
        assert_eq!(detector.backend_priority(&WhisperBackend::Vulkan), 2);
        assert_eq!(detector.backend_priority(&WhisperBackend::CPU), 5);
    }
}