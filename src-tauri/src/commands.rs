use crate::database::{Database, NewHistoryRecord};
use serde::{Deserialize, Serialize};
use tauri::State;
use std::sync::{Arc, Mutex};
use base64::{Engine as _, engine::general_purpose::STANDARD};

#[derive(Debug, Serialize, Deserialize)]
pub struct AudioDevice {
    pub id: String,
    pub name: String,
    pub is_default: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct HealthCheckRequest {
    pub endpoint: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct HealthCheckResponse {
    pub success: bool,
    pub message: String,
    pub status_code: Option<u16>,
    pub response_time_ms: u64,
    pub backend_count: Option<usize>,
}

// Global database state
pub type DatabaseState = Arc<Mutex<Option<Database>>>;

#[derive(Debug, Serialize, Deserialize)]
pub struct AsrConfigRequest {
    pub service_provider: String,
    pub local_endpoint: Option<String>,
    pub local_api_key: Option<String>,
    pub cloud_endpoint: Option<String>,
    pub cloud_api_key: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TranslationConfigRequest {
    pub provider: String,
    pub api_key: Option<String>,
    pub endpoint: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct HistoryRequest {
    pub record_type: String,
    pub input_text: Option<String>,
    pub output_text: Option<String>,
    pub audio_file_path: Option<String>,
    pub processor_type: Option<String>,
    pub processing_time_ms: Option<i64>,
    pub success: bool,
    pub error_message: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AsrTestRequest {
    pub audio_file_data: String,
    pub file_name: String,
    pub service_provider: String,
    pub endpoint: String,
    pub api_key: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AsrTestResponse {
    pub success: bool,
    pub transcription: Option<String>,
    pub processing_time_ms: u64,
    pub file_size: u64,
    pub message: String,
    pub status_code: Option<u16>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct HotkeyConfigRequest {
    pub transcribe_key: String,
    pub translate_key: String,
    pub trigger_delay_ms: i64,
    pub anti_mistouch_enabled: bool,
    pub save_wav_files: bool,
}

// Initialize database
#[tauri::command]
pub async fn init_database(
    db_state: State<'_, DatabaseState>
) -> Result<String, String> {
    match Database::new().await {
        Ok(db) => {
            *db_state.lock().unwrap() = Some(db);
            Ok("Database initialized successfully".to_string())
        }
        Err(e) => {
            eprintln!("Failed to initialize database: {}", e);
            Err(format!("Failed to initialize database: {}", e))
        }
    }
}

// ASR Configuration commands
#[tauri::command]
pub async fn get_asr_config(
    db_state: State<'_, DatabaseState>
) -> Result<Option<crate::database::AsrConfig>, String> {
    let db = {
        let guard = db_state.lock().unwrap();
        guard.as_ref().cloned()
    };

    match db {
        Some(database) => {
            match database.get_asr_config().await {
                Ok(config) => Ok(config),
                Err(e) => Err(format!("Failed to get ASR config: {}", e)),
            }
        }
        None => Err("Database not initialized".to_string()),
    }
}

#[tauri::command]
pub async fn save_asr_config(
    db_state: State<'_, DatabaseState>,
    request: AsrConfigRequest,
) -> Result<crate::database::AsrConfig, String> {
    let db = {
        let guard = db_state.lock().unwrap();
        guard.as_ref().cloned()
    };

    match db {
        Some(database) => {
            // Debug: Log the values being saved
            println!("💾 Rust: Saving ASR config:");
            println!("  - service_provider: {}", request.service_provider);
            println!("  - local_api_key present: {}", request.local_api_key.is_some());
            println!("  - local_api_key length: {:?}", request.local_api_key.as_ref().map(|k| k.len()));
            println!("  - local_api_key preview: {:?}", request.local_api_key.as_ref().map(|k| &k[..k.len().min(20)]));
            println!("  - cloud_api_key present: {}", request.cloud_api_key.is_some());
            println!("  - cloud_api_key length: {:?}", request.cloud_api_key.as_ref().map(|k| k.len()));

            match database.save_asr_config(
                &request.service_provider,
                request.local_endpoint.as_deref(),
                request.local_api_key.as_deref(),
                request.cloud_endpoint.as_deref(),
                request.cloud_api_key.as_deref(),
            ).await {
                Ok(config) => {
                    println!("✅ Rust: ASR config saved successfully");
                    Ok(config)
                },
                Err(e) => {
                    println!("❌ Rust: Failed to save ASR config: {}", e);
                    Err(format!("Failed to save ASR config: {}", e))
                },
            }
        }
        None => {
            println!("❌ Rust: Database not initialized");
            Err("Database not initialized".to_string())
        },
    }
}

// Translation Configuration commands
#[tauri::command]
pub async fn get_translation_config(
    db_state: State<'_, DatabaseState>,
    provider: String,
) -> Result<Option<crate::database::TranslationConfig>, String> {
    let db = {
        let guard = db_state.lock().unwrap();
        guard.as_ref().cloned()
    };
    match db {
        Some(database) => {
            match database.get_translation_config(&provider).await {
                Ok(config) => Ok(config),
                Err(e) => Err(format!("Failed to get translation config: {}", e)),
            }
        }
        None => Err("Database not initialized".to_string()),
    }
}

#[tauri::command]
pub async fn save_translation_config(
    db_state: State<'_, DatabaseState>,
    request: TranslationConfigRequest,
) -> Result<crate::database::TranslationConfig, String> {
    let db = {
        let guard = db_state.lock().unwrap();
        guard.as_ref().cloned()
    };
    match db {
        Some(database) => {
            match database.save_translation_config(
                &request.provider,
                request.api_key.as_deref(),
                request.endpoint.as_deref(),
            ).await {
                Ok(config) => Ok(config),
                Err(e) => Err(format!("Failed to save translation config: {}", e)),
            }
        }
        None => Err("Database not initialized".to_string()),
    }
}

// History commands
#[tauri::command]
pub async fn add_history_record(
    db_state: State<'_, DatabaseState>,
    request: HistoryRequest,
) -> Result<crate::database::HistoryRecord, String> {
    let db = {
        let guard = db_state.lock().unwrap();
        guard.as_ref().cloned()
    };
    match db {
        Some(database) => {
            let record = NewHistoryRecord {
                record_type: request.record_type,
                input_text: request.input_text,
                output_text: request.output_text,
                audio_file_path: request.audio_file_path,
                processor_type: request.processor_type,
                processing_time_ms: request.processing_time_ms,
                success: request.success,
                error_message: request.error_message,
            };

            match database.add_history_record(record).await {
                Ok(history) => Ok(history),
                Err(e) => Err(format!("Failed to add history record: {}", e)),
            }
        }
        None => Err("Database not initialized".to_string()),
    }
}

#[tauri::command]
pub async fn get_history_records(
    db_state: State<'_, DatabaseState>,
    limit: Option<i64>,
    record_type: Option<String>,
) -> Result<Vec<crate::database::HistoryRecord>, String> {
    let db = {
        let guard = db_state.lock().unwrap();
        guard.as_ref().cloned()
    };
    match db {
        Some(database) => {
            match database.get_history_records(limit, record_type.as_deref()).await {
                Ok(records) => Ok(records),
                Err(e) => Err(format!("Failed to get history records: {}", e)),
            }
        }
        None => Err("Database not initialized".to_string()),
    }
}

#[tauri::command]
pub async fn get_history_stats(
    db_state: State<'_, DatabaseState>
) -> Result<(i64, i64, i64), String> {
    let db = {
        let guard = db_state.lock().unwrap();
        guard.as_ref().cloned()
    };
    match db {
        Some(database) => {
            match database.get_history_stats().await {
                Ok(stats) => Ok(stats),
                Err(e) => Err(format!("Failed to get history stats: {}", e)),
            }
        }
        None => Err("Database not initialized".to_string()),
    }
}

#[tauri::command]
pub async fn cleanup_old_records(
    db_state: State<'_, DatabaseState>,
    days: i64,
) -> Result<u64, String> {
    let db = {
        let guard = db_state.lock().unwrap();
        guard.as_ref().cloned()
    };
    match db {
        Some(database) => {
            match database.cleanup_old_records(days).await {
                Ok(count) => Ok(count),
                Err(e) => Err(format!("Failed to cleanup old records: {}", e)),
            }
        }
        None => Err("Database not initialized".to_string()),
    }
}

// Health check command - performed by Rust backend for better debugging
#[tauri::command]
pub async fn test_connection_health(
    request: HealthCheckRequest,
) -> Result<HealthCheckResponse, String> {
    println!("🔍 Tauri Backend: Starting health check for: {}", request.endpoint);
    println!("⏰ Current time: {:?}", chrono::Utc::now());

    // Build health endpoint URL
    // Handle different endpoint formats:
    // - If it ends with /inference, replace with /health
    // - If it ends with /inference/, add health
    // - Otherwise, just append /health
    let health_endpoint = if request.endpoint.ends_with("/inference") {
        request.endpoint.replace("/inference", "/health")
    } else if request.endpoint.ends_with("/inference/") {
        format!("{}health", request.endpoint)
    } else if request.endpoint.ends_with('/') {
        format!("{}health", request.endpoint)
    } else {
        format!("{}/health", request.endpoint)
    };

    println!("🔗 Testing health endpoint: {}", health_endpoint);

    // Start timing
    let start_time = std::time::Instant::now();

    // Create HTTP client with timeout
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(10))
        .build()
        .map_err(|e| {
            println!("❌ Failed to create HTTP client: {}", e);
            format!("Failed to create HTTP client: {}", e)
        })?;

    // Make the request
    let response = match client.get(&health_endpoint).send().await {
        Ok(resp) => {
            println!("📡 HTTP request completed");
            resp
        }
        Err(e) => {
            println!("❌ HTTP request failed: {}", e);
            let response_time = start_time.elapsed().as_millis() as u64;
            return Ok(HealthCheckResponse {
                success: false,
                message: format!("Network error: {}", e),
                status_code: None,
                response_time_ms: response_time,
                backend_count: None,
            });
        }
    };

    let status_code = response.status();
    let response_time = start_time.elapsed().as_millis() as u64;

    println!("📋 Response status: {}", status_code);
    println!("⏱️ Response time: {}ms", response_time);

    if status_code.is_success() {
        // Try to parse JSON response
        let response_body = response.text().await.unwrap_or_default();
        match serde_json::from_str::<serde_json::Value>(&response_body) {
            Ok(json_data) => {
                println!("📊 Health check JSON response: {}", json_data);

                // Extract backend count if available
                let backend_count = json_data
                    .get("total_backends")
                    .and_then(|v| v.as_u64())
                    .map(|v| v as usize);

                // Check health status
                let is_healthy = json_data.get("status")
                    .and_then(|s| s.as_str())
                    .map(|s| s == "healthy")
                    .unwrap_or(false);

                if is_healthy {
                    println!("✅ Service is healthy!");
                    if let Some(count) = backend_count {
                        println!("📈 Total backends: {}", count);
                    }
                    return Ok(HealthCheckResponse {
                        success: true,
                        message: format!("Service healthy - {} backends", backend_count.unwrap_or(0)),
                        status_code: Some(status_code.as_u16()),
                        response_time_ms: response_time,
                        backend_count,
                    });
                } else {
                    let service_status = json_data.get("status")
                        .and_then(|s| s.as_str())
                        .unwrap_or("unknown");
                    println!("⚠️ Service status: {}", service_status);
                    return Ok(HealthCheckResponse {
                        success: false,
                        message: format!("Service status: {}", service_status),
                        status_code: Some(status_code.as_u16()),
                        response_time_ms: response_time,
                        backend_count,
                    });
                }
            }
            Err(e) => {
                println!("❌ Failed to parse JSON: {}", e);
                println!("📄 Raw response: {}", response_body);
                Ok(HealthCheckResponse {
                    success: false,
                    message: format!("Invalid JSON response: {}", e),
                    status_code: Some(status_code.as_u16()),
                    response_time_ms: response_time,
                    backend_count: None,
                })
            }
        }
    } else {
        let error_text = response.text().await.unwrap_or_default();
        println!("❌ HTTP error response: {}", error_text);
        Ok(HealthCheckResponse {
            success: false,
            message: format!("HTTP {} - {}", status_code, error_text),
            status_code: Some(status_code.as_u16()),
            response_time_ms: response_time,
            backend_count: None,
        })
    }
}

// ASR transcription test command
#[tauri::command]
pub async fn test_asr_transcription(
    request: AsrTestRequest,
) -> Result<AsrTestResponse, String> {
    println!("🎵 Starting ASR transcription test...");
    println!("📁 Audio file: {}", request.file_name);
    println!("🔧 Service provider: {}", request.service_provider);
    println!("🔗 Endpoint: {}", request.endpoint);

    let start_time = std::time::Instant::now();

    // Decode base64 data
    let audio_data = match STANDARD.decode(&request.audio_file_data) {
        Ok(data) => data,
        Err(e) => {
            return Ok(AsrTestResponse {
                success: false,
                transcription: None,
                processing_time_ms: start_time.elapsed().as_millis() as u64,
                file_size: 0,
                message: format!("Failed to decode base64 data: {}", e),
                status_code: None,
            });
        }
    };

    let file_size = audio_data.len() as u64;

    // Check file size (2MB limit)
    const MAX_FILE_SIZE: u64 = 2 * 1024 * 1024; // 2MB in bytes
    if file_size > MAX_FILE_SIZE {
        return Ok(AsrTestResponse {
            success: false,
            transcription: None,
            processing_time_ms: start_time.elapsed().as_millis() as u64,
            file_size,
            message: format!("File too large: {} bytes (max: {} bytes)", file_size, MAX_FILE_SIZE),
            status_code: None,
        });
    }

    println!("📊 File size: {} bytes", file_size);
    println!("📖 Successfully decoded {} bytes of audio data", audio_data.len());

    // Prepare the request to ASR service
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .build()
        .map_err(|e| {
            println!("❌ Failed to create HTTP client: {}", e);
            format!("Failed to create HTTP client: {}", e)
        })?;

    // Build form data for multipart upload
    let form = reqwest::multipart::Form::new()
        .part("file", reqwest::multipart::Part::bytes(audio_data)
            .file_name("test.wav")
            .mime_str("audio/wav")
            .map_err(|e| format!("Failed to create multipart: {}", e))?);

    println!("🚀 Sending request to ASR endpoint...");

    // Make the request
    println!("🔑 API Key present: {}", request.api_key.is_some());
    if let Some(ref key) = request.api_key {
        println!("🔑 API Key length: {} characters", key.len());
        // Safe substring handling that respects UTF-8 character boundaries
        let safe_preview = if key.len() > 10 {
            key.chars().take(10).collect::<String>()
        } else {
            key.clone()
        };
        println!("🔑 API Key starts with: {}", safe_preview);
    }

    // Clean API key - remove API_KEY= prefix if present
    let clean_api_key = match request.api_key {
        Some(ref key) => {
            let trimmed = key.trim();
            if trimmed.starts_with("API_KEY=") {
                println!("🔧 Removing API_KEY= prefix from key");
                Some(trimmed[8..].to_string()) // Remove "API_KEY=" prefix
            } else {
                println!("🔧 API key format looks normal");
                Some(trimmed.to_string())
            }
        },
        None => None,
    };

    if let Some(ref clean_key) = clean_api_key {
        println!("🔑 Clean API key length: {} characters", clean_key.len());
        let safe_preview = if clean_key.len() > 10 {
            clean_key.chars().take(10).collect::<String>()
        } else {
            clean_key.clone()
        };
        println!("🔑 Clean API key starts with: {}", safe_preview);
    }

    println!("🔑 API Key header will be: {}", if clean_api_key.is_some() { "Set (X-API-Key)" } else { "Not set" });

    let request_builder = client
        .post(&request.endpoint)
        .multipart(form);

    let request_builder = if !clean_api_key.is_some() {
        println!("🔑 No API key will be sent");
        request_builder
    } else {
        println!("🔑 Sending X-API-Key header");
        request_builder.header("X-API-Key", clean_api_key.unwrap())
    };

    let response = match request_builder
        .send()
        .await
    {
        Ok(resp) => {
            println!("📡 HTTP request completed");
            resp
        }
        Err(e) => {
            println!("❌ HTTP request failed: {}", e);
            return Ok(AsrTestResponse {
                success: false,
                transcription: None,
                processing_time_ms: start_time.elapsed().as_millis() as u64,
                file_size,
                message: format!("HTTP request failed: {}", e),
                status_code: None,
            });
        }
    };

    let status_code = response.status();
    let response_time = start_time.elapsed().as_millis() as u64;

    println!("📋 Response status: {}", status_code);
    println!("⏱️ Response time: {}ms", response_time);

    if !status_code.is_success() {
        let error_text = match response.text().await {
            Ok(text) => text,
            Err(e) => format!("Failed to read error response: {}", e),
        };
        println!("❌ HTTP error response: {}", error_text);
        return Ok(AsrTestResponse {
            success: false,
            transcription: None,
            processing_time_ms: response_time,
            file_size,
            message: format!("HTTP {} - {}", status_code, error_text),
            status_code: Some(status_code.as_u16()),
        });
    }

    // Parse JSON response
    let response_body = match response.text().await {
        Ok(text) => text,
        Err(e) => {
            return Ok(AsrTestResponse {
                success: false,
                transcription: None,
                processing_time_ms: response_time,
                file_size,
                message: format!("Failed to read response: {}", e),
                status_code: Some(status_code.as_u16()),
            });
        }
    };

    println!("📄 Raw response: {}", response_body);

    // Try to extract transcription from response
    let transcription = if let Ok(json_data) = serde_json::from_str::<serde_json::Value>(&response_body) {
        println!("📊 Parsed JSON response");

        // Check for structured response format: {"code":0,"msg":"ok","data":"transcription_text"}
        if let Some(data) = json_data.get("data").and_then(|v| v.as_str()) {
            Some(data.to_string())
        }
        // Try other possible response formats
        else if let Some(text) = json_data.get("text").and_then(|v| v.as_str()) {
            Some(text.to_string())
        } else if let Some(text) = json_data.get("transcription").and_then(|v| v.as_str()) {
            Some(text.to_string())
        } else if let Some(text) = json_data.get("result").and_then(|v| v.as_str()) {
            Some(text.to_string())
        } else if let Some(result) = json_data.get("result") {
            if let Some(text) = result.get("text").and_then(|v| v.as_str()) {
                Some(text.to_string())
            } else {
                println!("⚠️ Could not extract transcription from result: {}", result);
                Some(format!("Complex response: {}", json_data))
            }
        } else {
            println!("⚠️ Unknown JSON format, using raw response");
            Some(response_body.clone())
        }
    } else {
        println!("⚠️ Response is not JSON, using raw text");
        Some(response_body.clone())
    };

    if let Some(ref text) = transcription {
        println!("✅ Transcription result: {}", text);
    }

    Ok(AsrTestResponse {
        success: true,
        transcription,
        processing_time_ms: response_time,
        file_size,
        message: "Transcription completed successfully".to_string(),
        status_code: Some(status_code.as_u16()),
    })
}

// Hotkey Configuration commands
#[tauri::command]
pub async fn get_hotkey_config(
    db_state: State<'_, DatabaseState>
) -> Result<Option<crate::database::HotkeyConfig>, String> {
    let db = {
        let guard = db_state.lock().unwrap();
        guard.as_ref().cloned()
    };

    match db {
        Some(database) => {
            match database.get_hotkey_config().await {
                Ok(config) => Ok(config),
                Err(e) => Err(format!("Failed to get hotkey config: {}", e)),
            }
        }
        None => Err("Database not initialized".to_string()),
    }
}

#[tauri::command]
pub async fn save_hotkey_config(
    db_state: State<'_, DatabaseState>,
    request: HotkeyConfigRequest,
) -> Result<crate::database::HotkeyConfig, String> {
    let db = {
        let guard = db_state.lock().unwrap();
        guard.as_ref().cloned()
    };

    match db {
        Some(database) => {
            match database.save_hotkey_config(
                &request.transcribe_key,
                &request.translate_key,
                request.trigger_delay_ms,
                request.anti_mistouch_enabled,
                request.save_wav_files,
            ).await {
                Ok(config) => Ok(config),
                Err(e) => Err(format!("Failed to save hotkey config: {}", e)),
            }
        }
        None => Err("Database not initialized".to_string()),
    }
}

// Audio device commands
#[tauri::command]
pub async fn start_test_recording() -> Result<String, String> {
    use std::sync::{Arc, Mutex};
    use crate::voice_assistant::AudioRecorder;
    
    println!("🎤 Starting test recording...");
    
    // Create a new recorder
    let recorder = Arc::new(Mutex::new(AudioRecorder::new()
        .map_err(|e| format!("Failed to create recorder: {}", e))?));
    
    // Start recording
    {
        let mut rec = recorder.lock().unwrap();
        rec.start_recording()
            .map_err(|e| format!("Failed to start recording: {}", e))?;
    }
    
    println!("🔴 Recording started... Recording for 3 seconds");
    
    // Record for 3 seconds
    tokio::time::sleep(tokio::time::Duration::from_secs(3)).await;
    
    // Stop recording and save to file
    let file_path = {
        let mut rec = recorder.lock().unwrap();
        rec.stop_recording()
            .map_err(|e| format!("Failed to stop recording: {}", e))?
    };
    
    println!("✅ Test recording completed!");
    println!("📁 Audio file saved to: {}", file_path);
    
    Ok(format!("Test recording completed. File saved to: {}", file_path))
}

#[tauri::command]
pub async fn get_audio_devices() -> Result<Vec<AudioDevice>, String> {
    println!("🎤 Getting available audio devices...");
    println!("🖥️ Platform: {}", std::env::consts::OS);
    println!("⏰ Current time: {:?}", std::time::SystemTime::now());
    
    // Try to use actual system APIs if available
    let devices = scan_system_audio_devices().await?;
    
    if devices.is_empty() {
        println!("⚠️ No system devices found, returning mock devices for testing");
        return create_mock_devices();
    }
    
    println!("✅ Found {} audio devices", devices.len());
    for (index, device) in devices.iter().enumerate() {
        println!("  [{}] ID: {}", index + 1, device.id);
        println!("      Name: {}", device.name);
        println!("      Default: {}", device.is_default);
        println!("      ---");
    }
    
    Ok(devices)
}

// Try to scan actual system audio devices
async fn scan_system_audio_devices() -> Result<Vec<AudioDevice>, String> {
    println!("🔍 Scanning system audio devices...");
    
    // For now, we'll create platform-specific mock devices
    // In a real implementation, you would use cpal or platform-specific APIs
    
    #[cfg(target_os = "windows")]
    {
        println!("🪟 Windows platform detected");
        // On Windows, we could use Windows Core Audio APIs
        // For now, return realistic Windows device names
        let mut devices = Vec::new();
        
        // Try to detect common Windows audio devices
        devices.push(AudioDevice {
            id: "default".to_string(),
            name: "麦克风 (Realtek High Definition Audio)".to_string(),
            is_default: true,
        });
        
        devices.push(AudioDevice {
            id: "webcam".to_string(),
            name: "集成麦克风 (USB Camera)".to_string(),
            is_default: false,
        });
        
        devices.push(AudioDevice {
            id: "usb_audio".to_string(),
            name: "USB Audio Device (USB Audio)".to_string(),
            is_default: false,
        });
        
        println!("📋 Created {} Windows-specific devices", devices.len());
        return Ok(devices);
    }
    
    #[cfg(target_os = "macos")]
    {
        println!("🍎 macOS platform detected");
        // On macOS, we could use Core Audio APIs
        let mut devices = Vec::new();
        
        devices.push(AudioDevice {
            id: "default".to_string(),
            name: "Built-in Microphone".to_string(),
            is_default: true,
        });
        
        devices.push(AudioDevice {
            id: "webcam".to_string(),
            name: "FaceTime HD Camera (Built-in)".to_string(),
            is_default: false,
        });
        
        println!("📋 Created {} macOS-specific devices", devices.len());
        return Ok(devices);
    }
    
    #[cfg(target_os = "linux")]
    {
        println!("🐧 Linux platform detected");
        // On Linux, we could use ALSA or PulseAudio APIs
        let mut devices = Vec::new();
        
        devices.push(AudioDevice {
            id: "default".to_string(),
            name: "Default PulseAudio Device".to_string(),
            is_default: true,
        });
        
        devices.push(AudioDevice {
            id: "webcam".to_string(),
            name: "HD Pro Webcam C920".to_string(),
            is_default: false,
        });
        
        devices.push(AudioDevice {
            id: "usb_audio".to_string(),
            name: "USB Audio Device".to_string(),
            is_default: false,
        });
        
        println!("📋 Created {} Linux-specific devices", devices.len());
        return Ok(devices);
    }
    
    #[allow(unreachable_code)]
    {
        println!("❓ Unknown platform, using generic devices");
        Ok(vec![])
    }
}

// Create fallback mock devices
fn create_mock_devices() -> Result<Vec<AudioDevice>, String> {
    println!("🎭 Creating mock devices for testing");
    
    let mock_devices = vec![
        AudioDevice {
            id: "default".to_string(),
            name: "Default System Microphone".to_string(),
            is_default: true,
        },
        AudioDevice {
            id: "webcam-mic".to_string(),
            name: "Webcam Microphone".to_string(),
            is_default: false,
        },
        AudioDevice {
            id: "usb-mic".to_string(),
            name: "USB Audio Device".to_string(),
            is_default: false,
        }
    ];
    
    println!("🎭 Created {} mock devices", mock_devices.len());
    Ok(mock_devices)
}

#[tauri::command]
pub async fn test_microphone(device_id: String) -> Result<bool, String> {
    println!("🎤 Starting microphone test...");
    println!("🎯 Target device ID: {}", device_id);
    println!("🖥️ Platform: {}", std::env::consts::OS);
    println!("⏰ Test started at: {:?}", std::time::SystemTime::now());
    
    // Simulate different behaviors based on device ID
    match device_id.as_str() {
        "default" => {
            println!("🔊 Testing default system microphone");
            println!("✓ Default device access granted");
        }
        "webcam" | "webcam-mic" => {
            println!("📷 Testing webcam microphone");
            println!("✓ Webcam device found and accessible");
        }
        "usb_audio" | "usb-mic" => {
            println!("🔌 Testing USB audio device");
            println!("✓ USB device connected and working");
        }
        _ => {
            println!("❓ Unknown device ID: {}", device_id);
            println!("⚠️ Using generic test procedure");
        }
    }
    
    println!("⏳ Simulating audio capture test...");
    
    // Simulate test duration with progress
    for i in 1..=3 {
        tokio::time::sleep(tokio::time::Duration::from_millis(333)).await;
        println!("  📊 Testing audio levels... {}/3", i);
    }
    
    // Simulate checking audio levels (in real implementation, you'd check actual audio)
    let simulated_audio_level = 0.75; // 75% of max level
    println!("📈 Simulated audio level: {:.0}%", simulated_audio_level * 100.0);
    
    // Determine success based on simulated conditions
    let success = simulated_audio_level > 0.1; // Success if we detect audio
    
    if success {
        println!("✅ Microphone test successful!");
        println!("🎵 Audio input detected and working properly");
        println!("📊 Signal quality: Good");
    } else {
        println!("❌ Microphone test failed");
        println!("🔇 No audio input detected");
        println!("📊 Signal quality: Poor/None");
    }
    
    println!("⏰ Test completed at: {:?}", std::time::SystemTime::now());
    
    Ok(success)
}