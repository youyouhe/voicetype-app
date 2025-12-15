#!/usr/bin/env node

/**
 * API Bridge Server
 * This server provides HTTP endpoints that bridge to Tauri commands
 * It allows the web frontend to communicate with the Tauri backend
 */

import express from 'express';
import cors from 'cors';

const app = express();
const PORT = process.env.API_PORT || 3001;

// Middleware
app.use(cors());
app.use(express.json());
app.use(express.urlencoded({ extended: true }));

// Mock database storage (in production, this would be SQLite)
const mockDb = {
  asrConfig: null,
  translationConfigs: {},
  historyRecords: []
};

// ASR Config endpoints
app.get('/api/asr/config', (req, res) => {
  console.log('GET /api/asr/config');
  try {
    if (mockDb.asrConfig) {
      res.json(mockDb.asrConfig);
    } else {
      res.status(404).json({ error: 'Configuration not found' });
    }
  } catch (error) {
    console.error('Error getting ASR config:', error);
    res.status(500).json({ error: error.message });
  }
});

app.post('/api/asr/config', (req, res) => {
  console.log('POST /api/asr/config', req.body);
  try {
    const config = {
      id: 'web-api-config',
      service_provider: req.body.service_provider,
      local_endpoint: req.body.local_endpoint,
      local_api_key: req.body.local_api_key,
      cloud_endpoint: req.body.cloud_endpoint,
      cloud_api_key: req.body.cloud_api_key,
      created_at: new Date().toISOString(),
      updated_at: new Date().toISOString()
    };

    mockDb.asrConfig = config;

    console.log('✅ Saved ASR config via API bridge');
    console.log('📊 Config details:', {
      provider: config.service_provider,
      endpoint: config.local_endpoint,
      has_key: !!config.local_api_key
    });

    res.json(config);
  } catch (error) {
    console.error('Error saving ASR config:', error);
    res.status(500).json({ error: error.message });
  }
});

// Health check endpoint
app.get('/api/health', (req, res) => {
  res.json({
    status: 'ok',
    message: 'API Bridge Server is running',
    timestamp: new Date().toISOString()
  });
});

// Start server
app.listen(PORT, () => {
  console.log(`🌐 API Bridge Server running on http://localhost:${PORT}`);
  console.log(`📊 Health check: http://localhost:${PORT}/api/health`);
  console.log(`💾 ASR config API: http://localhost:${PORT}/api/asr/config`);
});

// Handle graceful shutdown
process.on('SIGINT', () => {
  console.log('\n🛑 Shutting down API Bridge Server...');
  process.exit(0);
});

process.on('SIGTERM', () => {
  console.log('\n🛑 Shutting down API Bridge Server...');
  process.exit(0);
});