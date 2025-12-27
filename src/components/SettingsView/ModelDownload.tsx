import React, { useState, useEffect } from 'react';
import { Download, Trash2, Play, Check, AlertCircle, Loader2, HardDrive } from 'lucide-react';
import { TauriService } from '../../services/tauriService';
import { useLanguage } from '../../contexts/LanguageContext';

interface WhisperModel {
  name: string;
  display_name: string;
  file_name: string;
  size_mb: number;
  description: string;
  download_url: string;
  is_downloaded: boolean;
  file_path?: string;
  download_progress: number;
  is_downloading: boolean;
}

interface ModelStats {
  total_models: number;
  downloaded_models: number;
  total_size_mb: number;
  downloaded_size_mb: number;
  models_dir: string;
}

// Model descriptions mapping for i18n
const MODEL_DESCRIPTIONS: Record<string, { zh: string; en: string }> = {
  'large-v3-turbo': {
    zh: '最新的高效模型，在保持高准确性的同时显著提升推理速度，适合生产环境使用',
    en: 'The latest efficient model that significantly improves inference speed while maintaining high accuracy, suitable for production use'
  },
  'large-v2': {
    zh: '成熟稳定的模型，具有良好的准确性和兼容性',
    en: 'A mature and stable model with good accuracy and compatibility'
  }
};

