import React from 'react';
import { Mic, Settings, Minus, Clock } from 'lucide-react';

const App: React.FC = () => {
  return (
    <div className="min-h-screen bg-[#F3F4F6] text-gray-900 font-sans p-8">
      <div className="max-w-4xl mx-auto">
        <div className="bg-white rounded-2xl shadow-lg p-8 border border-white">
          <div className="flex flex-col items-center text-center">
            <div className="bg-primary-50 p-4 rounded-full mb-6">
              <Mic className="w-12 h-12 text-primary-600" />
            </div>

            <h1 className="text-3xl font-bold text-gray-900 mb-4">
              EchoType
            </h1>

            <p className="text-gray-600 mb-8 text-lg max-w-md">
              Real-time voice-to-text application built with Tauri and React
            </p>

            <div className="text-sm text-gray-500 space-y-2">
              <p>✅ Rust backend with cpal audio processing</p>
              <p>✅ React frontend with TypeScript</p>
              <p>✅ Zustand state management</p>
              <p>✅ TailwindCSS styling</p>
              <p>✅ Whisper ASR integration ready</p>
              <p>✅ Global shortcuts support</p>
            </div>

            <div className="flex space-x-4 mt-8">
              <button className="px-6 py-3 bg-primary-500 text-white rounded-lg hover:bg-primary-600 transition-colors">
                <Settings className="w-5 h-5 mr-2" />
                Settings
              </button>
            </div>
          </div>
        </div>
      </div>
    </div>
  );
};

export default App;