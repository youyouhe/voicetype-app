use sqlx::{SqlitePool, sqlite::SqliteConnectOptions};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use tracing::info;
use std::str::FromStr;

// Database models
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct AsrConfig {
    pub id: String,
    pub service_provider: String, // "local" or "cloud"
    pub local_endpoint: Option<String>,
    pub local_api_key: Option<String>,
    pub cloud_endpoint: Option<String>,
    pub cloud_api_key: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct HotkeyConfig {
    pub id: String,
    pub transcribe_key: String,
    pub translate_key: String,
    pub trigger_delay_ms: i64,
    pub anti_mistouch_enabled: bool,
    pub save_wav_files: bool,
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

#[derive(Clone)]
pub struct Database {
    pool: SqlitePool,
}

impl Database {
    pub async fn new() -> Result<Self, sqlx::Error> {
        // Use a hidden directory to avoid triggering file watches
        let app_dir = std::env::current_dir().unwrap().join(".tauri-data");
        std::fs::create_dir_all(&app_dir).ok();

        let db_dir = app_dir.join("databases");
        std::fs::create_dir_all(&db_dir).ok();

        let db_path = db_dir.join("voice_assistant.db");
        let connection_string = format!("sqlite:{}", db_path.display());

        info!("Initializing database at: {}", connection_string);

        let connect_options = SqliteConnectOptions::from_str(&connection_string)?
            .create_if_missing(true);

        let pool = SqlitePool::connect_with(connect_options).await?;

        // Run migrations
        let db = Self { pool };
        db.migrate().await?;

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
                created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
                updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
            )
            "#
        )
        .execute(&self.pool)
        .await?;

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
        .execute(&self.pool)
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
        .execute(&self.pool)
        .await?;

        // Create indexes for better query performance
        sqlx::query("CREATE INDEX IF NOT EXISTS idx_history_type ON history_records(record_type)")
            .execute(&self.pool)
            .await?;

        sqlx::query("CREATE INDEX IF NOT EXISTS idx_history_created ON history_records(created_at)")
            .execute(&self.pool)
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
        .execute(&self.pool)
        .await?;

        // Add the save_wav_files column if it doesn't exist (for existing databases)
        sqlx::query(
            r#"
            ALTER TABLE hotkey_configs ADD COLUMN save_wav_files BOOLEAN NOT NULL DEFAULT TRUE
            "#
        )
        .execute(&self.pool)
        .await
        .ok(); // Ignore error if column already exists

        info!("Database migrations completed successfully");
        Ok(())
    }

    // Hotkey Configuration methods
    pub async fn get_hotkey_config(&self) -> Result<Option<HotkeyConfig>, sqlx::Error> {
        let config = sqlx::query_as::<_, HotkeyConfig>(
            "SELECT * FROM hotkey_configs ORDER BY updated_at DESC LIMIT 1"
        )
        .fetch_optional(&self.pool)
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
    ) -> Result<HotkeyConfig, sqlx::Error> {
        let now = Utc::now();

        // First, try to update existing record
        let update_result = sqlx::query_as::<_, HotkeyConfig>(
            r#"
            UPDATE hotkey_configs
            SET transcribe_key = $1,
                translate_key = $2,
                trigger_delay_ms = $3,
                anti_mistouch_enabled = $4,
                save_wav_files = $5,
                updated_at = $6
            WHERE id = (SELECT id FROM hotkey_configs ORDER BY updated_at DESC LIMIT 1)
            RETURNING *
            "#
        )
        .bind(transcribe_key)
        .bind(translate_key)
        .bind(trigger_delay_ms)
        .bind(anti_mistouch_enabled)
        .bind(save_wav_files)
        .bind(now)
        .fetch_optional(&self.pool)
        .await?;

        if let Some(config) = update_result {
            info!("Updated hotkey config");
            println!("✅ Database: Updated hotkey config with new shortcuts");
            Ok(config)
        } else {
            // If no existing record, insert new one
            println!("⚠️ Database: No hotkey config found, creating new one...");
            let id = Uuid::new_v4().to_string();
            println!("🆔 Database: New hotkey config ID: {}", id);
            println!("💾 Database: Inserting transcribe_key: {}, translate_key: {}", transcribe_key, translate_key);

            let config = sqlx::query_as::<_, HotkeyConfig>(
                r#"
                INSERT INTO hotkey_configs (id, transcribe_key, translate_key, trigger_delay_ms, anti_mistouch_enabled, save_wav_files, created_at, updated_at)
                VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
                RETURNING *
                "#
            )
            .bind(&id)
            .bind(transcribe_key)
            .bind(translate_key)
            .bind(trigger_delay_ms)
            .bind(anti_mistouch_enabled)
            .bind(save_wav_files)
            .bind(now)
            .bind(now)
            .fetch_one(&self.pool)
            .await?;

            info!("Created new hotkey config");
            println!("✅ Database: Created new hotkey config");
            Ok(config)
        }
    }

    // ASR Configuration methods
    pub async fn get_asr_config(&self) -> Result<Option<AsrConfig>, sqlx::Error> {
        let config = sqlx::query_as::<_, AsrConfig>(
            "SELECT * FROM asr_configs ORDER BY updated_at DESC LIMIT 1"
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(config)
    }

    pub async fn save_asr_config(
        &self,
        service_provider: &str,
        local_endpoint: Option<&str>,
        local_api_key: Option<&str>,
        cloud_endpoint: Option<&str>,
        cloud_api_key: Option<&str>,
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
                updated_at = $6
            WHERE id = (SELECT id FROM asr_configs ORDER BY updated_at DESC LIMIT 1)
            RETURNING *
            "#
        )
        .bind(service_provider)
        .bind(local_endpoint)
        .bind(local_api_key)
        .bind(cloud_endpoint)
        .bind(cloud_api_key)
        .bind(now)
        .fetch_optional(&self.pool)
        .await?;

        if let Some(config) = update_result {
            info!("Updated ASR config for provider: {}", service_provider);
            println!("✅ Database: Updated existing ASR config with new API key");
            Ok(config)
        } else {
            // If no existing record, insert new one
            println!("⚠️ Database: No existing record found, creating new one...");
            let id = Uuid::new_v4().to_string();
            println!("🆔 Database: New record ID: {}", id);
            println!("💾 Database: Inserting API key: {:?}", local_api_key);

            let config = sqlx::query_as::<_, AsrConfig>(
                r#"
                INSERT INTO asr_configs (id, service_provider, local_endpoint, local_api_key, cloud_endpoint, cloud_api_key, created_at, updated_at)
                VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
                RETURNING *
                "#
            )
            .bind(&id)
            .bind(service_provider)
            .bind(local_endpoint)
            .bind(local_api_key)
            .bind(cloud_endpoint)
            .bind(cloud_api_key)
            .bind(now)
            .bind(now)
            .fetch_one(&self.pool)
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
        .fetch_optional(&self.pool)
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
        .fetch_one(&self.pool)
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
        .fetch_one(&self.pool)
        .await?;

        Ok(history)
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
            .fetch_all(&self.pool)
            .await?;

        Ok(records)
    }

    pub async fn get_history_stats(&self) -> Result<(i64, i64, i64), sqlx::Error> {
        let total_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM history_records")
            .fetch_one(&self.pool)
            .await?;

        let success_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM history_records WHERE success = true")
            .fetch_one(&self.pool)
            .await?;

        let transcribe_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM history_records WHERE record_type = 'transcribe'")
            .fetch_one(&self.pool)
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
        .execute(&self.pool)
        .await?;

        let deleted_count = result.rows_affected();
        info!("Cleaned up {} old records older than {} days", deleted_count, days);

        Ok(deleted_count)
    }
}

impl Drop for Database {
    fn drop(&mut self) {
        // Note: In a real application, you might want to close the pool gracefully
        info!("Database connection dropped");
    }
}