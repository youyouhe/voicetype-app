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

  // Legacy functions (keep for compatibility)
  static async greet(name: string): Promise<string> {
    return await invoke<string>('greet', { name });
  }

  static async add(a: number, b: number): Promise<number> {
    return await invoke<number>('add', { a, b });
  }
}