pub mod voice_assistant;
pub mod commands;
pub mod database;

// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/


#[tauri::command]
fn greet(name: &str) -> String {
    let len = name.chars().count();
    if name.trim().is_empty() {
        return "嘿！你没有告诉我你的名字！".to_string();
    }
    format!("你好，{}！\n你的名字有 {} 个字。\n这条消息是 Rust 计算后返回的。", name, len)
}

#[tauri::command]
fn add(a: i32, b: i32) -> i32 {
    println!("Rust 收到了请求：计算 {} + {}", a, b); // 这行会在终端打印日志，方便调试
    a + b
}

// Re-export VoiceAssistant commands
use voice_assistant::{
    start_voice_assistant, stop_voice_assistant, get_voice_assistant_state,
    get_voice_assistant_config, test_asr, test_translation, get_system_info,
    // SystemTrayManager, GlobalHotkeyManager, ensure_dependencies,
    GlobalHotkeyManager, ensure_dependencies,
    // Model management commands
    get_available_models, download_model, delete_model, set_active_model,
    get_active_model_info, get_model_stats
};

// Import commands module
use commands::{
    test_frontend_backend_connection, test_connection_health,
    init_database, get_asr_config, save_asr_config,
    get_translation_config, save_translation_config,
    add_history_record, get_history_records, get_history_stats, cleanup_old_records,
    get_hotkey_config, save_hotkey_config,
    start_test_recording, get_audio_devices, test_microphone,
    test_asr_transcription,
    get_service_status, get_latency_data, get_usage_data,
    handle_asr_result,
    scan_whisper_models, set_active_whisper_model, get_active_whisper_model
};

// Import global whisper manager commands
use voice_assistant::global_whisper::{get_whisper_manager_status, reload_whisper_processor, clear_whisper_processor};

// Import GPU backend commands
use commands::gpu_backend::{
    get_gpu_backend_status, set_preferred_gpu_backend, redetect_gpu_backends,
    get_backend_details, test_backend_performance
};

use std::sync::{Arc, Mutex};
use commands::DatabaseState;


#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // Ensure system dependencies are available
    if let Err(e) = ensure_dependencies() {
        eprintln!("⚠️  Warning: Could not ensure system dependencies: {}", e);
    }

    // Initialize database state
    let db_state: DatabaseState = Arc::new(Mutex::new(None));

    // Initialize database immediately before creating the app
    println!("🚀 Initializing database on app startup...");
    let db_for_init = db_state.clone();
    tauri::async_runtime::block_on(async move {
        match commands::init_database_direct().await {
            Ok(db) => {
                println!("✅ Database initialization successful");
                *db_for_init.lock().unwrap() = Some(db);
            }
            Err(e) => eprintln!("❌ Failed to initialize database on startup: {}", e),
        }
    });

    // Initialize and perform GPU backend detection on startup
    println!("🔍 Initializing GPU backend detection...");
    let gpu_detector = voice_assistant::asr::gpu_detector::get_gpu_detector();
    let detector_guard = gpu_detector.lock().unwrap();

    println!("📊 GPU Backend Detection Results:");
    println!("   Available Backends:");
    for backend in detector_guard.get_available_backends() {
        let priority = detector_guard.backend_priority(&backend);
        let status = if backend == detector_guard.get_preferred_backend() {
            "✅ SELECTED"
        } else {
            "✓ Available"
        };
        println!("     - {} (Priority: {}) {}", backend, priority, status);
    }

    println!("   Preferred Backend: {}", detector_guard.get_preferred_backend());
    println!("   Backend Info: {}", detector_guard.get_backend_info());
    println!("🎯 GPU backend detection completed!");

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .setup(|app| {
            // Set the global app handle for event emission
            crate::voice_assistant::coordinator::set_app_handle(app.handle().clone());
            println!("✅ Global app handle set for event emission");

            // Initialize system tray manager - DISABLED DUE TO COMPILATION ISSUES
            // let system_tray_manager = Arc::new(Mutex::new(
            //     SystemTrayManager::new(app.handle().clone())
            // ));
            // app.manage(system_tray_manager.clone());

            // // Create system tray icon with menu items
            // match SystemTrayManager::create_tray_icon() {
            //     Ok(tray) => {
            //         if let Err(e) = tray.build(app) {
            //             eprintln!("⚠️  Failed to build system tray: {}", e);
            //         } else {
            //             println!("✅ System tray created successfully");
            //         }
            //     }
            //     Err(e) => eprintln!("⚠️  Failed to create system tray: {}", e),
            // }

            // Create overlay window (initially hidden) - TEMPORARILY DISABLED
            // let tray_manager_ref = app.state::<Arc<Mutex<SystemTrayManager>>>();
            // if let Ok(tray_manager) = tray_manager_ref.try_lock() {
            //     if let Err(e) = tray_manager.create_overlay_window() {
            //         eprintln!("⚠️  Failed to create overlay window: {}", e);
            //     } else {
            //         println!("✅ Overlay window created successfully");
            //     }
            // }
            println!("ℹ️  Overlay window creation disabled for evaluation");

            // Initialize and register global hotkeys
            let hotkey_manager = GlobalHotkeyManager::new(app.handle().clone());

            if let Err(e) = hotkey_manager.register_global_hotkeys() {
                eprintln!("❌ Failed to register global hotkeys: {}", e);
            } else {
                println!("ℹ️  Global hotkey registration skipped (feature disabled)");
            }

            Ok(())
        })
        .manage(db_state)
        .invoke_handler(tauri::generate_handler![
            greet,
            add,
            start_voice_assistant,
            stop_voice_assistant,
            get_voice_assistant_state,
            get_voice_assistant_config,
            test_asr,
            test_translation,
            get_system_info,
            test_frontend_backend_connection,
            test_connection_health,
            // Database commands
            init_database,
            get_asr_config,
            save_asr_config,
            get_translation_config,
            save_translation_config,
            add_history_record,
            get_history_records,
            get_history_stats,
            cleanup_old_records,
            get_hotkey_config,
            save_hotkey_config,
            // Audio and testing commands
            start_test_recording,
            get_audio_devices,
            test_microphone,
            test_asr_transcription,
            // Live data commands
            get_service_status,
            get_latency_data,
            get_usage_data,
            handle_asr_result,
            // Model management commands - ONLY use file-based scanning commands
            // scan_whisper_models,      // ⭐️ ACTIVE - Scans actual model files
            // set_active_whisper_model, // ⭐️ ACTIVE - Sets model via environment
            // get_active_whisper_model, // ⭐️ ACTIVE - Gets active model from env
            
            // ❌ DISABLED - Redundant hardcoded model management
            // get_available_models,     // Conflicts with scan_whisper_models
            // download_model,           // Uses hardcoded URLs, not flexible
            // delete_model,             // Uses hardcoded model list
            // set_active_model,         // Conflicts with set_active_whisper_model  
            // get_active_model_info,    // Uses hardcoded model list
            // get_model_stats,          // Uses hardcoded model list
            
            // 🎯 TEMP: Keep both for now during transition
            scan_whisper_models,
            set_active_whisper_model,
            get_active_whisper_model,
            get_available_models,
            download_model,
            delete_model,
            set_active_model,
            get_active_model_info,
            get_model_stats,
            // Global WhisperRS manager commands
            get_whisper_manager_status,
            reload_whisper_processor,
            clear_whisper_processor,
            // GPU backend management commands
            get_gpu_backend_status,
            set_preferred_gpu_backend,
            redetect_gpu_backends,
            get_backend_details,
            test_backend_performance
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
