// Global configuration manager to prevent duplicate loading
import { DatabaseService, AsrConfig } from './database';

interface ConfigManager {
  isLoading: boolean;
  lastLoadTime: number;
  promise: Promise<AsrConfig | null> | null;
}

class ConfigurationManager {
  private manager: ConfigManager = {
    isLoading: false,
    lastLoadTime: 0,
    promise: null
  };

  private readonly CACHE_DURATION = 5000; // 5 seconds cache

  async loadConfiguration(): Promise<AsrConfig | null> {
    const now = Date.now();
    
    // If we have a recent load result, return it
    if (!this.manager.isLoading && (now - this.manager.lastLoadTime) < this.CACHE_DURATION) {
      console.log('📦 Using cached config from:', new Date(this.manager.lastLoadTime));
      return this.manager.promise ? await this.manager.promise : null;
    }

    // If currently loading, wait for the existing promise
    if (this.manager.isLoading && this.manager.promise) {
      console.log('⏳ Config already loading, waiting for existing promise');
      return await this.manager.promise;
    }

    // Start new loading process
    console.log('🔄 Starting new config load');
    this.manager.isLoading = true;
    
    this.manager.promise = DatabaseService.getAsrConfig()
      .finally(() => {
        this.manager.isLoading = false;
        this.manager.lastLoadTime = Date.now();
      });

    return await this.manager.promise;
  }

  // Method to force reload if needed
  async forceReload(): Promise<AsrConfig | null> {
    this.manager.lastLoadTime = 0; // Reset cache
    this.manager.promise = null;
    return this.loadConfiguration();
  }
}

export const configManager = new ConfigurationManager();