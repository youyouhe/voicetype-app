import React, { useState, useEffect } from 'react';
import { Monitor, Cpu, HardDrive, Wifi, Zap, RefreshCw } from 'lucide-react';
import { Button } from '../ui/Button';
import { TauriService } from '../../services/tauriService';
import { useLanguage } from '../../contexts/LanguageContext';

export const SystemInfoSettings = () => {
  const { t } = useLanguage();

  const [systemInfo, setSystemInfo] = useState<Record<string, string>>({});
  const [isLoading, setIsLoading] = useState(false);

  useEffect(() => {
    loadSystemInfo();
  }, []);

  const loadSystemInfo = async () => {
    setIsLoading(true);
    try {
      const info = await TauriService.getSystemInfo();
      setSystemInfo(info);
    } catch (error) {
      console.error('Failed to load system info:', error);
    } finally {
      setIsLoading(false);
    }
  };

  const renderInfoCard = (title: string, icon: React.ReactNode, items: Array<{label: string, value: string}>) => (
    <div className="bg-gray-50 dark:bg-dark-surface rounded-xl p-5 border border-gray-200 dark:border-dark-border">
      <div className="flex items-center mb-4">
        <div className="p-2 bg-gray-100 dark:bg-gray-800 rounded-lg text-gray-600 dark:text-gray-400 mr-3">
          {icon}
        </div>
        <h3 className="text-lg font-semibold text-gray-900 dark:text-dark-text">{title}</h3>
      </div>
      <div className="space-y-3">
        {items.map((item, index) => (
          <div key={index} className="flex justify-between items-center">
            <span className="text-sm font-medium text-gray-600 dark:text-dark-muted">{item.label}</span>
            <span className="text-sm text-gray-900 dark:text-dark-text font-mono bg-white dark:bg-dark-surface px-3 py-1 rounded border border-gray-200 dark:border-dark-border">
              {item.value}
            </span>
          </div>
        ))}
      </div>
    </div>
  );

  // Organize system info into logical groups
  const getSystemInfoGroups = () => {
    const groups = {
      system: [] as Array<{label: string, value: string}>,
      hardware: [] as Array<{label: string, value: string}>,
      software: [] as Array<{label: string, value: string}>,
      status: [] as Array<{label: string, value: string}>
    };

    Object.entries(systemInfo).forEach(([key, value]) => {
      const normalizedKey = key.toLowerCase();
      const stringValue = String(value);
      if (normalizedKey.includes('platform') || normalizedKey.includes('arch') || normalizedKey.includes('os')) {
        groups.system.push({ label: key, value: stringValue });
      } else if (normalizedKey.includes('cpu') || normalizedKey.includes('memory') || normalizedKey.includes('disk')) {
        groups.hardware.push({ label: key, value: stringValue });
      } else if (normalizedKey.includes('version') || normalizedKey.includes('rust') || normalizedKey.includes('tauri')) {
        groups.software.push({ label: key, value: stringValue });
      } else {
        groups.status.push({ label: key, value: stringValue });
      }
    });

    return groups;
  };

  const infoGroups = getSystemInfoGroups();

  return (
    <div className="space-y-6">
      {/* Header */}
      <div className="flex items-center justify-between">
        <div>
          <h2 className="text-2xl font-bold text-gray-900 dark:text-dark-text">{t.systemInformationWithIcon}</h2>
          <p className="text-gray-500 dark:text-dark-muted mt-1">
            {t.monitorSystemStatus}
          </p>
        </div>
        <Button
          variant="ghost"
          size="sm"
          onClick={loadSystemInfo}
          disabled={isLoading}
          icon={<RefreshCw className={`w-4 h-4 ${isLoading ? 'animate-spin' : ''}`} />}
        >
          {t.refresh}
        </Button>
      </div>

      {/* System Status Overview */}
      <div className="bg-white dark:bg-dark-surface rounded-xl p-6 border border-gray-200 dark:border-dark-border">
        <div className="flex items-center mb-4">
          <Monitor className="w-5 h-5 text-primary-500 mr-2" />
          <h3 className="text-lg font-semibold text-gray-900 dark:text-dark-text">{t.systemStatusCard}</h3>
        </div>

        <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
          {Object.entries(systemInfo)
            .slice(0, 6) // Show first 6 items as overview
            .map(([key, value]) => (
              <div key={key} className="flex justify-between items-center p-3 bg-gray-50 dark:bg-dark-surface rounded-lg">
                <span className="text-sm font-medium text-gray-600 dark:text-dark-muted truncate">{key}:</span>
                <span className="text-sm font-mono text-gray-900 dark:text-dark-text truncate ml-2">{value}</span>
              </div>
            ))}
        </div>
      </div>

      {/* Detailed Information */}
      <div className="grid grid-cols-1 lg:grid-cols-2 gap-6">
        {/* System Information */}
        {infoGroups.system.length > 0 && renderInfoCard(
          t.systemInfo,
          <Monitor className="w-5 h-5" />,
          infoGroups.system
        )}

        {/* Hardware Information */}
        {infoGroups.hardware.length > 0 && renderInfoCard(
          t.hardwareInformation,
          <HardDrive className="w-5 h-5" />,
          infoGroups.hardware
        )}

        {/* Software Information */}
        {infoGroups.software.length > 0 && renderInfoCard(
          t.softwareInformation,
          <Cpu className="w-5 h-5" />,
          infoGroups.software
        )}

        {/* Status Information */}
        {infoGroups.status.length > 0 && renderInfoCard(
          t.voiceAssistantStatusCard,
          <Zap className="w-5 h-5" />,
          infoGroups.status
        )}
      </div>

      {/* Empty State */}
      {Object.keys(systemInfo).length === 0 && !isLoading && (
        <div className="text-center py-12 bg-gray-50 dark:bg-dark-surface rounded-xl border border-gray-200 dark:border-dark-border">
          <Monitor className="w-12 h-12 text-gray-400 mx-auto mb-4" />
          <h3 className="text-lg font-medium text-gray-900 dark:text-dark-text mb-2">{t.noSystemInformation}</h3>
          <p className="text-gray-500 dark:text-dark-muted mb-4">
            {t.unableToRetrieveSystemInfo}
          </p>
          <Button onClick={loadSystemInfo} disabled={isLoading}>
            {isLoading ? t.loading : t.retrySystemInfo}
          </Button>
        </div>
      )}
    </div>
  );
};