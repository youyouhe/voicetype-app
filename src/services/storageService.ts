import { invoke } from '@tauri-apps/api/core';

export class StorageService {
  private static isTauri(): boolean {
    return typeof window !== 'undefined' && '__TAURI__' in window;
  }

  static async getItem(key: string): Promise<string | null> {
    try {
      if (this.isTauri()) {
        // Try to use Tauri's persistent storage
        return await invoke('get_storage_item', { key });
      } else {
        // Fallback to localStorage for web development
        return localStorage.getItem(key);
      }
    } catch (error) {
      console.warn('Failed to get storage item:', error);
      // Fallback to localStorage
      return localStorage.getItem(key);
    }
  }

  static async setItem(key: string, value: string): Promise<void> {
    try {
      if (this.isTauri()) {
        // Try to use Tauri's persistent storage
        await invoke('set_storage_item', { key, value });
      } else {
        // Fallback to localStorage for web development
        localStorage.setItem(key, value);
      }
    } catch (error) {
      console.warn('Failed to set storage item:', error);
      // Fallback to localStorage
      localStorage.setItem(key, value);
    }
  }

  static async removeItem(key: string): Promise<void> {
    try {
      if (this.isTauri()) {
        // Try to use Tauri's persistent storage
        await invoke('remove_storage_item', { key });
      } else {
        // Fallback to localStorage for web development
        localStorage.removeItem(key);
      }
    } catch (error) {
      console.warn('Failed to remove storage item:', error);
      // Fallback to localStorage
      localStorage.removeItem(key);
    }
  }
}

// For synchronous operations (use with caution)
export class StorageServiceSync {
  static getItem(key: string): string | null {
    try {
      return localStorage.getItem(key);
    } catch (error) {
      console.warn('Failed to get localStorage item:', error);
      return null;
    }
  }

  static setItem(key: string, value: string): void {
    try {
      localStorage.setItem(key, value);
    } catch (error) {
      console.warn('Failed to set localStorage item:', error);
    }
  }

  static removeItem(key: string): void {
    try {
      localStorage.removeItem(key);
    } catch (error) {
      console.warn('Failed to remove localStorage item:', error);
    }
  }
}