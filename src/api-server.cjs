#!/usr/bin/env node

/**
 * API Bridge Server (CommonJS version)
 * This server provides HTTP endpoints that bridge to Tauri commands
 */

const express = require('express');
const cors = require('cors');

const app = express();
const PORT = process.env.API_PORT || 3001;

// Middleware
app.use(cors());
app.use(express.json());
app.use(express.urlencoded({ extended: true }));

// Mock database storage
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
    const now = new Date().toISOString();
    let config;

    // Simulate UPSERT behavior - update existing or insert new
    if (mockDb.asrConfig) {
      // Update existing config
      config = {
        ...mockDb.asrConfig,
        service_provider: req.body.service_provider,
        local_endpoint: req.body.local_endpoint,
        local_api_key: req.body.local_api_key,
        cloud_endpoint: req.body.cloud_endpoint,
        cloud_api_key: req.body.cloud_api_key,
        updated_at: now
      };
      console.log('🔄 Updated existing ASR config via API bridge');
    } else {
      // Insert new config
      config = {
        id: 'web-api-config',
        service_provider: req.body.service_provider,
        local_endpoint: req.body.local_endpoint,
        local_api_key: req.body.local_api_key,
        cloud_endpoint: req.body.cloud_endpoint,
        cloud_api_key: req.body.cloud_api_key,
        created_at: now,
        updated_at: now
      };
      console.log('➕ Created new ASR config via API bridge');
    }

    mockDb.asrConfig = config;

    console.log('📊 Config details:', {
      provider: config.service_provider,
      endpoint: config.local_endpoint,
      has_key: !!config.local_api_key,
      updated: config.updated_at
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