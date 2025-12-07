import React, { useState, useEffect, useCallback, useRef } from 'react';
import { Server, Cloud, Globe, Wifi, Check, AlertTriangle, Save } from 'lucide-react';
import { ServiceProvider, ServiceOptionProps } from '../../types';
import { Input, ToggleInput } from '../ui/Input';
import { HotkeyInput } from '../ui/HotkeyInput';
import { Button } from '../ui/Button';
import { DatabaseService, AsrConfigRequest, StorageMigration } from '../../services/database';
import { configManager } from '../../services/configManager';
import { invoke } from '@tauri-apps/api/core';
import { HotkeyConfig, HotkeyConfigRequest } from '../../types';

// Service Option Component
const ServiceOption: React.FC<ServiceOptionProps> = ({ id, title, description, icon, selected, onSelect, disabled = false }) => (
  <div
    onClick={disabled ? undefined : onSelect}
    className={`
      relative flex items-start p-4 rounded-xl border-2 transition-all duration-200
      ${selected
        ? 'border-primary-500 bg-primary-50'
        : disabled
          ? 'border-gray-200 bg-gray-100 cursor-not-allowed opacity-60'
          : 'border-gray-200 bg-white hover:border-gray-300 hover:bg-gray-50 cursor-pointer'}
    `}
  >
    <div className={`p-2 rounded-lg mr-4 ${selected ? 'bg-primary-100 text-primary-600' : disabled ? 'bg-gray-100 text-gray-400' : 'bg-gray-100 text-gray-500'}`}>
      {icon}
    </div>
    <div className="flex-1">
      <div className="flex justify-between">
        <h3 className={`font-semibold ${selected ? 'text-primary-900' : disabled ? 'text-gray-500' : 'text-gray-900'}`}>{title}</h3>
        {selected && <Check className="w-5 h-5 text-primary-500" />}
      </div>
      <p className={`text-sm mt-1 ${selected ? 'text-primary-700' : disabled ? 'text-gray-400' : 'text-gray-500'}`}>{description}</p>
    </div>
  </div>
);

