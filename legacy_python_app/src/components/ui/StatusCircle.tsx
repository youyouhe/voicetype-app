import React from 'react';
import { motion } from 'framer-motion';
import { Mic, Activity, CheckCircle, AlertCircle, XCircle, Languages, Loader } from 'lucide-react';
import { AppStatus } from '../../types';

interface StatusCircleProps {
  status: AppStatus;
  size?: 'sm' | 'md' | 'lg';
}

export const StatusCircle: React.FC<StatusCircleProps> = ({ status, size = 'md' }) => {
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

  const sizeClasses = {
    sm: 'w-16 h-16',
    md: 'w-24 h-24',
    lg: 'w-32 h-32'
  };

  return (
    <div className="relative flex items-center justify-center">
      {/* Outer pulsing ring for recording */}
      {status === AppStatus.Recording && (
        <motion.div
          className="absolute inset-0 rounded-full bg-red-500 opacity-20"
          animate={{
            scale: [1, 1.2, 1],
            opacity: [0.2, 0.4, 0.2],
          }}
          transition={{
            duration: 1.5,
            repeat: Infinity,
            ease: "easeInOut"
          }}
        />
      )}
      
      {/* Spinning ring for processing/translating */}
      {(status === AppStatus.Processing || status === AppStatus.Translating) && (
        <motion.div
          className="absolute inset-0 rounded-full border-4 border-t-transparent border-r-transparent border-b-primary-500 border-l-transparent"
          animate={{ rotate: 360 }}
          transition={{ duration: 2, repeat: Infinity, ease: "linear" }}
        />
      )}

      {/* Main Circle */}
      <motion.div
        className={`
          relative rounded-full flex items-center justify-center
          transition-all duration-300 ease-in-out shadow-lg
          ${config.color}
          ${config.ring}
          ${sizeClasses[size]}
        `}
        initial={{ scale: 0.8, opacity: 0 }}
        animate={{ scale: 1, opacity: 1 }}
        transition={{ duration: 0.3, ease: "easeInOut" }}
      >
        <Icon className={`transition-transform duration-300 ${status === AppStatus.Processing ? 'animate-spin' : ''}`} />
      </motion.div>
    </div>
  );
};

export const StatusIndicator: React.FC<{ status: AppStatus }> = ({ status }) => {
  const map = {
      [AppStatus.Idle]: { color: 'bg-gray-400', label: '空闲' },
      [AppStatus.Recording]: { color: 'bg-red-500', label: '录音中' },
      [AppStatus.Processing]: { color: 'bg-blue-500', label: '处理中' },
      [AppStatus.Translating]: { color: 'bg-green-500', label: '翻译中' },
      [AppStatus.Error]: { color: 'bg-red-600', label: '错误' },
  };
  const current = map[status];
  return (
      <div className="flex items-center space-x-2 px-3 py-1 bg-gray-50 rounded-full border border-gray-200">
          <motion.div 
            className={`w-2.5 h-2.5 rounded-full ${current.color}`}
            animate={status === AppStatus.Recording ? { scale: [1, 1.2, 1] } : { scale: 1 }}
            transition={{ duration: 0.5, repeat: status === AppStatus.Recording ? Infinity : 0 }}
          />
          <span className="text-xs font-medium text-gray-600 uppercase tracking-wide">{current.label}</span>
      </div>
  )
}