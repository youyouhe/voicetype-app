/// 跨平台工具模块，处理Windows/Linux/macOS平台差异
use std::path::PathBuf;

/// 获取用户数据目录
pub fn get_user_data_dir() -> PathBuf {
    #[cfg(target_os = "windows")]
    {
        // Windows: %APPDATA%/com.martin.flash-input/
        if let Ok(appdata) = std::env::var("APPDATA") {
            PathBuf::from(appdata).join("com.martin.flash-input")
        } else if let Ok(userprofile) = std::env::var("USERPROFILE") {
            // Fallback to User Profile
            PathBuf::from(userprofile)
                .join("AppData")
                .join("Roaming")
                .join("com.martin.flash-input")
        } else {
            // Last resort
            PathBuf::from("C:\\Users\\Public\\AppData\\com.martin.flash-input")
        }
    }
    #[cfg(any(target_os = "macos", target_os = "linux"))]
    {
        // macOS/Linux: ~/.local/share/com.martin.flash-input/
        if let Ok(home) = std::env::var("HOME") {
            PathBuf::from(home)
                .join(".local")
                .join("share")
                .join("com.martin.flash-input")
        } else {
            PathBuf::from("./data")  // Fallback
        }
    }
}

/// 获取用户主目录
pub fn get_home_dir() -> PathBuf {
    #[cfg(target_os = "windows")]
    {
        // Windows: %USERPROFILE%
        std::env::var("USERPROFILE")
            .map(PathBuf::from)
            .unwrap_or_else(|_| PathBuf::from("C:\\Users\\Public"))
    }
    #[cfg(any(target_os = "macos", target_os = "linux"))]
    {
        // macOS/Linux: $HOME
        std::env::var("HOME")
            .map(PathBuf::from)
            .unwrap_or_else(|_| PathBuf::from("/"))
    }
}

/// 获取模型存储目录
pub fn get_models_dir() -> PathBuf {
    get_user_data_dir().join("models")
}

/// 获取数据库存储目录
pub fn get_database_dir() -> PathBuf {
    get_user_data_dir().join("databases")
}

/// 获取平台信息
pub fn get_platform_info() -> (String, String) {
    let os = std::env::consts::OS.to_string();
    let arch = std::env::consts::ARCH.to_string();
    (os, arch)
}

/// 检查是否为Windows平台
pub fn is_windows() -> bool {
    cfg!(target_os = "windows")
}

/// 检查是否为macOS平台
pub fn is_macos() -> bool {
    cfg!(target_os = "macos")
}

/// 检查是否为Linux平台
pub fn is_linux() -> bool {
    cfg!(target_os = "linux")
}

/// 获取CUDA相关环境变量
pub fn get_cuda_env_vars() -> Vec<String> {
    #[cfg(target_os = "windows")]
    {
        vec![
            "CUDA_PATH".to_string(),
            "CUDA_HOME".to_string(),
            "CUDA_ROOT".to_string(),
        ]
    }
    #[cfg(any(target_os = "macos", target_os = "linux"))]
    {
        vec![
            "CUDA_PATH".to_string(),
            "CUDA_HOME".to_string(),
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_platform_detection() {
        println!("Platform detection:");
        println!("  OS: {}", std::env::consts::OS);
        println!("  ARCH: {}", std::env::consts::ARCH);
        println!("  Family: {}", std::env::consts::FAMILY);
    }

    #[test]
    fn test_directories() {
        let user_data = get_user_data_dir();
        let models_dir = get_models_dir();
        let db_dir = get_database_dir();

        println!("User data dir: {:?}", user_data);
        println!("Models dir: {:?}", models_dir);
        println!("Database dir: {:?}", db_dir);
    }

    #[test]
    fn test_cuda_env_vars() {
        let cuda_vars = get_cuda_env_vars();
        println!("CUDA environment variables: {:?}", cuda_vars);

        for var in &cuda_vars {
            if let Ok(value) = std::env::var(var) {
                println!("  {} = {}", var, value);
            }
        }
    }
}