// ASR Settings Form
export const ASRSettings: React.FC = () => {
  // Track component mount/remount
  const mountId = useRef(`mount-${Date.now()}-${Math.random()}`);
  
  // Prevent multiple instances of ASRSettings from mounting simultaneously
  if (typeof window !== 'undefined') {
    const existingInstance = (window as any).__asrSettingsInstance;
    if (existingInstance && existingInstance !== mountId.current) {
      console.warn('⚠️ Multiple ASRSettings instances detected! Existing:', existingInstance, 'New:', mountId.current);
    }
    (window as any).__asrSettingsInstance = mountId.current;
  }

  // Debug state (moved to the top to avoid dependency issues)
  const [debugLogs, setDebugLogs] = useState<string[]>([]);
  const [showDebugPanel, setShowDebugPanel] = useState(true); // 默认显示调试面板

  const addDebugLog = useCallback((message: string) => {
    const timestamp = new Date().toLocaleTimeString();
    const logEntry = `[${timestamp}] ${message}`;
    setDebugLogs(prev => [...prev.slice(-49), logEntry]); // 保留最近50条日志
    console.log(message); // 同时输出到控制台（如果开启了的话）
  }, []);

  useEffect(() => {
    console.log('🏗️ ASRSettings component mounted - ID:', mountId.current);
    console.log('🪟 Window.__TAURI__ available:', !!window.__TAURI__);
    console.log('🪟 Window.__TAURI_INTERNALS__ available:', !!(window as any).__TAURI_INTERNALS__);
    console.log('⚠️ CHECK: Multiple instances of ASRSettings might be mounting!');
    addDebugLog('Component mounted. Environment: ' + (typeof window !== 'undefined' && window.__TAURI__ ? 'Tauri' : 'Browser') + ' - ID: ' + mountId.current);
    return () => {
      console.log('🏗️ ASRSettings component unmounting - ID:', mountId.current);
      // Clean up window reference
      if (typeof window !== 'undefined' && (window as any).__asrSettingsInstance === mountId.current) {
        delete (window as any).__asrSettingsInstance;
      }
    };
  }, [addDebugLog]);

  // Initialize with undefined to avoid premature rendering and focus issues
  const [selectedService, setSelectedService] = useState<ServiceProvider | undefined>(undefined);
  const [hasLoadedFromDatabase, setHasLoadedFromDatabase] = useState(false);
  const [hasUserSelection, setHasUserSelection] = useState(false);
  const [apiKey, setApiKey] = useState('');
  const [localApiKey, setLocalApiKey] = useState('');
  const [localEndpoint, setLocalEndpoint] = useState('http://localhost:5001/inference');
  const [cloudEndpoint, setCloudEndpoint] = useState('https://api.siliconflow.cn/v1/audio/transcriptions');
  const [isTesting, setIsTesting] = useState(false);
  const [status, setStatus] = useState<'idle' | 'success' | 'failed'>('idle');
  const [testResult, setTestResult] = useState<'idle' | 'success' | 'failed'>('idle');
  const [testMessage, setTestMessage] = useState<string>('');

  // Health check state
  const [healthStatus, setHealthStatus] = useState<'idle' | 'checking' | 'healthy' | 'unhealthy'>('idle');
  const [healthMessage, setHealthMessage] = useState<string>('');

  // ASR test state
  const [isTestingAsr, setIsTestingAsr] = useState(false);
  const [asrResult, setAsrResult] = useState<string | null>(null);
  const [asrTestMessage, setAsrTestMessage] = useState<string>('');

  // File input state
  const [selectedFile, setSelectedFile] = useState<File | null>(null);

  // Check if running in Tauri environment
  const isTauriEnvironment = () => {
    return typeof window !== 'undefined' && window.__TAURI__;
  };

  // Add a flag to prevent unnecessary health checks
  const healthCheckRef = useRef(false);

  // Health check function (simplified version for auto-check)
  const checkHealthStatus = useCallback(async (serviceProvider: string, endpoint: string) => {
    if (!endpoint) {
      addDebugLog('❌ No endpoint provided for health check');
      return;
    }

    // Prevent multiple concurrent health checks
    if (healthCheckRef.current) {
      addDebugLog('⚠️ Health check already in progress, skipping...');
      return;
    }

    // Check if we're in Tauri environment
    if (!isTauriEnvironment()) {
      setHealthStatus('unhealthy');
      setHealthMessage('Tauri environment required');
      addDebugLog('⚠️ Health check not available in browser');
      return;
    }

    setHealthStatus('checking');
    healthCheckRef.current = true;
    addDebugLog('🏥 Auto health check started for: ' + serviceProvider);
    addDebugLog('🔗 Endpoint: ' + endpoint);
    addDebugLog('🌐 Window.__TAURI__ available: ' + !!window.__TAURI__);

    // Test basic connection first
    try {
      addDebugLog('📞 Testing basic backend connection...');
      const testResponse = await invoke('test_frontend_backend_connection') as string;
      addDebugLog('📨 Backend test response: ' + testResponse);

      addDebugLog('📞 Invoking Rust command: test_connection_health');
      const response = await invoke('test_connection_health', {
        request: { endpoint }
      }) as {
        success: boolean;
        message: string;
        backend_count?: number;
      };

      addDebugLog('📨 Received response from backend: ' + JSON.stringify(response));

      if (response.success) {
        setHealthStatus('healthy');
        setHealthMessage(response.message + (response.backend_count ? ` (${response.backend_count} backends)` : ''));
        addDebugLog('✅ Health check passed: ' + response.message);
      } else {
        setHealthStatus('unhealthy');
        setHealthMessage(response.message);
        addDebugLog('❌ Health check failed: ' + response.message);
      }
    } catch (error) {
      setHealthStatus('unhealthy');
      setHealthMessage('Connection failed');
      addDebugLog('💥 Health check error: ' + String(error));
      addDebugLog('🔍 Error type: ' + typeof error);
      addDebugLog('🔍 Error details: ' + JSON.stringify(error, null, 2));
    } finally {
      healthCheckRef.current = false;
    }
  }, []); // Remove addDebugLog dependency to prevent infinite loops
  const [isLoading, setIsLoading] = useState(true);
  const [isSaving, setIsSaving] = useState(false);

  // Prevent auto-focus on input fields when component mounts and data changes
  useEffect(() => {
    // Remove focus from any input field that might have been auto-focused
    const timer = setTimeout(() => {
      const activeElement = document.activeElement as HTMLElement;
      if (activeElement && activeElement.tagName === 'INPUT') {
        activeElement.blur();
        console.log('🚫 Prevented auto-focus on input field');
      }
    }, 100); // Small delay to ensure DOM is ready

    return () => clearTimeout(timer);
  }, []); // Only run on mount

  // Prevent focus when service selection changes (only once)
  useEffect(() => {
    if (!isLoading && selectedService) {
      const timer = setTimeout(() => {
        const activeElement = document.activeElement as HTMLElement;
        if (activeElement && activeElement.tagName === 'INPUT') {
          activeElement.blur();
        }
      }, 50);

      return () => clearTimeout(timer);
    }
  }, [selectedService]); // Removed isLoading to prevent loops

  // Load configuration from database on component mount - only run once
  useEffect(() => {
    let isMounted = true;
    let hasStarted = false; // Prevent multiple concurrent loads

    const loadConfig = async () => {
      // Prevent multiple concurrent loads
      if (hasStarted) {
        console.log('⚠️ Load config already in progress, skipping...');
        return;
      }
      hasStarted = true;
      console.log('🚀 ASRSettings loadConfig() starting - ID:', mountId.current);

      try {
        console.log('🚀 SettingsContent: loadConfig() started');
        setIsLoading(true);

        // Initialize database if needed (with protection)
        console.log('📊 SettingsContent: Initializing database...');
        try {
          await DatabaseService.initDatabase();
          console.log('✅ SettingsContent: Database initialized');
        } catch (dbError) {
          if (dbError instanceof Error && dbError.message.includes('already initialized')) {
            console.log('📦 Database already initialized, skipping');
          } else {
            throw dbError;
          }
        }

        // Try to migrate from localStorage (only if database is empty)
        console.log('📦 SettingsContent: Checking for localStorage migration...');
        try {
          await StorageMigration.migrateFromLocalStorage();
          console.log('✅ SettingsContent: Migration check completed');
        } catch (migrationError) {
          if (migrationError instanceof Error && migrationError.message.includes('already checked')) {
            console.log('📦 Migration already checked, skipping');
          } else {
            throw migrationError;
          }
        }

        // Load current configuration
        console.log('📥 SettingsContent: Loading ASR configuration...');
        const config = await DatabaseService.getAsrConfig();

        if (isMounted) {
          if (config) {
            console.log('📥 Loaded config from database:', config.service_provider, 'updated_at:', config.updated_at);
            // Only apply database config if user hasn't made a selection
            if (!hasUserSelection) {
              setSelectedService(config.service_provider === 'local' ? ServiceProvider.Local : ServiceProvider.Cloud);
            } else {
              console.log('🔄 User has made selection, keeping user choice');
            }
            setApiKey(config.cloud_api_key || '');
            setLocalApiKey(config.local_api_key || '');
            setLocalEndpoint(config.local_endpoint || 'http://localhost:5001/inference');
            setCloudEndpoint(config.cloud_endpoint || 'https://api.siliconflow.cn/v1/audio/transcriptions');

            // Auto run health check after loading config - DISABLED to prevent infinite loops
            // if (config.service_provider && (config.local_endpoint || config.cloud_endpoint)) {
            //   setTimeout(() => {
            //     if (isMounted) { // Check if still mounted before running health check
            //       checkHealthStatus(config.service_provider, config.local_endpoint || config.cloud_endpoint || '');
            //     }
            //   }, 1000); // Delay 1 second after config load
            // }
          } else {
            // No config found - only set default if user hasn't already made a selection
            if (!hasUserSelection && selectedService === undefined) {
              console.log('📥 No config found, setting default to Cloud');
              setSelectedService(ServiceProvider.Cloud);
            } else {
              console.log('📥 No config found, keeping user selection:', selectedService);
            }
          }
          setHasLoadedFromDatabase(true);
        }
      } catch (error) {
        console.error('Failed to load ASR configuration:', error);
        if (isMounted) {
          // Only set fallback if user hasn't made a selection yet
          if (!hasUserSelection && selectedService === undefined) {
            setSelectedService(ServiceProvider.Cloud); // Fallback to Cloud
            console.log('⚠️ Error loading config, set fallback to Cloud');
          }
        }
      } finally {
        if (isMounted) {
          setIsLoading(false);
        }
        hasStarted = false; // Reset flag
      }
    };

    // Small delay to ensure component is fully mounted before loading
    const timeoutId = setTimeout(loadConfig, 100);

    return () => {
      isMounted = false;
      hasStarted = false;
      clearTimeout(timeoutId);
    };
  }, []); // Empty dependency array - run only once

  // Manual refresh function for debugging
  const manualRefreshConfig = useCallback(async () => {
    addDebugLog('🔄 Manual config refresh requested by user');
    try {
      setIsLoading(true);
      await DatabaseService.initDatabase();
      const config = await DatabaseService.getAsrConfig();
      
      if (config) {
        addDebugLog(`✅ Manual refresh found config: ${config.service_provider}`);
        setSelectedService(config.service_provider === 'local' ? ServiceProvider.Local : ServiceProvider.Cloud);
        setApiKey(config.cloud_api_key || '');
        setLocalApiKey(config.local_api_key || '');
        setLocalEndpoint(config.local_endpoint || 'http://localhost:5001/inference');
        setCloudEndpoint(config.cloud_endpoint || 'https://api.siliconflow.cn/v1/audio/transcriptions');
      } else {
        addDebugLog('📥 Manual refresh found no config');
      }
    } catch (error) {
      addDebugLog(`❌ Manual refresh failed: ${error}`);
    } finally {
      setIsLoading(false);
    }
  }, [addDebugLog]);

  // Save configuration to database - simple immediate save
  const saveConfiguration = useCallback(async () => {
    // Don't save if still loading or service is not set
    if (selectedService === undefined) {
      return;
    }

    try {
      setIsSaving(true);

      const config: AsrConfigRequest = {
        service_provider: selectedService === ServiceProvider.Local ? 'local' : 'cloud',
        local_endpoint: localEndpoint || undefined,
        local_api_key: localApiKey || undefined,
        cloud_endpoint: cloudEndpoint || undefined,
        cloud_api_key: apiKey || undefined,
      };

      // Debug: Log the values being saved
      console.log('💾 Saving ASR config:');
      console.log('  - service_provider:', config.service_provider);
      console.log('  - local_api_key present:', !!config.local_api_key);
      console.log('  - local_api_key length:', config.local_api_key?.length || 0);
      console.log('  - local_api_key preview:', config.local_api_key?.substring(0, 20) || 'none');
      console.log('  - cloud_api_key present:', !!config.cloud_api_key);
      console.log('  - cloud_api_key length:', config.cloud_api_key?.length || 0);

      await DatabaseService.saveAsrConfig(config);
      console.log('✅ ASR configuration saved successfully');

      // Trigger health check after saving - DISABLED to prevent infinite loops
      // const endpoint = selectedService === ServiceProvider.Local ? localEndpoint : cloudEndpoint;
      // if (endpoint) {
      //   setTimeout(() => {
      //     checkHealthStatus(
      //       selectedService === ServiceProvider.Local ? 'local' : 'cloud',
      //       endpoint
      //     );
      //   }, 500);
      // }
    } catch (error) {
      console.error('Failed to save ASR configuration:', error);
    } finally {
      setIsSaving(false);
    }
  }, [selectedService, apiKey, localApiKey, localEndpoint, cloudEndpoint, checkHealthStatus]);

  // Handle service selection change
  const handleServiceChange = useCallback((service: ServiceProvider) => {
    setSelectedService(service);
    setHasUserSelection(true);
    console.log('🎯 User selected service:', service);
    // Note: No automatic save - user must click Save button
  }, []);

  // onChange handlers for input fields (no auto-save)
  const handleApiKeyChange = useCallback((value: string) => {
    setApiKey(value);
  }, []);

  const handleLocalApiKeyChange = useCallback((value: string) => {
    setLocalApiKey(value);
  }, []);

  const handleCloudEndpointChange = useCallback((value: string) => {
    setCloudEndpoint(value);
  }, []);

  const handleLocalEndpointChange = useCallback((value: string) => {
    setLocalEndpoint(value);
  }, []);

  // Check if endpoint is HTTP (insecure)
  const isEndpointInsecure = (endpoint: string) => {
    return endpoint.startsWith('http://');
  };

  
// Handle file selection
  const handleFileSelect = (event: React.ChangeEvent<HTMLInputElement>) => {
    const file = event.target.files?.[0];
    if (!file) {
      setSelectedFile(null);
      return;
    }

    // Check file type
    if (!file.name.toLowerCase().endsWith('.wav')) {
      setAsrTestMessage('❌ Please select a WAV file');
      setSelectedFile(null);
      return;
    }

    // Check file size (2MB limit)
    if (file.size > 2 * 1024 * 1024) {
      setAsrTestMessage('❌ File too large (max 2MB)');
      setSelectedFile(null);
      return;
    }

    setSelectedFile(file);
    setAsrTestMessage(`Selected: ${file.name} (${Math.round(file.size / 1024)}KB)`);
    addDebugLog('📁 Selected file: ' + file.name);
  };

  // ASR transcription test function
  const handleAsrTest = async () => {
    if (isTestingAsr || !selectedFile) {
      if (!selectedFile) {
        setAsrTestMessage('Please select a WAV file first');
      }
      return;
    }

    // Check if we're in Tauri environment
    if (!isTauriEnvironment()) {
      setAsrTestMessage('❌ ASR testing only available in Tauri desktop app');
      addDebugLog('⚠️ ASR test not available in browser - requires Tauri desktop app');
      return;
    }

    try {
      // Save current configuration first
      await saveConfiguration();

      // Get current config
      const config = await DatabaseService.getAsrConfig();
      if (!config) {
        setAsrTestMessage('No configuration found');
        return;
      }

      // Determine endpoint and API key
      const endpoint = config.service_provider === 'local'
        ? config.local_endpoint
        : config.cloud_endpoint;
      const apiKey = config.service_provider === 'local'
        ? config.local_api_key
        : config.cloud_api_key;

      if (!endpoint) {
        setAsrTestMessage('No endpoint configured');
        return;
      }

      addDebugLog('🔧 Testing with service: ' + config.service_provider);
      addDebugLog('📊 File size: ' + selectedFile.size + ' bytes');
      addDebugLog('🔑 API Key present: ' + (apiKey ? 'YES (' + apiKey.length + ' chars)' : 'NO'));
      if (apiKey && apiKey.length > 0) {
        addDebugLog('🔑 API Key preview: ' + apiKey.substring(0, Math.min(10, apiKey.length)) + '...');
      }

      setIsTestingAsr(true);
      setAsrResult(null);
      setAsrTestMessage('Processing audio file...');

      // Convert file to base64 for transfer to Rust
      const arrayBuffer = await selectedFile.arrayBuffer();
      const uint8Array = new Uint8Array(arrayBuffer);
      const base64Data = btoa(String.fromCharCode(...uint8Array));

      // Call Rust backend for ASR transcription
      const response = await invoke('test_asr_transcription', {
        request: {
          audio_file_data: base64Data,
          file_name: selectedFile.name,
          service_provider: config.service_provider,
          endpoint: endpoint,
          api_key: apiKey || undefined
        }
      }) as {
        success: boolean;
        transcription?: string;
        processing_time_ms: number;
        file_size: number;
        message: string;
        status_code?: number;
      };

      addDebugLog('📊 ASR response: ' + JSON.stringify(response));

      if (response.success && response.transcription) {
        setAsrResult(response.transcription);
        setAsrTestMessage(`✅ Transcribed in ${response.processing_time_ms}ms (${response.file_size} bytes)`);
        addDebugLog('✅ ASR success: ' + response.transcription);
      } else {
        setAsrTestMessage('❌ ' + response.message);
        addDebugLog('❌ ASR failed: ' + response.message);
      }

    } catch (error) {
      const errorMessage = error instanceof Error ? error.message : String(error);
      setAsrTestMessage('❌ Error: ' + errorMessage);
      addDebugLog('💥 ASR test error: ' + errorMessage);
    } finally {
      setIsTestingAsr(false);
    }
  };

  return (
    <div className="max-w-3xl animate-in fade-in duration-500">
      <div className="flex justify-between items-center mb-6">
        <div className="flex items-center space-x-3">
          <h2 className="text-2xl font-bold text-gray-900">ASR Service Settings</h2>
          
          {/* Debug refresh button */}
          <Button
            onClick={manualRefreshConfig}
            loading={isLoading}
            variant="secondary"
            size="sm"
            className="text-xs bg-blue-50 hover:bg-blue-100 text-blue-600 border-blue-200"
          >
            🔄 Debug Refresh
          </Button>

          {/* Health Status Indicator */}
          {hasLoadedFromDatabase && healthStatus !== 'idle' && (
            <div className={`flex items-center px-3 py-1 rounded-full text-xs font-medium ${
              healthStatus === 'healthy'
                ? 'bg-green-100 text-green-800 border border-green-200'
                : healthStatus === 'checking'
                ? 'bg-yellow-100 text-yellow-800 border border-yellow-200'
                : 'bg-red-100 text-red-800 border border-red-200'
            }`}>
              {healthStatus === 'checking' && (
                <div className="animate-spin rounded-full h-3 w-3 border-b-2 border-yellow-600 mr-1"></div>
              )}
              {healthStatus === 'healthy' && (
                <div className="w-3 h-3 bg-green-500 rounded-full mr-1"></div>
              )}
              {healthStatus === 'unhealthy' && (
                <div className="w-3 h-3 bg-red-500 rounded-full mr-1"></div>
              )}
              {healthStatus === 'checking' ? 'Checking' :
               healthStatus === 'healthy' ? 'Healthy' : 'Unhealthy'}
            </div>
          )}
        </div>

        <div className="flex items-center space-x-3">
          {isLoading && (
            <span className="flex items-center text-sm text-gray-500">
              <div className="animate-spin rounded-full h-4 w-4 border-b-2 border-primary-500 mr-2"></div>
              Loading...
            </span>
          )}
          {isSaving && (
            <span className="flex items-center text-sm text-blue-600">
              <div className="animate-spin rounded-full h-4 w-4 border-b-2 border-blue-500 mr-2"></div>
              Saving...
            </span>
          )}
          {status === 'success' && !isTesting && (
            <span className="flex items-center text-sm text-green-600">
              <Check className="w-4 h-4 mr-1" /> Configuration Saved
            </span>
          )}
        </div>
      </div>

      {/* Service Selection */}
      <section className="bg-white rounded-xl border border-gray-200 p-6 mb-6 shadow-sm">
        <h3 className="text-lg font-semibold text-gray-900 mb-4">Voice Recognition Provider</h3>
        <div className="grid gap-4">
          <ServiceOption
            id={ServiceProvider.Local}
            title="Local ASR"
            description="Runs on device. Privacy focused. Requires powerful GPU."
            icon={<Server className="w-6 h-6" />}
            selected={selectedService === ServiceProvider.Local}
            onSelect={() => handleServiceChange(ServiceProvider.Local)}
            disabled={isLoading}
          />
          <ServiceOption
            id={ServiceProvider.Cloud}
            title="Cloud ASR"
            description="Ultra-fast cloud inference. Supports multiple providers (Whisper, SenseVoice). Requires API Key."
            icon={<Cloud className="w-6 h-6" />}
            selected={selectedService === ServiceProvider.Cloud}
            onSelect={() => handleServiceChange(ServiceProvider.Cloud)}
            disabled={isLoading}
          />
        </div>
      </section>

      {/* Configuration */}
      <section className="bg-white rounded-xl border border-gray-200 p-6 shadow-sm">
        <h3 className="text-lg font-semibold text-gray-900 mb-4">Connection Config</h3>
        <div className="space-y-4">
          {hasLoadedFromDatabase && selectedService === ServiceProvider.Local ? (
            <>
              <Input
                label="Local ASR API Endpoint"
                value={localEndpoint}
                onChange={handleLocalEndpointChange}
                placeholder="http://localhost:5001/inference"
                disabled={isLoading}
                autoFocus={false}
                error={isEndpointInsecure(localEndpoint) ? 'Warning: Using HTTP is insecure. Credentials may be transmitted unencrypted.' : undefined}
              />
              <Input
                label="Local ASR API Key"
                type="password"
                value={localApiKey}
                onChange={handleLocalApiKeyChange}
                placeholder="Enter your local ASR API key..."
                required
                disabled={isLoading}
                autoFocus={false}
              />
              <p className="text-sm text-amber-600 bg-amber-50 border border-amber-200 rounded-lg p-3">
                ⚠️ <strong>Security Notice:</strong> API keys are sensitive credentials. Never share them publicly or commit to version control. Use HTTPS endpoints when possible.
              </p>
              <p className="text-sm text-gray-500">
                Local ASR runs on your device. Configure the endpoint where your local ASR service is running.
              </p>
            </>
          ) : hasLoadedFromDatabase && selectedService === ServiceProvider.Cloud ? (
            <>
              <Input
                label="Cloud ASR API Endpoint"
                value={cloudEndpoint}
                onChange={handleCloudEndpointChange}
                placeholder="https://api.example.com/v1/audio/transcriptions"
                disabled={isLoading}
                autoFocus={false}
                error={isEndpointInsecure(cloudEndpoint) ? 'Warning: Using HTTP is insecure. Credentials may be transmitted unencrypted.' : undefined}
              />
              <Input
                label="Cloud ASR API Key"
                type="password"
                value={apiKey}
                onChange={handleApiKeyChange}
                placeholder="sk-..."
                required
                disabled={isLoading}
                autoFocus={false}
              />
              <p className="text-sm text-amber-600 bg-amber-50 border border-amber-200 rounded-lg p-3">
                ⚠️ <strong>Security Notice:</strong> API keys are sensitive credentials. Never share them publicly or commit to version control. Use HTTPS endpoints when possible.
              </p>
              <p className="text-sm text-gray-500">
                Cloud ASR supports multiple providers (SiliconFlow, Groq). The endpoint determines which provider to use.
              </p>
            </>
          ) : (
            <div className="text-center py-8 text-gray-500">
              <div className="animate-spin rounded-full h-6 w-6 border-b-2 border-primary-500 mx-auto mb-3"></div>
              <p>Loading configuration...</p>
            </div>
          )}

          <div className="flex items-center space-x-4 pt-4 border-t border-gray-200">
            <Button
              onClick={saveConfiguration}
              loading={isSaving}
              disabled={isLoading || isTesting || !hasLoadedFromDatabase}
              variant="primary"
              icon={<Save className="w-4 h-4"/>}
            >
              Save Configuration
            </Button>
            <Button
              onClick={handleAsrTest}
              loading={isTestingAsr}
              disabled={isLoading || isSaving || !hasLoadedFromDatabase || !selectedFile}
              icon={<Globe className="w-4 h-4"/>}
            >
              Test ASR with WAV File
            </Button>

            {/* File Input */}
            <div className="flex items-center space-x-3">
              <label className="relative cursor-pointer">
                <input
                  type="file"
                  accept=".wav"
                  onChange={handleFileSelect}
                  className="absolute inset-0 w-full h-full opacity-0 cursor-pointer"
                  disabled={isTestingAsr}
                />
                <div className={`flex items-center px-4 py-2 border-2 border-dashed rounded-lg ${
                  selectedFile
                    ? 'border-green-300 bg-green-50 text-green-800'
                    : 'border-gray-300 bg-gray-50 text-gray-700 hover:border-gray-400'
                } transition-colors`}>
                  <Globe className="w-4 h-4 mr-2" />
                  <span className="text-sm font-medium">
                    {selectedFile ? selectedFile.name : 'Choose WAV File'}
                  </span>
                </div>
              </label>
            </div>

            {/* ASR Test Results */}
            {asrTestMessage && (
              <div className={`mt-4 p-4 rounded-lg border ${
                asrResult
                  ? 'bg-green-50 border-green-200'
                  : 'bg-red-50 border-red-200'
              }`}>
                <div className="flex items-start space-x-2">
                  {asrResult ? (
                    <Check className="w-5 h-5 text-green-600 mt-0.5 flex-shrink-0" />
                  ) : (
                    <AlertTriangle className="w-5 h-5 text-red-600 mt-0.5 flex-shrink-0" />
                  )}
                  <div className="flex-1">
                    <p className={`text-sm font-medium ${
                      asrResult ? 'text-green-800' : 'text-red-800'
                    }`}>
                      {asrTestMessage}
                    </p>
                    {asrResult && (
                      <div className="mt-3 p-3 bg-white rounded border border-green-200">
                        <p className="text-xs text-gray-500 mb-2 font-medium">Transcription Result:</p>
                        <p className="text-sm text-gray-900 leading-relaxed whitespace-pre-wrap">
                          {asrResult}
                        </p>
                      </div>
                    )}
                  </div>
                </div>
              </div>
            )}
          </div>
        </div>
      </section>

      {/* Debug Panel Toggle */}
      <div className="mt-6 flex justify-center">
        <Button
          onClick={() => setShowDebugPanel(!showDebugPanel)}
          variant="secondary"
          size="sm"
          className="text-xs bg-gray-100 hover:bg-gray-200 text-gray-600 border-gray-300"
        >
          {showDebugPanel ? 'Hide' : 'Show'} Debug Panel ({debugLogs.length} logs)
        </Button>
      </div>

      {/* Debug Panel */}
      {showDebugPanel && (
        <section className="mt-4 bg-gray-900 text-green-400 rounded-xl border border-gray-700 p-4 shadow-sm font-mono text-xs">
          <div className="flex justify-between items-center mb-3">
            <h3 className="text-sm font-bold text-green-300">🔍 ASR Debug Console</h3>
            <div className="flex space-x-2">
              <Button
                onClick={() => {
                  // 复制所有日志到剪贴板
                  navigator.clipboard.writeText(debugLogs.join('\n'));
                }}
                variant="secondary"
                size="sm"
                className="text-xs bg-gray-800 hover:bg-gray-700 text-gray-300 border-gray-600"
              >
                Copy
              </Button>
              <Button
                onClick={() => setDebugLogs([])}
                variant="secondary"
                size="sm"
                className="text-xs bg-gray-800 hover:bg-gray-700 text-gray-300 border-gray-600"
              >
                Clear
              </Button>
            </div>
          </div>
          <div className="max-h-80 overflow-y-auto space-y-1">
            {debugLogs.length === 0 ? (
              <div className="text-gray-500 text-center py-4">No debug logs yet. Try performing an action...</div>
            ) : (
              debugLogs.map((log, index) => (
                <div key={index} className="border-b border-gray-800 pb-1 last:border-b-0">
                  {log}
                </div>
              ))
            )}
          </div>
          <div className="mt-3 pt-3 border-t border-gray-700">
            <div className="flex justify-between items-center text-xs text-gray-400">
              <span>Environment: {typeof window !== 'undefined' && window.__TAURI__ ? 'Tauri Desktop' : 'Browser'}</span>
              <span>Logs: {debugLogs.length}/50</span>
            </div>
          </div>
        </section>
      )}
    </div>
  );
};

