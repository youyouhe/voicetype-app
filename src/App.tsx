import React, { useState, useEffect } from 'react';
import { Mic, Settings } from 'lucide-react';
import { AppStatus, HistoryItem } from './types';
import { StatusIndicator } from './components/MainView/StatusCircle';
import { VoiceAssistantPanel } from './components/MainView/VoiceAssistantPanel';
import { Button } from './components/ui/Button';
import { SettingsView } from './components/SettingsView';
import { DashboardView } from './components/DashboardView';
import { TauriService } from './services/tauriService';
import { ThemeProvider } from './contexts/ThemeContext';



// --- Main App Component ---

const App: React.FC = () => {
  const [currentView, setCurrentView] = useState<'dashboard' | 'settings'>('dashboard');
  const [appStatus, setAppStatus] = useState<AppStatus>(AppStatus.Idle);
  const [activeSettingsTab, setActiveSettingsTab] = useState('asr');
  const [isVoiceAssistantRunning, setIsVoiceAssistantRunning] = useState(false);
  const [voiceAssistantState, setVoiceAssistantState] = useState<string>('Idle');
  
  // History State
  const [history, setHistory] = useState<HistoryItem[]>([]);
  const [lastSaved, setLastSaved] = useState<Date | null>(null);

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
    <header className="bg-white/80 backdrop-blur-md border-b border-gray-200 px-6 py-3 sticky top-0 z-50">
      <div className="flex items-center justify-between max-w-7xl mx-auto">
        <div className="flex items-center space-x-4">
          <div className="bg-primary-50 p-2 rounded-lg">
             <Mic className="w-5 h-5 text-primary-600" />
          </div>
          <div>
            <h1 className="text-lg font-bold text-gray-900 leading-none">Flash-Input</h1>
            <p className="text-xs text-gray-500 mt-0.5">Tauri Client v1.0.0</p>
          </div>
          <div className="hidden sm:block h-6 w-px bg-gray-200 mx-2" />
          <div className="hidden sm:block">
            <StatusIndicator status={appStatus} />
          </div>
        </div>

        <div className="flex items-center space-x-2">
          <Button
            variant={isVoiceAssistantRunning ? "secondary" : "ghost"}
            size="sm"
            icon={<Mic className="w-4 h-4"/>}
            onClick={isVoiceAssistantRunning ? stopVoiceAssistant : startVoiceAssistant}
            disabled={appStatus === AppStatus.Recording || appStatus === AppStatus.Processing}
            className="flex"
            title={isVoiceAssistantRunning ? "Stop Voice Assistant" : "Start Voice Assistant"}
          >
            <span className="hidden sm:inline">{isVoiceAssistantRunning ? 'Stop' : 'Start'}</span>
            <span className="sm:hidden">{isVoiceAssistantRunning ? 'Stop' : '▶'}</span>
          </Button>
          <Button
            variant="ghost"
            size="sm"
            icon={<Settings className="w-4 h-4"/>}
            onClick={() => setCurrentView(currentView === 'dashboard' ? 'settings' : 'dashboard')}
          >
            {currentView === 'dashboard' ? 'Settings' : 'Dashboard'}
          </Button>
        </div>
      </div>
    </header>
  );

  

  

  return (
    <ThemeProvider>
      <div className="min-h-screen bg-[#F3F4F6] text-gray-900 font-sans selection:bg-primary-100 selection:text-primary-900">
        <TopBar />
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
        <div className="lg:hidden fixed bottom-0 left-0 right-0 bg-white border-t border-gray-200 p-4 flex justify-between items-center z-40">
           <div className="flex items-center space-x-2">
              <div className={`w-2 h-2 rounded-full ${appStatus === AppStatus.Recording ? 'bg-red-500 animate-pulse' : 'bg-gray-400'}`} />
              <span className="text-sm font-medium">{appStatus === AppStatus.Idle ? 'Idle' : 'Active'}</span>
           </div>
           <span className="text-xs text-gray-400">Cloud ASR</span>
        </div>
      </div>
    </ThemeProvider>
  );
};

export default App;