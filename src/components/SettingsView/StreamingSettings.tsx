import React, { useState, useEffect, useCallback } from 'react';
import { Waves, Zap, Clock, Volume2, Save, AlertCircle } from 'lucide-react';
import { Button } from '../ui/Button';
import { ToggleInput } from '../ui/Input';
import { TauriService, StreamingConfig } from '../../services/tauriService';
import { useLanguage } from '../../contexts/LanguageContext';

const defaultStreamingConfig: StreamingConfig = {
  enabled: false,
  chunk_interval_ms: 500,
  vad_threshold: 0.5,
  min_speech_duration_ms: 1000,
  min_silence_duration_ms: 2000,
  max_segment_length_ms: 30000,
};

export const StreamingSettings: React.FC = () => {
  const { t } = useLanguage();
  const [config, setConfig] = useState<StreamingConfig>(defaultStreamingConfig);
  const [isLoading, setIsLoading] = useState(true);
  const [isSaving, setIsSaving] = useState(false);
  const [saveMessage, setSaveMessage] = useState<{ type: 'success' | 'error'; text: string } | null>(null);

  // Load streaming config
  const loadConfig = useCallback(async () => {
    setIsLoading(true);
    try {
      const savedConfig = await TauriService.getStreamingConfig();
      if (savedConfig) {
        setConfig(savedConfig);
      }
    } catch (error) {
      console.error('Failed to load streaming config:', error);
    } finally {
      setIsLoading(false);
    }
  }, []);

  useEffect(() => {
    loadConfig();
  }, [loadConfig]);

  const handleSave = async () => {
    setIsSaving(true);
    setSaveMessage(null);
    try {
      await TauriService.saveStreamingConfig(config);
      setSaveMessage({ type: 'success', text: 'Streaming settings saved successfully!' });
      setTimeout(() => setSaveMessage(null), 3000);
    } catch (error) {
      console.error('Failed to save streaming config:', error);
      setSaveMessage({ type: 'error', text: `Failed to save: ${error}` });
    } finally {
      setIsSaving(false);
    }
  };

  const handleToggleStreaming = async (enabled: boolean) => {
    setConfig(prev => ({ ...prev, enabled }));
    try {
      await TauriService.toggleStreamingMode(enabled);
      setSaveMessage({ type: 'success', text: `Streaming ${enabled ? 'enabled' : 'disabled'}!` });
      setTimeout(() => setSaveMessage(null), 2000);
    } catch (error) {
      console.error('Failed to toggle streaming:', error);
      setSaveMessage({ type: 'error', text: `Failed to toggle: ${error}` });
      // Revert on error
      setConfig(prev => ({ ...prev, enabled: !enabled }));
    }
  };

  if (isLoading) {
    return (
      <div className="flex items-center justify-center py-12">
        <div className="animate-spin rounded-full h-8 w-8 border-b-2 border-primary-500"></div>
      </div>
    );
  }

  return (
    <div className="space-y-6">
      {/* Header */}
      <div className="flex items-center justify-between">
        <div>
          <h2 className="text-2xl font-bold text-gray-900 dark:text-gray-100 flex items-center gap-2">
            <Waves className="w-6 h-6 text-primary-500" />
            Streaming Settings
          </h2>
          <p className="text-gray-500 dark:text-gray-400 mt-1">
            Configure real-time streaming ASR with VAD (Voice Activity Detection)
          </p>
        </div>
        <Button
          onClick={handleSave}
          disabled={isSaving}
          className="flex items-center gap-2"
        >
          <Save className="w-4 h-4" />
          {isSaving ? 'Saving...' : 'Save Settings'}
        </Button>
      </div>

      {/* Save Message */}
      {saveMessage && (
        <div className={`
          flex items-center gap-2 px-4 py-3 rounded-lg
          ${saveMessage.type === 'success'
            ? 'bg-green-50 dark:bg-green-900/30 text-green-700 dark:text-green-300 border border-green-200 dark:border-green-800'
            : 'bg-red-50 dark:bg-red-900/30 text-red-700 dark:text-red-300 border border-red-200 dark:border-red-800'}
        `}>
          {saveMessage.type === 'success' ? (
            <Save className="w-4 h-4" />
          ) : (
            <AlertCircle className="w-4 h-4" />
          )}
          <span className="text-sm font-medium">{saveMessage.text}</span>
        </div>
      )}

      {/* Streaming Toggle */}
      <div className="bg-gray-50 dark:bg-dark-bg rounded-xl p-6 border border-gray-200 dark:border-dark-border">
        <div className="flex items-center justify-between">
          <div className="flex items-center gap-3">
            <div className="p-2 bg-primary-100 dark:bg-primary-900/50 rounded-lg">
              <Waves className="w-5 h-5 text-primary-600 dark:text-primary-400" />
            </div>
            <div>
              <h3 className="font-semibold text-gray-900 dark:text-gray-100">
                Enable Streaming Mode
              </h3>
              <p className="text-sm text-gray-500 dark:text-gray-400">
                Transcribe speech in real-time while recording
              </p>
            </div>
          </div>
          <ToggleInput
            checked={config.enabled}
            onChange={handleToggleStreaming}
            disabled={isSaving}
          />
        </div>

        {config.enabled && (
          <div className="mt-4 p-4 bg-blue-50 dark:bg-blue-900/30 rounded-lg border border-blue-200 dark:border-blue-800">
            <p className="text-sm text-blue-700 dark:text-blue-300">
              <strong>Note:</strong> Streaming mode will transcribe audio as you speak. Text will appear
              at the cursor position in real-time. Make sure your cursor is in a text input field before
              pressing the hotkey.
            </p>
          </div>
        )}
      </div>

      {/* VAD Settings */}
      <div className="bg-gray-50 dark:bg-dark-bg rounded-xl p-6 border border-gray-200 dark:border-dark-border">
        <h3 className="font-semibold text-gray-900 dark:text-gray-100 flex items-center gap-2 mb-4">
          <Zap className="w-5 h-5 text-primary-500" />
          VAD (Voice Activity Detection) Settings
        </h3>
        <p className="text-sm text-gray-500 dark:text-gray-400 mb-6">
          Configure how the system detects speech segments
        </p>

        {/* VAD Threshold */}
        <div className="space-y-4">
          <div>
            <label className="flex items-center justify-between text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
              <span>VAD Threshold (Energy Level)</span>
              <span className="text-primary-600 dark:text-primary-400 font-mono">
                {config.vad_threshold.toFixed(2)}
              </span>
            </label>
            <input
              type="range"
              min="0.01"
              max="1.0"
              step="0.01"
              value={config.vad_threshold}
              onChange={(e) => setConfig(prev => ({ ...prev, vad_threshold: parseFloat(e.target.value) }))}
              className="w-full h-2 bg-gray-200 dark:bg-gray-700 rounded-lg appearance-none cursor-pointer accent-primary-500"
              disabled={isSaving}
            />
            <div className="flex justify-between text-xs text-gray-500 dark:text-gray-400 mt-1">
              <span>More sensitive (0.01)</span>
              <span>Less sensitive (1.0)</span>
            </div>
          </div>

          {/* Min Speech Duration */}
          <div>
            <label className="flex items-center justify-between text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
              <span className="flex items-center gap-1">
                <Clock className="w-4 h-4" />
                Minimum Speech Duration
              </span>
              <span className="text-primary-600 dark:text-primary-400 font-mono">
                {config.min_speech_duration_ms} ms
              </span>
            </label>
            <input
              type="range"
              min="100"
              max="5000"
              step="100"
              value={config.min_speech_duration_ms}
              onChange={(e) => setConfig(prev => ({ ...prev, min_speech_duration_ms: parseInt(e.target.value) }))}
              className="w-full h-2 bg-gray-200 dark:bg-gray-700 rounded-lg appearance-none cursor-pointer accent-primary-500"
              disabled={isSaving}
            />
            <div className="flex justify-between text-xs text-gray-500 dark:text-gray-400 mt-1">
              <span>100ms</span>
              <span>5000ms</span>
            </div>
          </div>

          {/* Min Silence Duration */}
          <div>
            <label className="flex items-center justify-between text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
              <span className="flex items-center gap-1">
                <Volume2 className="w-4 h-4" />
                Minimum Silence Duration
              </span>
              <span className="text-primary-600 dark:text-primary-400 font-mono">
                {config.min_silence_duration_ms} ms
              </span>
            </label>
            <input
              type="range"
              min="100"
              max="5000"
              step="100"
              value={config.min_silence_duration_ms}
              onChange={(e) => setConfig(prev => ({ ...prev, min_silence_duration_ms: parseInt(e.target.value) }))}
              className="w-full h-2 bg-gray-200 dark:bg-gray-700 rounded-lg appearance-none cursor-pointer accent-primary-500"
              disabled={isSaving}
            />
            <div className="flex justify-between text-xs text-gray-500 dark:text-gray-400 mt-1">
              <span>100ms</span>
              <span>5000ms</span>
            </div>
          </div>
        </div>
      </div>

      {/* Advanced Settings */}
      <div className="bg-gray-50 dark:bg-dark-bg rounded-xl p-6 border border-gray-200 dark:border-dark-border">
        <h3 className="font-semibold text-gray-900 dark:text-gray-100 mb-4">
          Advanced Settings
        </h3>

        <div className="space-y-4">
          {/* Chunk Interval */}
          <div>
            <label className="flex items-center justify-between text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
              <span>Processing Chunk Interval</span>
              <span className="text-primary-600 dark:text-primary-400 font-mono">
                {config.chunk_interval_ms} ms
              </span>
            </label>
            <input
              type="range"
              min="100"
              max="2000"
              step="50"
              value={config.chunk_interval_ms}
              onChange={(e) => setConfig(prev => ({ ...prev, chunk_interval_ms: parseInt(e.target.value) }))}
              className="w-full h-2 bg-gray-200 dark:bg-gray-700 rounded-lg appearance-none cursor-pointer accent-primary-500"
              disabled={isSaving}
            />
            <div className="flex justify-between text-xs text-gray-500 dark:text-gray-400 mt-1">
              <span>100ms (Faster, more CPU)</span>
              <span>2000ms (Slower, less CPU)</span>
            </div>
          </div>

          {/* Max Segment Length */}
          <div>
            <label className="flex items-center justify-between text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
              <span>Maximum Segment Length</span>
              <span className="text-primary-600 dark:text-primary-400 font-mono">
                {config.max_segment_length_ms} ms
              </span>
            </label>
            <input
              type="range"
              min="5000"
              max="60000"
              step="1000"
              value={config.max_segment_length_ms}
              onChange={(e) => setConfig(prev => ({ ...prev, max_segment_length_ms: parseInt(e.target.value) }))}
              className="w-full h-2 bg-gray-200 dark:bg-gray-700 rounded-lg appearance-none cursor-pointer accent-primary-500"
              disabled={isSaving}
            />
            <div className="flex justify-between text-xs text-gray-500 dark:text-gray-400 mt-1">
              <span>5s</span>
              <span>60s</span>
            </div>
          </div>
        </div>
      </div>

      {/* Info Box */}
      <div className="bg-yellow-50 dark:bg-yellow-900/20 rounded-xl p-4 border border-yellow-200 dark:border-yellow-800">
        <div className="flex items-start gap-3">
          <AlertCircle className="w-5 h-5 text-yellow-600 dark:text-yellow-400 flex-shrink-0 mt-0.5" />
          <div className="text-sm text-yellow-700 dark:text-yellow-300">
            <p className="font-medium mb-1">Streaming Mode Notes:</p>
            <ul className="list-disc list-inside space-y-1 text-xs">
              <li>Streaming mode requires Whisper model to be loaded</li>
              <li>VAD threshold controls speech sensitivity - lower = more sensitive</li>
              <li>Min speech/silence duration helps avoid false triggers</li>
              <li>Text appears at cursor position as you speak</li>
              <li>Release hotkey to finalize the transcription</li>
            </ul>
          </div>
        </div>
      </div>
    </div>
  );
};
