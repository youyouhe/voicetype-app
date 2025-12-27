import { LucideIcon } from 'lucide-react';
import { ReactNode } from 'react';

// Tauri environment type declarations
declare global {
  interface Window {
    __TAURI_INTERNALS__?: {
      platform: string;
      version: string;
    };
  }
}

export enum AppStatus {
  Idle = 'idle',
  Recording = 'recording',
  Processing = 'processing',
  Translating = 'translating',
  Error = 'error',
}

export enum ServiceProvider {
  Local = 'local',
  Cloud = 'cloud',
}

export interface NavTab {
  id: string;
  label: string;
  icon: LucideIcon;
}

export interface ServiceOptionProps {
  id: ServiceProvider;
  title: string;
  description: string;
  icon: ReactNode;
  selected: boolean;
  onSelect: () => void;
  disabled?: boolean;
}

export interface MetricData {
  name: string;
  value: number;
}

export interface HistoryItem {
  id: string;
  type: 'transcribe' | 'translate' | 'asr';  // Added 'asr' type
  text: string;
  timestamp: Date;  // Changed from number to Date
}

export interface HotkeyConfig {
  id: string;
  transcribe_key: string;
  translate_key: string;
  trigger_delay_ms: number;
  anti_mistouch_enabled: boolean;
  save_wav_files: boolean;
  created_at: string;
  updated_at: string;
}

export interface TypingDelays {
  clipboard_update_ms: number;     // 剪贴板更新等待时间
  keyboard_events_settle_ms: number; // 键盘事件处理等待时间
  typing_complete_ms: number;      // 打字完成后等待时间
  character_interval_ms: number;   // 字符间延迟时间
  short_operation_ms: number;      // 其他短操作延迟时间
}

export interface HotkeyConfigRequest {
  transcribe_key: string;
  translate_key: string;
  trigger_delay_ms: number;
  anti_mistouch_enabled: boolean;
  save_wav_files: boolean;
  typing_delays: TypingDelays;
}