import { invoke } from '@tauri-apps/api/core';

// Check if we're running in Tauri environment
const isTauriEnvironment = () => {
  return typeof window !== 'undefined' && window.__TAURI_INTERNALS__;
};

// Prevent repeated database initialization
let databaseInitialized = false;
let migrationChecked = false;

// Cache for getAsrConfig results to avoid redundant calls
let cachedAsrConfig: AsrConfig | null = null;
let cacheTimestamp = 0;
const CACHE_DURATION_MS = 1000; // 1秒缓存

// Console helper to show current environment
if (typeof window !== 'undefined') {
  if (isTauriEnvironment()) {
    console.log('🚀 Running in Tauri environment - using Rust backend');
  } else {
    console.log('🌐 Running in browser environment - using localStorage fallback');
    console.log('💡 Tip: Use "npm run tauri dev" to get the full experience with SQLite database');
  }
}

export interface AsrConfig {
  id: string;
  service_provider: string;
  local_endpoint?: string;
  local_api_key?: string;
  cloud_endpoint?: string;
  cloud_api_key?: string;
  created_at: string;
  updated_at: string;
}

export interface TranslationConfig {
  id: string;
  provider: string;
  api_key?: string;
  endpoint?: string;
  created_at: string;
  updated_at: string;
}

export interface HistoryRecord {
  id: string;
  record_type: string;
  input_text?: string;
  output_text?: string;
  audio_file_path?: string;
  processor_type?: string;
  processing_time_ms?: number;
  success: boolean;
  error_message?: string;
  created_at: string;
}

export interface AsrConfigRequest {
  service_provider: string;
  local_endpoint?: string;
  local_api_key?: string;
  cloud_endpoint?: string;
  cloud_api_key?: string;
}

export interface TranslationConfigRequest {
  provider: string;
  api_key?: string;
  endpoint?: string;
}

// Database service class
export class DatabaseService {
  // ASR Configuration
  static async getAsrConfig(): Promise<AsrConfig | null> {
    try {
      // 检查缓存
      const now = Date.now();
      if (cachedAsrConfig && (now - cacheTimestamp) < CACHE_DURATION_MS) {
        console.log('⚡ getAsrConfig() returning cached result');
        return cachedAsrConfig;
      }

      console.log('🚀 Frontend: getAsrConfig() called (fresh fetch)');
      console.log('🌍 Frontend: Environment check - isTauriEnvironment:', isTauriEnvironment());
      console.log('🪟 Frontend: window.__TAURI_INTERNALS__ exists:', !!(typeof window !== 'undefined' && window.__TAURI_INTERNALS__));

      let config: AsrConfig | null = null;

      if (isTauriEnvironment()) {
        // In Tauri environment, load from SQLite database via Rust backend
        console.log('🔍 Frontend: Attempting to get ASR config from SQLite via Tauri backend');
        console.log('📞 Frontend: Invoking Rust command "get_asr_config"...');
        try {
          config = await invoke<AsrConfig>('get_asr_config');
          console.log('✅ Frontend: Rust command "get_asr_config" completed successfully');
          if (config) {
            console.log('✅ Frontend: Successfully loaded ASR config from SQLite database:', {
              id: config.id,
              service_provider: config.service_provider,
              has_local_endpoint: !!config.local_endpoint,
              has_local_api_key: !!config.local_api_key,
              has_cloud_endpoint: !!config.cloud_endpoint,
              has_cloud_api_key: !!config.cloud_api_key,
              created_at: config.created_at,
              updated_at: config.updated_at
            });
          } else {
            console.log('📥 Frontend: No ASR config found in SQLite database (returned null)');
          }
        } catch (invokeError) {
          console.error('❌ Frontend: Rust command "get_asr_config" failed:', invokeError);
          console.log('🔍 Frontend: Error details:', {
            message: invokeError instanceof Error ? invokeError.message : String(invokeError),
            type: typeof invokeError,
            stack: invokeError instanceof Error ? invokeError.stack : 'No stack trace'
          });
          throw invokeError;
        }
      } else {
        // In browser environment, try to load via API bridge first
        console.log('🌐 Browser mode - attempting to load ASR config via API');
        try {
          const response = await fetch('/api/asr/config');
          if (response.ok) {
            config = await response.json();
            console.log('✅ Loaded ASR config via API bridge from SQLite database');
          } else if (response.status === 404) {
            console.log('📥 No ASR config found via API, checking localStorage');
          } else {
            throw new Error(`HTTP ${response.status}: ${response.statusText}`);
          }
        } catch (apiError) {
          console.warn('⚠️ API bridge failed, loading from localStorage:', apiError);
          console.log('💡 Tip: Make sure API bridge is running or use "npm run tauri dev" for direct SQLite access');
        }

        // Fallback to localStorage if API failed or no config found
        if (!config) {
          config = this.getAsrConfigFromStorage();
          if (config) {
            console.log('📦 Loaded ASR config from localStorage');
          }
        }
      }

      // 更新缓存
      if (config) {
        cachedAsrConfig = config;
        cacheTimestamp = Date.now();
        console.log('💾 Updated ASR config cache');
      }

      return config;
    } catch (error) {
      console.error('Failed to get ASR config:', error);

      // In browser mode, always fallback to localStorage
      if (!isTauriEnvironment()) {
        console.warn('⚠️ Fallback to localStorage');
        return this.getAsrConfigFromStorage();
      }

      throw error;
    }
  }