// Shortcuts Settings
export const ShortcutSettings: React.FC = () => {
    const [transcribeKey, setTranscribeKey] = useState('F4');
    const [translateKey, setTranslateKey] = useState('Shift + F4');
    const [delay, setDelay] = useState(0.3);
    const [antiTouch, setAntiTouch] = useState(true);
    const [saveWavFiles, setSaveWavFiles] = useState(true);
    const [isSaving, setIsSaving] = useState(false);
    const [hasLoadedFromDatabase, setHasLoadedFromDatabase] = useState(false);

    // Load hotkey configuration from database on component mount
    useEffect(() => {
        let isMounted = true;

        const loadHotkeyConfiguration = async () => {
            try {
                // Initialize database using DatabaseService (with built-in protection)
                await DatabaseService.initDatabase();

                if (typeof window !== 'undefined' && window.__TAURI__) {
                    // In Tauri environment, load from SQLite database
                    const config = await invoke<HotkeyConfig | null>('get_hotkey_config');
                    if (isMounted && config) {
                        console.log('📥 Loaded hotkey config from database:', {
                            transcribe_key: config.transcribe_key,
                            translate_key: config.translate_key,
                            trigger_delay_ms: config.trigger_delay_ms,
                            anti_mistouch_enabled: config.anti_mistouch_enabled,
                            updated_at: config.updated_at,
                        });
                        setTranscribeKey(config.transcribe_key);
                        setTranslateKey(config.translate_key);
                        setDelay(config.trigger_delay_ms / 1000); // Convert ms to seconds
                        setAntiTouch(config.anti_mistouch_enabled);
                        setSaveWavFiles(config.save_wav_files);
                    } else if (isMounted) {
                        console.log('📥 No hotkey config found, using defaults');
                    }
                }
                setHasLoadedFromDatabase(true);
            } catch (error) {
                console.error('Failed to load hotkey configuration:', error);
                setHasLoadedFromDatabase(true);
            }
        };

        loadHotkeyConfiguration();

        return () => {
            isMounted = false;
        };
    }, []);

    // Save hotkey configuration
    const saveHotkeyConfiguration = useCallback(async () => {
        try {
            setIsSaving(true);

            const config: HotkeyConfigRequest = {
                transcribe_key: transcribeKey,
                translate_key: translateKey,
                trigger_delay_ms: Math.round(delay * 1000), // Convert seconds to ms
                anti_mistouch_enabled: antiTouch,
                save_wav_files: saveWavFiles,
            };

            console.log('💾 Saving hotkey config:');
            console.log('  - transcribe_key:', config.transcribe_key);
            console.log('  - translate_key:', config.translate_key);
            console.log('  - trigger_delay_ms:', config.trigger_delay_ms);
            console.log('  - anti_mistouch_enabled:', config.anti_mistouch_enabled);
            console.log('  - save_wav_files:', config.save_wav_files);

            if (typeof window !== 'undefined' && window.__TAURI__) {
                await invoke('save_hotkey_config', { request: config });
                console.log('✅ Hotkey configuration saved successfully');
            }
        } catch (error) {
            console.error('Failed to save hotkey configuration:', error);
        } finally {
            setIsSaving(false);
        }
    }, [transcribeKey, translateKey, delay, antiTouch, saveWavFiles]);

    return (
        <div className="max-w-3xl animate-in fade-in duration-500">
             <h2 className="text-2xl font-bold text-gray-900 mb-6">Shortcuts & Behaviors</h2>
             
             <section className="bg-white rounded-xl border border-gray-200 p-6 mb-6 shadow-sm">
                <h3 className="text-lg font-semibold text-gray-900 mb-4">Global Hotkeys</h3>
                <div className="space-y-6">
                    <HotkeyInput label="Start Transcription" value={transcribeKey} onChange={setTranscribeKey} placeholder="Press keys..." autoFocus={false} />
                    <HotkeyInput label="Start Translation" value={translateKey} onChange={setTranslateKey} placeholder="Press keys..." autoFocus={false} />
                </div>
             </section>

             <section className="bg-white rounded-xl border border-gray-200 p-6 shadow-sm">
                <h3 className="text-lg font-semibold text-gray-900 mb-4">Prevention</h3>
                <div className="space-y-4">
                    <Input label="Trigger Delay (seconds)" type="number" step={0.1} value={delay} onChange={setDelay} unit="s" autoFocus={false} />
                    <ToggleInput label="Enable Anti-Mistouch" checked={antiTouch} onChange={setAntiTouch} description="Prevents accidental recording when holding keys briefly." />
                </div>
             </section>

    

             {/* Save Button */}
             <div className="flex justify-end space-x-3 mt-8">
                 <Button
                     onClick={saveHotkeyConfiguration}
                     disabled={isSaving || !hasLoadedFromDatabase}
                     className="px-6 py-2 text-sm font-medium"
                 >
                     {isSaving ? (
                         <div className="flex items-center">
                             <div className="animate-spin rounded-full h-4 w-4 border-2 border-white border-t-transparent mr-2" />
                             Saving...
                         </div>
                     ) : (
                         <div className="flex items-center">
                             <Save className="w-4 h-4 mr-2" />
                             Save Shortcuts
                         </div>
                     )}
                 </Button>
             </div>
        </div>
    )
}

export const PlaceholderSettings: React.FC<{title: string}> = ({title}) => (
    <div className="max-w-3xl animate-in fade-in duration-500">
        <h2 className="text-2xl font-bold text-gray-900 mb-6">{title}</h2>
        <div className="bg-white rounded-xl border border-gray-200 p-12 text-center shadow-sm">
            <div className="inline-block p-4 rounded-full bg-gray-50 mb-4">
                <Sliders className="w-8 h-8 text-gray-400" />
            </div>
            <h3 className="text-gray-900 font-medium text-lg">Coming Soon</h3>
            <p className="text-gray-500 mt-2">This settings module is currently under development.</p>
        </div>
    </div>
);

import { Sliders } from 'lucide-react';