#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_user_data_dir() {
        let data_dir = get_user_data_dir();
        println!("User data directory: {}", data_dir.display());

        // On Linux, should be something like /home/user/.local/share/com.martin.flash-input
        #[cfg(target_os = "linux")]
        assert!(data_dir.to_string_lossy().contains(".local"));

        // On Windows, should be something like C:\Users\User\AppData\Roaming\com.martin.flash-input
        #[cfg(target_os = "windows")]
        assert!(data_dir.to_string_lossy().contains("AppData"));

        // On macOS, should be something like /Users/user/Library/Application Support/com.martin.flash-input
        #[cfg(target_os = "macos")]
        assert!(data_dir.to_string_lossy().contains("Library"));
    }

    #[test]
    fn test_get_models_dir() {
        let models_dir = get_models_dir();
        println!("Models directory: {}", models_dir.display());

        // Should always end with "models" directory
        assert!(models_dir.to_string_lossy().ends_with("models"));

        // Parent should be the user data directory
        assert_eq!(models_dir.parent(), Some(get_user_data_dir().as_path()));
    }

    #[test]
    fn test_get_logs_dir() {
        let logs_dir = get_logs_dir();
        println!("Logs directory: {}", logs_dir.display());

        // Should always end with "logs" directory
        assert!(logs_dir.to_string_lossy().ends_with("logs"));

        // Parent should be the user data directory
        assert_eq!(logs_dir.parent(), Some(get_user_data_dir().as_path()));
    }

    #[test]
    fn test_is_windows() {
        let result = is_windows();
        println!("Is Windows platform: {}", result);

        #[cfg(target_os = "windows")]
        assert!(result);

        #[cfg(not(target_os = "windows"))]
        assert!(!result);
    }

    #[test]
    fn test_get_cuda_env_vars() {
        let vars = get_cuda_env_vars();
        println!("CUDA environment variables to check: {:?}", vars);

        // Should contain common CUDA environment variables
        assert!(vars.contains(&"CUDA_PATH".to_string()));
        assert!(vars.contains(&"CUDA_HOME".to_string()));
        assert!(vars.contains(&"CUDA_ROOT".to_string()));
    }

    #[test]
    fn test_platform_consistency() {
        // Test that all directory functions return valid paths
        let data_dir = get_user_data_dir();
        let models_dir = get_models_dir();
        let logs_dir = get_logs_dir();

        // Models and logs should be subdirectories of data dir
        assert!(models_dir.starts_with(&data_dir));
        assert!(logs_dir.starts_with(&data_dir));

        // All should have consistent structure
        assert_eq!(models_dir.parent(), Some(&data_dir));
        assert_eq!(logs_dir.parent(), Some(&data_dir));
    }
}