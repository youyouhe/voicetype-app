// API Bridge for Tauri Commands
// This file provides HTTP API endpoints that bridge to Tauri commands

import { invoke } from '@tauri-apps/api/core';

// Check if we're running in Tauri environment
const isTauriEnvironment = () => {
  return typeof window !== 'undefined' && window.__TAURI_INTERNALS__;
};

// ASR Config API endpoints
export const asrConfigApi = {
  // GET /api/asr/config - Get ASR configuration
  async getConfig(req, res) {
    try {
      if (!isTauriEnvironment()) {
        return res.status(503).json({ error: 'Tauri environment not available' });
      }

      const config = await invoke('get_asr_config');
      if (config) {
        res.json(config);
      } else {
        res.status(404).json({ error: 'Configuration not found' });
      }
    } catch (error) {
      console.error('Failed to get ASR config:', error);
      res.status(500).json({ error: error.message });
    }
  },

  // POST /api/asr/config - Save ASR configuration
  async saveConfig(req, res) {
    try {
      if (!isTauriEnvironment()) {
        return res.status(503).json({ error: 'Tauri environment not available' });
      }

      const config = req.body;
      const result = await invoke('save_asr_config', { request: config });
      res.json(result);
    } catch (error) {
      console.error('Failed to save ASR config:', error);
      res.status(500).json({ error: error.message });
    }
  }
};

// Translation Config API endpoints
export const translationConfigApi = {
  async getConfig(req, res) {
    try {
      if (!isTauriEnvironment()) {
        return res.status(503).json({ error: 'Tauri environment not available' });
      }

      const { provider } = req.params;
      const config = await invoke('get_translation_config', { provider });
      if (config) {
        res.json(config);
      } else {
        res.status(404).json({ error: 'Translation configuration not found' });
      }
    } catch (error) {
      console.error('Failed to get translation config:', error);
      res.status(500).json({ error: error.message });
    }
  },

  async saveConfig(req, res) {
    try {
      if (!isTauriEnvironment()) {
        return res.status(503).json({ error: 'Tauri environment not available' });
      }

      const config = req.body;
      const result = await invoke('save_translation_config', { request: config });
      res.json(result);
    } catch (error) {
      console.error('Failed to save translation config:', error);
      res.status(500).json({ error: error.message });
    }
  }
};

// History Records API endpoints
export const historyApi = {
  async getRecords(req, res) {
    try {
      if (!isTauriEnvironment()) {
        return res.status(503).json({ error: 'Tauri environment not available' });
      }

      const { limit, recordType } = req.query;
      const records = await invoke('get_history_records', {
        limit: limit ? parseInt(limit) : undefined,
        recordType: recordType || undefined
      });
      res.json(records);
    } catch (error) {
      console.error('Failed to get history records:', error);
      res.status(500).json({ error: error.message });
    }
  },

  async addRecord(req, res) {
    try {
      if (!isTauriEnvironment()) {
        return res.status(503).json({ error: 'Tauri environment not available' });
      }

      const record = req.body;
      const result = await invoke('add_history_record', { request: record });
      res.json(result);
    } catch (error) {
      console.error('Failed to add history record:', error);
      res.status(500).json({ error: error.message });
    }
  },

  async getStats(req, res) {
    try {
      if (!isTauriEnvironment()) {
        return res.status(503).json({ error: 'Tauri environment not available' });
      }

      const stats = await invoke('get_history_stats');
      res.json({
        total: stats[0],
        successful: stats[1],
        transcribeCount: stats[2]
      });
    } catch (error) {
      console.error('Failed to get history stats:', error);
      res.status(500).json({ error: error.message });
    }
  }
};

// Export all API handlers
export default {
  asrConfig: asrConfigApi,
  translationConfig: translationConfigApi,
  history: historyApi
};