import React, { useState, useEffect } from 'react';
import { Button } from '../ui/Button';
import { TauriService } from '../../services/tauriService';

export const VoiceAssistantPanel = () => {
  const [isRunning, setIsRunning] = useState(false);
  const [systemInfo, setSystemInfo] = useState<Record<string, string>>({});
  const [testResults, setTestResults] = useState<string[]>([]);

  useEffect(() => {
    loadSystemInfo();
  }, []);

  const loadSystemInfo = async () => {
    try {
      const info = await TauriService.getSystemInfo();
      setSystemInfo(info);
    } catch (error) {
      console.error('Failed to load system info:', error);
    }
  };

  const startVoiceAssistant = async () => {
    try {
      setIsRunning(true);
      addTestResult('🚀 Starting Voice Assistant...');
      const result = await TauriService.startVoiceAssistant();
      addTestResult('✅ ' + result, 'success');
    } catch (error) {
      addTestResult('❌ Failed to start: ' + error, 'error');
      setIsRunning(false);
    }
  };

  const stopVoiceAssistant = async () => {
    try {
      setIsRunning(false);
      addTestResult('⏹️ Stopping Voice Assistant...');
      const result = await TauriService.stopVoiceAssistant();
      addTestResult('📴 ' + result, 'info');
    } catch (error) {
      addTestResult('❌ Failed to stop: ' + error, 'error');
    }
  };

  const testASR = async (processor: 'cloud' | 'local') => {
    try {
      addTestResult(`🧪 Testing ${processor} ASR...`);
      const result = await TauriService.testASR(processor);
      addTestResult('✅ ASR Test: ' + result, 'success');
    } catch (error) {
      addTestResult('❌ ASR Test Failed: ' + error, 'error');
    }
  };

  const testTranslation = async (translator: 'siliconflow' | 'ollama') => {
    try {
      addTestResult(`🧪 Testing ${translator} translation...`);
      const result = await TauriService.testTranslation(translator);
      addTestResult('✅ Translation Test: ' + result, 'success');
    } catch (error) {
      addTestResult('❌ Translation Test Failed: ' + error, 'error');
    }
  };

  const addTestResult = (message: string, type?: string) => {
    const timestamp = new Date().toLocaleTimeString();
    const result = `[${timestamp}] ${message}`;
    setTestResults(prev => [...prev, result]);

    // Keep only last 100 results (increase limit and prevent auto-clear)
    if (testResults.length > 100) {
      setTestResults(prev => prev.slice(-100));
    }
  };

  const clearResults = () => {
    setTestResults([]);
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

      {/* Test Buttons */}
      <div className="mb-6">
        <h4 className="text-sm font-semibold text-gray-700 mb-3">🧪 Test Processors</h4>
        <div className="grid grid-cols-1 sm:grid-cols-2 gap-3">
          <Button
            variant="ghost"
            size="sm"
            onClick={() => testASR('cloud')}
            className="justify-start"
          >
            ☁️ Test Cloud ASR
          </Button>
          <Button
            variant="ghost"
            size="sm"
            onClick={() => testASR('local')}
            className="justify-start"
          >
            🏠 Test Local ASR
          </Button>
          <Button
            variant="ghost"
            size="sm"
            onClick={() => testTranslation('siliconflow')}
            className="justify-start"
          >
            🌐 Test SiliconFlow
          </Button>
          <Button
            variant="ghost"
            size="sm"
            onClick={() => testTranslation('ollama')}
            className="justify-start"
          >
            🦙 Test Ollama
          </Button>
          <Button
            variant="ghost"
            size="sm"
            onClick={clearResults}
            className="justify-start text-red-500 hover:bg-red-50"
          >
            🗑️ Clear Results
          </Button>
        </div>
      </div>

      {/* Test Results */}
      {testResults.length > 0 && (
        <div className="mb-6">
          <div className="flex items-center justify-between mb-3">
            <h4 className="text-sm font-semibold text-gray-700">📋 Test Results ({testResults.length})</h4>
            <Button
              variant="ghost"
              size="sm"
              onClick={clearResults}
              className="text-xs text-red-500 hover:bg-red-50"
            >
              🗑️ Clear All
            </Button>
          </div>
          <div className="bg-gray-900 text-green-400 rounded-lg p-3 max-h-64 overflow-y-auto border border-gray-700">
            {testResults.map((result, index) => (
              <div key={index} className="text-xs font-mono mb-1 break-all leading-relaxed">
                {result}
              </div>
            ))}
          </div>
        </div>
      )}

      {/* System Information */}
      <div>
        <div className="flex items-center justify-between mb-3">
          <h4 className="text-sm font-semibold text-gray-700">💻 System Information</h4>
          <Button
            variant="ghost"
            size="sm"
            onClick={loadSystemInfo}
            className="text-xs"
          >
            🔄 Refresh
          </Button>
        </div>
        <div className="bg-gray-50 rounded-lg p-3">
          {Object.entries(systemInfo).map(([key, value]) => (
            <div key={key} className="flex justify-between text-sm mb-1">
              <span className="font-medium text-gray-600">{key}:</span>
              <span className="text-gray-800 font-mono text-xs">{value}</span>
            </div>
          ))}
        </div>
      </div>
    </div>
  );
};