  static async saveAsrConfig(config: AsrConfigRequest): Promise<AsrConfig> {
    try {
      // 清除缓存，因为配置即将更新
      cachedAsrConfig = null;
      cacheTimestamp = 0;
      console.log('🗑️ Cleared ASR config cache for update');

      let result: AsrConfig;

      if (isTauriEnvironment()) {
        // In Tauri environment, save to SQLite database via Rust backend
        result = await invoke<AsrConfig>('save_asr_config', { request: config });
        console.log('💾 Saved ASR config to SQLite database');
      } else {
        // In browser environment, try to save via API bridge first
        console.log('🌐 Browser mode - attempting to sync ASR config via API');
        try {
          const response = await fetch('/api/asr/config', {
            method: 'POST',
            headers: {
              'Content-Type': 'application/json',
            },
            body: JSON.stringify(config),
          });

          if (response.ok) {
            result = await response.json();
            console.log('✅ Saved ASR config via API bridge to SQLite database');
          } else {
            throw new Error(`HTTP ${response.status}: ${response.statusText}`);
          }
        } catch (apiError) {
          console.warn('⚠️ API bridge failed, saving to localStorage:', apiError);
          console.log('💡 Tip: Make sure API bridge is running or use "npm run tauri dev" for direct SQLite access');
          result = this.saveAsrConfigToStorage(config);
        }

        // Always also save to localStorage as backup/cache
        this.saveAsrConfigToStorage(config);
      }

      return result;
    } catch (error) {
      console.error('Failed to save ASR config:', error);

      // In browser mode, always fallback to localStorage
      if (!isTauriEnvironment()) {
        console.warn('⚠️ Final fallback to localStorage');
        return this.saveAsrConfigToStorage(config);
      }

      throw error;
    }
  }

  

  // Translation Configuration
  static async getTranslationConfig(provider: string): Promise<TranslationConfig | null> {
    try {
      return await invoke<TranslationConfig>('get_translation_config', { provider });
    } catch (error) {
      console.error('Failed to get translation config:', error);
      return null;
    }
  }

  static async saveTranslationConfig(config: TranslationConfigRequest): Promise<TranslationConfig> {
    try {
      return await invoke<TranslationConfig>('save_translation_config', { request: config });
    } catch (error) {
      console.error('Failed to save translation config:', error);
      throw error;
    }
  }

  // History Management
  static async addHistoryRecord(record: {
    record_type: string;
    input_text?: string;
    output_text?: string;
    audio_file_path?: string;
    processor_type?: string;
    processing_time_ms?: number;
    success: boolean;
    error_message?: string;
  }): Promise<HistoryRecord> {
    try {
      return await invoke<HistoryRecord>('add_history_record', { request: record });
    } catch (error) {
      console.error('Failed to add history record:', error);
      throw error;
    }
  }

  static async getHistoryRecords(limit?: number, recordType?: string): Promise<HistoryRecord[]> {
    try {
      return await invoke<HistoryRecord[]>('get_history_records', { limit, recordType });
    } catch (error) {
      console.error('Failed to get history records:', error);
      return [];
    }
  }

  static async getHistoryStats(): Promise<[total: number, successful: number, transcribeCount: number]> {
    try {
      return await invoke<[total: number, successful: number, transcribeCount: number]>('get_history_stats');
    } catch (error) {
      console.error('Failed to get history stats:', error);
      return [0, 0, 0];
    }
  }

  static async cleanupOldRecords(days: number): Promise<number> {
    try {
      return await invoke<number>('cleanup_old_records', { days });
    } catch (error) {
      console.error('Failed to cleanup old records:', error);
      return 0;
    }
  }

