import React, { useState, useEffect } from 'react';
import { Button } from '../ui/Button';
import { TauriService } from '../../services/tauriService';

interface VoiceAssistantPanelProps {
  isRunning?: boolean;
  onStatusChange?: (running: boolean) => void;
}

export const VoiceAssistantPanel: React.FC<VoiceAssistantPanelProps> = ({
  isRunning: externalIsRunning,
  onStatusChange
}) => {
  const [internalIsRunning, setInternalIsRunning] = useState(false);

  // Use external state if provided, otherwise use internal state
  const isRunning = externalIsRunning !== undefined ? externalIsRunning : internalIsRunning;

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
    <div className="bg-white rounded-2xl shadow-lg shadow-gray-200/50 p-6 mb-8 border border-white">
      <h3 className="text-lg font-bold text-gray-900 mb-4">🎤 Voice Assistant Status</h3>

      {/* Status Display */}
      <div className="p-4 bg-gray-50 rounded-lg">
        <div className="flex items-center justify-between">
          <span className="text-sm font-medium">Service Status:</span>
          <span className={`text-sm font-bold ${isRunning ? 'text-green-600' : 'text-gray-600'}`}>
            {isRunning ? '🟢 Active' : '⚪ Inactive'}
          </span>
        </div>
        <div className="mt-2 text-xs text-gray-500">
          {isRunning
            ? 'Voice Assistant is running and listening for hotkeys (F4, Shift+F4)'
            : 'Use the Start button in the top bar to activate Voice Assistant'}
        </div>
      </div>
    </div>
  );
};