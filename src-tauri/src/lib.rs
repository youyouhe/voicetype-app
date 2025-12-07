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
    get_voice_assistant_config, test_asr, test_translation, get_system_info
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
    handle_asr_result
};

use std::sync::{Arc, Mutex};
use commands::DatabaseState;


#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
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

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
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
            handle_asr_result
        ])
        .setup(|app| {
            // Setup application if needed
            // Set the global app handle for event emission
            crate::voice_assistant::coordinator::set_app_handle(app.handle().clone());
            println!("✅ Global app handle set for event emission");
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
