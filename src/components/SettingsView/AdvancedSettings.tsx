import React, { useState, useEffect } from 'react';
import { Mic, Settings, RefreshCw, Volume2, Wand2, Check, AlertCircle } from 'lucide-react';
import { invoke } from '@tauri-apps/api/core';
import { ToggleInput } from '../ui/Input';
import { Input } from '../ui/Input';
import { HotkeyConfig, PostProcessConfig } from '../../types';
import { TauriService } from '../../services/tauriService';
import { useLanguage } from '../../contexts/LanguageContext';

interface AudioDevice {
  id: string;
  name: string;
  is_default: boolean;
}

export const AdvancedSettings: React.FC = () => {
  const { t } = useLanguage();

  const [micDevices, setMicDevices] = useState<AudioDevice[]>([]);
  const [selectedMic, setSelectedMic] = useState<string>('');
  const [isLoading, setIsLoading] = useState(false);
  const [isTauri, setIsTauri] = useState(false);

  // WAV Files Settings
  const [saveWavFiles, setSaveWavFiles] = useState(true);
  const [hasLoadedFromDatabase, setHasLoadedFromDatabase] = useState(false);

  // Post-process Settings (AI text correction)
  const defaultPostProcessConfig: PostProcessConfig = {
    enabled: false,
    provider: 'ollama',
    endpoint: 'http://localhost:11434/api/chat',
    api_key: '',
    model: 'llama3.2:latest',
    system_prompt: 'You are a text correction assistant. Fix grammar, spelling, and punctuation errors in the user input. Return only the corrected text without explanation.',
    timeout_seconds: 30,
  };
  const [postProcessConfig, setPostProcessConfig] = useState<PostProcessConfig>(defaultPostProcessConfig);
  const [hasLoadedPostProcessConfig, setHasLoadedPostProcessConfig] = useState(false);
  const [postProcessSaveStatus, setPostProcessSaveStatus] = useState<'idle' | 'saving' | 'success' | 'error'>('idle');
  const [postProcessSaveError, setPostProcessSaveError] = useState<string>('');

  // Check if we're running in Tauri
  const checkTauriEnvironment = () => {
    const inTauri = typeof window !== 'undefined' && '__TAURI__' in window;
    setIsTauri(inTauri);
    return inTauri;
  };

  // Load available microphone devices
  const loadMicrophones = async () => {
    setIsLoading(true);
    try {
      const inTauri = checkTauriEnvironment();

      if (inTauri) {
        // Use Tauri backend to get audio devices
        console.log('Using Tauri backend for audio device detection');
        const devices = await invoke<AudioDevice[]>('get_audio_devices');
        console.log('Tauri audio devices:', devices);

        const mics = devices.map(device => ({
          id: device.id,
          name: device.name,
          is_default: device.is_default
        }));

        setMicDevices(mics);

        // Load saved preference
        const saved = localStorage.getItem('selected-microphone');
        console.log('Saved microphone:', saved);

        if (saved && mics.find(m => m.id === saved)) {
          console.log('Using saved microphone:', saved);
          setSelectedMic(saved);
        } else if (mics.length > 0) {
          // Select default or first microphone
          const defaultMic = mics.find(m => m.is_default) || mics[0];
          console.log('Auto-selecting microphone:', defaultMic);
          setSelectedMic(defaultMic.id);
        }
      } else {
        // Use WebRTC for web development
        console.log('Using WebRTC for audio device detection');

        try {
          const devices = await navigator.mediaDevices.enumerateDevices();
          console.log('All audio devices:', devices);

          const mics = devices
            .filter(device => device.kind === 'audioinput')
            .map(device => ({
              id: device.deviceId,
              name: device.label || `Microphone ${device.deviceId.slice(0, 8)}...`,
              is_default: device.deviceId === 'default' || device.label?.includes('default')
            }));

          console.log('Microphone devices found:', mics);
          setMicDevices(mics);

          // Load saved preference
          const saved = localStorage.getItem('selected-microphone');

          if (saved && mics.find(m => m.id === saved)) {
            setSelectedMic(saved);
          } else if (mics.length > 0) {
            const defaultMic = mics[0];
            setSelectedMic(defaultMic.id);
          }
        } catch (error) {
          console.warn('WebRTC device enumeration failed:', error);
          // Create fallback mock devices for development
          const fallbackDevices: AudioDevice[] = [
            {
              id: 'default',
              name: 'Default Microphone (Development)',
              is_default: true
            }
          ];
          setMicDevices(fallbackDevices);
          setSelectedMic('default');
        }
      }
    } catch (error) {
      console.error('Failed to load microphones:', error);
      // Create fallback devices if everything fails
      const fallbackDevices: AudioDevice[] = [
        {
          id: 'default',
          name: isTauri ? 'Default System Microphone' : 'Default Microphone (Development)',
          is_default: true
        }
      ];
      setMicDevices(fallbackDevices);
      setSelectedMic('default');
    } finally {
      setIsLoading(false);
    }
  };

  // Handle microphone selection change
  const handleMicChange = (micId: string) => {
    console.log('Microphone selection changed to:', micId);
    setSelectedMic(micId);
    localStorage.setItem('selected-microphone', micId);
  };

  // Test microphone
  const testMicrophone = async () => {
    if (!selectedMic) return;

    try {
      if (isTauri) {
        // Use Tauri backend to test microphone
        const success = await invoke<boolean>('test_microphone', { deviceId: selectedMic });
        if (success) {
          alert(t.micTestSuccess);
        } else {
          alert(t.micTestFailed);
        }
      } else {
        // Use WebRTC for web development
        const constraints = {
          audio: selectedMic === 'default' ? true : { deviceId: { exact: selectedMic } }
        };

        const stream = await navigator.mediaDevices.getUserMedia(constraints);

        // Simple visual feedback
        const audioContext = new AudioContext();
        const analyser = audioContext.createAnalyser();
        const source = audioContext.createMediaStreamSource(stream);
        source.connect(analyser);

        // Check if we're getting audio data
        const dataArray = new Uint8Array(analyser.frequencyBinCount);
        analyser.getByteFrequencyData(dataArray);

        // Clean up
        stream.getTracks().forEach(track => track.stop());
        audioContext.close();

        alert(t.micTestSuccess);
      }
    } catch (error) {
      console.error('Microphone test failed:', error);
      alert(t.micTestFailed);
    }
  };

  // Load microphones on component mount
  useEffect(() => {
    loadMicrophones();
  }, []);

  // Load hotkey configuration from database on component mount
  useEffect(() => {
    const loadHotkeyConfiguration = async () => {
      if (isTauri && hasLoadedFromDatabase === false) {
        try {
          console.log('📥 Loading hotkey config from database...');
          const config = await invoke<HotkeyConfig | null>('get_hotkey_config');
          console.log('📋 Raw config from database:', config);

          if (config) {
            console.log('📥 Loaded hotkey config from database:');
            console.log('  - save_wav_files:', config.save_wav_files);
            console.log('🔄 Setting state with loaded values...');
            setSaveWavFiles(config.save_wav_files);
            console.log('✅ State updated with save_wav_files:', config.save_wav_files);
          } else {
            console.log('📥 No hotkey config found, using defaults');
          }
        } catch (error) {
          console.error('❌ Failed to load hotkey configuration:', error);
          console.error('❌ Error details:', JSON.stringify(error, null, 2));
        }
        console.log('🔔 Setting hasLoadedFromDatabase to true');
        setHasLoadedFromDatabase(true);
      } else {
        console.log('⏸️ Skipping config load - isTauri:', isTauri, ', hasLoadedFromDatabase:', hasLoadedFromDatabase);
      }
    };

    console.log('🚀 Calling loadHotkeyConfiguration...');
    loadHotkeyConfiguration();
  }, [isTauri]); // Removed hasLoadedFromDatabase from dependencies to prevent infinite loop

  // Handle saveWavFiles toggle with immediate save
  const handleSaveWavFilesToggle = async (enabled: boolean) => {
    // Always update the UI state first
    setSaveWavFiles(enabled);

    // Only try to save in Tauri environment
    if (!isTauri) {
      return;
    }

    try {
      // First, load existing configuration to preserve user settings
      const existingConfig = await invoke<HotkeyConfig | null>('get_hotkey_config');

      const config = {
        transcribe_key: existingConfig?.transcribe_key || 'F4',
        translate_key: existingConfig?.translate_key || 'Shift + F4',
        trigger_delay_ms: existingConfig?.trigger_delay_ms || 300,
        anti_mistouch_enabled: existingConfig?.anti_mistouch_enabled ?? true,
        save_wav_files: enabled,
        typing_delays: existingConfig?.typing_delays || {
          clipboard_update_ms: 100,
          keyboard_events_settle_ms: 300,
          typing_complete_ms: 500,
          character_interval_ms: 100,
          short_operation_ms: 100,
        },
      };

      await invoke('save_hotkey_config', { request: config });
    } catch (_error) {
      // Error - save failed, but don't revert the UI state
      // The user can try again by toggling
    }
  };

  // Load post-process config from database on component mount
  useEffect(() => {
    const loadPostProcessConfig = async () => {
      if (!isTauri) {
        // Not in Tauri environment, skip
        return;
      }

      // Always load when in Tauri environment, even if already loaded
      // This ensures we have the latest config when isTauri changes from false to true
      try {
        console.log('📥 Loading post-process config from database... isTauri:', isTauri, 'hasLoaded:', hasLoadedPostProcessConfig);
        const config = await TauriService.getPostProcessConfig();
        if (config) {
          console.log('📥 Loaded post-process config from database:', config);
          setPostProcessConfig(config);
        } else {
          console.log('📥 No post-process config found, using defaults');
        }
      } catch (error) {
        console.error('❌ Failed to load post-process configuration:', error);
      }
      setHasLoadedPostProcessConfig(true);
    };
    loadPostProcessConfig();
  }, [isTauri]);

  // Save post-process config when it changes
  useEffect(() => {
    console.log('🔍 Post-process config useEffect triggered:', {
      hasLoadedPostProcessConfig,
      isTauri,
      enabled: postProcessConfig.enabled,
      provider: postProcessConfig.provider,
    });

    // Skip during initialization and if not in Tauri
    if (!hasLoadedPostProcessConfig || !isTauri) {
      console.log('⏭️ Skipping save - hasLoadedPostProcessConfig:', hasLoadedPostProcessConfig, ', isTauri:', isTauri);
      return;
    }

    console.log('💾 Saving post-process config...');
    console.log('📤 Data being sent:', JSON.stringify(postProcessConfig, null, 2));
    setPostProcessSaveStatus('saving');
    setPostProcessSaveError('');
    TauriService.savePostProcessConfig(postProcessConfig)
      .then((result) => {
        console.log('✅ Post-process config saved successfully!');
        console.log('📥 Result:', result);
        setPostProcessSaveStatus('success');
        setPostProcessSaveError('');
        setTimeout(() => setPostProcessSaveStatus('idle'), 2000);
      })
      .catch((error) => {
        const errorMsg = String(error);
        console.error('❌ Failed to save post-process config. Error:', error);
        console.error('❌ Error type:', typeof error);
        console.error('❌ Error message:', errorMsg);
        setPostProcessSaveStatus('error');
        setPostProcessSaveError(errorMsg);
        setTimeout(() => setPostProcessSaveStatus('idle'), 5000);
      });
  }, [postProcessConfig, hasLoadedPostProcessConfig, isTauri]);

  return (
    <div className="space-y-6">
      <div>
        <h2 className="text-2xl font-bold text-gray-900 dark:text-dark-text mb-2 flex items-center">
          <Settings className="w-6 h-6 mr-2" />
          {t.advancedSettingsFullTitle}
        </h2>
        <p className="text-gray-600 dark:text-dark-muted">
          {t.configureAdvancedSettings}
        </p>
        {isTauri && (
          <p className="text-xs text-green-600 dark:text-green-400 mt-1">
            {t.runningInTauriNative}
          </p>
        )}
      </div>

      {/* Microphone Settings */}
      <div className="border-t border-gray-200 dark:border-dark-border pt-6">
        <div className="flex items-center justify-between mb-4">
          <h3 className="text-lg font-semibold text-gray-900 dark:text-dark-text flex items-center">
            <Mic className="w-5 h-5 mr-2" />
            {t.audioInputDevice}
          </h3>
          <button
            onClick={loadMicrophones}
            disabled={isLoading}
            className="flex items-center px-3 py-1 text-sm bg-gray-100 dark:bg-dark-bg hover:bg-gray-200 dark:hover:bg-dark-border rounded-md transition-colors"
          >
            <RefreshCw className={`w-4 h-4 mr-1 ${isLoading ? 'animate-spin' : ''}`} />
            {t.refresh}
          </button>
        </div>

        {micDevices.length > 0 ? (
          <div className="space-y-4">
            <div>
              <label className="block text-sm font-medium text-gray-700 dark:text-dark-text mb-2">
                {t.selectMicrophone}
              </label>
              <select
                value={selectedMic || ''}
                onChange={(e) => handleMicChange(e.target.value)}
                className="w-full px-3 py-2 border border-gray-300 dark:border-dark-border bg-white dark:bg-dark-surface text-gray-900 dark:text-dark-text rounded-lg focus:ring-2 focus:ring-primary-500 focus:border-primary-500"
              >
                <option value="" disabled>
                  {t.selectAMicrophone}
                </option>
                {micDevices.map((mic) => (
                  <option key={mic.id} value={mic.id}>
                    {mic.name} {mic.is_default && t.default}
                  </option>
                ))}
              </select>
              {selectedMic && (
                <p className="mt-1 text-xs text-gray-500 dark:text-dark-muted">
                  {t.selected} {micDevices.find(m => m.id === selectedMic)?.name}
                </p>
              )}
            </div>

            <div className="flex items-center justify-between p-4 bg-gray-50 dark:bg-dark-bg rounded-lg border border-gray-200 dark:border-dark-border">
              <div className="flex items-center">
                <Volume2 className="w-5 h-5 text-gray-400 dark:text-dark-muted mr-3" />
                <div>
                  <p className="text-sm font-medium text-gray-900 dark:text-dark-text">
                    {t.currentSelection}
                  </p>
                  <p className="text-xs text-gray-500 dark:text-dark-muted">
                    {micDevices.find(m => m.id === selectedMic)?.name || t.noMicrophoneSelected}
                  </p>
                </div>
              </div>
              <button
                onClick={testMicrophone}
                disabled={!selectedMic}
                className="px-3 py-1 text-sm bg-primary-500 hover:bg-primary-600 text-white rounded-md transition-colors disabled:opacity-50 disabled:cursor-not-allowed"
              >
                {t.testMicrophone}
              </button>
            </div>

            <div className="text-xs text-gray-500 dark:text-dark-muted">
              <p>{t.selectMicrophoneDesc}</p>
              <p>{t.testButtonDesc}</p>
              <p>{t.autoSaveDesc}</p>
              {isTauri && <p>{t.systemAudioApiDesc}</p>}
            </div>
          </div>
        ) : (
          !isLoading && (
            <div className="text-center py-8 text-gray-500 dark:text-dark-muted">
              <Mic className="w-12 h-12 mx-auto mb-3 opacity-30" />
              <p>{t.noMicrophonesDetected}</p>
              <p className="text-sm">
                {isTauri
                  ? t.checkSystemAudioSettings
                  : t.grantMicrophonePermission
                }
              </p>
              <button
                onClick={loadMicrophones}
                className="mt-3 px-4 py-2 bg-primary-500 hover:bg-primary-600 text-white rounded-md text-sm transition-colors"
              >
                {isTauri ? t.refreshDevices : t.requestMicrophoneAccess}
              </button>
            </div>
          )
        )}
      </div>

      {/* Audio Settings */}
      <div className="border-t border-gray-200 dark:border-dark-border pt-6">
        <h3 className="text-lg font-semibold text-gray-900 dark:text-dark-text mb-4">
          <Volume2 className="w-5 h-5 inline mr-2" />
          {t.audioSettings}
        </h3>
        <div className="bg-white dark:bg-dark-secondary rounded-lg border border-gray-200 dark:border-dark-border p-4">
          <div className="space-y-4">
            <ToggleInput
              label={t.saveWavFiles}
              checked={saveWavFiles}
              onChange={handleSaveWavFilesToggle}
              description={t.saveWavFilesLongDesc}
            />

            {isTauri && (
              <div className="mt-4 p-3 bg-gray-50 dark:bg-dark-primary rounded-md">
                <p className="text-sm text-gray-600 dark:text-dark-muted" dangerouslySetInnerHTML={{ __html: t.noteChangesSavedAutomatically }} />
              </div>
            )}
          </div>
        </div>
      </div>

      {/* AI Text Correction Settings */}
      {isTauri && (
        <div className="border-t border-gray-200 dark:border-dark-border pt-6">
          <div className="flex items-center justify-between mb-4">
            <h3 className="text-lg font-semibold text-gray-900 dark:text-dark-text flex items-center">
              <Wand2 className="w-5 h-5 mr-2" />
              {t.postProcessTextCorrection}
            </h3>
            {/* Save Status Indicator */}
            {postProcessSaveStatus !== 'idle' && (
              <div className="flex items-center text-sm">
                {postProcessSaveStatus === 'saving' && (
                  <span className="flex items-center text-blue-600">
                    <div className="animate-spin rounded-full h-4 w-4 border-b-2 border-blue-500 mr-2"></div>
                    Saving...
                  </span>
                )}
                {postProcessSaveStatus === 'success' && (
                  <span className="flex items-center text-green-600">
                    <Check className="w-4 h-4 mr-1" />
                    Saved
                  </span>
                )}
                {postProcessSaveStatus === 'error' && (
                  <span className="flex items-center text-red-600" title={postProcessSaveError}>
                    <AlertCircle className="w-4 h-4 mr-1" />
                    Save Failed: {postProcessSaveError || 'Unknown error'}
                  </span>
                )}
              </div>
            )}
          </div>
          <div className="bg-white dark:bg-dark-secondary rounded-lg border border-gray-200 dark:border-dark-border p-4">
            <div className="space-y-4">
              <ToggleInput
                label={t.postProcessEnabled}
                checked={postProcessConfig.enabled}
                onChange={(checked) => setPostProcessConfig(prev => ({ ...prev, enabled: checked }))}
                description={t.postProcessEnabledDesc}
              />

              {postProcessConfig.enabled && (
                <div className="mt-4 space-y-4">
                  {/* Provider Selection */}
                  <div>
                    <label className="block text-sm font-medium text-gray-700 dark:text-dark-text mb-2">
                      {t.postProcessProvider}
                    </label>
                    <select
                      value={postProcessConfig.provider}
                      onChange={(e) => {
                        const provider = e.target.value;
                        setPostProcessConfig(prev => ({
                          ...prev,
                          provider,
                          // Update defaults based on provider
                          endpoint: provider === 'deepseek'
                            ? 'https://api.deepseek.com/v1/chat/completions'
                            : 'http://localhost:11434/api/chat',
                          model: provider === 'deepseek' ? 'deepseek-chat' : 'llama3.2:latest',
                        }));
                      }}
                      className="w-full px-3 py-2 border border-gray-300 dark:border-dark-border bg-white dark:bg-dark-surface text-gray-900 dark:text-dark-text rounded-lg focus:ring-2 focus:ring-primary-500 focus:border-primary-500"
                    >
                      <option value="ollama">{t.postProcessProviderOllama}</option>
                      <option value="deepseek">{t.postProcessProviderDeepSeek}</option>
                    </select>
                  </div>

                  {/* DeepSeek API Key - only show when DeepSeek is selected */}
                  {postProcessConfig.provider === 'deepseek' && (
                    <div>
                      <label className="block text-sm font-medium text-gray-700 dark:text-dark-text mb-2">
                        {t.postProcessApiKey}
                      </label>
                      <input
                        type="password"
                        value={postProcessConfig.api_key || ''}
                        onChange={(e) => setPostProcessConfig(prev => ({ ...prev, api_key: e.target.value }))}
                        placeholder={t.postProcessApiKeyPlaceholder}
                        className="w-full px-3 py-2 border border-gray-300 dark:border-dark-border bg-white dark:bg-dark-surface text-gray-900 dark:text-dark-text rounded-lg focus:ring-2 focus:ring-primary-500 focus:border-primary-500"
                      />
                    </div>
                  )}

                  {/* Endpoint URL */}
                  <div>
                    <label className="block text-sm font-medium text-gray-700 dark:text-dark-text mb-2">
                      {t.postProcessEndpoint}
                    </label>
                    <Input
                      label={t.postProcessEndpoint}
                      type="text"
                      value={postProcessConfig.endpoint}
                      onChange={(e) => setPostProcessConfig(prev => ({ ...prev, endpoint: e.target.value }))}
                      placeholder={t.postProcessEndpointPlaceholder}
                    />
                  </div>

                  {/* Model Name */}
                  <div>
                    <label className="block text-sm font-medium text-gray-700 dark:text-dark-text mb-2">
                      {t.postProcessModel}
                    </label>
                    <Input
                      label={t.postProcessModel}
                      type="text"
                      value={postProcessConfig.model}
                      onChange={(e) => setPostProcessConfig(prev => ({ ...prev, model: e.target.value }))}
                      placeholder={t.postProcessModelPlaceholder}
                    />
                  </div>

                  {/* System Prompt */}
                  <div>
                    <label className="block text-sm font-medium text-gray-700 dark:text-dark-text mb-2">
                      {t.postProcessSystemPrompt}
                    </label>
                    <textarea
                      value={postProcessConfig.system_prompt}
                      onChange={(e) => setPostProcessConfig(prev => ({ ...prev, system_prompt: e.target.value }))}
                      rows={3}
                      className="w-full px-3 py-2 border border-gray-300 dark:border-dark-border bg-white dark:bg-dark-surface text-gray-900 dark:text-dark-text rounded-lg focus:ring-2 focus:ring-primary-500 focus:border-primary-500"
                      placeholder={t.postProcessSystemPromptPlaceholder}
                    />
                  </div>

                  {/* Timeout */}
                  <div>
                    <label className="block text-sm font-medium text-gray-700 dark:text-dark-text mb-2">
                      {t.postProcessTimeout}
                    </label>
                    <input
                      type="number"
                      value={postProcessConfig.timeout_seconds}
                      onChange={(e) => setPostProcessConfig(prev => ({ ...prev, timeout_seconds: parseInt(e.target.value) || 30 }))}
                      className="w-full px-3 py-2 border border-gray-300 dark:border-dark-border bg-white dark:bg-dark-surface text-gray-900 dark:text-dark-text rounded-lg focus:ring-2 focus:ring-primary-500 focus:border-primary-500"
                      min="5"
                      max="120"
                    />
                  </div>

                  {/* Info Box */}
                  <div className="bg-blue-50 dark:bg-blue-900/20 rounded-lg p-3 border border-blue-200 dark:border-blue-800">
                    <p className="text-sm text-blue-700 dark:text-blue-300">
                      {t.postProcessInfoBox}
                    </p>
                  </div>
                </div>
              )}

              {/* Auto-save notice */}
              <div className="mt-4 p-3 bg-gray-50 dark:bg-dark-primary rounded-md">
                <p className="text-sm text-gray-600 dark:text-dark-muted">
                  {t.postProcessAutoSaved}
                </p>
              </div>
            </div>
          </div>
        </div>
      )}

    </div>
  );
};