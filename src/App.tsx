import React, { useState, useEffect, useRef } from 'react';
import {
  Mic, Settings, Minus, Keyboard,
  Palette, Sliders, Languages, FileText, Clock, Save, Trash2
} from 'lucide-react';
import { AppStatus, NavTab, HistoryItem } from './types';
import { StatusCircle, StatusIndicator } from './components/MainView/StatusCircle';
import { LiveData } from './components/MainView/LiveData';
import { VoiceAssistantPanel } from './components/MainView/VoiceAssistantPanel';
import { Button } from './components/ui/Button';
import { ASRSettings, ShortcutSettings, PlaceholderSettings } from './components/SettingsView/SettingsContent';
import { TauriService } from './services/tauriService';

// --- Constants ---
const STORAGE_KEY = 'whisper_input_history';
const AUTOSAVE_INTERVAL_MS = 5000; // Auto-save every 5 seconds

// --- Main App Component ---

const App: React.FC = () => {
  const [currentView, setCurrentView] = useState<'dashboard' | 'settings'>('dashboard');
  const [appStatus, setAppStatus] = useState<AppStatus>(AppStatus.Idle);
  const [activeSettingsTab, setActiveSettingsTab] = useState('asr');
  
  // History State
  const [history, setHistory] = useState<HistoryItem[]>([]);
  const [lastSaved, setLastSaved] = useState<Date | null>(null);
  
  // Ref to access latest state inside interval closure
  const historyRef = useRef(history);

  // Sync ref with state
  useEffect(() => {
    historyRef.current = history;
  }, [history]);

  // Load history on mount
  useEffect(() => {
    const saved = localStorage.getItem(STORAGE_KEY);
    if (saved) {
      try {
        const parsed = JSON.parse(saved);
        setHistory(parsed);
      } catch (e) {
        console.error("Failed to parse history from localStorage", e);
      }
    }
  }, []);

  // Auto-save Interval
  useEffect(() => {
    const intervalId = setInterval(() => {
      // Save current state to local storage
      localStorage.setItem(STORAGE_KEY, JSON.stringify(historyRef.current));
      setLastSaved(new Date());
    }, AUTOSAVE_INTERVAL_MS);

    return () => clearInterval(intervalId);
  }, []);

  const handleClearHistory = () => {
    setHistory([]);
    localStorage.removeItem(STORAGE_KEY);
  };

  // Real Voice Assistant Actions with Tauri Backend
  const startRecording = async (type: 'transcribe' | 'translate') => {
    if (appStatus !== AppStatus.Idle) return;

    try {
      setAppStatus(AppStatus.Recording);

      // In a real implementation, this would trigger the actual recording
      // For now, we'll simulate with the Tauri backend

      setAppStatus(AppStatus.Processing);

      // Add a placeholder for the recording in progress
      const processingItem: HistoryItem = {
        id: Date.now().toString(),
        type,
        text: 'Processing audio...',
        timestamp: Date.now()
      };
      setHistory(prev => [processingItem, ...prev]);

      // Simulate processing time (in real app, this would be handled by backend)
      setTimeout(async () => {
        try {
          // In real implementation, this would get actual transcription/translation result
          // For now, we'll use mock data but could call actual Tauri service
          const mockTexts = type === 'transcribe'
            ? [
                "The quick brown fox jumps over the lazy dog.",
                "Meeting notes: Discussed Q3 roadmap and performance optimizations.",
                "React 19 introduces new hooks and compiler improvements.",
                "Don't forget to buy milk and eggs on the way home."
              ]
            : [
                "El zorro marrón rápido salta sobre el perro perezoso.",
                "Notes de réunion : Discussion sur la feuille de route du T3.",
                "React 19 introduce nuevos hooks y mejoras en el compilador.",
                "N'oubliez pas d'acheter du lait et des œufs sur le chemin du retour."
              ];

          const randomText = mockTexts[Math.floor(Math.random() * mockTexts.length)];

          // Update the processing item with the final result
          setHistory(prev => prev.map(item =>
            item.id === processingItem.id
              ? { ...item, text: randomText }
              : item
          ));

          if (type === 'translate') {
            setAppStatus(AppStatus.Translating);
            setTimeout(() => setAppStatus(AppStatus.Idle), 2000);
          } else {
            setAppStatus(AppStatus.Idle);
          }
        } catch (error) {
          console.error('Error processing audio:', error);
          setAppStatus(AppStatus.Idle);

          // Update history with error message
          setHistory(prev => prev.map(item =>
            item.id === processingItem.id
              ? { ...item, text: 'Error processing audio: ' + error }
              : item
          ));
        }
      }, 2000);

    } catch (error) {
      console.error('Error starting recording:', error);
      setAppStatus(AppStatus.Idle);
    }
  };

  // Voice Assistant Control Functions
  const startVoiceAssistant = async () => {
    try {
      setAppStatus(AppStatus.Processing);
      const result = await TauriService.startVoiceAssistant();
      console.log('Voice Assistant started:', result);
      setAppStatus(AppStatus.Idle);
    } catch (error) {
      console.error('Failed to start Voice Assistant:', error);
      setAppStatus(AppStatus.Idle);
    }
  };

  const stopVoiceAssistant = async () => {
    try {
      setAppStatus(AppStatus.Processing);
      const result = await TauriService.stopVoiceAssistant();
      console.log('Voice Assistant stopped:', result);
      setAppStatus(AppStatus.Idle);
    } catch (error) {
      console.error('Failed to stop Voice Assistant:', error);
      setAppStatus(AppStatus.Idle);
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
            <h1 className="text-lg font-bold text-gray-900 leading-none">Whisper-Input</h1>
            <p className="text-xs text-gray-500 mt-0.5">Tauri Client v1.0.0</p>
          </div>
          <div className="hidden sm:block h-6 w-px bg-gray-200 mx-2" />
          <div className="hidden sm:block">
            <StatusIndicator status={appStatus} />
          </div>
        </div>

        <div className="flex items-center space-x-2">
          <Button
            variant="ghost"
            size="sm"
            icon={<Mic className="w-4 h-4"/>}
            onClick={startVoiceAssistant}
            disabled={appStatus !== AppStatus.Idle}
            className="hidden md:flex"
            title="Start Voice Assistant"
          >
            Start
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

  const DashboardView = () => (
    <div className="max-w-6xl mx-auto px-4 py-8 animate-in fade-in slide-in-from-bottom-4 duration-500">
      {/* Voice Assistant Panel */}
      <VoiceAssistantPanel />

      {/* Main Status Card */}
      <div className="bg-white rounded-2xl shadow-lg shadow-gray-200/50 p-8 mb-8 border border-white">
        <div className="flex flex-col items-center">
          <div className="mb-8">
            <StatusCircle status={appStatus} />
          </div>

          <div className="text-center mb-8">
            <h2 className="text-3xl font-bold text-gray-900 mb-2 transition-all">
              {appStatus === AppStatus.Idle ? 'Ready to Listen' :
               appStatus === AppStatus.Recording ? 'Listening...' :
               appStatus === AppStatus.Processing ? 'Processing Audio...' : 'Translating...'}
            </h2>
            <p className="text-gray-500 max-w-md mx-auto">
              {appStatus === AppStatus.Idle
                ? 'Press hotkey or buttons below to start capturing audio.'
                : 'Speak clearly into your microphone.'}
            </p>
          </div>

          <div className="flex flex-col sm:flex-row space-y-3 sm:space-y-0 sm:space-x-4 w-full sm:w-auto">
            <Button
              size="lg"
              onClick={() => startRecording('transcribe')}
              disabled={appStatus !== AppStatus.Idle}
              icon={<FileText className="w-5 h-5"/>}
              className="w-full sm:w-48"
            >
              Transcribe
              <span className="ml-2 text-xs opacity-60 bg-white/20 px-1.5 py-0.5 rounded">F4</span>
            </Button>
            <Button
              size="lg"
              variant="secondary"
              onClick={() => startRecording('translate')}
              disabled={appStatus !== AppStatus.Idle}
              icon={<Languages className="w-5 h-5"/>}
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
                 {lastSaved && (
                   <span className="text-[10px] text-green-600 bg-green-50 px-2 py-0.5 rounded-full flex items-center animate-in fade-in duration-300">
                     <Save className="w-3 h-3 mr-1" />
                     Saved {lastSaved.toLocaleTimeString()}
                   </span>
                 )}
               </div>
               {history.length > 0 && (
                 <Button variant="ghost" size="sm" className="text-xs h-7 text-red-500 hover:bg-red-50 hover:text-red-600" onClick={handleClearHistory} icon={<Trash2 className="w-3 h-3"/>}>
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
      { id: 'appearance', label: 'Appearance', icon: Palette },
      { id: 'advanced', label: 'Advanced', icon: Sliders },
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
          {activeSettingsTab === 'asr' && <ASRSettings />}
          {activeSettingsTab === 'shortcuts' && <ShortcutSettings />}
          {activeSettingsTab === 'appearance' && <PlaceholderSettings title="Appearance" />}
          {activeSettingsTab === 'advanced' && <PlaceholderSettings title="Advanced Configuration" />}
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
            <div className={`w-2 h-2 rounded-full ${appStatus === AppStatus.Recording ? 'bg-red-500 animate-pulse' : 'bg-gray-400'}`} />
            <span className="text-sm font-medium">{appStatus === AppStatus.Idle ? 'Idle' : 'Active'}</span>
         </div>
         <span className="text-xs text-gray-400">Cloud ASR</span>
      </div>
    </div>
  );
};

export default App;