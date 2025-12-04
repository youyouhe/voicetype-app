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
  type: 'transcribe' | 'translate';
  text: string;
  timestamp: number;
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

export interface HotkeyConfigRequest {
  transcribe_key: string;
  translate_key: string;
  trigger_delay_ms: number;
  anti_mistouch_enabled: boolean;
  save_wav_files: boolean;
}