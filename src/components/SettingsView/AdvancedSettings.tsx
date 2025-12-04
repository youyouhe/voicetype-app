import React, { useState, useEffect, useCallback } from 'react';
import { Mic, Settings, RefreshCw, Volume2 } from 'lucide-react';
import { invoke } from '@tauri-apps/api/core';
import { ToggleInput } from '../ui/Input';
import { HotkeyConfig } from '../../types';

interface AudioDevice {
  id: string;
  name: string;
  is_default: boolean;
}

export const AdvancedSettings: React.FC = () => {
  const [micDevices, setMicDevices] = useState<AudioDevice[]>([]);
  const [selectedMic, setSelectedMic] = useState<string>('');
  const [isLoading, setIsLoading] = useState(false);
  const [isTauri, setIsTauri] = useState(false);
  
  // WAV Files Settings
  const [saveWavFiles, setSaveWavFiles] = useState(true);
  const [hasLoadedFromDatabase, setHasLoadedFromDatabase] = useState(false);

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
          alert('Microphone test successful! Audio input is working.');
        } else {
          alert('Microphone test failed. Please check your microphone settings.');
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
        
        alert('Microphone test successful! Audio input is working.');
      }
    } catch (error) {
      console.error('Microphone test failed:', error);
      alert('Microphone test failed. Please check your microphone settings.');
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
          
          if (config) {
            console.log('📥 Loaded hotkey config from database:', {
              save_wav_files: config.save_wav_files
            });
            setSaveWavFiles(config.save_wav_files);
          } else {
            console.log('📥 No hotkey config found, using defaults');
          }
        } catch (error) {
          console.error('Failed to load hotkey configuration:', error);
        }
        setHasLoadedFromDatabase(true);
      }
    };

    loadHotkeyConfiguration();
  }, [isTauri, hasLoadedFromDatabase]);

  // Save hotkey configuration when saveWavFiles changes
  const saveHotkeyConfig = useCallback(async () => {
    if (isTauri && hasLoadedFromDatabase) {
      try {
        const config = {
          transcribe_key: 'F4', // Default values - user can change these in Shortcuts tab
          translate_key: 'Shift + F4',
          trigger_delay_ms: 300,
          anti_mistouch_enabled: true,
          save_wav_files: saveWavFiles,
        };

        console.log('💾 Saving hotkey config (Audio Settings):');
        console.log('  - save_wav_files:', config.save_wav_files);

        await invoke('save_hotkey_config', { request: config });
        console.log('✅ Hotkey configuration saved successfully');
      } catch (error) {
        console.error('Failed to save hotkey configuration:', error);
      }
    }
  }, [saveWavFiles, isTauri, hasLoadedFromDatabase]);

  // Trigger save when saveWavFiles changes
  useEffect(() => {
    if (hasLoadedFromDatabase) {
      saveHotkeyConfig();
    }
  }, [saveWavFiles, saveHotkeyConfig, hasLoadedFromDatabase]);

  return (
    <div className="space-y-6">
      <div>
        <h2 className="text-2xl font-bold text-gray-900 dark:text-dark-text mb-2 flex items-center">
          <Settings className="w-6 h-6 mr-2" />
          Advanced Settings
        </h2>
        <p className="text-gray-600 dark:text-dark-muted">
          Configure advanced audio and system settings.
        </p>
        {isTauri && (
          <p className="text-xs text-green-600 dark:text-green-400 mt-1">
            ✓ Running in Tauri native environment
          </p>
        )}
      </div>

      {/* Microphone Settings */}
      <div className="border-t border-gray-200 dark:border-dark-border pt-6">
        <div className="flex items-center justify-between mb-4">
          <h3 className="text-lg font-semibold text-gray-900 dark:text-dark-text flex items-center">
            <Mic className="w-5 h-5 mr-2" />
            Audio Input Device
          </h3>
          <button
            onClick={loadMicrophones}
            disabled={isLoading}
            className="flex items-center px-3 py-1 text-sm bg-gray-100 dark:bg-dark-bg hover:bg-gray-200 dark:hover:bg-dark-border rounded-md transition-colors"
          >
            <RefreshCw className={`w-4 h-4 mr-1 ${isLoading ? 'animate-spin' : ''}`} />
            Refresh
          </button>
        </div>

        {micDevices.length > 0 ? (
          <div className="space-y-4">
            <div>
              <label className="block text-sm font-medium text-gray-700 dark:text-dark-text mb-2">
                Select Microphone
              </label>
              <select
                value={selectedMic || ''}
                onChange={(e) => handleMicChange(e.target.value)}
                className="w-full px-3 py-2 border border-gray-300 dark:border-dark-border bg-white dark:bg-dark-surface text-gray-900 dark:text-dark-text rounded-lg focus:ring-2 focus:ring-primary-500 focus:border-primary-500"
              >
                <option value="" disabled>
                  Select a microphone...
                </option>
                {micDevices.map((mic) => (
                  <option key={mic.id} value={mic.id}>
                    {mic.name} {mic.is_default && '(Default)'}
                  </option>
                ))}
              </select>
              {selectedMic && (
                <p className="mt-1 text-xs text-gray-500 dark:text-dark-muted">
                  Selected: {micDevices.find(m => m.id === selectedMic)?.name}
                </p>
              )}
            </div>

            <div className="flex items-center justify-between p-4 bg-gray-50 dark:bg-dark-bg rounded-lg border border-gray-200 dark:border-dark-border">
              <div className="flex items-center">
                <Volume2 className="w-5 h-5 text-gray-400 dark:text-dark-muted mr-3" />
                <div>
                  <p className="text-sm font-medium text-gray-900 dark:text-dark-text">
                    Current Selection
                  </p>
                  <p className="text-xs text-gray-500 dark:text-dark-muted">
                    {micDevices.find(m => m.id === selectedMic)?.name || 'No microphone selected'}
                  </p>
                </div>
              </div>
              <button
                onClick={testMicrophone}
                disabled={!selectedMic}
                className="px-3 py-1 text-sm bg-primary-500 hover:bg-primary-600 text-white rounded-md transition-colors disabled:opacity-50 disabled:cursor-not-allowed"
              >
                Test Microphone
              </button>
            </div>

            <div className="text-xs text-gray-500 dark:text-dark-muted">
              <p>• Select your preferred microphone for voice input</p>
              <p>• Use the test button to verify microphone functionality</p>
              <p>• Your selection will be saved automatically</p>
              {isTauri && <p>• Using system audio API for device detection</p>}
            </div>
          </div>
        ) : (
          !isLoading && (
            <div className="text-center py-8 text-gray-500 dark:text-dark-muted">
              <Mic className="w-12 h-12 mx-auto mb-3 opacity-30" />
              <p>No microphones detected</p>
              <p className="text-sm">
                {isTauri 
                  ? 'Please check your system audio settings' 
                  : 'Please grant microphone permission to detect audio devices'
                }
              </p>
              <button
                onClick={loadMicrophones}
                className="mt-3 px-4 py-2 bg-primary-500 hover:bg-primary-600 text-white rounded-md text-sm transition-colors"
              >
                {isTauri ? 'Refresh Devices' : 'Request Microphone Access'}
              </button>
            </div>
          )
        )}
      </div>

      {/* Audio Settings */}
      <div className="border-t border-gray-200 dark:border-dark-border pt-6">
        <h3 className="text-lg font-semibold text-gray-900 dark:text-dark-text mb-4">
          <Volume2 className="w-5 h-5 inline mr-2" />
          Audio Settings
        </h3>
        <div className="bg-white dark:bg-dark-secondary rounded-lg border border-gray-200 dark:border-dark-border p-4">
          <div className="space-y-4">
            <ToggleInput 
              label="Save WAV Files" 
              checked={saveWavFiles} 
              onChange={setSaveWavFiles} 
              description="Save recorded audio as WAV files after processing for debugging and backup purposes."
            />
            
            {isTauri && (
              <div className="mt-4 p-3 bg-gray-50 dark:bg-dark-primary rounded-md">
                <p className="text-sm text-gray-600 dark:text-dark-muted">
                  <strong>Note:</strong> Changes to this setting will be saved automatically.
                </p>
              </div>
            )}
          </div>
        </div>
      </div>

      {/* Additional Advanced Settings Placeholder */}
      <div className="border-t border-gray-200 dark:border-dark-border pt-6">
        <h3 className="text-lg font-semibold text-gray-900 dark:text-dark-text mb-4">
          System Configuration
        </h3>
        <div className="text-center py-8 text-gray-500 dark:text-dark-muted">
          <p>Additional advanced settings coming soon...</p>
        </div>
      </div>
    </div>
  );
};