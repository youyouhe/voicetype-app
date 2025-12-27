import React from 'react';
import { Mic, Activity, CheckCircle, AlertCircle, XCircle, Languages, Loader } from 'lucide-react';
import { AppStatus } from '../../types';

interface StatusCircleProps {
  status: AppStatus;
}

export const StatusCircle: React.FC<StatusCircleProps> = ({ status }) => {
  const getStatusConfig = () => {
    switch (status) {
      case AppStatus.Idle:
        return { color: 'bg-gray-100 dark:bg-slate-800 text-gray-400 dark:text-gray-500', icon: Mic, ring: 'ring-gray-200 dark:ring-slate-700' };
      case AppStatus.Recording:
        return { color: 'bg-red-50 dark:bg-red-900/30 text-red-500 dark:text-red-400', icon: Mic, ring: 'ring-red-100 dark:ring-red-900/50' };
      case AppStatus.Processing:
        return { color: 'bg-blue-50 dark:bg-blue-900/30 text-blue-500 dark:text-blue-400', icon: Loader, ring: 'ring-blue-100 dark:ring-blue-900/50' };
      case AppStatus.Translating:
        return { color: 'bg-green-50 dark:bg-green-900/30 text-green-500 dark:text-green-400', icon: Languages, ring: 'ring-green-100 dark:ring-green-900/50' };
      case AppStatus.Error:
        return { color: 'bg-red-50 dark:bg-red-900/30 text-red-600 dark:text-red-400', icon: XCircle, ring: 'ring-red-200 dark:ring-red-900/50' };
      default:
        return { color: 'bg-gray-100 dark:bg-slate-800', icon: Mic, ring: 'ring-gray-200 dark:ring-slate-700' };
    }
  };

  const config = getStatusConfig();
  const Icon = config.icon;

  return (
    <div className="relative flex items-center justify-center w-48 h-48">
      {/* Outer pulsing ring for recording */}
      {status === AppStatus.Recording && (
        <div className="absolute inset-0 rounded-full bg-red-500 opacity-20 animate-ping" />
      )}
      
      {/* Spinning ring for processing/translating */}
      {(status === AppStatus.Processing || status === AppStatus.Translating) && (
        <div className="absolute inset-0 rounded-full border-4 border-t-primary-500 border-r-transparent border-b-primary-500 border-l-transparent animate-spin" />
      )}

      {/* Main Circle */}
      <div className={`
        relative w-32 h-32 rounded-full flex items-center justify-center
        transition-all duration-300 ease-in-out shadow-lg
        ${config.color}
        ring-8 ${config.ring}
      `}>
        <Icon className={`w-12 h-12 transition-transform duration-300 ${status === AppStatus.Processing ? 'animate-pulse' : ''}`} />
      </div>
    </div>
  );
};

export const StatusIndicator: React.FC<{ status: AppStatus }> = ({ status }) => {
    const map = {
        [AppStatus.Idle]: { color: 'bg-gray-400 dark:bg-gray-500', label: 'Idle' },
        [AppStatus.Recording]: { color: 'bg-red-500 dark:bg-red-400', label: 'Recording' },
        [AppStatus.Processing]: { color: 'bg-blue-500 dark:bg-blue-400', label: 'Processing' },
        [AppStatus.Translating]: { color: 'bg-green-500 dark:bg-green-400', label: 'Translating' },
        [AppStatus.Error]: { color: 'bg-red-600 dark:bg-red-500', label: 'Error' },
    }
    const current = map[status];
    return (
        <div className="flex items-center space-x-2 px-3 py-1 bg-gray-50 dark:bg-slate-800 rounded-full border border-gray-200 dark:border-slate-700">
            <div className={`w-2.5 h-2.5 rounded-full ${current.color} ${status === AppStatus.Recording ? 'animate-pulse' : ''}`} />
            <span className="text-xs font-medium text-gray-600 dark:text-gray-400 uppercase tracking-wide">{current.label}</span>
        </div>
    )
}