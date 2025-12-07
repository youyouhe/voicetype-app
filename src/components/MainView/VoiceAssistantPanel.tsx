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
      const result = await TauriService.startVoiceAssistant();
      console.log('Voice Assistant started:', result);
      // Optimistically set to true, but let events handle the real state
      if (onStatusChange) {
        onStatusChange(true);
      }
    } catch (error) {
      console.error('Failed to start Voice Assistant:', error);
      setIsRunning(false);
    }
  };

  const stopVoiceAssistant = async () => {
    try {
      const result = await TauriService.stopVoiceAssistant();
      console.log('Voice Assistant stopped:', result);
      // Optimistically set to false, but let events handle the real state
      if (onStatusChange) {
        onStatusChange(false);
      }
    } catch (error) {
      console.error('Failed to stop Voice Assistant:', error);
    }
  };

  return (
    <div className="bg-white rounded-2xl shadow-lg shadow-gray-200/50 p-6 mb-8 border border-white">
      <h3 className="text-lg font-bold text-gray-900 mb-4">🎤 Voice Assistant Control</h3>

      {/* Control Buttons */}
      <div className="flex flex-col sm:flex-row space-y-3 sm:space-y-0 sm:space-x-4 mb-6">
        <Button
          variant={isRunning ? "secondary" : "primary"}
          size="lg"
          onClick={isRunning ? stopVoiceAssistant : startVoiceAssistant}
          className="flex-1"
        >
          {isRunning ? '⏹️ Stop Voice Assistant' : '▶️ Start Voice Assistant'}
        </Button>
      </div>

      {/* Status */}
      <div className="mb-6 p-4 bg-gray-50 rounded-lg">
        <div className="flex items-center justify-between">
          <span className="text-sm font-medium">Status:</span>
          <span className={`text-sm font-bold ${isRunning ? 'text-green-600' : 'text-gray-600'}`}>
            {isRunning ? '🟢 Running' : '⚪ Stopped'}
          </span>
        </div>
      </div>

  
    </div>
  );
};