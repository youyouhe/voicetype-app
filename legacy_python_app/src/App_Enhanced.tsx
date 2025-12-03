import React, { useState, useEffect, useRef } from 'react';
import { motion, AnimatePresence } from 'framer-motion';
import { 
  Mic, Settings, Minus, Keyboard, 
  Palette, Sliders, Languages, FileText, Clock, Save, Trash2,
  Volume2, Zap, Activity
} from 'lucide-react';
import { AppStatus, HistoryItem } from './types';
import { StatusCircle, StatusIndicator } from './components/ui/StatusCircle';
import { Button } from './components/ui/Button';
import { Card } from './components/ui/Card';
import { Input, Select, Toggle } from './components/ui/Input';
import { useAppStore } from './store';

// --- Constants ---
const STORAGE_KEY = 'echotype_history';
const AUTO_SAVE_INTERVAL_MS = 5000; // Auto-save every 5 seconds

// --- Main App Component ---
const App: React.FC = () => {
  const [currentView, setCurrentView] = useState<'dashboard' | 'settings'>('dashboard');
  const [activeSettingsTab, setActiveSettingsTab] = useState('asr');
  
  // Get app store
  const {
    isRecording,
    isProcessing,
    lastResult,
    history,
    startRecording,
    stopRecording,
    addToHistory,
    clearHistory
  } = useAppStore();
  
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
        // Restore history via store methods
        parsed.forEach((item: HistoryItem) => {
          addToHistory(item);
        });
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
    }, AUTO_SAVE_INTERVAL_MS);

    return () => clearInterval(intervalId);
  }, []);

  const handleClearHistory = () => {
    clearHistory();
    localStorage.removeItem(STORAGE_KEY);
  };

  // Mock Actions with History Update
  const handleStartRecording = async (type: 'transcribe' | 'translate') => {
    if (isRecording || isProcessing) return;
    
    try {
      await startRecording(type);
      
      // Mock processing for demonstration
      setTimeout(() => {
        const mockTexts = type === 'transcribe' 
          ? [
              "The quick brown fox jumps over the lazy dog.",
              "Meeting notes: Discussed Q3 roadmap and performance optimizations.",
              "React 19 introduces new hooks and compiler improvements.",
              "Don't forget to buy milk and eggs on your way home."
            ] 
          : [
              "El zorro marrón rápido salta sobre el perro perezoso.",
              "Notes de réunion : Discussion sur la feuille de route du Q3.",
              "React 19 introduce nouveaux hooks et améliorations dans le compilateur.",
              "N'oubliez pas d'acheter du lait et des œufs sur le chemin du retour."
            ];
            
        const randomText = mockTexts[Math.floor(Math.random() * mockTexts.length)];
        
        const newItem: HistoryItem = {
          id: Date.now().toString(),
          type,
          text: randomText,
          timestamp: Date.now()
        };

        addToHistory(newItem);
      }, 3000);
    } catch (error) {
      console.error('Failed to start recording:', error);
    }
  };

  const getStatusText = () => {
    if (isRecording) return '正在录音...';
    if (isProcessing) return '正在处理...';
    return '准备就绪';
  };

  const getStatusDescription = () => {
    if (isRecording) return '请对着麦克风清晰地说话';
    if (isProcessing) return '正在处理您的音频...';
    return '按住快捷键或点击下方按钮开始录音';
  };

  const TopBar = () => (
    <motion.header 
      initial={{ y: -100 }}
      animate={{ y: 0 }}
      className="bg-white/80 backdrop-blur-md border-b border-gray-200 px-6 py-3 sticky top-0 z-50"
    >
      <div className="flex items-center justify-between max-w-7xl mx-auto">
        <div className="flex items-center space-x-4">
          <motion.div 
            whileHover={{ scale: 1.1, rotate: 5 }}
            className="bg-primary-50 p-2 rounded-lg"
          >
            <Mic className="w-5 h-5 text-primary-600" />
          </motion.div>
          <div>
            <h1 className="text-lg font-bold text-gray-900 leading-none">EchoType</h1>
            <p className="text-xs text-gray-500 mt-0.5">Tauri Client v1.0.0</p>
          </div>
          <div className="hidden sm:block h-6 w-px bg-gray-200 mx-2" />
          <div className="hidden sm:block">
            <StatusIndicator status={isRecording ? AppStatus.Recording : isProcessing ? AppStatus.Processing : AppStatus.Idle} />
          </div>
        </div>

        <div className="flex items-center space-x-2">
          <Button 
            variant="ghost" 
            size="sm" 
            icon={<Minus className="w-4 h-4"/>} 
            onClick={() => {}} 
            className="hidden md:flex"
          >
            最小化
          </Button>
          <Button 
            variant={currentView === 'settings' ? 'primary' : 'ghost'} 
            size="sm" 
            icon={<Settings className="w-4 h-4"/>} 
            onClick={() => setCurrentView(currentView === 'dashboard' ? 'settings' : 'dashboard')}
          >
            {currentView === 'dashboard' ? '设置' : '主页'}
          </Button>
        </div>
      </div>
    </motion.header>
  );

  const DashboardView = () => (
    <motion.div
      initial={{ opacity: 0, y: 20 }}
      animate={{ opacity: 1, y: 0 }}
      transition={{ duration: 0.5 }}
      className="max-w-5xl mx-auto px-4 py-8"
    >
      {/* Main Status Card */}
      <Card hover className="mb-8">
        <div className="flex flex-col items-center">
          <div className="mb-8">
            <StatusCircle 
              status={isRecording ? AppStatus.Recording : isProcessing ? AppStatus.Processing : AppStatus.Idle} 
              size="lg"
            />
          </div>
          
          <div className="text-center mb-8">
            <motion.h2 
              className="text-3xl font-bold text-gray-900 mb-2"
              key={isRecording ? 'recording' : isProcessing ? 'processing' : 'idle'}
              initial={{ opacity: 0, y: -10 }}
              animate={{ opacity: 1, y: 0 }}
              transition={{ duration: 0.3 }}
            >
              {getStatusText()}
            </motion.h2>
            <motion.p 
              className="text-gray-500 max-w-md mx-auto"
              initial={{ opacity: 0 }}
              animate={{ opacity: 1 }}
              transition={{ duration: 0.3, delay: 0.1 }}
            >
              {getStatusDescription()}
            </motion.p>
          </div>

          <div className="flex flex-col sm:flex-row space-y-3 sm:space-y-0 sm:space-x-4 w-full sm:w-auto">
            <Button 
              size="lg" 
              onClick={() => handleStartRecording('transcribe')}
              disabled={isRecording || isProcessing}
              icon={<FileText className="w-5 h-5"/>}
              className="w-full sm:w-48"
              hotkey="F4"
            >
              语音转录
              <span className="ml-2 text-xs opacity-60 bg-white/20 px-1.5 py-0.5 rounded">F4</span>
            </Button>
            <Button 
              size="lg" 
              variant="secondary"
              onClick={() => handleStartRecording('translate')}
              disabled={isRecording || isProcessing}
              icon={<Languages className="w-5 h-5"/>}
              className="w-full sm:w-48"
              hotkey="⇧F4"
            >
              语音翻译
              <span className="ml-2 text-xs opacity-60 bg-white/20 px-1.5 py-0.5 rounded">⇧F4</span>
            </Button>
          </div>
        </div>

        {/* History Section */}
        <div className="mt-10 pt-6 border-t border-gray-100">
          <div className="flex items-center justify-between mb-4">
            <div className="flex items-center space-x-3">
              <span className="text-xs font-semibold uppercase tracking-wider text-gray-400">最近历史</span>
            </div>
            {history.length > 0 && (
              <Button 
                variant="ghost" 
                size="sm" 
                className="text-xs h-7 text-red-500 hover:bg-red-50 hover:text-red-600" 
                onClick={handleClearHistory} 
                icon={<Trash2 className="w-3 h-3"/>}
              >
                清空
              </Button>
            )}
          </div>
          
          <div className="space-y-3">
            {history.length === 0 ? (
              <motion.div
                initial={{ opacity: 0, scale: 0.9 }}
                animate={{ opacity: 1, scale: 1 }}
                className="text-center py-8 text-gray-400 text-sm bg-gray-50 rounded-xl border border-gray-200 border-dashed flex flex-col items-center"
              >
                <Clock className="w-8 h-8 mb-2 opacity-20" />
                <p>暂无历史记录。开始录音以查看结果。</p>
              </motion.div>
            ) : (
              <AnimatePresence>
                {history.slice(0, 3).map((item, index) => (
                  <motion.div
                    key={item.id}
                    initial={{ opacity: 0, y: 20, scale: 0.9 }}
                    animate={{ opacity: 1, y: 0, scale: 1 }}
                    exit={{ opacity: 0, y: -20, scale: 0.9 }}
                    transition={{ duration: 0.3, delay: index * 0.1 }}
                    className="bg-white p-4 rounded-xl border border-gray-100 shadow-sm hover:shadow-md transition-all duration-200 group"
                  >
                    <div className="flex justify-between items-start mb-2">
                      <span className={`text-[10px] px-2 py-0.5 rounded-full uppercase font-bold tracking-wide ${
                        item.type === 'translate' 
                          ? 'bg-green-100 text-green-700' 
                          : 'bg-blue-100 text-blue-700'
                      }`}>
                        {item.type === 'translate' ? '翻译' : '转录'}
                      </span>
                      <span className="text-xs text-gray-400 flex items-center">
                        <Clock className="w-3 h-3 mr-1" />
                        {new Date(item.timestamp).toLocaleTimeString()}
                      </span>
                    </div>
                    <p className="text-gray-700 text-sm leading-relaxed font-medium">
                      "{item.text}"
                    </p>
                  </motion.div>
                ))}
              </AnimatePresence>
            )}
          </div>
        </div>
      </Card>

      {/* Live Data Section */}
      <motion.div
        initial={{ opacity: 0, y: 20 }}
        animate={{ opacity: 1, y: 0 }}
        transition={{ duration: 0.5, delay: 0.2 }}
      >
        <div className="grid grid-cols-1 md:grid-cols-3 gap-4">
          <Card
            icon={<Activity className="w-5 h-5 text-blue-500" />}
            title="服务状态"
            value="在线"
            description="连接正常"
          />
          <Card
            icon={<Zap className="w-5 h-5 text-green-500" />}
            title="响应时间"
            value="45ms"
            trend="down"
          />
          <Card
            icon={<Volume2 className="w-5 h-5 text-purple-500" />}
            title="今日使用"
            value={history.length}
            description="次语音识别"
          />
        </div>
      </motion.div>
    </motion.div>
  );

  const SettingsView = () => {
    const tabs = [
      { id: 'asr', label: 'ASR 服务', icon: Mic },
      { id: 'shortcuts', label: '快捷键', icon: Keyboard },
      { id: 'appearance', label: '外观设置', icon: Palette },
      { id: 'advanced', label: '高级设置', icon: Sliders },
    ];

    return (
      <motion.div
        initial={{ opacity: 0 }}
        animate={{ opacity: 1 }}
        transition={{ duration: 0.3 }}
        className="max-w-6xl mx-auto px-4 py-8 flex flex-col md:flex-row gap-8"
      >
        {/* Sidebar */}
        <aside className="w-full md:w-64 flex-shrink-0">
          <motion.nav 
            initial={{ x: -100 }}
            animate={{ x: 0 }}
            className="space-y-1"
          >
            {tabs.map((tab, index) => {
              const Icon = tab.icon;
              const isActive = activeSettingsTab === tab.id;
              return (
                <motion.button
                  key={tab.id}
                  whileHover={{ scale: 1.02, x: 5 }}
                  whileTap={{ scale: 0.95 }}
                  onClick={() => setActiveSettingsTab(tab.id)}
                  transition={{ delay: index * 0.05 }}
                  className={`
                    w-full flex items-center space-x-3 px-4 py-3 rounded-xl text-sm font-medium transition-all duration-200
                    ${isActive 
                      ? 'bg-white text-primary-600 shadow-sm ring-1 ring-gray-200' 
                      : 'text-gray-600 hover:bg-white/60 hover:text-gray-900'
                    }
                  `}
                >
                  <Icon className={`w-5 h-5 ${isActive ? 'text-primary-500' : 'text-gray-400'}`} />
                  <span>{tab.label}</span>
                </motion.button>
              );
            })}
          </motion.nav>
        </aside>

        {/* Content Area */}
        <main className="flex-1 min-h-[500px] bg-white rounded-xl shadow-lg p-6">
          {activeSettingsTab === 'asr' && (
            <motion.div
              initial={{ opacity: 0, y: 20 }}
              animate={{ opacity: 1, y: 0 }}
              transition={{ duration: 0.3 }}
            >
              <h2 className="text-2xl font-bold text-gray-900 mb-6">ASR 服务设置</h2>
              <div className="space-y-6">
                <div className="space-y-4">
                  <h3 className="text-lg font-semibold text-gray-800 mb-3">选择服务</h3>
                  <div className="grid grid-cols-1 md:grid-cols-3 gap-4">
                    {[
                      { id: 'groq', title: 'Groq Whisper', description: '云端服务，速度快，精度高' },
                      { id: 'siliconflow', title: 'SiliconFlow', description: '国内网络友好，自带标点' },
                      { id: 'local', title: '本地 ASR', description: '私有部署，无网络依赖' }
                    ].map(service => (
                      <Card key={service.id} title={service.title} description={service.description} />
                    ))}
                  </div>
                </div>
                
                <div className="space-y-4">
                  <h3 className="text-lg font-semibold text-gray-800 mb-3">API 配置</h3>
                  <Input label="API 密钥" type="password" placeholder="请输入您的 API 密钥" />
                  <Input label="服务 URL" placeholder="https://api.example.com/v1" />
                </div>
              </div>
            </motion.div>
          )}
          
          {activeSettingsTab === 'shortcuts' && (
            <motion.div
              initial={{ opacity: 0, y: 20 }}
              animate={{ opacity: 1, y: 0 }}
              transition={{ duration: 0.3 }}
            >
              <h2 className="text-2xl font-bold text-gray-900 mb-6">快捷键设置</h2>
              <div className="space-y-4">
                <Input 
                  label="语音转录快捷键" 
                  value="F4" 
                  onChange={() => {}}
                  placeholder="按下此键开始录音"
                />
                <Input 
                  label="语音翻译快捷键" 
                  value="Shift+F4" 
                  onChange={() => {}}
                  placeholder="按下此键开始录音并翻译"
                />
              </div>
            </motion.div>
          )}
          
          {activeSettingsTab === 'appearance' && (
            <motion.div
              initial={{ opacity: 0, y: 20 }}
              animate={{ opacity: 1, y: 0 }}
              transition={{ duration: 0.3 }}
            >
              <h2 className="text-2xl font-bold text-gray-900 mb-6">外观设置</h2>
              <div className="space-y-4">
                <div className="space-y-2">
                  <label className="text-sm font-medium text-gray-700">主题</label>
                  <Select
                    value="auto"
                    onChange={() => {}}
                    options={[
                      { label: '自动', value: 'auto' },
                      { label: '亮色', value: 'light' },
                      { label: '暗色', value: 'dark' }
                    ]}
                  />
                </div>
                <Toggle label="启用动画效果" checked={true} onChange={() => {}} />
                <Toggle label="显示状态栏" checked={true} onChange={() => {}} />
              </div>
            </motion.div>
          )}
          
          {activeSettingsTab === 'advanced' && (
            <motion.div
              initial={{ opacity: 0, y: 20 }}
              animate={{ opacity: 1, y: 0 }}
              transition={{ duration: 0.3 }}
            >
              <h2 className="text-2xl font-bold text-gray-900 mb-6">高级设置</h2>
              <div className="space-y-4">
                <Input label="超时时间（秒）" type="number" value="10" onChange={() => {}} min={5} max={60} />
                <Input label="最短录音时长（秒）" type="number" value="1" onChange={() => {}} min={0.5} max={5} step={0.5} />
                <Toggle label="启用自动语言检测" checked={true} onChange={() => {}} />
                <Toggle label="开启错误恢复" checked={true} onChange={() => {}} />
              </div>
            </motion.div>
          )}
        </main>
      </motion.div>
    );
  };

  return (
    <div className="min-h-screen bg-gradient-to-br from-gray-50 to-gray-100 text-gray-900 font-sans selection:bg-primary-100 selection:text-primary-900">
      <TopBar />
      
      <AnimatePresence mode="wait">
        <motion.div
          key={currentView}
          initial={{ opacity: 0, x: 100 }}
          animate={{ opacity: 1, x: 0 }}
          exit={{ opacity: 0, x: -100 }}
          transition={{ duration: 0.3 }}
        >
          {currentView === 'dashboard' ? <DashboardView /> : <SettingsView />}
        </motion.div>
      </AnimatePresence>
      
      {/* Mobile Status Bar (Visible only on small screens) */}
      <motion.div 
        className="lg:hidden fixed bottom-0 left-0 right-0 bg-white border-t border-gray-200 p-4 flex justify-between items-center z-40"
        initial={{ y: 100 }}
        animate={{ y: 0 }}
        transition={{ duration: 0.3 }}
      >
        <div className="flex items-center space-x-2">
          <motion.div 
            className={`w-2 h-2 rounded-full ${
              isRecording ? 'bg-red-500 animate-pulse' : 'bg-gray-400'
            }`}
            animate={isRecording ? { scale: [1, 1.3, 1] } : { scale: 1 }}
            transition={{ duration: 0.5, repeat: isRecording ? Infinity : 0 }}
          />
          <span className="text-sm font-medium">
            {isRecording ? '录音中' : '空闲'}
          </span>
        </div>
        <span className="text-xs text-gray-400">EchoType v1.0</span>
      </motion.div>
    </div>
  );
};

export default App;