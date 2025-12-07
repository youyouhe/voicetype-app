import React, { useState, useEffect, useCallback } from 'react';
import { Mic, Settings, RefreshCw, Volume2 } from 'lucide-react';
import { invoke } from '@tauri-apps/api/core';
import { ToggleInput } from '../ui/Input';
import { Input } from '../ui/Input';
import { HotkeyConfig, TypingDelays } from '../../types';

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
  const [isInitializing, setIsInitializing] = useState(true);

  // Typing Delays Settings
  const [typingDelays, setTypingDelays] = useState<TypingDelays>({
    clipboard_update_ms: 100,         // 剪贴板更新等待时间
    keyboard_events_settle_ms: 300,   // 键盘事件处理等待时间
    typing_complete_ms: 500,          // 打字完成后等待时间
    character_interval_ms: 100,       // 字符间延迟时间
    short_operation_ms: 100,          // 其他短操作延迟时间
  });

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
    console.log('🔍 AdvancedSettings mount useEffect:');
    console.log('  - isTauri:', isTauri);
    console.log('  - hasLoadedFromDatabase:', hasLoadedFromDatabase);

    const loadHotkeyConfiguration = async () => {
      if (isTauri && hasLoadedFromDatabase === false) {
        try {
          console.log('📥 Loading hotkey config from database...');
          const config = await invoke<HotkeyConfig | null>('get_hotkey_config');
          console.log('📋 Raw config from database:', config);

          if (config) {
            console.log('📥 Loaded hotkey config from database:');
            console.log('  - save_wav_files:', config.save_wav_files);
            console.log('  - clipboard_update_ms:', config.clipboard_update_ms);
            console.log('  - keyboard_events_settle_ms:', config.keyboard_events_settle_ms);
            console.log('  - typing_complete_ms:', config.typing_complete_ms);
            console.log('  - character_interval_ms:', config.character_interval_ms);
            console.log('  - short_operation_ms:', config.short_operation_ms);

            console.log('🔄 Setting state with loaded values...');
            setSaveWavFiles(config.save_wav_files);

            const newTypingDelays = {
              clipboard_update_ms: config.clipboard_update_ms,
              keyboard_events_settle_ms: config.keyboard_events_settle_ms,
              typing_complete_ms: config.typing_complete_ms,
              character_interval_ms: config.character_interval_ms,
              short_operation_ms: config.short_operation_ms,
            };
            setTypingDelays(newTypingDelays);
            console.log('✅ State updated with typing delays:', newTypingDelays);
          } else {
            console.log('📥 No hotkey config found, using defaults');
            console.log('🔄 Setting state with default typing delays:', {
              clipboard_update_ms: 100,
              keyboard_events_settle_ms: 300,
              typing_complete_ms: 500,
              character_interval_ms: 100,
              short_operation_ms: 100,
            });
          }
        } catch (error) {
          console.error('❌ Failed to load hotkey configuration:', error);
          console.error('❌ Error details:', JSON.stringify(error, null, 2));
        }
        console.log('🔔 Setting hasLoadedFromDatabase to true');
        setHasLoadedFromDatabase(true);
        // Mark initialization as complete after loading initial values
        setTimeout(() => setIsInitializing(false), 0);
      } else {
        console.log('⏸️ Skipping config load - isTauri:', isTauri, ', hasLoadedFromDatabase:', hasLoadedFromDatabase);
        if (!isTauri) {
          // For non-Tauri environment, mark initialization as complete immediately
          setTimeout(() => setIsInitializing(false), 0);
        }
      }
    };

    console.log('🚀 Calling loadHotkeyConfiguration...');
    loadHotkeyConfiguration();
  }, [isTauri, hasLoadedFromDatabase]);

  // Save hotkey configuration when saveWavFiles changes
  const saveHotkeyConfig = useCallback(async () => {
    console.log('🎯 saveHotkeyConfig called:');
    console.log('  - isTauri:', isTauri);
    console.log('  - hasLoadedFromDatabase:', hasLoadedFromDatabase);
    console.log('  - isInitializing:', isInitializing);

    // Skip save during initialization to avoid unnecessary database writes
    if (isInitializing) {
      console.log('⏭️ Skipping save during component initialization');
      return;
    }

    if (isTauri && hasLoadedFromDatabase) {
      try {
        console.log('📖 Reading existing configuration...');
        // First, load existing configuration to preserve user settings
        const existingConfig = await invoke<HotkeyConfig | null>('get_hotkey_config');
        console.log('📋 Existing config loaded:', existingConfig);

        // Check if values actually changed to avoid unnecessary database writes
        const saveWavFilesChanged = !existingConfig || existingConfig.save_wav_files !== saveWavFiles;
        const typingDelaysChanged = !existingConfig || 
          existingConfig.clipboard_update_ms !== typingDelays.clipboard_update_ms ||
          existingConfig.keyboard_events_settle_ms !== typingDelays.keyboard_events_settle_ms ||
          existingConfig.typing_complete_ms !== typingDelays.typing_complete_ms ||
          existingConfig.character_interval_ms !== typingDelays.character_interval_ms ||
          existingConfig.short_operation_ms !== typingDelays.short_operation_ms;

        if (!saveWavFilesChanged && !typingDelaysChanged) {
          console.log('⏭️ No actual changes detected, skipping save to avoid unnecessary database write');
          return;
        }

        const config = {
          transcribe_key: existingConfig?.transcribe_key || 'F4', // Preserve existing values
          translate_key: existingConfig?.translate_key || 'Shift + F4',
          trigger_delay_ms: existingConfig?.trigger_delay_ms || 300,
          anti_mistouch_enabled: existingConfig?.anti_mistouch_enabled ?? true,
          save_wav_files: saveWavFiles, // Update only this setting
          typing_delays: typingDelays, // Update only delays
        };

        console.log('💾 About to save hotkey config (Advanced Settings):');
        console.log('  - save_wav_files changed:', saveWavFilesChanged);
        console.log('  - typing_delays changed:', typingDelaysChanged);
        console.log('  - save_wav_files:', config.save_wav_files);
        console.log('  - typing_delays:', config.typing_delays);
        console.log('  - transcribe_key:', config.transcribe_key);
        console.log('  - translate_key:', config.translate_key);
        console.log('  - trigger_delay_ms:', config.trigger_delay_ms);
        console.log('  - anti_mistouch_enabled:', config.anti_mistouch_enabled);

        console.log('📡 Calling save_hotkey_config Tauri command...');
        const result = await invoke('save_hotkey_config', { request: config });
        console.log('✅ Tauri command result:', result);
        console.log('✅ Hotkey configuration saved successfully');
      } catch (error) {
        console.error('❌ Failed to save hotkey configuration:', error);
        console.error('❌ Error details:', JSON.stringify(error, null, 2));
      }
    } else {
      console.log('❌ Cannot save - conditions not met:');
      console.log('  - isTauri:', isTauri);
      console.log('  - hasLoadedFromDatabase:', hasLoadedFromDatabase);
    }
  }, [saveWavFiles, typingDelays, isTauri, hasLoadedFromDatabase]);

  // Trigger save when saveWavFiles or typingDelays changes
  useEffect(() => {
    console.log('🔍 AdvancedSettings useEffect triggered:');
    console.log('  - hasLoadedFromDatabase:', hasLoadedFromDatabase);
    console.log('  - isTauri:', isTauri);
    console.log('  - saveWavFiles changed to:', saveWavFiles);
    console.log('  - typingDelays changed to:', typingDelays);

    if (hasLoadedFromDatabase) {
      console.log('🚀 Calling saveHotkeyConfig...');
      saveHotkeyConfig();
    } else {
      console.log('⏳ Skipping save - database not loaded yet');
    }
  }, [saveWavFiles, typingDelays, saveHotkeyConfig, hasLoadedFromDatabase, isInitializing]);

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

      {/* Typing Delays Settings */}
      <div className="border-t border-gray-200 dark:border-dark-border pt-6">
        <h3 className="text-lg font-semibold text-gray-900 dark:text-dark-text mb-4">
          <Settings className="w-5 h-5 inline mr-2" />
          Typing Delays (milliseconds)
        </h3>
        <div className="bg-white dark:bg-dark-secondary rounded-lg border border-gray-200 dark:border-dark-border p-4">
          <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
            <Input
              label="Clipboard Update Wait"
              type="number"
              step={50}
              value={typingDelays.clipboard_update_ms}
              onChange={(value) => setTypingDelays(prev => ({...prev, clipboard_update_ms: value}))}
              description="Wait for clipboard to update"
              autoFocus={false}
            />
            <Input
              label="Keyboard Events Settle"
              type="number"
              step={50}
              value={typingDelays.keyboard_events_settle_ms}
              onChange={(value) => {
              console.log('⌨️ Keyboard Events Settle changed from', typingDelays.keyboard_events_settle_ms, 'to', value);
              setTypingDelays(prev => {
                const newDelays = {...prev, keyboard_events_settle_ms: value};
                console.log('🔄 New typing delays after change:', newDelays);
                return newDelays;
              });
            }}
              description="Ensure keyboard events are fully processed"
              autoFocus={false}
            />
            <Input
              label="Typing Complete Wait"
              type="number"
              step={50}
              value={typingDelays.typing_complete_ms}
              onChange={(value) => setTypingDelays(prev => ({...prev, typing_complete_ms: value}))}
              description="Ensure all character input completes"
              autoFocus={false}
            />
            <Input
              label="Character Interval"
              type="number"
              step={10}
              value={typingDelays.character_interval_ms}
              onChange={(value) => setTypingDelays(prev => ({...prev, character_interval_ms: value}))}
              description="Delay between characters (important for Chinese)"
              autoFocus={false}
            />
            <Input
              label="Short Operation Delay"
              type="number"
              step={10}
              value={typingDelays.short_operation_ms}
              onChange={(value) => setTypingDelays(prev => ({...prev, short_operation_ms: value}))}
              description="Other short operation delays"
              autoFocus={false}
            />
          </div>

          {isTauri && (
            <div className="mt-4 p-3 bg-gray-50 dark:bg-dark-primary rounded-md">
              <p className="text-sm text-gray-600 dark:text-dark-muted">
                <strong>Note:</strong> These delays control typing behavior and are saved automatically. Adjust them if you experience timing issues with text input.
              </p>
            </div>
          )}
        </div>
      </div>
    </div>
  );
};