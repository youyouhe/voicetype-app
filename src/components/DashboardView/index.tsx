import React, { memo } from 'react';
import { FileText, Languages, Clock, Save, Trash2 } from 'lucide-react';
import { AppStatus, HistoryItem } from '../../types';
import { StatusCircle } from '../MainView/StatusCircle';
import { LiveData } from '../MainView/LiveData';
import { VoiceAssistantPanel } from '../MainView/VoiceAssistantPanel';
import { Button } from '../ui/Button';

export interface DashboardViewProps {
  appStatus: AppStatus;
  history: HistoryItem[];
  lastSaved: Date | null;
  startRecording: (type: 'transcribe' | 'translate') => void;
  handleClearHistory: () => void;
  startVoiceAssistant: () => Promise<void>;
  stopVoiceAssistant: () => Promise<void>;
  isVoiceAssistantRunning: boolean;
}

export const DashboardView = memo<DashboardViewProps>(({
  appStatus,
  history,
  lastSaved,
  startRecording,
  handleClearHistory,
  startVoiceAssistant,
  stopVoiceAssistant,
  isVoiceAssistantRunning
}) => {
  return (
    <div className="max-w-6xl mx-auto px-4 py-8 animate-in fade-in slide-in-from-bottom-4 duration-500">
      {/* Voice Assistant Panel */}
      <VoiceAssistantPanel
        isRunning={isVoiceAssistantRunning}
        onStatusChange={(running) => {
          // Optional: handle status changes from VoiceAssistantPanel
          console.log('🔄 VoiceAssistantPanel status change:', running);
        }}
      />

      {/* Main Status Card */}
      <div className="bg-white dark:bg-dark-surface rounded-2xl shadow-lg shadow-gray-200/50 dark:shadow-black/20 p-8 mb-8 border border-white dark:border-dark-border">
        <div className="flex flex-col items-center">
          <div className="mb-8">
            <StatusCircle status={appStatus} />
          </div>

          <div className="text-center mb-8">
            <h2 className="text-3xl font-bold text-gray-900 dark:text-dark-text mb-2 transition-all">
              {!isVoiceAssistantRunning ? 'Voice Assistant Offline' :
               appStatus === AppStatus.Idle ? 'Ready to Listen' :
               appStatus === AppStatus.Recording ? 'Listening...' :
               appStatus === AppStatus.Processing ? 'Processing Audio...' : 'Translating...'}
            </h2>
            <p className="text-gray-500 dark:text-dark-muted max-w-md mx-auto">
              {!isVoiceAssistantRunning
                ? 'Please start Voice Assistant first to use transcription and translation features.'
                : appStatus === AppStatus.Idle
                ? 'Press F4 (transcribe) or Shift+F4 (translate) hotkeys to start capturing audio.'
                : 'Speak clearly into your microphone.'}
            </p>
          </div>

          <div className="flex flex-col sm:flex-row space-y-3 sm:space-y-0 sm:space-x-4 w-full sm:w-auto">
            <Button
              size="lg"
              onClick={() => startRecording('transcribe')}
              disabled={!isVoiceAssistantRunning || appStatus !== AppStatus.Idle}
              icon={<FileText className="w-5 h-5"/>}
              className="w-full sm:w-48 relative"
              title={
                !isVoiceAssistantRunning
                  ? "Start Voice Assistant first to enable transcription"
                  : appStatus === AppStatus.Idle
                  ? "Press F4 hotkey to start transcribing"
                  : "Voice Assistant is busy - please wait"
              }
            >
              {!isVoiceAssistantRunning ? (
                <>
                  Start Voice Assistant
                  <span className="ml-2 text-xs opacity-60 bg-white/20 px-1.5 py-0.5 rounded">↑</span>
                </>
              ) : appStatus === AppStatus.Idle ? (
                <>
                  Transcribe
                  <span className="ml-2 text-xs opacity-60 bg-white/20 px-1.5 py-0.5 rounded">F4</span>
                </>
              ) : (
                <>
                  Use Hotkey F4
                  <span className="ml-2 text-xs opacity-60 bg-white/20 px-1.5 py-0.5 rounded">Active</span>
                </>
              )}
            </Button>
            <Button
              size="lg"
              variant="secondary"
              onClick={() => startRecording('translate')}
              disabled={!isVoiceAssistantRunning || appStatus !== AppStatus.Idle}
              icon={<Languages className="w-5 h-5"/>}
              className="w-full sm:w-48"
              title={
                !isVoiceAssistantRunning
                  ? "Start Voice Assistant first to enable translation"
                  : appStatus === AppStatus.Idle
                  ? "Press Shift+F4 hotkey to start translating"
                  : "Voice Assistant is busy - please wait"
              }
            >
              {!isVoiceAssistantRunning ? 'Unavailable' : 'Translate'}
              <span className="ml-2 text-xs opacity-60 bg-white/20 px-1.5 py-0.5 rounded">⇧F4</span>
            </Button>
          </div>
        </div>

        {/* History Section */}
        <div className="mt-10 pt-6 border-t border-gray-100 dark:border-dark-border">
           <div className="flex items-center justify-between mb-4">
               <div className="flex items-center space-x-3">
                 <span className="text-xs font-semibold uppercase tracking-wider text-gray-400 dark:text-dark-muted">Recent History</span>
                 {lastSaved && (
                   <span className="text-[10px] text-green-600 dark:text-green-400 bg-green-50 dark:bg-green-900/30 px-2 py-0.5 rounded-full flex items-center animate-in fade-in duration-300">
                     <Save className="w-3 h-3 mr-1" />
                     Saved {lastSaved.toLocaleTimeString()}
                   </span>
                 )}
               </div>
               {history.length > 0 && (
                 <Button variant="ghost" size="sm" className="text-xs h-7 text-red-500 dark:text-red-400 hover:bg-red-50 dark:hover:bg-red-900/20 hover:text-red-600 dark:hover:text-red-300" onClick={handleClearHistory} icon={<Trash2 className="w-3 h-3"/>}>
                    Clear
                 </Button>
               )}
           </div>

           <div className="space-y-3">
             {history.length === 0 ? (
               <div className="text-center py-8 text-gray-400 dark:text-dark-muted text-sm bg-gray-50 dark:bg-dark-surface rounded-xl border border-gray-200 dark:border-dark-border border-dashed flex flex-col items-center">
                 <Clock className="w-8 h-8 mb-2 opacity-20" />
                 <p>No history yet. Start recording to see results here.</p>
               </div>
             ) : (
               history.slice(0, 3).map((item) => (
                 <div key={item.id} className="bg-white dark:bg-dark-surface p-4 rounded-xl border border-gray-100 dark:border-dark-border shadow-sm hover:shadow-md transition-all duration-200 group">
                    <div className="flex justify-between items-start mb-2">
                       <span className={`text-[10px] px-2 py-0.5 rounded-full uppercase font-bold tracking-wide ${item.type === 'translate' ? 'bg-green-100 text-green-700 dark:bg-green-900/30 dark:text-green-400' : 'bg-blue-100 text-blue-700 dark:bg-blue-900/30 dark:text-blue-400'}`}>
                         {item.type}
                       </span>
                       <span className="text-xs text-gray-400 dark:text-dark-muted flex items-center">
                         <Clock className="w-3 h-3 mr-1" />
                         {new Date(item.timestamp).toLocaleTimeString()}
                       </span>
                    </div>
                    <p className="text-gray-700 dark:text-dark-text text-sm leading-relaxed font-medium">
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
});