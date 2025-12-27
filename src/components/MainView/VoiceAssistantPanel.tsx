import React, { useState, useEffect } from 'react';
import { Button } from '../ui/Button';
import { TauriService } from '../../services/tauriService';
import type { HotkeyConfig } from '../../types';
import { useLanguage } from '../../contexts/LanguageContext';

interface VoiceAssistantPanelProps {
  isRunning?: boolean;
  onStatusChange?: (running: boolean) => void;
}

export const VoiceAssistantPanel: React.FC<VoiceAssistantPanelProps> = ({
  isRunning: externalIsRunning,
  onStatusChange
}) => {
  const { t } = useLanguage();
  const [internalIsRunning, setInternalIsRunning] = useState(false);
  const [hotkeyConfig, setHotkeyConfig] = useState<HotkeyConfig | null>(null);

  // Use external state if provided, otherwise use internal state
  const isRunning = externalIsRunning !== undefined ? externalIsRunning : internalIsRunning;

  // Fetch hotkey config on mount
  useEffect(() => {
    const fetchHotkeyConfig = async () => {
      try {
        const config = await TauriService.getHotkeyConfig();
        console.log('📥 VoiceAssistantPanel - Loaded hotkey config:', config);
        setHotkeyConfig(config);
      } catch (error) {
        console.error('❌ VoiceAssistantPanel - Failed to load hotkey config:', error);
      }
    };

    fetchHotkeyConfig();
  }, []);

  // Update internal state only if we don't have external control
  const setIsRunning = (running: boolean) => {
    if (onStatusChange) {
      onStatusChange(running);
    } else {
      setInternalIsRunning(running);
    }
  };

  // Debug logging
  console.log('🎤 VoiceAssistantPanel - externalIsRunning:', externalIsRunning);
  console.log('🎤 VoiceAssistantPanel - final isRunning:', isRunning);
  

  const startVoiceAssistant = async () => {
    try {
      console.log('🎤 VoiceAssistantPanel starting Voice Assistant...');
      const result = await TauriService.startVoiceAssistant();
      console.log('Voice Assistant started:', result);

      // Wait a bit for backend to initialize, then check actual state
      setTimeout(async () => {
        try {
          const actualState = await TauriService.getVoiceAssistantState();
          console.log('🔍 VoiceAssistantPanel - Actual state after start:', actualState);
          const cleanState = actualState.replace(/"/g, '').trim();
          const isActuallyRunning = cleanState === 'Running' ||
                                  cleanState === 'Recording' ||
                                  cleanState === 'RecordingTranslate' ||
                                  cleanState === 'Processing' ||
                                  cleanState === 'Translating';

          console.log('🟢 VoiceAssistantPanel - Actually running after start:', isActuallyRunning);
          if (onStatusChange) {
            onStatusChange(isActuallyRunning);
          }
        } catch (error) {
          console.error('VoiceAssistantPanel - Failed to verify Voice Assistant state:', error);
          if (onStatusChange) {
            onStatusChange(false);
          }
        }
      }, 500); // Wait 500ms for backend to initialize

    } catch (error) {
      console.error('VoiceAssistantPanel - Failed to start Voice Assistant:', error);
      if (onStatusChange) {
        onStatusChange(false);
      }
    }
  };

  const stopVoiceAssistant = async () => {
    try {
      console.log('🎤 VoiceAssistantPanel stopping Voice Assistant...');
      const result = await TauriService.stopVoiceAssistant();
      console.log('Voice Assistant stopped:', result);

      // Wait a bit for backend to stop, then verify actual state
      setTimeout(async () => {
        try {
          const actualState = await TauriService.getVoiceAssistantState();
          console.log('🔍 VoiceAssistantPanel - Actual state after stop:', actualState);
          const cleanState = actualState.replace(/"/g, '').trim();
          const isActuallyRunning = cleanState === 'Running' ||
                                  cleanState === 'Recording' ||
                                  cleanState === 'RecordingTranslate' ||
                                  cleanState === 'Processing' ||
                                  cleanState === 'Translating';

          console.log('🟢 VoiceAssistantPanel - Actually running after stop:', isActuallyRunning);
          if (onStatusChange) {
            onStatusChange(isActuallyRunning);
          }
        } catch (error) {
          console.error('VoiceAssistantPanel - Failed to verify stop state:', error);
          if (onStatusChange) {
            onStatusChange(false);
          }
        }
      }, 300); // Wait 300ms for backend to stop

    } catch (error) {
      console.error('VoiceAssistantPanel - Failed to stop Voice Assistant:', error);
      if (onStatusChange) {
        onStatusChange(false);
      }
    }
  };

  return (
    <div className="bg-white dark:bg-dark-surface rounded-2xl shadow-lg shadow-gray-200/50 dark:shadow-black/20 p-6 mb-8 border border-white dark:border-dark-border">
      <h3 className="text-lg font-bold text-gray-900 dark:text-gray-100 mb-4">🎤 {t.voiceAssistantStatus}</h3>

      {/* Status Display */}
      <div className="p-4 bg-gray-50 dark:bg-slate-800 rounded-lg">
        <div className="flex items-center justify-between">
          <span className="text-sm font-medium text-gray-700 dark:text-gray-300">{t.serviceStatusText}</span>
          <span className={`text-sm font-bold ${isRunning ? 'text-green-600 dark:text-green-400' : 'text-gray-600 dark:text-gray-400'}`}>
            {isRunning ? `🟢 ${t.activeText}` : `⚪ ${t.inactiveText}`}
          </span>
        </div>
        <div className="mt-2 text-xs text-gray-500 dark:text-gray-400">
          {isRunning
            ? t.runningListening(hotkeyConfig?.transcribe_key || 'F4', hotkeyConfig?.translate_key || 'Shift+F4')
            : t.useStartButton}
        </div>
      </div>
    </div>
  );
};