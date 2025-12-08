import { invoke } from '@tauri-apps/api/core';

// Types for our Voice Assistant
export interface VoiceAssistantConfig {
  service_platform: string;
  asr_processor: 'cloud' | 'local';
  translate_processor: 'siliconflow' | 'ollama';
  convert_to_simplified: boolean;
  add_symbol: boolean;
  optimize_result: boolean;
}

export interface SystemInfo {
  [key: string]: string;
}

export interface ServiceStatus {
  active_service: string;
  status: 'online' | 'offline' | 'error';
  endpoint?: string;
}

export interface LatencyData {
  current: number;
  trend: 'up' | 'down' | 'neutral';
  trend_value: number;
  history: { time: string; val: number }[];
}

export interface UsageData {
  today_seconds: number;
  success_rate: number;
  total_requests: number;
  successful_requests: number;
}

export interface ProcessorType {
  Whisper: 'whisper';
  SenseVoice: 'sensevoice';
  LocalASR: 'local';
}

export interface TranslateType {
  SiliconFlow: 'siliconflow';
  Ollama: 'ollama';
}

// Voice Assistant Service
export class TauriService {
  // Voice Assistant Control
  static async startVoiceAssistant(): Promise<string> {
    return await invoke<string>('start_voice_assistant');
  }

  static async stopVoiceAssistant(): Promise<string> {
    return await invoke<string>('stop_voice_assistant');
  }

  static async getVoiceAssistantState(): Promise<string> {
    return await invoke<string>('get_voice_assistant_state');
  }

  static async getVoiceAssistantConfig(): Promise<VoiceAssistantConfig> {
    return await invoke<VoiceAssistantConfig>('get_voice_assistant_config');
  }

  // Testing
  static async testASR(processorType: 'cloud' | 'local'): Promise<string> {
    return await invoke<string>('test_asr', { processorType });
  }

  static async testTranslation(translateType: 'siliconflow' | 'ollama'): Promise<string> {
    return await invoke<string>('test_translation', { translateType });
  }

  static async getSystemInfo(): Promise<SystemInfo> {
    return await invoke<SystemInfo>('get_system_info');
  }

  // Live Data Methods
  static async getServiceStatus(): Promise<ServiceStatus> {
    return await invoke<ServiceStatus>('get_service_status');
  }

  static async getLatencyData(): Promise<LatencyData> {
    return await invoke<LatencyData>('get_latency_data');
  }

  static async getUsageData(): Promise<UsageData> {
    return await invoke<UsageData>('get_usage_data');
  }

  // Model Management Methods
  static async getAvailableModels(): Promise<any[]> {
    return await invoke<any[]>('get_available_models');
  }

  static async downloadModel(modelName: string): Promise<string> {
    return await invoke<string>('download_model', { modelName });
  }

  static async deleteModel(modelName: string): Promise<string> {
    return await invoke<string>('delete_model', { modelName });
  }

  static async setActiveModel(modelName: string): Promise<string> {
    return await invoke<string>('set_active_model', { modelName });
  }

  static async getActiveModelInfo(): Promise<string | null> {
    return await invoke<string | null>('get_active_model_info');
  }

  static async getModelStats(): Promise<any> {
    return await invoke<any>('get_model_stats');
  }

  // Legacy functions (keep for compatibility)
  static async greet(name: string): Promise<string> {
    return await invoke<string>('greet', { name });
  }

  static async add(a: number, b: number): Promise<number> {
    return await invoke<number>('add', { a, b });
  }
}