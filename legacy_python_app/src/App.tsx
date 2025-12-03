import React, { useEffect } from 'react';
import { Mic, Settings, Minimize, Keyboard, Clock, Trash2 } from 'lucide-react';
import { AppStatus, NavTab, HistoryItem } from './types';
import { StatusCircle } from './components/MainView/StatusCircle';
import { LiveData } from './components/MainView/LiveData';
import { Button } from './components/ui/Button';
import { PlaceholderSettings } from './components/SettingsView/SettingsContent';
import { useAppStore, setupTauriListeners } from './store';

// --- Constants ---
const AUTO_SAVE_INTERVAL_MS = 5000; // Auto-save every 5 seconds

// --- Main App Component ---
const App: React.FC = () => {
  const {
    // State
    isRecording,
    isProcessing,
    lastResult,
    currentView,
    activeSettingsTab,
    history,

    // Actions
    startRecording,
    setCurrentView,
    setActiveSettingsTab,
    addToHistory,
    clearHistory,
    updateSettings,
  } = useAppStore();

  // Auto-save Interval
  useEffect(() => {
    const intervalId = setInterval(() => {
      // Auto-save is handled by Zustand persistence if needed
    }, AUTO_SAVE_INTERVAL_MS);

    return () => clearInterval(intervalId);
  }, []);

  // Setup Tauri event listeners
  useEffect(() => {
    setupTauriListeners();
  }, []);

  const handleClearHistory = () => {
    clearHistory();
  };

  // Mock actions - these will be replaced by real Tauri calls
  const handleStartRecording = async (type: 'transcribe' | 'translate') => {
    if (isRecording || isProcessing) return;

    await startRecording(type);
  };

  const getDisplayStatus = () => {
    switch (true) { // This will be replaced with actual status
      case isRecording:
        return 'Listening...';
      case isProcessing:
        return type === 'translate' ? 'Translating...' : 'Processing Audio...';
      default:
        return 'Ready to Listen';
    }
  };

  const getCurrentAppStatus = (): AppStatus => {
    if (isRecording) return AppStatus.Recording;
    if (isProcessing) return AppStatus.Processing;
    return AppStatus.Idle;
  };

  const TopBar = () => (
    <header className="bg-white/80 backdrop-blur-md border-b border-gray-200 px-6 py-3 sticky top-0 z-50">
      <div className="flex items-center justify-between max-w-7xl mx-auto">
        <div className="flex items-center space-x-4">
          <div className="bg-primary-50 p-2 rounded-lg">
             <Mic className="w-5 h-5 text-primary-600" />
          </div>
          <div>
            <h1 className="text-lg font-bold text-gray-900 leading-none">EchoType</h1>
            <p className="text-xs text-gray-500 mt-0.5">Tauri Client v1.0.0</p>
          </div>
          <div className="hidden sm:block h-6 w-px bg-gray-200 mx-2" />
          <div className="hidden sm:block">
            <StatusCircle status={getCurrentAppStatus()} size="sm" />
          </div>
        </div>

        <div className="flex items-center space-x-2">
          <Button variant="ghost" size="sm" icon={<Minimize className="w-4 h-4" />} onClick={() => {}} className="hidden md:flex">
             Minimize
          </Button>
          <Button
            variant={currentView === 'settings' ? 'primary' : 'ghost'}
            size="sm"
            icon={<Settings className="w-4 h-4" />}
            onClick={() => setCurrentView(currentView === 'dashboard' ? 'settings' : 'dashboard')}
          >
            {currentView === 'dashboard' ? 'Settings' : 'Dashboard'}
          </Button>
        </div>
      </div>
    </header>
  );

  const DashboardView = () => (
    <div className="max-w-5xl mx-auto px-4 py-8 animate-in fade-in slide-in-from-bottom-4 duration-500">
      {/* Main Status Card */}
      <div className="bg-white rounded-2xl shadow-lg shadow-gray-200/50 p-8 mb-8 border border-white">
        <div className="flex flex-col items-center">
          <div className="mb-8">
            <StatusCircle status={getCurrentAppStatus()} />
          </div>

          <div className="text-center mb-8">
            <h2 className="text-3xl font-bold text-gray-900 mb-2 transition-all">
              {getDisplayStatus()}
            </h2>
            <p className="text-gray-500 max-w-md mx-auto">
              {isRecording || isProcessing
                ? 'Speak clearly into your microphone.'
                : 'Press shortcut or buttons below to start capturing audio.'
              }
            </p>
          </div>

          <div className="flex flex-col sm:flex-row space-y-3 sm:space-y-0 sm:space-x-4 w-full sm:w-auto">
            <Button
              size="lg"
              onClick={() => handleStartRecording('transcribe')}
              disabled={isRecording || isProcessing}
              className="w-full sm:w-48"
            >
              Transcribe
              <span className="ml-2 text-xs opacity-60 bg-white/20 px-1.5 py-0.5 rounded">F4</span>
            </Button>
            <Button
              size="lg"
              variant="secondary"
              onClick={() => handleStartRecording('translate')}
              disabled={isRecording || isProcessing}
              className="w-full sm:w-48"
            >
              Translate
              <span className="ml-2 text-xs opacity-60 bg-white/20 px-1.5 py-0.5 rounded">⇧F4</span>
            </Button>
          </div>
        </div>

        {/* History Section */}
        <div className="mt-10 pt-6 border-t border-gray-100">
           <div className="flex items-center justify-between mb-4">
               <div className="flex items-center space-x-3">
                 <span className="text-xs font-semibold uppercase tracking-wider text-gray-400">Recent History</span>
               </div>
               {history.length > 0 && (
                 <Button variant="ghost" size="sm" className="text-xs h-7 text-red-500 hover:bg-red-50 hover:text-red-600" onClick={handleClearHistory} icon={<Trash2 className="w-3 h-3" />}>
                    Clear
                 </Button>
               )}
           </div>

           <div className="space-y-3">
             {history.length === 0 ? (
               <div className="text-center py-8 text-gray-400 text-sm bg-gray-50 rounded-xl border border-gray-200 border-dashed flex flex-col items-center">
                 <Clock className="w-8 h-8 mb-2 opacity-20" />
                 <p>No history yet. Start recording to see results here.</p>
               </div>
             ) : (
               history.slice(0, 3).map((item) => (
                 <div key={item.id} className="bg-white p-4 rounded-xl border border-gray-100 shadow-sm hover:shadow-md transition-all duration-200 group">
                    <div className="flex justify-between items-start mb-2">
                      <span className={`text-[10px] px-2 py-0.5 rounded-full uppercase font-bold tracking-wide ${item.type === 'translate' ? 'bg-green-100 text-green-700' : 'bg-blue-100 text-blue-700'}`}>
                         {item.type}
                      </span>
                      <span className="text-xs text-gray-400 flex items-center">
                         <Clock className="w-3 h-3 mr-1" />
                         {new Date(item.timestamp).toLocaleTimeString()}
                      </span>
                    </div>
                    <p className="text-gray-700 text-sm leading-relaxed font-medium">
                      "{item.text}"
                    </p>
                 </div>
               ))
             )}
           </div>
        </div>
      </div>

      <LiveData />
    </div>
  );

  const SettingsView = () => {
    const tabs: NavTab[] = [
      { id: 'asr', label: 'ASR Service', icon: Mic },
      { id: 'shortcuts', label: 'Shortcuts', icon: Keyboard },
      { id: 'appearance', label: 'Appearance', icon: Settings },
      { id: 'advanced', label: 'Advanced', icon: Settings },
    ];

    return (
      <div className="max-w-6xl mx-auto px-4 py-8 flex flex-col md:flex-row gap-8 animate-in fade-in duration-300">
        {/* Sidebar */}
        <aside className="w-full md:w-64 flex-shrink-0">
          <nav className="space-y-1">
            {tabs.map((tab) => {
              const Icon = tab.icon;
              const isActive = activeSettingsTab === tab.id;
              return (
                <button
                  key={tab.id}
                  onClick={() => setActiveSettingsTab(tab.id)}
                  className={`
                    w-full flex items-center space-x-3 px-4 py-3 rounded-xl text-sm font-medium transition-all duration-200
                    ${isActive
                      ? 'bg-white text-primary-600 shadow-sm ring-1 ring-gray-200'
                      : 'text-gray-600 hover:bg-white/60 hover:text-gray-900'}
                  `}
                >
                  <Icon className={`w-5 h-5 ${isActive ? 'text-primary-500' : 'text-gray-400'}`} />
                  <span>{tab.label}</span>
                </button>
              );
            })}
          </nav>
        </aside>

        {/* Content Area */}
        <main className="flex-1 min-h-[500px]">
          <PlaceholderSettings title={tabs.find(t => t.id === activeSettingsTab)?.label || 'Settings'} />
        </main>
      </div>
    );
  };

  return (
    <div className="min-h-screen bg-[#F3F4F6] text-gray-900 font-sans selection:bg-primary-100 selection:text-primary-900">
      <TopBar />
      {currentView === 'dashboard' ? <DashboardView /> : <SettingsView />}

      {/* Mobile Status Bar (Visible only on small screens) */}
      <div className="lg:hidden fixed bottom-0 left-0 right-0 bg-white border-t border-gray-200 p-4 flex justify-between items-center z-40">
         <div className="flex items-center space-x-2">
            <div className={`w-2 h-2 rounded-full ${isRecording ? 'bg-red-500 animate-pulse' : 'bg-gray-400'}`} />
            <span className="text-sm font-medium">{isRecording ? 'Active' : 'Idle'}</span>
         </div>
         <span className="text-xs text-gray-400">EchoType</span>
      </div>
    </div>
  );
};

export default App;