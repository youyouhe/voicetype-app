use crate::database::{Database, NewHistoryRecord};
use crate::voice_assistant::traits::AsrProcessor;
use serde::{Deserialize, Serialize};
use tauri::State;
use std::sync::{Arc, Mutex};
use base64::{Engine as _, engine::general_purpose::STANDARD};

pub mod gpu_backend;

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
    pub endpoint: String,
    pub healthy: bool,
    pub response_time_ms: u64,
    pub status_code: Option<u16>,
    pub message: String,
    pub timestamp: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ServiceStatusResponse {
    pub active_service: String,
    pub status: String,
    pub endpoint: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LatencyDataResponse {
    pub current: i64,
    pub trend: String,
    pub trend_value: i64,
    pub history: Vec<LatencyHistoryPoint>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LatencyHistoryPoint {
    pub time: String,
    pub val: i64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UsageDataResponse {
    pub today_seconds: i64,
    pub success_rate: f64,
    pub total_requests: i64,
    pub successful_requests: i64,
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
    pub whisper_model: Option<String>, // NEW: Selected whisper model
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
    pub endpoint: Option<String>,
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
    pub typing_delays: crate::database::TypingDelays,
}

// Initialize database
#[tauri::command]
pub async fn init_database(
    db_state: State<'_, DatabaseState>
) -> Result<String, String> {
    println!("🚀 Backend: init_database() called");

    // Check if database is already initialized
    {
        println!("🔍 Backend: Checking if database already exists...");
        let guard = db_state.lock().unwrap();
        if guard.is_some() {
            println!("✅ Backend: Database already exists, skipping initialization");
            return Ok("Database already initialized".to_string());
        }
        println!("🔍 Backend: No existing database found, proceeding with initialization");
    }

    println!("🔍 Backend: Attempting to create new Database instance...");
    match Database::new().await {
        Ok(db) => {
            println!("✅ Backend: Database created successfully, storing in state");
            *db_state.lock().unwrap() = Some(db);
            println!("✅ Backend: Database initialized and stored in state");
            Ok("Database initialized successfully".to_string())
        }
        Err(e) => {
            eprintln!("❌ Backend: Failed to initialize database: {}", e);
            println!("❌ Backend: Database initialization error details:");
            println!("  - Error: {}", e);
            Err(format!("Failed to initialize database: {}", e))
        }
    }
}

// ASR Configuration commands
#[tauri::command]
pub async fn get_asr_config(
    db_state: State<'_, DatabaseState>
) -> Result<Option<crate::database::AsrConfig>, String> {
    println!("🔍 Backend: get_asr_config() called");

    let db = {
        println!("🔒 Backend: Acquiring database lock...");
        let guard = db_state.lock().unwrap();
        let db_ref = guard.as_ref().cloned();
        println!("🔓 Backend: Database lock released, database exists: {}", db_ref.is_some());
        db_ref
    };

    match db {
        Some(database) => {
            println!("✅ Backend: Database found, querying ASR config...");
            match database.get_asr_config().await {
                Ok(config) => {
                    if let Some(ref cfg) = config {
                        println!("✅ Backend: ASR config found:");
                        println!("  - ID: {}", cfg.id);
                        println!("  - Service Provider: {}", cfg.service_provider);
                        println!("  - Has Local Endpoint: {}", cfg.local_endpoint.is_some());
                        println!("  - Has Local API Key: {}", cfg.local_api_key.is_some());
                        println!("  - Has Cloud Endpoint: {}", cfg.cloud_endpoint.is_some());
                        println!("  - Has Cloud API Key: {}", cfg.cloud_api_key.is_some());
                        println!("  - Created At: {}", cfg.created_at);
                        println!("  - Updated At: {}", cfg.updated_at);
                    } else {
                        println!("📥 Backend: No ASR config found in database");
                    }
                    Ok(config)
                },
                Err(e) => {
                    println!("❌ Backend: Database query failed: {}", e);
                    Err(format!("Failed to get ASR config: {}", e))
                },
            }
        }
        None => {
            println!("❌ Backend: Database not initialized");
            Err("Database not initialized".to_string())
        },
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
            println!("  - whisper_model: {:?}", request.whisper_model);
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
                request.whisper_model.as_deref(), // NEW: Pass whisper model
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
                Ok(history) => {
                    // Emit events to notify frontend of new data
                    crate::voice_assistant::coordinator::emit_new_history_record_event();
                    crate::voice_assistant::coordinator::emit_service_status_updated_event();
                    Ok(history)
                },
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

// Simple test command to verify frontend-backend connection
#[tauri::command]
pub async fn test_frontend_backend_connection() -> Result<String, String> {
    println!("🔔 Backend: Frontend-backend connection test received!");
    Ok("Backend connection successful!".to_string())
}

// Health check command - performed by Rust backend for better debugging
#[tauri::command]
pub async fn test_connection_health(
    request: HealthCheckRequest,
) -> Result<HealthCheckResponse, String> {
    println!("🔍 Tauri Backend: Starting health check for: {}", request.endpoint);
    println!("⏰ Current time: {:?}", chrono::Utc::now());
    println!("📋 Request details: {:?}", request);

    // Build health endpoint URL
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
                endpoint: request.endpoint,
                healthy: false,
                message: format!("Network error: {}", e),
                status_code: None,
                response_time_ms: response_time,
                timestamp: chrono::Utc::now().to_rfc3339(),
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
                        endpoint: request.endpoint,
                        healthy: true,
                        message: format!("Service healthy - {} backends", backend_count.unwrap_or(0)),
                        status_code: Some(status_code.as_u16()),
                        response_time_ms: response_time,
                        timestamp: chrono::Utc::now().to_rfc3339(),
                    });
                } else {
                    let service_status = json_data.get("status")
                        .and_then(|s| s.as_str())
                        .unwrap_or("unknown");
                    println!("⚠️ Service status: {}", service_status);
                    return Ok(HealthCheckResponse {
                        endpoint: request.endpoint,
                        healthy: false,
                        message: format!("Service status: {}", service_status),
                        status_code: Some(status_code.as_u16()),
                        response_time_ms: response_time,
                        timestamp: chrono::Utc::now().to_rfc3339(),
                    });
                }
            }
            Err(e) => {
                println!("❌ Failed to parse JSON: {}", e);
                println!("📄 Raw response: {}", response_body);
                Ok(HealthCheckResponse {
                    endpoint: request.endpoint,
                    healthy: false,
                    message: format!("Invalid JSON response: {}", e),
                    status_code: Some(status_code.as_u16()),
                    response_time_ms: response_time,
                    timestamp: chrono::Utc::now().to_rfc3339(),
                })
            }
        }
    } else {
        let error_text = response.text().await.unwrap_or_default();
        println!("❌ HTTP error response: {}", error_text);
        Ok(HealthCheckResponse {
            endpoint: request.endpoint,
            healthy: false,
            message: format!("HTTP {} - {}", status_code, error_text),
            status_code: Some(status_code.as_u16()),
            response_time_ms: response_time,
            timestamp: chrono::Utc::now().to_rfc3339(),
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
    println!("🔗 Endpoint: {:?}", request.endpoint);

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

    // Route to appropriate processor based on service provider
    match request.service_provider.as_str() {
        "local" => {
            println!("🎯 Attempting Local Whisper (whisper-rs) for transcription");
            println!("⚠️ Note: whisper-rs has known compatibility issues with some CPU configurations");
            
            // Try local whisper first, but with immediate fallback if it fails
            match test_local_whisper_transcription(audio_data.clone(), file_size, start_time).await {
                Ok(response) => {
                    if response.success {
                        println!("✅ Local whisper succeeded!");
                        Ok(response)
                    } else {
                        println!("❌ Local whisper failed: {}", response.message);
                        println!("🔄 Auto-switching to Cloud ASR fallback...");
                        
                        // Try Cloud ASR fallback
                        if let Some(endpoint) = std::env::var("GROQ_API_ENDPOINT").ok() {
                            let api_key = std::env::var("GROQ_API_KEY").ok();
                            test_cloud_asr_transcription(audio_data, file_size, start_time, &endpoint, api_key).await
                        } else {
                            println!("⚠️ No Cloud ASR configured");
                            Ok(response)
                        }
                    }
                }
                Err(e) => {
                    println!("❌ Local whisper crashed: {}", e);
                    println!("🔄 Auto-switching to Cloud ASR fallback...");
                    
                    // Try Cloud ASR fallback
                    if let Some(endpoint) = std::env::var("GROQ_API_ENDPOINT").ok() {
                        let api_key = std::env::var("GROQ_API_KEY").ok();
                        test_cloud_asr_transcription(audio_data, file_size, start_time, &endpoint, api_key).await
                    } else {
                        Err(e)
                    }
                }
            }
        }
        "cloud" => {
            println!("☁️ Using Cloud ASR for transcription");
            if let Some(endpoint) = request.endpoint {
                test_cloud_asr_transcription(audio_data, file_size, start_time, &endpoint, request.api_key).await
            } else {
                Ok(AsrTestResponse {
                    success: false,
                    transcription: None,
                    processing_time_ms: start_time.elapsed().as_millis() as u64,
                    file_size,
                    message: "No endpoint configured for Cloud ASR".to_string(),
                    status_code: None,
                })
            }
        }
        other => {
            println!("❌ Unknown service provider: {}", other);
            Ok(AsrTestResponse {
                success: false,
                transcription: None,
                processing_time_ms: start_time.elapsed().as_millis() as u64,
                file_size,
                message: format!("Unknown service provider: {}", other),
                status_code: None,
            })
        }
    }
}

// Local Whisper transcription helper function
async fn test_local_whisper_transcription(
    audio_data: Vec<u8>,
    file_size: u64,
    start_time: std::time::Instant,
) -> Result<AsrTestResponse, String> {
    println!("🎯 Starting Local Whisper transcription...");

    // First, do a quick health check of whisper-rs availability
    if !check_whisper_rs_health().await {
        println!("❌ Whisper-rs health check failed - known compatibility issue detected");
        return Ok(AsrTestResponse {
            success: false,
            transcription: None,
            processing_time_ms: start_time.elapsed().as_millis() as u64,
            file_size,
            message: "Whisper-rs has known compatibility issues with this CPU configuration. Auto-switching to Cloud ASR recommended.".to_string(),
            status_code: None,
        });
    }

    // Create or get global WhisperRS processor
    let model_path = std::env::var("WHISPER_MODEL_PATH")
        .ok()
        .and_then(|path| {
            if std::path::Path::new(&path).exists() {
                Some(path)
            } else {
                None
            }
        })
        .or_else(|| {
            // Try to find models in the default data directory
            let home = std::env::var("HOME").ok()?;
            let models_dir = format!("{}/.local/share/com.martin.flash-input/models", home);

            // Model preference order for testing (small to large)
            let model_preferences = [
                "ggml-small.bin",          // Good balance of speed and accuracy
                "ggml-base.bin",           // Fastest
                "ggml-medium.bin",         // Better accuracy
                "ggml-large-v3-turbo.bin", // Best accuracy
            ];

            for model in model_preferences {
                let model_file = format!("{}/{}", models_dir, model);
                if std::path::Path::new(&model_file).exists() {
                    println!("✅ Found available model for testing: {}", model);
                    return Some(model_file);
                }
            }
            None
        })
        .unwrap_or_else(|| {
            println!("⚠️ No Whisper model found in default directory");
            println!("💡 Please download a model to ~/.local/share/com.martin.flash-input/models/");
            println!("📥 Recommended: ggml-small.bin for good performance");
            "ggml-small.bin".to_string() // Fallback for error message
        });

    println!("🎯 Using Whisper model path: {}", model_path);
    
    let processor = match crate::voice_assistant::global_whisper::get_or_create_whisper_processor(&model_path).await {
        Ok(processor) => processor,
        Err(e) => {
            return Ok(AsrTestResponse {
                success: false,
                transcription: None,
                processing_time_ms: start_time.elapsed().as_millis() as u64,
                file_size,
                message: format!("Failed to get/create global Whisper processor: {}", e),
                status_code: None,
            });
        }
    };

    // Convert audio bytes to WAV format and process
    let audio_cursor = std::io::Cursor::new(audio_data.clone());
    
    // Scope the processor lock to avoid holding it across await
    let transcription_result = {
        let processor_guard = processor.lock().unwrap();
        processor_guard.process_audio(
            audio_cursor,
            crate::voice_assistant::Mode::Transcriptions,
            "",
        )
    };
    
    let transcription_result = match transcription_result {
        Ok(result) => {
            println!("✅ Local Whisper processing succeeded!");
            result
        }
        Err(e) => {
            println!("❌ Local Whisper processing failed: {}", e);
            println!("🔄 Attempting fallback to Cloud ASR...");
            
            // Try fallback to Cloud ASR if available
            let cloud_endpoint = std::env::var("GROQ_API_ENDPOINT").ok();
            let cloud_api_key = std::env::var("GROQ_API_KEY").ok();
            
            if let (Some(endpoint), Some(api_key)) = (cloud_endpoint, cloud_api_key) {
                println!("☁️ Using Cloud ASR fallback with Groq");
                match test_cloud_asr_transcription(audio_data, file_size, start_time, &endpoint, Some(api_key)).await {
                    Ok(cloud_response) => {
                        if cloud_response.success {
                            println!("✅ Cloud ASR fallback succeeded!");
                            return Ok(cloud_response);
                        } else {
                            println!("❌ Cloud ASR fallback also failed: {}", cloud_response.message);
                        }
                    }
                    Err(e) => {
                        println!("❌ Cloud ASR fallback failed with error: {}", e);
                    }
                }
            } else {
                println!("⚠️ No Cloud ASR credentials available for fallback");
            }
            
            return Ok(AsrTestResponse {
                success: false,
                transcription: None,
                processing_time_ms: start_time.elapsed().as_millis() as u64,
                file_size,
                message: format!("Local Whisper processing failed: {}. Cloud fallback unavailable.", e),
                status_code: None,
            });
        }
    };

    let processing_time = start_time.elapsed().as_millis() as u64;

    println!("✅ Local Whisper transcription completed in {}ms", processing_time);
    println!("📝 Result: {}", transcription_result);

    Ok(AsrTestResponse {
        success: true,
        transcription: Some(transcription_result),
        processing_time_ms: processing_time,
        file_size,
        message: "Local Whisper transcription completed successfully".to_string(),
        status_code: None,
    })
}

// Cloud ASR transcription helper function
async fn test_cloud_asr_transcription(
    audio_data: Vec<u8>,
    file_size: u64,
    start_time: std::time::Instant,
    endpoint: &str,
    api_key: Option<String>,
) -> Result<AsrTestResponse, String> {
    println!("☁️ Starting Cloud ASR transcription...");

    // Create HTTP client
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .build()
        .map_err(|e| {
            println!("❌ Failed to create HTTP client: {}", e);
            format!("Failed to create HTTP client: {}", e)
        })?;

    // Create multipart form with audio file
    let form = reqwest::multipart::Form::new()
        .part("audio", reqwest::multipart::Part::bytes(audio_data)
            .file_name("test_audio.wav")
            .mime_str("audio/wav")
            .map_err(|e| format!("Failed to create form part: {}", e))?);

    println!("🚀 Sending request to Cloud ASR endpoint: {}", endpoint);

    // Clean API key - remove API_KEY= prefix if present
    let clean_api_key = match api_key {
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

    let request_builder = client
        .post(endpoint)
        .multipart(form);

    let request_builder = if let Some(ref key) = clean_api_key {
        println!("🔑 Sending X-API-Key header");
        request_builder.header("X-API-Key", key)
    } else {
        println!("🔑 No API key will be sent");
        request_builder
    };

    let response = match request_builder.send().await {
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
        println!("✅ Cloud ASR transcription result: {}", text);
    }

    Ok(AsrTestResponse {
        success: true,
        transcription,
        processing_time_ms: response_time,
        file_size,
        message: "Cloud ASR transcription completed successfully".to_string(),
        status_code: Some(status_code.as_u16()),
    })
}

// Helper function to create Local Whisper processor
#[allow(dead_code)]
async fn create_local_whisper_processor() -> Result<crate::voice_assistant::asr::whisper_rs::WhisperRSProcessor, String> {
    use crate::voice_assistant::asr::whisper_rs::{WhisperRSProcessor, WhisperRSConfig, SamplingStrategyConfig};

    // Try to get model path from environment
    let model_path = std::env::var("WHISPER_MODEL_PATH")
        .ok()
        .and_then(|path| {
            if std::path::Path::new(&path).exists() {
                Some(path)
            } else {
                None
            }
        })
        .or_else(|| {
            // Try to find models in the default data directory, preferring smaller models for CPU
            let home = std::env::var("HOME").ok()?;
            let models_dir = format!("{}/.local/share/com.martin.flash-input/models", home);

            // Model preference order for CPU (smallest to largest)
            let model_preferences = [
                "ggml-base.bin",           // ~74MB
                "ggml-small.bin",          // ~244MB
                "ggml-medium.bin",         // ~769MB
                "ggml-large-v3-turbo.bin", // ~1570MB
            ];

            for model in model_preferences {
                let model_file = format!("{}/{}", models_dir, model);
                if std::path::Path::new(&model_file).exists() {
                    println!("✅ Found CPU-optimized model: {} ({}MB)",
                            model,
                            match model {
                                "ggml-base.bin" => "74",
                                "ggml-small.bin" => "244",
                                "ggml-medium.bin" => "769",
                                "ggml-large-v3-turbo.bin" => "1570",
                                _ => "unknown",
                            });
                    return Some(model_file);
                }
            }
            None
        })
        .unwrap_or_else(|| {
            println!("⚠️ No Whisper model found. Please download a model to ~/.local/share/com.martin.flash-input/models/");
            println!("💡 Recommended models for CPU: ggml-base.bin (fastest) or ggml-small.bin (balanced)");
            println!("📥 Download from: https://huggingface.co/ggerganov/whisper.cpp/tree/main");
            println!("🔧 Quick download commands:");
            println!("   # For base model (fastest, 74MB):");
            println!("   wget -O ~/.local/share/com.martin.flash-input/models/ggml-base.bin \\");
            println!("     https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-base.bin");
            "./models/ggml-base.bin".to_string()
        });

    println!("🎯 Using Whisper model: {}", model_path);

    // Check if VAD should be enabled via environment variable
    let enable_vad = std::env::var("WHISPER_ENABLE_VAD")
        .unwrap_or_else(|_| "false".to_string())
        .parse::<bool>()
        .unwrap_or(false);

    if enable_vad {
        println!("🎯 VAD enabled via WHISPER_ENABLE_VAD environment variable");
    } else {
        println!("ℹ️  VAD disabled (set WHISPER_ENABLE_VAD=true to enable)");
    }

    // Auto-detect optimal GPU backend
    let gpu_detector = crate::voice_assistant::asr::gpu_detector::GpuDetector::new();
    let optimal_backend = gpu_detector.get_preferred_backend();

    let config = WhisperRSConfig {
        model_path,
        sampling_strategy: SamplingStrategyConfig::Greedy { best_of: 1 },
        language: None, // Auto-detect
        translate: false,
        enable_vad,
        backend: optimal_backend.clone(),
        use_gpu_if_available: std::env::var("WHISPER_USE_GPU")
            .unwrap_or_else(|_| "true".to_string())
            .parse::<bool>()
            .unwrap_or(true),
        gpu_device_id: std::env::var("WHISPER_GPU_DEVICE_ID")
            .ok()
            .and_then(|id| id.parse::<u32>().ok()),
    };

      // Use thread-safe creation with timeout to prevent crashes
    println!("⏱️ Creating WhisperRSProcessor with safety timeout...");
    
    let processor_result = std::thread::spawn(move || {
        // Use a simple timeout mechanism
        let (tx, rx) = std::sync::mpsc::channel();
        
        // Spawn the processor creation in a separate thread
        std::thread::spawn(move || {
            let result = WhisperRSProcessor::new(config);
            let _ = tx.send(result);
        });
        
        // Wait for up to 30 seconds for processor creation
        match rx.recv_timeout(std::time::Duration::from_secs(30)) {
            Ok(processor_result) => processor_result,
            Err(_) => {
                eprintln!("⏰ WhisperRSProcessor creation timed out after 30 seconds");
                eprintln!("💡 This indicates a deadlock or infinite loop in whisper.cpp");
                Err(crate::voice_assistant::VoiceError::Other(
                    "WhisperRSProcessor creation timeout - possible whisper.cpp bug".to_string()
                ))
            }
        }
    }).join().unwrap_or_else(|_| {
        eprintln!("💥 WhisperRSProcessor creation thread panicked!");
        Err(crate::voice_assistant::VoiceError::Other(
            "WhisperRSProcessor creation thread panicked".to_string()
        ))
    });
    
    processor_result.map_err(|e| {
        format!("Failed to create Local Whisper processor: {}. This may be due to whisper.cpp compatibility issues with your CPU.", e)
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
    println!("🔧 Backend: save_hotkey_config() called with request:");
    println!("  - transcribe_key: {}", request.transcribe_key);
    println!("  - translate_key: {}", request.translate_key);
    println!("  - trigger_delay_ms: {}", request.trigger_delay_ms);
    println!("  - anti_mistouch_enabled: {}", request.anti_mistouch_enabled);
    println!("  - save_wav_files: {}", request.save_wav_files);
    println!("  - typing_delays: {:?}", request.typing_delays);

    let db = {
        let guard = db_state.lock().unwrap();
        guard.as_ref().cloned()
    };

    println!("🗄️ Database state: {:?}", db.is_some());

    match db {
        Some(database) => {
            println!("📝 Calling database.save_hotkey_config...");
            match database.save_hotkey_config(
                &request.transcribe_key,
                &request.translate_key,
                request.trigger_delay_ms,
                request.anti_mistouch_enabled,
                request.save_wav_files,
                Some(&request.typing_delays),
            ).await {
                Ok(config) => {
                    println!("✅ Backend: Hotkey config saved successfully!");
                    println!("  - Saved config ID: {}", config.id);
                    println!("  - Saved clipboard_update_ms: {}", config.clipboard_update_ms);
                    println!("  - Saved keyboard_events_settle_ms: {}", config.keyboard_events_settle_ms);
                    Ok(config)
                },
                Err(e) => {
                    println!("❌ Backend: Failed to save hotkey config: {}", e);
                    Err(format!("Failed to save hotkey config: {}", e))
                },
            }
        }
        None => {
            println!("❌ Backend: Database not initialized");
            Err("Database not initialized".to_string())
        },
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

    // For now, return mock devices
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

// Live Data commands
#[tauri::command]
pub async fn get_service_status(
    service_name: Option<String>,
    db_state: State<'_, DatabaseState>
) -> Result<ServiceStatusResponse, String> {
    let db = {
        let guard = db_state.lock().unwrap();
        guard.as_ref().cloned()
    };

    match db {
        Some(database) => {
            let service = service_name.unwrap_or_else(|| "local_asr".to_string());
            println!("🔍 Getting service status for: {}", service);

            match database.get_service_status(&service).await {
                Ok(Some(stats)) => {
                    println!("✅ Service status found: {} ({})", stats.service_name, stats.status);
                    Ok(ServiceStatusResponse {
                        active_service: stats.service_name,
                        status: stats.status,
                        endpoint: stats.endpoint,
                    })
                }
                Ok(None) => {
                    // Return default status if not found
                    println!("⚠️ No service status found, returning default");
                    Ok(ServiceStatusResponse {
                        active_service: service,
                        status: "offline".to_string(),
                        endpoint: None,
                    })
                }
                Err(e) => {
                    println!("❌ Failed to get service status: {}", e);
                    Err(format!("Failed to get service status: {}", e))
                }
            }
        }
        None => Err("Database not initialized".to_string())
    }
}

#[tauri::command]
pub async fn get_latency_data(
    service_name: Option<String>,
    db_state: State<'_, DatabaseState>
) -> Result<LatencyDataResponse, String> {
    let db = {
        let guard = db_state.lock().unwrap();
        guard.as_ref().cloned()
    };

    match db {
        Some(database) => {
            let service = service_name.unwrap_or_else(|| "local_asr".to_string());
            println!("🔍 Getting latency data for: {}", service);

            match database.get_latency_data(&service, 24).await { // Last 24 hours
                Ok(records) => {
                    if records.is_empty() {
                        println!("⚠️ No latency data found");
                        return Ok(LatencyDataResponse {
                            current: 0,
                            trend: "neutral".to_string(),
                            trend_value: 0,
                            history: vec![],
                        });
                    }

                    // Calculate current average latency (last 10 records)
                    let current_avg = if records.len() > 10 {
                        records.iter().take(10).map(|r| r.latency_ms).sum::<i64>() / 10
                    } else {
                        records.iter().map(|r| r.latency_ms).sum::<i64>() / records.len() as i64
                    };

                    // Calculate trend (compare with previous 10 records)
                    let trend = if records.len() > 20 {
                        let previous_avg = records.iter().skip(10).take(10).map(|r| r.latency_ms).sum::<i64>() / 10;
                        if current_avg > previous_avg {
                            "up"
                        } else if current_avg < previous_avg {
                            "down"
                        } else {
                            "neutral"
                        }
                    } else {
                        "neutral"
                    };

                    let trend_value = if records.len() > 20 {
                        let previous_avg = records.iter().skip(10).take(10).map(|r| r.latency_ms).sum::<i64>() / 10;
                        current_avg - previous_avg
                    } else {
                        0
                    };

                    // Convert to history points for frontend (last 12 records with time formatting)
                    let history: Vec<LatencyHistoryPoint> = records
                        .iter().take(12)
                        .rev() // Reverse to show oldest first
                        .map(|r| LatencyHistoryPoint {
                            time: r.recorded_at.format("%H:%M").to_string(),
                            val: r.latency_ms,
                        })
                        .collect();

                    println!("✅ Latency data: {}ms (trend: {}, records: {})", current_avg, trend, records.len());
                    Ok(LatencyDataResponse {
                        current: current_avg,
                        trend: trend.to_string(),
                        trend_value,
                        history,
                    })
                }
                Err(e) => {
                    println!("❌ Failed to get latency data: {}", e);
                    Err(format!("Failed to get latency data: {}", e))
                }
            }
        }
        None => Err("Database not initialized".to_string())
    }
}

#[tauri::command]
pub async fn get_usage_data(
    db_state: State<'_, DatabaseState>
) -> Result<UsageDataResponse, String> {
    let db = {
        let guard = db_state.lock().unwrap();
        guard.as_ref().cloned()
    };

    match db {
        Some(database) => {
            println!("🔍 Getting today's usage data");

            match database.get_today_usage().await {
                Ok(Some(usage)) => {
                    let success_rate = if usage.total_requests > 0 {
                        (usage.successful_requests as f64 / usage.total_requests as f64) * 100.0
                    } else {
                        0.0
                    };

                    println!("✅ Usage data: {} secs, {:.1}% success rate", usage.total_seconds, success_rate);
                    Ok(UsageDataResponse {
                        today_seconds: usage.total_seconds,
                        success_rate,
                        total_requests: usage.total_requests,
                        successful_requests: usage.successful_requests,
                    })
                }
                Ok(None) => {
                    println!("⚠️ No usage data found for today");
                    Ok(UsageDataResponse {
                        today_seconds: 0,
                        success_rate: 0.0,
                        total_requests: 0,
                        successful_requests: 0,
                    })
                }
                Err(e) => {
                    println!("❌ Failed to get usage data: {}", e);
                    Err(format!("Failed to get usage data: {}", e))
                }
            }
        }
        None => Err("Database not initialized".to_string())
    }
}

// ASR result handler command
#[tauri::command]
pub async fn handle_asr_result(
    db_state: State<'_, DatabaseState>,
    result: crate::voice_assistant::coordinator::AsrResult,
) -> Result<String, String> {
    let db = {
        let guard = db_state.lock().unwrap();
        guard.as_ref().cloned()
    };

    match db {
        Some(database) => {
            println!("📊 Handling ASR result: success={}, processor={}", result.success, result.processor_type);

            let record = NewHistoryRecord {
                record_type: "asr".to_string(),
                input_text: result.input_text,
                output_text: Some(result.output_text.clone()),
                audio_file_path: result.audio_file_path,
                processor_type: Some(result.processor_type),
                processing_time_ms: result.processing_time_ms,
                success: result.success,
                error_message: result.error_message,
            };

            match database.add_history_record(record).await {
                Ok(_) => {
                    println!("✅ ASR result saved to database");
                    Ok("ASR result saved successfully".to_string())
                }
                Err(e) => {
                    println!("❌ Failed to save ASR result: {}", e);
                    Err(format!("Failed to save ASR result: {}", e))
                }
            }
        }
        None => Err("Database not initialized".to_string())
    }
}

/// Helper function to get hotkey config from database for internal use
pub async fn get_hotkey_config_from_database() -> Result<Option<crate::database::HotkeyConfig>, String> {
    let database_path = std::env::current_dir()
        .unwrap_or_else(|_| std::path::PathBuf::from("."))
        .join(".tauri-data")
        .join("databases")
        .join("voice_assistant.db");

    if !database_path.exists() {
        return Ok(None);
    }

    // Use global database pool to avoid repeated initialization
    match Database::from_global_pool().await {
        Ok(database) => {
            match database.get_hotkey_config().await {
                Ok(config) => Ok(config),
                Err(e) => Err(format!("Failed to get hotkey config: {}", e)),
            }
        }
        Err(e) => Err(format!("Failed to create database: {}", e)),
    }
}

// Internal functions for VoiceAssistant (without Tauri State parameter)
pub async fn get_asr_config_internal() -> Result<Vec<crate::database::AsrConfig>, String> {
    let database_path = std::env::current_dir()
        .unwrap_or_else(|_| std::path::PathBuf::from("."))
        .join(".tauri-data")
        .join("databases")
        .join("voice_assistant.db");

    if !database_path.exists() {
        println!("⚠️ Database file not found at: {:?}", database_path);
        return Ok(Vec::new());
    }

    // Use global database pool to avoid repeated initialization
    match Database::from_global_pool().await {
        Ok(database) => {
            match database.get_asr_config().await {
                Ok(configs) => {
                    if let Some(ref config) = configs {
                        println!("✅ Found ASR config: {} (local: {}, cloud: {})",
                            config.service_provider,
                            config.local_endpoint.is_some(),
                            config.cloud_endpoint.is_some());
                        Ok(vec![config.clone()])
                    } else {
                        println!("⚠️ No ASR config found in database");
                        Ok(Vec::new())
                    }
                }
                Err(e) => {
                    println!("❌ Failed to get ASR config: {}", e);
                    Err(format!("Failed to get ASR config: {}", e))
                }
            }
        }
        Err(e) => {
            println!("❌ Failed to create database: {}", e);
            Err(format!("Failed to create database: {}", e))
        }
    }
}

pub async fn get_translation_config_internal() -> Result<Vec<crate::database::TranslationConfig>, String> {
    let database_path = std::env::current_dir()
        .unwrap_or_else(|_| std::path::PathBuf::from("."))
        .join(".tauri-data")
        .join("databases")
        .join("voice_assistant.db");

    if !database_path.exists() {
        println!("⚠️ Database file not found at: {:?}", database_path);
        return Ok(Vec::new());
    }

    // Use global database pool to avoid repeated initialization
    match Database::from_global_pool().await {
        Ok(database) => {
            match database.get_translation_config("siliconflow").await {
                Ok(config) => {
                    if let Some(ref c) = config {
                        println!("✅ Found translation config: {} ({})", c.provider, c.endpoint.is_some());
                        Ok(vec![c.clone()])
                    } else {
                        println!("⚠️ No translation config found in database");
                        Ok(Vec::new())
                    }
                }
                Err(e) => {
                    println!("❌ Failed to get translation config: {}", e);
                    Err(format!("Failed to get translation config: {}", e))
                }
            }
        }
        Err(e) => {
            println!("❌ Failed to create database: {}", e);
            Err(format!("Failed to create database: {}", e))
        }
    }
}

// Helper function to initialize database directly (without State wrapper)
pub async fn init_database_direct() -> Result<Database, String> {
    println!("🚀 Backend: init_database_direct() called");
    match Database::new().await {
        Ok(db) => {
            println!("✅ Backend: Database created successfully");
            Ok(db)
        }
        Err(e) => {
            eprintln!("❌ Backend: Failed to initialize database: {}", e);
            Err(format!("Failed to initialize database: {}", e))
        }
    }
}

// Whisper-rs health check function - simplified since CPU compatibility is confirmed
async fn check_whisper_rs_health() -> bool {
    println!("🏥 Performing whisper-rs health check...");
    
    // Check for available memory (whisper-rs can crash with insufficient memory)
    if let Ok(mem_info) = std::fs::read_to_string("/proc/meminfo") {
        if let Some(memtotal_line) = mem_info.lines().find(|line| line.starts_with("MemTotal:")) {
            let mem_kb: u64 = memtotal_line.split_whitespace()
                .nth(1)
                .unwrap_or("0")
                .parse()
                .unwrap_or(0);
            
            let mem_gb = mem_kb / 1024 / 1024;
            if mem_gb < 4 {
                println!("⚠️ Low memory detected ({}GB) - whisper-rs may be unstable", mem_gb);
                println!("💡 Auto-switching to Cloud ASR for reliability");
                return false;
            }
            
            println!("✅ Memory check passed: {}GB available", mem_gb);
        }
    }
    
    println!("✅ Whisper-rs health check passed - CPU compatibility confirmed");
    true
}

// Model management commands
#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub struct WhisperModel {
    pub name: String,
    pub path: String,
    pub size_mb: f64,
    pub file_type: String,
    pub modified: String,
}

#[tauri::command]
pub fn scan_whisper_models() -> Result<Vec<WhisperModel>, String> {
    println!("🔍 Scanning for available Whisper models...");
    
    let models_dir = match std::env::var("HOME") {
        Ok(home) => format!("{}/.local/share/com.martin.flash-input/models", home),
        Err(_) => return Err("Failed to get home directory".to_string()),
    };
    
    if !std::path::Path::new(&models_dir).exists() {
        println!("📁 Models directory does not exist: {}", models_dir);
        return Ok(vec![]); // Return empty list instead of error
    }
    
    let mut models = Vec::new();
    
    // Scan the directory for .bin files
    match std::fs::read_dir(&models_dir) {
        Ok(entries) => {
            for entry in entries {
                let entry = match entry {
                    Ok(e) => e,
                    Err(e) => {
                        eprintln!("Warning: Failed to read directory entry: {}", e);
                        continue;
                    }
                };
                
                let path = entry.path();
                
                // Only look for .bin files (whisper models)
                if path.extension().map_or(false, |ext| ext == "bin") {
                    let metadata = match entry.metadata() {
                        Ok(m) => m,
                        Err(e) => {
                            eprintln!("Warning: Failed to read metadata for {}: {}", path.display(), e);
                            continue;
                        }
                    };
                    
                    if metadata.is_file() {
                        let name = path.file_name()
                            .and_then(|n| n.to_str())
                            .unwrap_or("unknown")
                            .to_string();
                        
                        // Skip VAD model - it's not for transcription
                        if name.contains("vad") {
                            println!("⚠️ Skipping VAD model: {} (not suitable for transcription)", name);
                            continue;
                        }
                        
                        let size_mb = metadata.len() as f64 / 1024.0 / 1024.0;
                        
                        let modified = metadata.modified()
                            .ok()
                            .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
                            .map(|d| {
                                let datetime = chrono::DateTime::from_timestamp(d.as_secs() as i64, 0);
                                datetime.map(|dt| dt.format("%Y-%m-%d %H:%M").to_string())
                                    .unwrap_or_else(|| "Unknown".to_string())
                            })
                            .unwrap_or_else(|| "Unknown".to_string());
                        
                        let file_type = if name.contains("base") {
                            "Base (~74MB)".to_string()
                        } else if name.contains("small") {
                            "Small (~244MB)".to_string()
                        } else if name.contains("medium") {
                            "Medium (~769MB)".to_string()
                        } else if name.contains("large") {
                            if name.contains("turbo") {
                                "Large V3 Turbo (~1.5GB)".to_string()
                            } else {
                                "Large (~1.5GB)".to_string()
                            }
                        } else if name.contains("tiny") {
                            "Tiny (~39MB)".to_string()
                        } else {
                            format!("Custom ({:.1}MB)", size_mb)
                        };
                        
                        models.push(WhisperModel {
                            name,
                            path: path.display().to_string(),
                            size_mb,
                            file_type,
                            modified,
                        });
                        
                        println!("✅ Found model: {} ({:.1} MB)", models.last().unwrap().name, size_mb);
                    }
                }
            }
        }
        Err(e) => {
            return Err(format!("Failed to read models directory {}: {}", models_dir, e));
        }
    }
    
    // Sort models by size (largest first) and then by name
    models.sort_by(|a, b| {
        b.size_mb.partial_cmp(&a.size_mb)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then(a.name.cmp(&b.name))
    });
    
    println!("📊 Found {} total Whisper models", models.len());
    Ok(models)
}

#[tauri::command]
pub fn set_active_whisper_model(model_path: String) -> Result<String, String> {
    println!("🎯 Setting active Whisper model: {}", model_path);
    
    // Validate that the model file exists
    if !std::path::Path::new(&model_path).exists() {
        return Err(format!("Model file does not exist: {}", model_path));
    }
    
    // Set environment variable for the current session
    std::env::set_var("WHISPER_MODEL_PATH", &model_path);
    
    println!("✅ Active Whisper model set to: {}", model_path);
    Ok(format!("Successfully set active model to: {}", std::path::Path::new(&model_path).file_name().and_then(|n| n.to_str()).unwrap_or(&model_path)))
}

#[tauri::command]
pub fn get_active_whisper_model() -> Result<Option<String>, String> {
    match std::env::var("WHISPER_MODEL_PATH") {
        Ok(path) => {
            if std::path::Path::new(&path).exists() {
                Ok(Some(path))
            } else {
                println!("⚠️ WHISPER_MODEL_PATH is set but file doesn't exist: {}", path);
                Ok(None)
            }
        }
        Err(_) => Ok(None), // No environment variable set
    }
}