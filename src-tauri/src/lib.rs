pub mod voice_assistant;

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

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            greet,
            add,
            start_voice_assistant,
            stop_voice_assistant,
            get_voice_assistant_state,
            get_voice_assistant_config,
            test_asr,
            test_translation,
            get_system_info
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
