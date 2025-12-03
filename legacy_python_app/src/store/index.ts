import { create } from 'zustand';
import { invoke } from '@tauri-apps/api/tauri';
import { AppStatus } from '../types';

export interface AppState {
  // Recording state
  isRecording: boolean;
  isProcessing: boolean;

  // ASR state
  selectedService: string;
  isConnected: boolean;
  lastResult: string | null;

  // UI state
  currentView: 'dashboard' | 'settings';
  activeSettingsTab: string;

  // History
  history: Array<{
    id: string;
    type: 'transcribe' | 'translate';
    text: string;
    timestamp: number;
  }>;

  // Device state
  audioDevices: Array<{
    id: string;
    name: string;
    is_default: boolean;
  }>;
  selectedDevice: string | null;

  // Settings
  settings: {
    api_key: string | null;
    model: string;
    language: string;
    theme: 'light' | 'dark' | 'auto';
    auto_start: boolean;
    show_notifications: boolean;
    transcription_shortcut: string;
    translation_shortcut: string;
  };

  // Error state
  error: string | null;

  // Audio levels
  audioLevel: number;
}

export interface AppActions {
  // Recording actions
  startRecording: (type: 'transcribe' | 'translate') => Promise<void>;
  stopRecording: () => Promise<void>;

  // ASR actions
  setSelectedService: (service: string) => void;
  testConnection: () => Promise<boolean>;

  // UI actions
  setCurrentView: (view: 'dashboard' | 'settings') => void;
  setActiveSettingsTab: (tab: string) => void;

  // History actions
  addToHistory: (item: AppState['history'][0]) => void;
  clearHistory: () => void;

  // Device actions
  refreshAudioDevices: () => Promise<void>;
  setSelectedDevice: (deviceId: string) => void;

  // Settings actions
  updateSettings: (settings: Partial<AppState['settings']>) => void;

  // Error handling
  clearError: () => void;

  // Audio level
  setAudioLevel: (level: number) => void;
}

export const useAppStore = create<AppState, AppActions>((set, get) => ({
  // Initial state
  isRecording: false,
  isProcessing: false,
  selectedService: 'groq',
  isConnected: false,
  lastResult: null,
  currentView: 'dashboard',
  activeSettingsTab: 'asr',
  history: [],
  audioDevices: [],
  selectedDevice: null,
  settings: {
    api_key: null,
    model: 'whisper-large-v3-turbo',
    language: 'zh',
    theme: 'auto',
    auto_start: false,
    show_notifications: true,
    transcription_shortcut: 'F4',
    translation_shortcut: 'Shift+F4',
  },
  error: null,
  audioLevel: 0,

  // Recording actions
  startRecording: async (type) => {
    try {
      set({ isRecording: true, error: null });

      // Start recording on Rust backend
      await invoke('start_recording');

      // Emit event for UI updates
      if (type === 'transcribe') {
        set({ isProcessing: true });
      }
    } catch (error) {
      set({
        isRecording: false,
        error: error instanceof Error ? error.message : 'Failed to start recording'
      });
    }
  },

  stopRecording: async () => {
    try {
      set({ isProcessing: true });

      // Stop recording and get audio data
      const audioData = await invoke('stop_recording');

      // Process audio with selected service
      const result = await invoke('process_audio', {
        audioData,
        mode: 'transcribe',
      });

      // Add to history
      const historyItem = {
        id: Date.now().toString(),
        type: 'transcribe',
        text: result.text || '',
        timestamp: Date.now(),
      };

      set(state => ({
        history: [historyItem, ...state.history],
        isRecording: false,
        isProcessing: false,
        lastResult: result.text || null,
        error: null,
      }));
    } catch (error) {
      set({
        isRecording: false,
        isProcessing: false,
        error: error instanceof Error ? error.message : 'Failed to stop recording'
      });
    }
  },

  // ASR actions
  setSelectedService: (service) => {
    set({ selectedService: service });
  },

  testConnection: async () => {
    try {
      const isConnected = await invoke('test_connection');
      set({ isConnected, error: null });
      return isConnected;
    } catch (error) {
      set({
        isConnected: false,
        error: error instanceof Error ? error.message : 'Connection test failed'
      });
      return false;
    }
  },

  // UI actions
  setCurrentView: (view) => {
    set({ currentView: view });
  },

  setActiveSettingsTab: (tab) => {
    set({ activeSettingsTab: tab });
  },

  // History actions
  addToHistory: (item) => {
    set(state => ({
      history: [item, ...state.history],
    }));
  },

  clearHistory: () => {
    set({ history: [] });
  },

  // Device actions
  refreshAudioDevices: async () => {
    try {
      const devices = await invoke('get_audio_devices');
      set({
        audioDevices: devices,
        selectedDevice: devices.find(d => d.is_default)?.id || null
      });
    } catch (error) {
      set({
        error: error instanceof Error ? error.message : 'Failed to get audio devices'
      });
    }
  },

  setSelectedDevice: (deviceId) => {
    set({ selectedDevice: deviceId });
  },

  // Settings actions
  updateSettings: (newSettings) => {
    set(state => ({
      settings: { ...state.settings, ...newSettings }
    }));
  },

  // Error handling
  clearError: () => {
    set({ error: null });
  },

  // Audio level
  setAudioLevel: (level) => {
    set({ audioLevel: Math.max(0, Math.min(1, level)) });
  },
}));

// Tauri event listeners
export const setupTauriListeners = () => {
  import('@tauri-apps/api/event').then(({ listen }) => {
    // Listen for recording events from Rust backend
    listen('recording-started', () => {
      useAppStore.setState({ isRecording: true });
    });

    listen('recording-stopped', () => {
      useAppStore.setState({ isRecording: false });
    });

    listen('asr-success', (event) => {
      const result = event.payload;
      useAppStore.setState({
        lastResult: result.text,
        isProcessing: false,
      });
    });

    listen('asr-error', (event) => {
      useAppStore.setState({
        error: event.payload,
        isProcessing: false,
      });
    });

    // Global shortcut listeners
    listen('start-transcription', () => {
      useAppStore.getState().startRecording('transcribe');
    });

    listen('start-translation', () => {
      useAppStore.getState().startRecording('translate');
    });
  });
};