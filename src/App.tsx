import React, { useState, useEffect } from 'react';
import { Mic, Settings, Languages } from 'lucide-react';
import { AppStatus, HistoryItem } from './types';
import { StatusIndicator } from './components/MainView/StatusCircle';
import { VoiceAssistantPanel } from './components/MainView/VoiceAssistantPanel';
import { Button } from './components/ui/Button';
import { SettingsView } from './components/SettingsView';
import { DashboardView } from './components/DashboardView';
import { TauriService } from './services/tauriService';
import { LanguageProvider, useLanguage } from './contexts/LanguageContext';

// --- Inner App Component (uses language context) ---
const AppContent: React.FC = () => {
  const { t, language, setLanguage } = useLanguage();
  const [currentView, setCurrentView] = useState<'dashboard' | 'settings'>('dashboard');
  const [appStatus, setAppStatus] = useState<AppStatus>(AppStatus.Idle);
  const [activeSettingsTab, setActiveSettingsTab] = useState('asr');
  const [isVoiceAssistantRunning, setIsVoiceAssistantRunning] = useState(false);
  const [voiceAssistantState, setVoiceAssistantState] = useState<string>('Idle');

  // History State
  const [history, setHistory] = useState<HistoryItem[]>([]);
  const [lastSaved, setLastSaved] = useState<Date | null>(null);

  // Driver check state
  const [driverCheckDone, setDriverCheckDone] = useState(false);
  const [driverWarning, setDriverWarning] = useState<string | null>(null);

  // Set window title on mount (only in Tauri)
  useEffect(() => {
    const setWindowTitle = async () => {
      try {
        // Dynamic import to avoid errors in web environment
        const webviewWindow = await import('@tauri-apps/api/webviewWindow');
        const window = await webviewWindow.getCurrentWebviewWindow();
        await window.setTitle('VoiceType - 语音助手');
      } catch (err) {
        // Silently fail - not in Tauri environment
        console.log('Not in Tauri environment');
      }
    };
    setWindowTitle();
  }, []);

  // Check NVIDIA driver on mount
  useEffect(() => {
    const checkDriver = async () => {
      try {
        const info = await TauriService.checkNvidiaDriver();
        console.log('🔍 NVIDIA Driver Check:', info);

        if (!info.installed) {
          setDriverWarning('No NVIDIA driver detected. CUDA acceleration will not work.');
        } else if (!info.is_compatible) {
          setDriverWarning(info.error_message || 'NVIDIA driver version is too old for CUDA 11.8.');
        }

        setDriverCheckDone(true);
      } catch (error) {
        console.log('ℹ️ Driver check skipped (not in Tauri or check failed):', error);
        setDriverCheckDone(true); // Silently continue if check fails
      }
    };
    checkDriver();
  }, []);

  // Load history from database
  const loadHistory = async () => {
    try {
      const records = await TauriService.getHistoryRecords(10);
      console.log('📋 Raw records from database:', records);
      const historyItems: HistoryItem[] = records
        .filter((record: any) => record.output_text) // Filter out records without output text
        .map((record: any) => ({
          id: record.id,
          type: record.record_type, // record_type, not type
          text: record.output_text, // output_text, not text
          timestamp: new Date(record.created_at), // created_at, not timestamp
        }));
      setHistory(historyItems);
      console.log('📋 Loaded history:', historyItems.length, 'records');
      console.log('📋 History items:', historyItems);
    } catch (error) {
      console.error('Failed to load history:', error);
    }
  };

  // History functions
  const handleClearHistory = () => {
    setHistory([]);
    localStorage.removeItem('voice-assistant-history');
  };

  const startRecording = (type: 'transcribe' | 'translate') => {
    console.log(`Starting ${type} recording...`);
    // Implementation will be added later
  };

  

  
  // Map Voice Assistant state to AppStatus
  const mapVoiceAssistantStateToAppStatus = (state: string): AppStatus => {
    switch (state) {
      case 'Recording':
      case 'RecordingTranslate':
        return AppStatus.Recording;
      case 'Processing':
      case 'Translating':
        return AppStatus.Processing;
      case 'Running':
        return AppStatus.Idle; // Running but not actively recording
      case 'Idle':
      default:
        return AppStatus.Idle;
    }
  };
  
  

  
  // Listen for Voice Assistant state changes via events
  useEffect(() => {
    console.log('🔧 Setting up Voice Assistant state event listener...');

    // Import the listen function dynamically
    const setupEventListener = async () => {
      try {
        const { listen } = await import('@tauri-apps/api/event');
        
        // Initial state check
        console.log('🚀 Getting initial state...');
        const initialState = await TauriService.getVoiceAssistantState();
        console.log('🎯 Initial Voice Assistant state:', initialState);
        console.log('🔍 State type:', typeof initialState);
        console.log('🔍 State length:', initialState.length);
        console.log('🔍 State includes "Running":', initialState.includes('Running'));
        
        const cleanInitialState = initialState.replace(/"/g, '').trim();
        console.log('🔧 Cleaned initial state:', cleanInitialState);

        const isInitiallyRunning = cleanInitialState === 'Running' ||
                                  cleanInitialState === 'Recording' ||
                                  cleanInitialState === 'RecordingTranslate' ||
                                  cleanInitialState === 'Processing' ||
                                  cleanInitialState === 'Translating';

        console.log('🟢 Initially running:', isInitiallyRunning);
        console.log('🔍 Running check - equals Running:', cleanInitialState === 'Running');
        console.log('🔍 Running check - equals Recording:', cleanInitialState === 'Recording');
        setIsVoiceAssistantRunning(isInitiallyRunning);
        setVoiceAssistantState(cleanInitialState);
        setAppStatus(mapVoiceAssistantStateToAppStatus(cleanInitialState));

        // Set up event listener for state changes
        console.log('👂 Setting up event listener for voice-assistant-state-changed...');
        const unlisten = await listen<string>('voice-assistant-state-changed', (event) => {
          console.log('📡 Received voice assistant state change event:', event.payload);

          const newState = event.payload;
          const isRunning = newState === 'Running' ||
                           newState === 'Recording' ||
                           newState === 'RecordingTranslate' ||
                           newState === 'Processing' ||
                           newState === 'Translating';

          console.log('🟢 Updated is running:', isRunning);
          console.log('🟢 Updated state:', newState);

          setIsVoiceAssistantRunning(isRunning);
          setVoiceAssistantState(newState);
          setAppStatus(mapVoiceAssistantStateToAppStatus(newState));
        });

        console.log('✅ Voice Assistant state event listener set up successfully');

        return unlisten;
      } catch (error) {
        console.error('❌ Failed to set up Voice Assistant state event listener:', error);
        return () => {}; // Return empty cleanup function
      }
    };

    let unlistenHandler: (() => void) | undefined;

    const setupCleanup = async () => {
      const unlisten = await setupEventListener();
      unlistenHandler = unlisten;
    };

    setupCleanup();

    return () => {
      console.log('🛑 Cleaning up Voice Assistant state event listener...');
      if (unlistenHandler) {
        unlistenHandler();
      }
    };
  }, []);

  // Listen for new history records and load initial history
  useEffect(() => {
    const setupHistoryListener = async () => {
      try {
        const { listen } = await import('@tauri-apps/api/event');

        // Load initial history
        await loadHistory();

        // Listen for new history records
        const unlisten = await listen('new-history-record', async () => {
          console.log('📋 New history record event received, reloading history...');
          await loadHistory();
          setLastSaved(new Date());
        });

        return unlisten;
      } catch (error) {
        console.error('Failed to set up history listener:', error);
        return () => {};
      }
    };

    let unlistenHandler: (() => void) | undefined;
    const setupCleanup = async () => {
      const unlisten = await setupHistoryListener();
      unlistenHandler = unlisten;
    };

    setupCleanup();

    return () => {
      if (unlistenHandler) {
        unlistenHandler();
      }
    };
  }, []);


  // Voice Assistant Control Functions
  const startVoiceAssistant = async () => {
    try {
      setAppStatus(AppStatus.Processing);
      const result = await TauriService.startVoiceAssistant();
      console.log('Voice Assistant started:', result);

      // Wait a bit for backend to initialize, then check actual state
      setTimeout(async () => {
        try {
          const actualState = await TauriService.getVoiceAssistantState();
          console.log('🔍 Actual state after start:', actualState);
          const cleanState = actualState.replace(/"/g, '').trim();
          const isActuallyRunning = cleanState === 'Running' ||
                                  cleanState === 'Recording' ||
                                  cleanState === 'RecordingTranslate' ||
                                  cleanState === 'Processing' ||
                                  cleanState === 'Translating';

          console.log('🟢 Actually running after start:', isActuallyRunning);
          setIsVoiceAssistantRunning(isActuallyRunning);
          setVoiceAssistantState(cleanState);
          setAppStatus(mapVoiceAssistantStateToAppStatus(cleanState));
        } catch (error) {
          console.error('Failed to verify Voice Assistant state:', error);
          setIsVoiceAssistantRunning(false);
          setAppStatus(AppStatus.Error);
        }
      }, 500); // Wait 500ms for backend to initialize

      // Set initial processing complete status while waiting for verification
      setAppStatus(AppStatus.Idle);

    } catch (error) {
      console.error('Failed to start Voice Assistant:', error);
      setIsVoiceAssistantRunning(false);
      setAppStatus(AppStatus.Error);
    }
  };

  const stopVoiceAssistant = async () => {
    try {
      setAppStatus(AppStatus.Processing);
      const result = await TauriService.stopVoiceAssistant();
      console.log('Voice Assistant stopped:', result);
      setIsVoiceAssistantRunning(false);
      setAppStatus(AppStatus.Idle);
    } catch (error) {
      console.error('Failed to stop Voice Assistant:', error);
      setIsVoiceAssistantRunning(false);
      setAppStatus(AppStatus.Error);
    }
  };

  const TopBar = () => (
    <header className="bg-white/80 dark:bg-slate-800/80 backdrop-blur-md border-b border-gray-200 dark:border-slate-700 px-6 py-3 sticky top-0 z-50 transition-colors duration-200">
      <div className="flex items-center justify-between max-w-7xl mx-auto">
        <div className="flex items-center space-x-4">
          <div className="bg-primary-50 dark:bg-primary-900/30 p-2 rounded-lg transition-colors duration-200">
             <Mic className="w-5 h-5 text-primary-600 dark:text-primary-400" />
          </div>
          <div>
            <h1 className="text-lg font-bold text-gray-900 dark:text-gray-100 leading-none transition-colors duration-200">{t.appName}</h1>
            <p className="text-xs text-gray-500 dark:text-gray-400 mt-0.5 transition-colors duration-200">{t.appVersion}</p>
          </div>
          <div className="hidden sm:block h-6 w-px bg-gray-200 dark:bg-slate-700 mx-2" />
          <div className="hidden sm:block">
            <StatusIndicator status={appStatus} />
          </div>
        </div>

        <div className="flex items-center space-x-2">
          {/* Language Switcher */}
          <Button
            variant="ghost"
            size="sm"
            icon={<Languages className="w-4 h-4"/>}
            onClick={() => setLanguage(language === 'zh-CN' ? 'en-US' : 'zh-CN')}
            title={t.language}
          >
            {language === 'zh-CN' ? 'EN' : '中'}
          </Button>

          <Button
            variant={isVoiceAssistantRunning ? "secondary" : "ghost"}
            size="sm"
            icon={<Mic className="w-4 h-4"/>}
            onClick={isVoiceAssistantRunning ? stopVoiceAssistant : startVoiceAssistant}
            disabled={appStatus === AppStatus.Recording || appStatus === AppStatus.Processing}
            className="flex"
            title={isVoiceAssistantRunning ? t.stopVoiceAssistant : t.startVoiceAssistant}
          >
            <span className="hidden sm:inline">{isVoiceAssistantRunning ? t.stop : t.start}</span>
            <span className="sm:hidden">{isVoiceAssistantRunning ? t.stop : '▶'}</span>
          </Button>
          <Button
            variant="ghost"
            size="sm"
            icon={<Settings className="w-4 h-4"/>}
            onClick={() => setCurrentView(currentView === 'dashboard' ? 'settings' : 'dashboard')}
          >
            {currentView === 'dashboard' ? t.settings : t.dashboard}
          </Button>
        </div>
      </div>
    </header>
  );

  

  

  return (
    <div className="min-h-screen bg-gray-50 dark:bg-slate-900 text-gray-900 dark:text-gray-100 font-sans selection:bg-primary-100 selection:text-primary-900 transition-colors duration-200">
        <TopBar />

        {/* Driver Warning Banner */}
        {driverCheckDone && driverWarning && (
          <div className="bg-amber-50 dark:bg-amber-900/20 border-b border-amber-200 dark:border-amber-800 px-6 py-3">
            <div className="max-w-7xl mx-auto flex items-start justify-between">
              <div className="flex items-start space-x-3">
                <svg className="w-5 h-5 text-amber-600 dark:text-amber-400 mt-0.5 flex-shrink-0" fill="currentColor" viewBox="0 0 20 20">
                  <path fillRule="evenodd" d="M8.257 3.099c.765-1.36 2.722-1.36 3.486 0l5.58 9.92c.75 1.334-.213 2.98-1.742 2.98H4.42c-1.53 0-2.493-1.646-1.743-2.98l5.58-9.92zM11 13a1 1 0 11-2 0 1 1 0 012 0zm-1-8a1 1 0 00-1 1v3a1 1 0 002 0V6a1 1 0 00-1-1z" clipRule="evenodd" />
                </svg>
                <div className="flex-1">
                  <p className="text-sm font-medium text-amber-800 dark:text-amber-200">
                    CUDA 驱动版本警告
                  </p>
                  <p className="text-sm text-amber-700 dark:text-amber-300 mt-1">
                    {driverWarning}
                  </p>
                  <p className="text-xs text-amber-600 dark:text-amber-400 mt-1">
                    请访问 <a href="https://www.nvidia.com/Download/index.aspx" target="_blank" rel="noopener noreferrer" className="underline hover:text-amber-800 dark:hover:text-amber-200">NVIDIA 官网</a> 下载最新驱动 (版本 522.xx 或更高)
                  </p>
                </div>
              </div>
              <button
                onClick={() => setDriverWarning(null)}
                className="text-amber-600 dark:text-amber-400 hover:text-amber-800 dark:hover:text-amber-200"
              >
                <svg className="w-5 h-5" fill="currentColor" viewBox="0 0 20 20">
                  <path fillRule="evenodd" d="M4.293 4.293a1 1 0 011.414 0L10 8.586l4.293-4.293a1 1 0 111.414 1.414L11.414 10l4.293 4.293a1 1 0 01-1.414 1.414L10 11.414l-4.293 4.293a1 1 0 01-1.414-1.414L8.586 10 4.293 5.707a1 1 0 010-1.414z" clipRule="evenodd" />
                </svg>
              </button>
            </div>
          </div>
        )}

        {currentView === 'dashboard' ? (
          <DashboardView
            appStatus={appStatus}
            history={history}
            lastSaved={lastSaved}
            startRecording={startRecording}
            handleClearHistory={handleClearHistory}
            startVoiceAssistant={startVoiceAssistant}
            stopVoiceAssistant={stopVoiceAssistant}
            isVoiceAssistantRunning={isVoiceAssistantRunning}
          />
        ) : (
          <SettingsView activeSettingsTab={activeSettingsTab} setActiveSettingsTab={setActiveSettingsTab} />
        )}

        {/* Mobile Status Bar (Visible only on small screens) */}
        <div className="lg:hidden fixed bottom-0 left-0 right-0 bg-white dark:bg-slate-800 border-t border-gray-200 dark:border-slate-700 p-4 flex justify-between items-center z-40 transition-colors duration-200">
           <div className="flex items-center space-x-2">
              <div className={`w-2 h-2 rounded-full ${appStatus === AppStatus.Recording ? 'bg-red-500 animate-pulse' : 'bg-gray-400 dark:bg-gray-500'}`} />
              <span className="text-sm font-medium text-gray-900 dark:text-gray-100 transition-colors duration-200">{appStatus === AppStatus.Idle ? t.idle : t.active}</span>
           </div>
           <span className="text-xs text-gray-400 dark:text-gray-500 transition-colors duration-200">Cloud ASR</span>
        </div>
      </div>
  );
};

// --- Wrapper with LanguageProvider ---
const App: React.FC = () => {
  return (
    <LanguageProvider>
      <AppContent />
    </LanguageProvider>
  );
};

export default App;