  // Initialize database
  static async initDatabase(): Promise<string> {
    try {
      if (!isTauriEnvironment()) {
        console.log('🌐 Browser mode - no database initialization needed');
        return 'Browser mode - using localStorage';
      }
      
      if (databaseInitialized) {
        console.log('📦 Database already initialized, skipping');
        return 'Database already initialized';
      }
      
      const result = await invoke<string>('init_database');
      databaseInitialized = true;
      return result;
    } catch (error) {
      console.error('Failed to initialize database:', error);
      if (isTauriEnvironment()) {
        throw error;
      } else {
        return 'Browser mode - using localStorage';
      }
    }
  }

  // Fallback methods for browser environment
  private static getAsrConfigFromStorage(): AsrConfig | null {
    try {
      const config = localStorage.getItem('asr_config');
      if (config) {
        const parsed = JSON.parse(config);
        return {
          id: parsed.id || 'browser-fallback',
          service_provider: parsed.service_provider || 'cloud',
          local_endpoint: parsed.local_endpoint,
          local_api_key: parsed.local_api_key,
          cloud_endpoint: parsed.cloud_endpoint,
          cloud_api_key: parsed.cloud_api_key,
          created_at: parsed.created_at || new Date().toISOString(),
          updated_at: parsed.updated_at || new Date().toISOString(),
        };
      }
      return null;
    } catch (error) {
      console.error('Failed to get config from localStorage:', error);
      return null;
    }
  }

  private static saveAsrConfigToStorage(config: AsrConfigRequest): AsrConfig {
    try {
      const now = new Date().toISOString();
      const savedConfig = {
        id: 'browser-fallback',
        ...config,
        created_at: now,
        updated_at: now,
      };
      
      localStorage.setItem('asr_config', JSON.stringify(savedConfig));
      console.log('✅ Saved ASR config to localStorage');
      
      return {
        id: savedConfig.id,
        service_provider: savedConfig.service_provider,
        local_endpoint: savedConfig.local_endpoint,
        local_api_key: savedConfig.local_api_key,
        cloud_endpoint: savedConfig.cloud_endpoint,
        cloud_api_key: savedConfig.cloud_api_key,
        created_at: savedConfig.created_at,
        updated_at: savedConfig.updated_at,
      };
    } catch (error) {
      console.error('Failed to save config to localStorage:', error);
      throw error;
    }
  }
}

// Legacy localStorage migration utilities
export class StorageMigration {
  static async migrateFromLocalStorage(): Promise<void> {
    try {
      // Only run migration in Tauri environment
      if (!isTauriEnvironment()) {
        console.log('🌐 Browser mode - skipping localStorage migration');
        migrationChecked = true;
        return;
      }

      // Prevent repeated migration checks
      if (migrationChecked) {
        console.log('📦 Migration already checked, skipping');
        return;
      }

      // First check if we already have config in database
      const existingConfig = await DatabaseService.getAsrConfig();
      if (existingConfig) {
        console.log('Database config already exists, skipping migration');
        migrationChecked = true;
        return;
      }

      // Migrate ASR config from localStorage to database
      const localStorageServiceProvider = localStorage.getItem('asr_service_provider');
      const localStorageCloudApiKey = localStorage.getItem('asr_cloud_api_key');
      const localStorageLocalApiKey = localStorage.getItem('asr_local_api_key');
      const localStorageLocalEndpoint = localStorage.getItem('asr_local_endpoint');
      const localStorageCloudEndpoint = localStorage.getItem('asr_cloud_endpoint');

      if (localStorageServiceProvider || localStorageCloudApiKey || localStorageLocalApiKey) {
        const config: AsrConfigRequest = {
          service_provider: localStorageServiceProvider || 'cloud',
          local_endpoint: localStorageLocalEndpoint || undefined,
          local_api_key: localStorageLocalApiKey || undefined,
          cloud_endpoint: localStorageCloudEndpoint || undefined,
          cloud_api_key: localStorageCloudApiKey || undefined,
        };

        await DatabaseService.saveAsrConfig(config);
        console.log('Migrated ASR config from localStorage to database');

        // Clear localStorage after successful migration
        localStorage.removeItem('asr_service_provider');
        localStorage.removeItem('asr_cloud_api_key');
        localStorage.removeItem('asr_local_api_key');
        localStorage.removeItem('asr_local_endpoint');
        localStorage.removeItem('asr_cloud_endpoint');
      }
      
      migrationChecked = true;
    } catch (error) {
      console.error('Failed to migrate from localStorage:', error);
    }
  }
}