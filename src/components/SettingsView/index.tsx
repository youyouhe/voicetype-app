import React, { memo } from 'react';
import { Mic, Keyboard, Palette, Sliders, Monitor } from 'lucide-react';
import { NavTab } from '../../types';
import { ASRSettings, ShortcutSettings, PlaceholderSettings } from './SettingsContent';
import { AppearanceSettings } from './AppearanceSettings';
import { AdvancedSettings } from './AdvancedSettings';
import { SystemInfoSettings } from './SystemInfoSettings';

export interface SettingsViewProps {
  activeSettingsTab: string;
  setActiveSettingsTab: (tab: string) => void;
}

export const SettingsView = memo<SettingsViewProps>(({ activeSettingsTab, setActiveSettingsTab }) => {
  const tabs: NavTab[] = [
    { id: 'asr', label: 'ASR Service', icon: Mic },
    { id: 'shortcuts', label: 'Shortcuts', icon: Keyboard },
    { id: 'appearance', label: 'Appearance', icon: Palette },
    { id: 'advanced', label: 'Advanced', icon: Sliders },
    { id: 'system', label: 'System', icon: Monitor },
  ];

  return (
    <div className="max-w-6xl mx-auto px-4 py-8 flex flex-col md:flex-row gap-8 animate-in fade-in duration-300">
      {/* Sidebar */}
      <aside className="w-full md:w-64 flex-shrink-0 bg-white dark:bg-dark-surface rounded-xl p-4 border border-gray-200 dark:border-dark-border">
        <nav className="space-y-1">
          {tabs.map((tab) => {
            const Icon = tab.icon;
            const isActive = activeSettingsTab === tab.id;
            return (
              <button
                key={tab.id}
                onClick={() => setActiveSettingsTab(tab.id)}
                className={`
                  w-full flex items-center space-x-3 px-4 py-3 rounded-xl text-sm font-medium transition-all duration-200
                  ${isActive
                    ? 'bg-gray-100 dark:bg-dark-bg text-primary-600 dark:text-primary-400 shadow-sm ring-1 ring-gray-200 dark:ring-dark-border'
                    : 'text-gray-600 dark:text-dark-muted hover:bg-gray-100 dark:hover:bg-dark-bg hover:text-gray-900 dark:hover:text-dark-text'}
                `}
              >
                <Icon className={`w-5 h-5 ${isActive ? 'text-primary-500' : 'text-gray-400'}`} />
                <span>{tab.label}</span>
              </button>
            );
          })}
        </nav>
      </aside>

      {/* Content Area */}
      <main className="flex-1 min-h-[500px] bg-white dark:bg-dark-surface rounded-xl p-6 border border-gray-200 dark:border-dark-border">
        {activeSettingsTab === 'asr' && <ASRSettings />}
        {activeSettingsTab === 'shortcuts' && <ShortcutSettings />}
        {activeSettingsTab === 'appearance' && <AppearanceSettings />}
        {activeSettingsTab === 'advanced' && <AdvancedSettings />}
        {activeSettingsTab === 'system' && <SystemInfoSettings />}
      </main>
    </div>
  );
});