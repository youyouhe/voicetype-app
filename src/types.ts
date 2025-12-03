import { LucideIcon } from 'lucide-react';
import { ReactNode } from 'react';

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