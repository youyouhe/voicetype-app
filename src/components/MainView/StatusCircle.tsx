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
        return { color: 'bg-gray-100 text-gray-400', icon: Mic, ring: 'ring-gray-200' };
      case AppStatus.Recording:
        return { color: 'bg-red-50 text-red-500', icon: Mic, ring: 'ring-red-100' };
      case AppStatus.Processing:
        return { color: 'bg-blue-50 text-blue-500', icon: Loader, ring: 'ring-blue-100' };
      case AppStatus.Translating:
        return { color: 'bg-green-50 text-green-500', icon: Languages, ring: 'ring-green-100' };
      case AppStatus.Error:
        return { color: 'bg-red-50 text-red-600', icon: XCircle, ring: 'ring-red-200' };
      default:
        return { color: 'bg-gray-100', icon: Mic, ring: 'ring-gray-200' };
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
        [AppStatus.Idle]: { color: 'bg-gray-400', label: 'Idle' },
        [AppStatus.Recording]: { color: 'bg-red-500', label: 'Recording' },
        [AppStatus.Processing]: { color: 'bg-blue-500', label: 'Processing' },
        [AppStatus.Translating]: { color: 'bg-green-500', label: 'Translating' },
        [AppStatus.Error]: { color: 'bg-red-600', label: 'Error' },
    }
    const current = map[status];
    return (
        <div className="flex items-center space-x-2 px-3 py-1 bg-gray-50 rounded-full border border-gray-200">
            <div className={`w-2.5 h-2.5 rounded-full ${current.color} ${status === AppStatus.Recording ? 'animate-pulse' : ''}`} />
            <span className="text-xs font-medium text-gray-600 uppercase tracking-wide">{current.label}</span>
        </div>
    )
}