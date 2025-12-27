// src-tauri/src/main.rs

// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    // ==========================================
    //  Voice Assistant Tauri Application
    //  Ported from Python to Rust with Tauri v2
    // ==========================================

    voicetype_lib::run()
}