export const ModelDownload: React.FC = () => {
  const { t, language } = useLanguage();

  // Helper function to get localized description
  const getModelDescription = (modelName: string, originalDesc: string): string => {
    const descMap = MODEL_DESCRIPTIONS[modelName];
    if (descMap) {
      return language === 'zh-CN' ? descMap.zh : descMap.en;
    }
    return originalDesc; // Fallback to original description
  };

  const [models, setModels] = useState<WhisperModel[]>([]);
  const [stats, setStats] = useState<ModelStats | null>(null);
  const [activeModel, setActiveModel] = useState<string | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  // Load models and stats
  useEffect(() => {
    loadModels();
    loadStats();
    
    // Set up event listeners for download progress
    const setupEventListeners = async () => {
      try {
        const { listen } = await import('@tauri-apps/api/event');
        
        // Download progress
        const unlistenProgress = await listen<{ model: string; progress: number }>(
          'model-download-progress',
          (event) => {
            const { model, progress } = event.payload;
            console.log(`📡 Frontend received download progress: ${model} = ${progress}%`);
            setModels(prev => prev.map(m =>
              m.name === model
                ? { ...m, download_progress: progress, is_downloading: progress < 100 }
                : m
            ));
          }
        );
        
        // Download complete
        const unlistenComplete = await listen<{ model: string; path: string }>(
          'model-download-complete',
          (event) => {
            const { model, path } = event.payload;
            console.log(`✅ Frontend received download complete: ${model} -> ${path}`);
            setModels(prev => prev.map(m =>
              m.name === model
                ? { ...m, is_downloaded: true, file_path: path, download_progress: 100, is_downloading: false }
                : m
            ));
            loadStats(); // Refresh stats
          }
        );
        
        // Download error
        const unlistenError = await listen<{ model: string; error: string }>(
          'model-download-error',
          (event) => {
            const { model, error } = event.payload;
            console.error(`❌ Frontend received download error: ${model} - ${error}`);
            setModels(prev => prev.map(m =>
              m.name === model
                ? { ...m, is_downloading: false, download_progress: 0 }
                : m
            ));
            setError(`Failed to download ${model}: ${error}`);
          }
        );
        
        return () => {
          unlistenProgress();
          unlistenComplete();
          unlistenError();
        };
      } catch (error) {
        console.error('Failed to set up event listeners:', error);
      }
    };
    
    const cleanup = setupEventListeners();
    return () => {
      cleanup.then(unlisten => unlisten?.());
    };
  }, []);

  const loadModels = async () => {
    try {
      setLoading(true);
      const modelsData = await TauriService.getAvailableModels();
      setModels(modelsData);
      
      // Load active model after models are loaded
      await loadActiveModel(modelsData);
    } catch (error) {
      console.error('Failed to load models:', error);
      setError('Failed to load models');
    } finally {
      setLoading(false);
    }
  };

  const loadStats = async () => {
    try {
      const statsData = await TauriService.getModelStats();
      setStats(statsData);
    } catch (error) {
      console.error('Failed to load stats:', error);
    }
  };

  const loadActiveModel = async (modelsList: any[]) => {
    try {
      console.log('🔍 Loading active model from ASR config...');
      
      // 1. Try to load from ASR config first (persistent storage)
      const { DatabaseService } = await import('../../services/database');
      const asrConfig = await DatabaseService.getAsrConfig();
      
      if (asrConfig?.whisper_model) {
        console.log('✅ Found model in ASR config:', asrConfig.whisper_model);
        const active = modelsList.find(m => m.name === asrConfig.whisper_model);
        if (active) {
          setActiveModel(active.name);
          console.log('✅ Active model set from ASR config:', active.name);
          return;
        } else {
          console.log('⚠️ Model from ASR config not found in models list:', asrConfig.whisper_model);
        }
      }
      
      // 2. Fallback to environment variable method
      console.log('⚠️ No valid model found in ASR config, checking environment variable...');
      const activeModelPath = await TauriService.getActiveModelInfo();
      if (activeModelPath) {
        const active = modelsList.find(m => m.file_path === activeModelPath);
        if (active) {
          setActiveModel(active.name);
          console.log('✅ Active model set from environment variable:', active.name);
          
          // Save to ASR config for future persistence
          const updatedConfig = {
            service_provider: asrConfig?.service_provider || 'local',
            local_endpoint: asrConfig?.local_endpoint,
            local_api_key: asrConfig?.local_api_key,
            cloud_endpoint: asrConfig?.cloud_endpoint,
            cloud_api_key: asrConfig?.cloud_api_key,
            whisper_model: active.name,
          };
          await DatabaseService.saveAsrConfig(updatedConfig);
          console.log('💾 Migrated model selection to ASR config');
          return;
        } else {
          console.log('⚠️ Model from environment not found in models list:', activeModelPath);
        }
      }
      
      console.log('ℹ️ No active model found, using default');
    } catch (error) {
      console.error('Failed to load active model:', error);
    }
  };

  const handleDownload = async (modelName: string) => {
    try {
      console.log(`🎯 Frontend: Starting download for model: ${modelName}`);
      setError(null);
      await TauriService.downloadModel(modelName);
      console.log(`✅ Frontend: Download command sent successfully for: ${modelName}`);
    } catch (error) {
      console.error('❌ Frontend: Failed to download model:', error);
      setError(`Failed to download ${modelName}: ${error}`);
      // Reset downloading state
      setModels(prev => prev.map(m =>
        m.name === modelName
          ? { ...m, is_downloading: false, download_progress: 0 }
          : m
      ));
    }
  };

  const handleDelete = async (modelName: string) => {
    if (!window.confirm(`Are you sure you want to delete the ${modelName} model?`)) {
      return;
    }
    
    try {
      await TauriService.deleteModel(modelName);
      setModels(prev => prev.map(m => 
        m.name === modelName 
          ? { ...m, is_downloaded: false, file_path: undefined, download_progress: 0 }
          : m
      ));
      
      if (activeModel === modelName) {
        setActiveModel(null);
      }
      
      loadStats(); // Refresh stats
    } catch (error) {
      console.error('Failed to delete model:', error);
      setError(`Failed to delete ${modelName}: ${error}`);
    }
  };

  const handleSetActive = async (modelName: string) => {
    try {
      console.log(`🎯 Setting active model: ${modelName}`);
      
      // 1. Set active model via environment variable (for compatibility)
      await TauriService.setActiveModel(modelName);
      
      // 2. Reload global WhisperRS processor with new model (primary method)
      console.log('🔄 Reloading global WhisperRS processor...');
      const { invoke } = await import('@tauri-apps/api/core');
      
      try {
        await invoke('reload_whisper_processor', { modelPath: `ggml-${modelName}.bin` });
        console.log('✅ Global WhisperRS processor reloaded successfully');
      } catch (reloadError) {
        console.warn('⚠️ Failed to reload global processor, continuing with environment variable method:', reloadError);
      }
      
      // 2. Save model selection to ASR config for persistence
      const { DatabaseService } = await import('../../services/database');
      
      // Get current ASR config to preserve other settings
      const currentConfig = await DatabaseService.getAsrConfig();
      
      // Create updated config with new model selection
      const updatedConfig = {
        service_provider: currentConfig?.service_provider || 'local',
        local_endpoint: currentConfig?.local_endpoint,
        local_api_key: currentConfig?.local_api_key,
        cloud_endpoint: currentConfig?.cloud_endpoint,
        cloud_api_key: currentConfig?.cloud_api_key,
        whisper_model: modelName, // NEW: Save selected model
      };
      
      console.log('💾 Saving model selection to ASR config:', updatedConfig);
      await DatabaseService.saveAsrConfig(updatedConfig);
      
      // Update local state
      setActiveModel(modelName);
      console.log('✅ Model selection and processor setup completed successfully');
      
    } catch (error) {
      console.error('Failed to set active model:', error);
      setError(`Failed to set active model: ${error}`);
    }
  };

  const formatSize = (sizeMb: number) => {
    if (sizeMb < 1024) {
      return `${sizeMb.toFixed(1)} MB`;
    }
    return `${(sizeMb / 1024).toFixed(1)} GB`;
  };

  if (loading) {
    return (
      <div className="flex items-center justify-center py-12">
        <Loader2 className="w-8 h-8 animate-spin text-blue-500" />
        <span className="ml-2 text-gray-600 dark:text-gray-400">{t.loadingModels}</span>
      </div>
    );
  }

  return (
    <div className="space-y-6">
      {/* Header */}
      <div>
        <h3 className="text-lg font-semibold text-gray-900 dark:text-gray-100 mb-2">{t.whisperModelsWithIcon}</h3>
        <p className="text-sm text-gray-600 dark:text-gray-400">
          {t.modelsDesc}
        </p>
      </div>

      {/* Error Display */}
      {error && (
        <div className="flex items-center p-4 bg-red-50 dark:bg-red-900/20 border border-red-200 dark:border-red-800 rounded-lg">
          <AlertCircle className="w-5 h-5 text-red-500 dark:text-red-400 mr-2" />
          <span className="text-red-700 dark:text-red-300 text-sm">{error}</span>
        </div>
      )}

      {/* Statistics */}
      {stats && (
        <div className="grid grid-cols-2 md:grid-cols-4 gap-4">
          <div className="bg-white dark:bg-dark-surface p-4 rounded-lg border border-gray-200 dark:border-dark-border">
            <div className="flex items-center">
              <HardDrive className="w-5 h-5 text-blue-500 mr-2" />
              <div>
                <p className="text-xs text-gray-500 dark:text-gray-400">{t.totalModels}</p>
                <p className="text-lg font-semibold text-gray-900 dark:text-gray-100">
                  {stats.downloaded_models}/{stats.total_models}
                </p>
              </div>
            </div>
          </div>

          <div className="bg-white dark:bg-dark-surface p-4 rounded-lg border border-gray-200 dark:border-dark-border">
            <div className="flex items-center">
              <Download className="w-5 h-5 text-green-500 mr-2" />
              <div>
                <p className="text-xs text-gray-500 dark:text-gray-400">{t.downloaded}</p>
                <p className="text-lg font-semibold text-gray-900 dark:text-gray-100">
                  {formatSize(stats.downloaded_size_mb)}
                </p>
              </div>
            </div>
          </div>

          <div className="bg-white dark:bg-dark-surface p-4 rounded-lg border border-gray-200 dark:border-dark-border">
            <div className="flex items-center">
              <Loader2 className="w-5 h-5 text-orange-500 mr-2" />
              <div>
                <p className="text-xs text-gray-500 dark:text-gray-400">{t.available}</p>
                <p className="text-lg font-semibold text-gray-900 dark:text-gray-100">
                  {formatSize(stats.total_size_mb - stats.downloaded_size_mb)}
                </p>
              </div>
            </div>
          </div>

          <div className="bg-white dark:bg-dark-surface p-4 rounded-lg border border-gray-200 dark:border-dark-border">
            <div className="flex items-center">
              <Play className="w-5 h-5 text-purple-500 mr-2" />
              <div>
                <p className="text-xs text-gray-500 dark:text-gray-400">{t.activeModel}</p>
                <p className="text-sm font-semibold text-gray-900 dark:text-gray-100">
                  {activeModel || t.none}
                </p>
              </div>
            </div>
          </div>
        </div>
      )}

      {/* Models List */}
      <div className="space-y-4">
        {models.map((model) => (
          <div key={model.name} className="bg-white dark:bg-dark-surface p-4 rounded-lg border border-gray-200 dark:border-dark-border">
            <div className="flex items-start justify-between">
              <div className="flex-1">
                <div className="flex items-center">
                  <h4 className="text-base font-semibold text-gray-900 dark:text-gray-100">{model.display_name}</h4>
                  {model.is_downloaded && (
                    <Check className="w-5 h-5 text-green-500 ml-2" />
                  )}
                  {activeModel === model.name && (
                    <span className="ml-2 px-2 py-1 text-xs bg-purple-100 dark:bg-purple-900/30 text-purple-700 dark:text-purple-300 rounded-full">
                      {t.activeModel}
                    </span>
                  )}
                </div>

                <p className="text-sm text-gray-600 dark:text-gray-400 mt-1">{getModelDescription(model.name, model.description)}</p>
                <div className="flex items-center mt-2 text-xs text-gray-500 dark:text-gray-400">
                  <span className="font-medium">{t.sizeLabel}</span> {formatSize(model.size_mb)}
                  <span className="mx-2">•</span>
                  <span className="font-medium">{t.fileLabel}</span> {model.file_name}
                </div>

                {/* Download Progress */}
                {model.is_downloading && (
                  <div className="mt-3">
                    <div className="flex items-center justify-between text-sm mb-1">
                      <span className="text-gray-600 dark:text-gray-400">{t.downloading}</span>
                      <span className="text-gray-900 dark:text-gray-100 font-medium">{model.download_progress.toFixed(1)}%</span>
                    </div>
                    <div className="w-full bg-gray-200 dark:bg-gray-700 rounded-full h-2">
                      <div
                        className="bg-blue-500 h-2 rounded-full transition-all duration-300"
                        style={{ width: `${model.download_progress}%` }}
                      />
                    </div>
                  </div>
                )}
              </div>

              {/* Action Buttons */}
              <div className="flex items-center ml-4 space-x-2">
                {model.is_downloaded ? (
                  <>
                    {activeModel !== model.name && (
                      <button
                        onClick={() => handleSetActive(model.name)}
                        className="px-3 py-2 bg-purple-500 text-white text-sm rounded-lg hover:bg-purple-600 transition-colors flex items-center"
                      >
                        <Play className="w-4 h-4 mr-1" />
                        {t.use}
                      </button>
                    )}
                    <button
                      onClick={() => handleDelete(model.name)}
                      className="px-3 py-2 bg-red-500 text-white text-sm rounded-lg hover:bg-red-600 transition-colors flex items-center"
                    >
                      <Trash2 className="w-4 h-4 mr-1" />
                      {t.delete}
                    </button>
                  </>
                ) : (
                  <button
                    onClick={() => handleDownload(model.name)}
                    disabled={model.is_downloading}
                    className="px-3 py-2 bg-blue-500 text-white text-sm rounded-lg hover:bg-blue-600 disabled:bg-gray-300 disabled:cursor-not-allowed transition-colors flex items-center"
                  >
                    {model.is_downloading ? (
                      <>
                        <Loader2 className="w-4 h-4 mr-1 animate-spin" />
                        {t.downloading}
                      </>
                    ) : (
                      <>
                        <Download className="w-4 h-4 mr-1" />
                        {t.download}
                      </>
                    )}
                  </button>
                )}
              </div>
            </div>
          </div>
        ))}
      </div>

      {/* Storage Location */}
      {stats && (
        <div className="bg-gray-50 dark:bg-slate-800 p-4 rounded-lg">
          <h4 className="text-sm font-medium text-gray-900 dark:text-gray-100 mb-1">{t.storageLocation}</h4>
          <p className="text-xs text-gray-600 dark:text-gray-400 font-mono">{stats.models_dir}</p>
        </div>
      )}
    </div>
  );
};