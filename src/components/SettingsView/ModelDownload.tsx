import React, { useState, useEffect } from 'react';
import { Download, Trash2, Play, Check, AlertCircle, Loader2, HardDrive } from 'lucide-react';
import { TauriService } from '../../services/tauriService';

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

export const ModelDownload: React.FC = () => {
  const [models, setModels] = useState<WhisperModel[]>([]);
  const [stats, setStats] = useState<ModelStats | null>(null);
  const [activeModel, setActiveModel] = useState<string | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  // Load models and stats
  useEffect(() => {
    loadModels();
    loadStats();
    loadActiveModel();
    
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

  const loadActiveModel = async () => {
    try {
      const activeModelPath = await TauriService.getActiveModelInfo();
      if (activeModelPath) {
        const active = models.find(m => m.file_path === activeModelPath);
        if (active) {
          setActiveModel(active.name);
        }
      }
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
      await TauriService.setActiveModel(modelName);
      setActiveModel(modelName);
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
        <span className="ml-2 text-gray-600">Loading models...</span>
      </div>
    );
  }

  return (
    <div className="space-y-6">
      {/* Header */}
      <div>
        <h3 className="text-lg font-semibold text-gray-900 mb-2">🎤 Whisper Models</h3>
        <p className="text-sm text-gray-600">
          Download and manage local Whisper models for offline speech recognition
        </p>
      </div>

      {/* Error Display */}
      {error && (
        <div className="flex items-center p-4 bg-red-50 border border-red-200 rounded-lg">
          <AlertCircle className="w-5 h-5 text-red-500 mr-2" />
          <span className="text-red-700 text-sm">{error}</span>
        </div>
      )}

      {/* Statistics */}
      {stats && (
        <div className="grid grid-cols-2 md:grid-cols-4 gap-4">
          <div className="bg-white p-4 rounded-lg border border-gray-200">
            <div className="flex items-center">
              <HardDrive className="w-5 h-5 text-blue-500 mr-2" />
              <div>
                <p className="text-xs text-gray-500">Total Models</p>
                <p className="text-lg font-semibold text-gray-900">
                  {stats.downloaded_models}/{stats.total_models}
                </p>
              </div>
            </div>
          </div>
          
          <div className="bg-white p-4 rounded-lg border border-gray-200">
            <div className="flex items-center">
              <Download className="w-5 h-5 text-green-500 mr-2" />
              <div>
                <p className="text-xs text-gray-500">Downloaded</p>
                <p className="text-lg font-semibold text-gray-900">
                  {formatSize(stats.downloaded_size_mb)}
                </p>
              </div>
            </div>
          </div>
          
          <div className="bg-white p-4 rounded-lg border border-gray-200">
            <div className="flex items-center">
              <Loader2 className="w-5 h-5 text-orange-500 mr-2" />
              <div>
                <p className="text-xs text-gray-500">Available</p>
                <p className="text-lg font-semibold text-gray-900">
                  {formatSize(stats.total_size_mb - stats.downloaded_size_mb)}
                </p>
              </div>
            </div>
          </div>
          
          <div className="bg-white p-4 rounded-lg border border-gray-200">
            <div className="flex items-center">
              <Play className="w-5 h-5 text-purple-500 mr-2" />
              <div>
                <p className="text-xs text-gray-500">Active</p>
                <p className="text-sm font-semibold text-gray-900">
                  {activeModel || 'None'}
                </p>
              </div>
            </div>
          </div>
        </div>
      )}

      {/* Models List */}
      <div className="space-y-4">
        {models.map((model) => (
          <div key={model.name} className="bg-white p-4 rounded-lg border border-gray-200">
            <div className="flex items-start justify-between">
              <div className="flex-1">
                <div className="flex items-center">
                  <h4 className="text-base font-semibold text-gray-900">{model.display_name}</h4>
                  {model.is_downloaded && (
                    <Check className="w-5 h-5 text-green-500 ml-2" />
                  )}
                  {activeModel === model.name && (
                    <span className="ml-2 px-2 py-1 text-xs bg-purple-100 text-purple-700 rounded-full">
                      Active
                    </span>
                  )}
                </div>
                
                <p className="text-sm text-gray-600 mt-1">{model.description}</p>
                <div className="flex items-center mt-2 text-xs text-gray-500">
                  <span className="font-medium">Size:</span> {formatSize(model.size_mb)}
                  <span className="mx-2">•</span>
                  <span className="font-medium">File:</span> {model.file_name}
                </div>

                {/* Download Progress */}
                {model.is_downloading && (
                  <div className="mt-3">
                    <div className="flex items-center justify-between text-sm mb-1">
                      <span className="text-gray-600">Downloading...</span>
                      <span className="text-gray-900 font-medium">{model.download_progress.toFixed(1)}%</span>
                    </div>
                    <div className="w-full bg-gray-200 rounded-full h-2">
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
                        Use
                      </button>
                    )}
                    <button
                      onClick={() => handleDelete(model.name)}
                      className="px-3 py-2 bg-red-500 text-white text-sm rounded-lg hover:bg-red-600 transition-colors flex items-center"
                    >
                      <Trash2 className="w-4 h-4 mr-1" />
                      Delete
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
                        Downloading...
                      </>
                    ) : (
                      <>
                        <Download className="w-4 h-4 mr-1" />
                        Download
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
        <div className="bg-gray-50 p-4 rounded-lg">
          <h4 className="text-sm font-medium text-gray-900 mb-1">Storage Location</h4>
          <p className="text-xs text-gray-600 font-mono">{stats.models_dir}</p>
        </div>
      )}
    </div>
  );
};