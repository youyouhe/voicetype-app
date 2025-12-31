use sqlx::{SqlitePool, sqlite::SqliteConnectOptions};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use tracing::info;
use std::str::FromStr;
use std::sync::{Arc, Mutex};
use std::sync::OnceLock;

// Database models
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct AsrConfig {
    pub id: String,
    pub service_provider: String, // "local" or "cloud"
    pub local_endpoint: Option<String>,
    pub local_api_key: Option<String>,
    pub cloud_endpoint: Option<String>,
    pub cloud_api_key: Option<String>,
    pub whisper_model: Option<String>, // 新增：选择的whisper模型
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TypingDelays {
    pub clipboard_update_ms: i64,
    pub keyboard_events_settle_ms: i64,
    pub typing_complete_ms: i64,
    pub character_interval_ms: i64,
    pub short_operation_ms: i64,
}

// 流式配置结构体
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct StreamingConfig {
    pub id: String,
    pub enabled: bool,                   // 是否启用流式（默认 false）
    pub chunk_interval_ms: i64,          // 处理间隔（默认 500ms）
    pub vad_threshold: f64,              // VAD 阈值（默认 0.5）
    pub min_speech_duration_ms: i64,     // 最小语音时长（默认 1000ms）
    pub min_silence_duration_ms: i64,    // 最小静默时长（默认 2000ms）
    pub max_segment_length_ms: i64,      // 最大段落长度（默认 30000ms）
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Default for StreamingConfig {
    fn default() -> Self {
        Self {
            id: String::new(),
            enabled: false,              // 默认关闭，需手动启用
            chunk_interval_ms: 500,
            vad_threshold: 0.5,
            min_speech_duration_ms: 1000,
            min_silence_duration_ms: 2000,
            max_segment_length_ms: 30000,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }
}

impl Default for TypingDelays {
    fn default() -> Self {
        Self {
            clipboard_update_ms: 100,
            keyboard_events_settle_ms: 300,
            typing_complete_ms: 500,
            character_interval_ms: 100,
            short_operation_ms: 100,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct HotkeyConfig {
    pub id: String,
    pub transcribe_key: String,
    pub translate_key: String,
    pub trigger_delay_ms: i64,
    pub anti_mistouch_enabled: bool,
    pub save_wav_files: bool,
    pub clipboard_update_ms: i64,
    pub keyboard_events_settle_ms: i64,
    pub typing_complete_ms: i64,
    pub character_interval_ms: i64,
    pub short_operation_ms: i64,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct TranslationConfig {
    pub id: String,
    pub provider: String, // "siliconflow" or "ollama"
    pub api_key: Option<String>,
    pub endpoint: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct HistoryRecord {
    pub id: String,
    pub record_type: String, // "transcribe" or "translate"
    pub input_text: Option<String>,
    pub output_text: Option<String>,
    pub audio_file_path: Option<String>,
    pub processor_type: Option<String>,
    pub processing_time_ms: Option<i64>,
    pub success: bool,
    pub error_message: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewHistoryRecord {
    pub record_type: String,
    pub input_text: Option<String>,
    pub output_text: Option<String>,
    pub audio_file_path: Option<String>,
    pub processor_type: Option<String>,
    pub processing_time_ms: Option<i64>,
    pub success: bool,
    pub error_message: Option<String>,
}

// Statistics models
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct ServiceStats {
    pub id: String,
    pub service_name: String,
    pub status: String, // "online", "offline", "error"
    pub endpoint: Option<String>,
    pub last_check: DateTime<Utc>,
    pub uptime_seconds: i64,
    pub total_requests: i64,
    pub successful_requests: i64,
    pub failed_requests: i64,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct LatencyRecord {
    pub id: String,
    pub service_name: String,
    pub latency_ms: i64,
    pub request_type: String, // "transcribe", "translate"
    pub recorded_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct UsageLog {
    pub id: String,
    pub date: String, // YYYY-MM-DD format
    pub total_seconds: i64,
    pub total_requests: i64,
    pub successful_requests: i64,
    pub failed_requests: i64,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewServiceStats {
    pub service_name: String,
    pub status: String,
    pub endpoint: Option<String>,
    pub uptime_seconds: i64,
    pub total_requests: i64,
    pub successful_requests: i64,
    pub failed_requests: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewLatencyRecord {
    pub service_name: String,
    pub latency_ms: i64,
    pub request_type: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewUsageLog {
    pub date: String,
    pub total_seconds: i64,
    pub total_requests: i64,
    pub successful_requests: i64,
    pub failed_requests: i64,
}

// 全局数据库连接池
static GLOBAL_DB_POOL: OnceLock<Arc<Mutex<Option<SqlitePool>>>> = OnceLock::new();

#[derive(Clone)]
pub struct Database {
    pool: Arc<SqlitePool>,
}

impl Database {
    pub async fn new() -> Result<Self, sqlx::Error> {
        println!("🗄️ Database: Database::new() called");

        // 获取或创建全局连接池
        let pool_guard = GLOBAL_DB_POOL.get_or_init(|| {
            Arc::new(Mutex::new(None))
        });

        // 尝试获取现有连接池
        {
            let pool_option = pool_guard.lock().unwrap();
            if let Some(ref pool) = *pool_option {
                println!("📦 Database: Using existing global pool");
                let db = Self { pool: Arc::new(pool.clone()) };
                return Ok(db);
            }
        }

        // 创建新连接池
        println!("🏗️ Database: Creating new global database pool...");

        // Use a hidden directory to avoid triggering file watches
        let app_dir = std::env::current_dir().unwrap().join(".tauri-data");
        println!("📁 Database: App dir: {:?}", app_dir);
        std::fs::create_dir_all(&app_dir).ok();

        let db_dir = app_dir.join("databases");
        println!("📁 Database: DB dir: {:?}", db_dir);
        std::fs::create_dir_all(&db_dir).ok();

        let db_path = db_dir.join("voice_assistant.db");
        println!("📁 Database: DB path: {:?}", db_path);
        let connection_string = format!("sqlite:{}", db_path.display());
        println!("🔗 Database: Connection string: {}", connection_string);

        info!("Initializing database at: {}", connection_string);

        println!("🔌 Database: Creating SQLite connection...");
        let connect_options = SqliteConnectOptions::from_str(&connection_string)
            .unwrap_or_else(|_| SqliteConnectOptions::new().filename(&db_path))
            .create_if_missing(true)
            .journal_mode(sqlx::sqlite::SqliteJournalMode::Wal) // 使用WAL模式提升性能
            .synchronous(sqlx::sqlite::SqliteSynchronous::Normal) // 适度同步
            .busy_timeout(std::time::Duration::from_secs(30)); // 30秒超时

        println!("🏊 Database: Connecting to database pool...");
        let pool = SqlitePool::connect_with(connect_options).await?;
        println!("✅ Database: Global database pool connected successfully");

        // 存储到全局变量
        {
            let mut pool_option = pool_guard.lock().unwrap();
            *pool_option = Some(pool.clone());
        }

        let db = Self { pool: Arc::new(pool) };

        // 运行迁移（只在第一次创建时）
        db.migrate().await?;
        println!("✅ Database: Migrations completed successfully");

        Ok(db)
    }

    async fn migrate(&self) -> Result<(), sqlx::Error> {
        info!("Running database migrations");

        // Create ASR config table
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS asr_configs (
                id TEXT PRIMARY KEY,
                service_provider TEXT NOT NULL,
                local_endpoint TEXT,
                local_api_key TEXT,
                cloud_endpoint TEXT,
                cloud_api_key TEXT,
                whisper_model TEXT,
                created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
                updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
            )
            "#
        )
        .execute(&*self.pool)
        .await?;

        // 添加 whisper_model 列如果不存在（为现有数据库）
        sqlx::query(
            "ALTER TABLE asr_configs ADD COLUMN whisper_model TEXT"
        )
        .execute(&*self.pool)
        .await
        .ok(); // 忽略错误，如果列已存在

        // Create translation config table
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS translation_configs (
                id TEXT PRIMARY KEY,
                provider TEXT NOT NULL,
                api_key TEXT,
                endpoint TEXT,
                created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
                updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
            )
            "#
        )
        .execute(&*self.pool)
        .await?;

        // Create history records table
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS history_records (
                id TEXT PRIMARY KEY,
                record_type TEXT NOT NULL,
                input_text TEXT,
                output_text TEXT,
                audio_file_path TEXT,
                processor_type TEXT,
                processing_time_ms INTEGER,
                success BOOLEAN NOT NULL DEFAULT FALSE,
                error_message TEXT,
                created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
            )
            "#
        )
        .execute(&*self.pool)
        .await?;

        // Create indexes for better query performance
        sqlx::query("CREATE INDEX IF NOT EXISTS idx_history_type ON history_records(record_type)")
            .execute(&*self.pool)
            .await?;

        sqlx::query("CREATE INDEX IF NOT EXISTS idx_history_created ON history_records(created_at)")
            .execute(&*self.pool)
            .await?;

        // Create hotkey configs table
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS hotkey_configs (
                id TEXT PRIMARY KEY,
                transcribe_key TEXT NOT NULL,
                translate_key TEXT NOT NULL,
                trigger_delay_ms INTEGER NOT NULL DEFAULT 300,
                anti_mistouch_enabled BOOLEAN NOT NULL DEFAULT TRUE,
                save_wav_files BOOLEAN NOT NULL DEFAULT TRUE,
                created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
                updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
            )
            "#,
        )
        .execute(&*self.pool)
        .await?;

        // Add the save_wav_files column if it doesn't exist (for existing databases)
        sqlx::query(
            r#"
            ALTER TABLE hotkey_configs ADD COLUMN save_wav_files BOOLEAN NOT NULL DEFAULT TRUE
            "#
        )
        .execute(&*self.pool)
        .await
        .ok(); // Ignore error if column already exists

        // Migrate usage_logs table from total_minutes to total_seconds if needed
        // First, check if total_seconds column exists
        let column_exists = sqlx::query_scalar::<_, bool>(
            "SELECT COUNT(*) > 0 FROM pragma_table_info('usage_logs') WHERE name = 'total_seconds'"
        )
        .fetch_one(&*self.pool)
        .await
        .unwrap_or(false);

        if !column_exists {
            println!("🔄 Database: Migrating usage_logs table from total_minutes to total_seconds");
            
            // Add total_seconds column
            sqlx::query(
                "ALTER TABLE usage_logs ADD COLUMN total_seconds INTEGER NOT NULL DEFAULT 0"
            )
            .execute(&*self.pool)
            .await
            .ok(); // Ignore error if column already exists
            
            // Migrate data from total_minutes to total_seconds (multiply by 60)
            sqlx::query(
                "UPDATE usage_logs SET total_seconds = total_minutes * 60 WHERE total_minutes > 0"
            )
            .execute(&*self.pool)
            .await
            .ok();
            
            println!("✅ Database: Migration from total_minutes to total_seconds completed");
        }

        // Add typing delays columns if they don't exist (for existing databases)
        sqlx::query(
            r#"
            ALTER TABLE hotkey_configs ADD COLUMN clipboard_update_ms INTEGER NOT NULL DEFAULT 100
            "#
        )
        .execute(&*self.pool)
        .await
        .ok(); // Ignore error if column already exists

        // Create streaming_config table for streaming ASR settings
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS streaming_config (
                id TEXT PRIMARY KEY,
                enabled BOOLEAN NOT NULL DEFAULT FALSE,
                chunk_interval_ms INTEGER NOT NULL DEFAULT 500,
                vad_threshold REAL NOT NULL DEFAULT 0.5,
                min_speech_duration_ms INTEGER NOT NULL DEFAULT 1000,
                min_silence_duration_ms INTEGER NOT NULL DEFAULT 2000,
                max_segment_length_ms INTEGER NOT NULL DEFAULT 30000,
                created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
                updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
            )
            "#,
        )
        .execute(&*self.pool)
        .await?;

        sqlx::query(
            r#"
            ALTER TABLE hotkey_configs ADD COLUMN keyboard_events_settle_ms INTEGER NOT NULL DEFAULT 300
            "#
        )
        .execute(&*self.pool)
        .await
        .ok(); // Ignore error if column already exists

        sqlx::query(
            r#"
            ALTER TABLE hotkey_configs ADD COLUMN typing_complete_ms INTEGER NOT NULL DEFAULT 500
            "#
        )
        .execute(&*self.pool)
        .await
        .ok(); // Ignore error if column already exists

        sqlx::query(
            r#"
            ALTER TABLE hotkey_configs ADD COLUMN character_interval_ms INTEGER NOT NULL DEFAULT 100
            "#
        )
        .execute(&*self.pool)
        .await
        .ok(); // Ignore error if column already exists

        sqlx::query(
            r#"
            ALTER TABLE hotkey_configs ADD COLUMN short_operation_ms INTEGER NOT NULL DEFAULT 100
            "#
        )
        .execute(&*self.pool)
        .await
        .ok(); // Ignore error if column already exists

        // Create service stats table
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS service_stats (
                id TEXT PRIMARY KEY,
                service_name TEXT NOT NULL UNIQUE,
                status TEXT NOT NULL DEFAULT 'offline',
                endpoint TEXT,
                last_check DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
                uptime_seconds INTEGER NOT NULL DEFAULT 0,
                total_requests INTEGER NOT NULL DEFAULT 0,
                successful_requests INTEGER NOT NULL DEFAULT 0,
                failed_requests INTEGER NOT NULL DEFAULT 0,
                created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
                updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
            )
            "#
        )
        .execute(&*self.pool)
        .await?;

        // Create latency records table
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS latency_records (
                id TEXT PRIMARY KEY,
                service_name TEXT NOT NULL,
                latency_ms INTEGER NOT NULL,
                request_type TEXT NOT NULL,
                recorded_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
            )
            "#
        )
        .execute(&*self.pool)
        .await?;

        // Create usage logs table
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS usage_logs (
                id TEXT PRIMARY KEY,
                date TEXT NOT NULL UNIQUE,
                total_seconds INTEGER NOT NULL DEFAULT 0,
                total_requests INTEGER NOT NULL DEFAULT 0,
                successful_requests INTEGER NOT NULL DEFAULT 0,
                failed_requests INTEGER NOT NULL DEFAULT 0,
                created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
                updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
            )
            "#
        )
        .execute(&*self.pool)
        .await?;

        // Create indexes for statistics tables
        sqlx::query("CREATE INDEX IF NOT EXISTS idx_latency_service ON latency_records(service_name)")
            .execute(&*self.pool)
            .await?;

        sqlx::query("CREATE INDEX IF NOT EXISTS idx_latency_recorded ON latency_records(recorded_at)")
            .execute(&*self.pool)
            .await?;

        sqlx::query("CREATE INDEX IF NOT EXISTS idx_usage_date ON usage_logs(date)")
            .execute(&*self.pool)
            .await?;

        info!("Database migrations completed successfully");
        Ok(())
    }

    // Hotkey Configuration methods
    pub async fn get_hotkey_config(&self) -> Result<Option<HotkeyConfig>, sqlx::Error> {
        let config = sqlx::query_as::<_, HotkeyConfig>(
            "SELECT * FROM hotkey_configs ORDER BY updated_at DESC LIMIT 1"
        )
        .fetch_optional(&*self.pool)
        .await?;

        Ok(config)
    }

    pub async fn save_hotkey_config(
        &self,
        transcribe_key: &str,
        translate_key: &str,
        trigger_delay_ms: i64,
        anti_mistouch_enabled: bool,
        save_wav_files: bool,
        typing_delays: Option<&TypingDelays>,
    ) -> Result<HotkeyConfig, sqlx::Error> {
        let now = Utc::now();

        let default_delays = TypingDelays::default();
        let delays = typing_delays.unwrap_or(&default_delays);

        // Check if columns exist by attempting a query first
        // The columns should already exist from migrations, so we skip ALTER TABLE attempts

        // First, try to update existing record
        println!("🔄 Database: Attempting to update existing record...");
        println!("  - transcribe_key: {}", transcribe_key);
        println!("  - clipboard_update_ms: {}", delays.clipboard_update_ms);
        println!("  - keyboard_events_settle_ms: {}", delays.keyboard_events_settle_ms);

        let update_result = sqlx::query_as::<_, HotkeyConfig>(
            r#"
            UPDATE hotkey_configs
            SET transcribe_key = $1,
                translate_key = $2,
                trigger_delay_ms = $3,
                anti_mistouch_enabled = $4,
                save_wav_files = $5,
                clipboard_update_ms = $6,
                keyboard_events_settle_ms = $7,
                typing_complete_ms = $8,
                character_interval_ms = $9,
                short_operation_ms = $10,
                updated_at = $11
            WHERE id = (SELECT id FROM hotkey_configs ORDER BY updated_at DESC LIMIT 1)
            RETURNING *
            "#
        )
        .bind(transcribe_key)
        .bind(translate_key)
        .bind(trigger_delay_ms)
        .bind(anti_mistouch_enabled)
        .bind(save_wav_files)
        .bind(delays.clipboard_update_ms)
        .bind(delays.keyboard_events_settle_ms)
        .bind(delays.typing_complete_ms)
        .bind(delays.character_interval_ms)
        .bind(delays.short_operation_ms)
        .bind(now)
        .fetch_optional(&*self.pool)
        .await?;

        if let Some(config) = update_result {
            info!("Updated hotkey config");
            println!("✅ Database: Successfully updated hotkey config!");
            println!("  - Updated clipboard_update_ms: {}", config.clipboard_update_ms);
            println!("  - Updated keyboard_events_settle_ms: {}", config.keyboard_events_settle_ms);
            println!("  - Updated typing_complete_ms: {}", config.typing_complete_ms);
            println!("  - Updated character_interval_ms: {}", config.character_interval_ms);
            println!("  - Updated short_operation_ms: {}", config.short_operation_ms);
            Ok(config)
        } else {
            // If no existing record, insert new one
            println!("⚠️ Database: No existing hotkey config found, creating new one...");
            let id = Uuid::new_v4().to_string();
            println!("🆔 Database: New hotkey config ID: {}", id);
            println!("💾 Database: Inserting transcribe_key: {}, translate_key: {}", transcribe_key, translate_key);

            let config = sqlx::query_as::<_, HotkeyConfig>(
                r#"
                INSERT INTO hotkey_configs (id, transcribe_key, translate_key, trigger_delay_ms, anti_mistouch_enabled, save_wav_files, clipboard_update_ms, keyboard_events_settle_ms, typing_complete_ms, character_interval_ms, short_operation_ms, created_at, updated_at)
                VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13)
                RETURNING *
                "#
            )
            .bind(&id)
            .bind(transcribe_key)
            .bind(translate_key)
            .bind(trigger_delay_ms)
            .bind(anti_mistouch_enabled)
            .bind(save_wav_files)
            .bind(delays.clipboard_update_ms)
            .bind(delays.keyboard_events_settle_ms)
            .bind(delays.typing_complete_ms)
            .bind(delays.character_interval_ms)
            .bind(delays.short_operation_ms)
            .bind(now)
            .bind(now)
            .fetch_one(&*self.pool)
            .await?;

            info!("Created new hotkey config");
            println!("✅ Database: Created new hotkey config");
            Ok(config)
        }
    }

    // ASR Configuration methods
    pub async fn get_asr_config(&self) -> Result<Option<AsrConfig>, sqlx::Error> {
        println!("🗄️ Database: get_asr_config() called");
        println!("🔍 Database: Querying asr_configs table...");
        
        let config = sqlx::query_as::<_, AsrConfig>(
            "SELECT * FROM asr_configs ORDER BY updated_at DESC LIMIT 1"
        )
        .fetch_optional(&*self.pool)
        .await?;

        if let Some(ref cfg) = config {
            println!("✅ Database: Query successful, found ASR config:");
            println!("  - ID: {}", cfg.id);
            println!("  - Service Provider: {}", cfg.service_provider);
            println!("  - Local Endpoint: {:?}", cfg.local_endpoint);
            println!("  - Local API Key: {}", cfg.local_api_key.is_some());
            println!("  - Cloud Endpoint: {:?}", cfg.cloud_endpoint);
            println!("  - Cloud API Key: {}", cfg.cloud_api_key.is_some());
            println!("  - Whisper Model: {:?}", cfg.whisper_model);
            println!("  - Created At: {}", cfg.created_at);
            println!("  - Updated At: {}", cfg.updated_at);
        } else {
            println!("📥 Database: Query successful, but no ASR config found");
        }

        Ok(config)
    }

    pub async fn save_asr_config(
        &self,
        service_provider: &str,
        local_endpoint: Option<&str>,
        local_api_key: Option<&str>,
        cloud_endpoint: Option<&str>,
        cloud_api_key: Option<&str>,
        whisper_model: Option<&str>,
    ) -> Result<AsrConfig, sqlx::Error> {
        let now = Utc::now();

        // First, try to update existing record
        let update_result = sqlx::query_as::<_, AsrConfig>(
            r#"
            UPDATE asr_configs
            SET service_provider = $1,
                local_endpoint = $2,
                local_api_key = $3,
                cloud_endpoint = $4,
                cloud_api_key = $5,
                whisper_model = $6,
                updated_at = $7
            WHERE id = (SELECT id FROM asr_configs ORDER BY updated_at DESC LIMIT 1)
            RETURNING *
            "#
        )
        .bind(service_provider)
        .bind(local_endpoint)
        .bind(local_api_key)
        .bind(cloud_endpoint)
        .bind(cloud_api_key)
        .bind(whisper_model)
        .bind(now)
        .fetch_optional(&*self.pool)
        .await?;

        if let Some(config) = update_result {
            info!("Updated ASR config for provider: {}", service_provider);
            println!("✅ Database: Updated existing ASR config with whisper model: {:?}", whisper_model);
            Ok(config)
        } else {
            // If no existing record, insert new one
            println!("⚠️ Database: No existing record found, creating new one...");
            let id = Uuid::new_v4().to_string();
            println!("🆔 Database: New record ID: {}", id);
            println!("💾 Database: Inserting API key: {:?}", local_api_key);

            let config = sqlx::query_as::<_, AsrConfig>(
                r#"
                INSERT INTO asr_configs (id, service_provider, local_endpoint, local_api_key, cloud_endpoint, cloud_api_key, whisper_model, created_at, updated_at)
                VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
                RETURNING *
                "#
            )
            .bind(&id)
            .bind(service_provider)
            .bind(local_endpoint)
            .bind(local_api_key)
            .bind(cloud_endpoint)
            .bind(cloud_api_key)
            .bind(whisper_model)
            .bind(now)
            .bind(now)
            .fetch_one(&*self.pool)
            .await?;

            info!("Created new ASR config for provider: {}", service_provider);
            Ok(config)
        }
    }

    // Translation Configuration methods
    pub async fn get_translation_config(&self, provider: &str) -> Result<Option<TranslationConfig>, sqlx::Error> {
        let config = sqlx::query_as::<_, TranslationConfig>(
            "SELECT * FROM translation_configs WHERE provider = $1 ORDER BY updated_at DESC LIMIT 1"
        )
        .bind(provider)
        .fetch_optional(&*self.pool)
        .await?;

        Ok(config)
    }

    pub async fn save_translation_config(
        &self,
        provider: &str,
        api_key: Option<&str>,
        endpoint: Option<&str>,
    ) -> Result<TranslationConfig, sqlx::Error> {
        let id = Uuid::new_v4().to_string();
        let now = Utc::now();

        let config = sqlx::query_as::<_, TranslationConfig>(
            r#"
            INSERT INTO translation_configs (id, provider, api_key, endpoint, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6)
            RETURNING *
            "#
        )
        .bind(&id)
        .bind(provider)
        .bind(api_key)
        .bind(endpoint)
        .bind(now)
        .bind(now)
        .fetch_one(&*self.pool)
        .await?;

        info!("Saved translation config for provider: {}", provider);
        Ok(config)
    }

    // History methods
    pub async fn add_history_record(&self, record: NewHistoryRecord) -> Result<HistoryRecord, sqlx::Error> {
        let id = Uuid::new_v4().to_string();
        let now = Utc::now();

        let history = sqlx::query_as::<_, HistoryRecord>(
            r#"
            INSERT INTO history_records (id, record_type, input_text, output_text, audio_file_path, processor_type, processing_time_ms, success, error_message, created_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
            RETURNING *
            "#
        )
        .bind(&id)
        .bind(&record.record_type)
        .bind(&record.input_text)
        .bind(&record.output_text)
        .bind(&record.audio_file_path)
        .bind(&record.processor_type)
        .bind(record.processing_time_ms)
        .bind(record.success)
        .bind(&record.error_message)
        .bind(now)
        .fetch_one(&*self.pool)
        .await?;

        // Update service statistics after successful history record addition
        if record.success {
            self.update_service_stats_from_record(&record, now).await?;
            self.update_latency_from_record(&record, now).await?;
            self.update_usage_from_record(&record, now).await?;
        }

        Ok(history)
    }

    // Helper function to update service stats from a new history record
    async fn update_service_stats_from_record(&self, record: &NewHistoryRecord, _timestamp: chrono::DateTime<chrono::Utc>) -> Result<(), sqlx::Error> {
        let service_name = match record.processor_type.as_deref() {
            Some("whisper") => "whisper_asr",
            Some("sensevoice") => "sensevoice_asr", 
            Some("local") => "local_asr",
            Some("siliconflow") => "siliconflow_translation",
            Some("ollama") => "ollama_translation",
            _ => "unknown_service",
        };

        let status = if record.success { "online" } else { "error" };
        
        self.update_service_status(service_name, status, None).await?;
        Ok(())
    }

    // Helper function to update latency from a new history record
    async fn update_latency_from_record(&self, record: &NewHistoryRecord, timestamp: chrono::DateTime<chrono::Utc>) -> Result<(), sqlx::Error> {
        let service_name = match record.processor_type.as_deref() {
            Some("whisper") | Some("whisper-rs") => "local_asr",  // whisper-rs maps to local_asr
            Some("sensevoice") => "sensevoice_asr",
            Some("local") => "local_asr",
            Some("cloud") => "cloud_asr",
            _ => "local_asr",  // Default to local_asr for unknown types
        };

        // Insert latency record
        let id = Uuid::new_v4().to_string();
        sqlx::query(
            r#"
            INSERT INTO latency_records (id, service_name, latency_ms, request_type, recorded_at)
            VALUES ($1, $2, $3, $4, $5)
            "#
        )
        .bind(&id)
        .bind(service_name)
        .bind(record.processing_time_ms.unwrap_or(0))
        .bind(&record.record_type)
        .bind(timestamp)
        .execute(&*self.pool)
        .await?;

        Ok(())
    }

    // Helper function to update usage from a new history record
    async fn update_usage_from_record(&self, record: &NewHistoryRecord, timestamp: chrono::DateTime<chrono::Utc>) -> Result<(), sqlx::Error> {
        // Update today's usage (calculate seconds from processing time)
        let seconds_today = (record.processing_time_ms.unwrap_or(0) / 1000).max(1); // Convert ms to seconds, at least 1 second
        
        // Update or insert today's usage record
        let today = timestamp.format("%Y-%m-%d").to_string();
        let id = Uuid::new_v4().to_string();
        
        sqlx::query(
            r#"
            INSERT OR REPLACE INTO usage_logs (id, date, total_seconds, total_requests, successful_requests)
            VALUES (
                $1,
                $2,
                COALESCE((SELECT total_seconds FROM usage_logs WHERE date = $2), 0) + $3,
                COALESCE((SELECT total_requests FROM usage_logs WHERE date = $2), 0) + 1,
                COALESCE((SELECT successful_requests FROM usage_logs WHERE date = $2), 0) + $4
            )
            "#
        )
        .bind(&id)
        .bind(&today)
        .bind(seconds_today)
        .bind(if record.success { 1 } else { 0 })
        .execute(&*self.pool)
        .await?;

        Ok(())
    }

    pub async fn get_history_records(
        &self,
        limit: Option<i64>,
        record_type: Option<&str>,
    ) -> Result<Vec<HistoryRecord>, sqlx::Error> {
        let mut query = "SELECT * FROM history_records".to_string();
        let mut conditions = Vec::new();

        if let Some(r_type) = record_type {
            conditions.push(format!("record_type = '{}'", r_type));
        }

        if !conditions.is_empty() {
            query += " WHERE ";
            query += &conditions.join(" AND ");
        }

        query += " ORDER BY created_at DESC";

        if let Some(limit_val) = limit {
            query += &format!(" LIMIT {}", limit_val);
        }

        let records = sqlx::query_as::<_, HistoryRecord>(&query)
            .fetch_all(&*self.pool)
            .await?;

        Ok(records)
    }

    pub async fn get_history_stats(&self) -> Result<(i64, i64, i64), sqlx::Error> {
        let total_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM history_records")
            .fetch_one(&*self.pool)
            .await?;

        let success_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM history_records WHERE success = true")
            .fetch_one(&*self.pool)
            .await?;

        let transcribe_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM history_records WHERE record_type = 'transcribe'")
            .fetch_one(&*self.pool)
            .await?;

        Ok((total_count, success_count, transcribe_count))
    }

    // Utility methods
    pub async fn cleanup_old_records(&self, days: i64) -> Result<u64, sqlx::Error> {
        let cutoff_date = Utc::now() - chrono::Duration::days(days);

        let result = sqlx::query(
            "DELETE FROM history_records WHERE created_at < $1"
        )
        .bind(cutoff_date)
        .execute(&*self.pool)
        .await?;

        let deleted_count = result.rows_affected();
        info!("Cleaned up {} old records older than {} days", deleted_count, days);

        Ok(deleted_count)
    }

    /// Create or get a global database pool instance
    pub async fn from_global_pool() -> Result<Self, sqlx::Error> {
        // 使用同一个全局连接池
        Self::new().await
    }

    // Statistics methods for frontend
    pub async fn get_service_status(&self, service_name: &str) -> Result<Option<ServiceStats>, sqlx::Error> {
        let stats = sqlx::query_as::<_, ServiceStats>(
            "SELECT * FROM service_stats WHERE service_name = ?"
        )
        .bind(service_name)
        .fetch_optional(&*self.pool)
        .await?;

        Ok(stats)
    }

    pub async fn get_all_service_stats(&self) -> Result<Vec<ServiceStats>, sqlx::Error> {
        let stats = sqlx::query_as::<_, ServiceStats>(
            "SELECT * FROM service_stats ORDER BY updated_at DESC"
        )
        .fetch_all(&*self.pool)
        .await?;

        Ok(stats)
    }

    pub async fn update_service_status(&self, service_name: &str, status: &str, endpoint: Option<String>) -> Result<(), sqlx::Error> {
        let now = Utc::now();

        let result = sqlx::query(
            r#"
            UPDATE service_stats SET
                status = ?1,
                endpoint = ?2,
                last_check = ?3,
                updated_at = ?3
            WHERE service_name = ?4
            "#
        )
        .bind(status)
        .bind(&endpoint)
        .bind(now)
        .bind(service_name)
        .execute(&*self.pool)
        .await?;

        // If no rows were affected, create a new service stats record
        if result.rows_affected() == 0 {
            let id = Uuid::new_v4().to_string();
            sqlx::query(
                r#"
                INSERT INTO service_stats (
                    id, service_name, status, endpoint, last_check, uptime_seconds,
                    total_requests, successful_requests, failed_requests, created_at, updated_at
                )
                VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)
                "#
            )
            .bind(&id)
            .bind(service_name)
            .bind(status)
            .bind(&endpoint)
            .bind(now)
            .bind(0i64)
            .bind(0i64)
            .bind(0i64)
            .bind(0i64)
            .bind(now)
            .bind(now)
            .execute(&*self.pool)
            .await?;
        }

        Ok(())
    }

    pub async fn get_latency_data(&self, service_name: &str, hours_back: i64) -> Result<Vec<LatencyRecord>, sqlx::Error> {
        let cutoff = Utc::now() - chrono::Duration::hours(hours_back);

        let records = sqlx::query_as::<_, LatencyRecord>(
            r#"
            SELECT * FROM latency_records
            WHERE service_name = ? AND recorded_at >= ?
            ORDER BY recorded_at DESC
            "#
        )
        .bind(service_name)
        .bind(cutoff)
        .fetch_all(&*self.pool)
        .await?;

        Ok(records)
    }

    pub async fn get_usage_data(&self, date: &str) -> Result<Option<UsageLog>, sqlx::Error> {
        let usage = sqlx::query_as::<_, UsageLog>(
            "SELECT * FROM usage_logs WHERE date = ?"
        )
        .bind(date)
        .fetch_optional(&*self.pool)
        .await?;

        Ok(usage)
    }

    pub async fn get_today_usage(&self) -> Result<Option<UsageLog>, sqlx::Error> {
        let today = Utc::now().format("%Y-%m-%d").to_string();
        self.get_usage_data(&today).await
    }

    // ========== Streaming Configuration methods ==========
    pub async fn get_streaming_config(&self) -> Result<Option<StreamingConfig>, sqlx::Error> {
        let config = sqlx::query_as::<_, StreamingConfig>(
            "SELECT * FROM streaming_config ORDER BY updated_at DESC LIMIT 1"
        )
        .fetch_optional(&*self.pool)
        .await?;

        Ok(config)
    }

    pub async fn save_streaming_config(
        &self,
        enabled: bool,
        chunk_interval_ms: i64,
        vad_threshold: f64,
        min_speech_duration_ms: i64,
        min_silence_duration_ms: i64,
        max_segment_length_ms: i64,
    ) -> Result<StreamingConfig, sqlx::Error> {
        let now = Utc::now();

        // Try to update existing record first
        let update_result = sqlx::query_as::<_, StreamingConfig>(
            r#"
            UPDATE streaming_config
            SET enabled = $1,
                chunk_interval_ms = $2,
                vad_threshold = $3,
                min_speech_duration_ms = $4,
                min_silence_duration_ms = $5,
                max_segment_length_ms = $6,
                updated_at = $7
            WHERE id = (SELECT id FROM streaming_config ORDER BY updated_at DESC LIMIT 1)
            RETURNING *
            "#
        )
        .bind(enabled)
        .bind(chunk_interval_ms)
        .bind(vad_threshold)
        .bind(min_speech_duration_ms)
        .bind(min_silence_duration_ms)
        .bind(max_segment_length_ms)
        .bind(now)
        .fetch_optional(&*self.pool)
        .await?;

        if let Some(config) = update_result {
            info!("Updated streaming config");
            Ok(config)
        } else {
            // Insert new record
            let id = Uuid::new_v4().to_string();
            let config = sqlx::query_as::<_, StreamingConfig>(
                r#"
                INSERT INTO streaming_config (id, enabled, chunk_interval_ms, vad_threshold, min_speech_duration_ms, min_silence_duration_ms, max_segment_length_ms, created_at, updated_at)
                VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
                RETURNING *
                "#
            )
            .bind(&id)
            .bind(enabled)
            .bind(chunk_interval_ms)
            .bind(vad_threshold)
            .bind(min_speech_duration_ms)
            .bind(min_silence_duration_ms)
            .bind(max_segment_length_ms)
            .bind(now)
            .bind(now)
            .fetch_one(&*self.pool)
            .await?;

            info!("Created new streaming config");
            Ok(config)
        }
    }
}

// 移除 Drop trait，因为使用全局连接池，不需要在 drop 时关闭连接
// impl Drop for Database {
//     fn drop(&mut self) {
//         // 不再输出 "Database connection dropped" 消息
//         // 因为使用全局连接池，连接会一直保持
//     }
// }