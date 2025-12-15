import React, { memo } from 'react';
import { Mic, Settings } from 'lucide-react';
import { AppStatus } from '../../types';
import { StatusIndicator } from '../MainView/StatusCircle';
import { Button } from '../ui/Button';
import { ThemeToggle } from '../ThemeToggle';

export interface TopBarProps {
  appStatus: AppStatus;
  currentView: 'dashboard' | 'settings';
  setCurrentView: (view: 'dashboard' | 'settings') => void;
}

export const TopBar = memo<TopBarProps>(({
  appStatus,
  currentView,
  setCurrentView
}) => {
  return (
    <header className="bg-white/80 dark:bg-dark-surface/80 backdrop-blur-md border-b border-gray-200 dark:border-dark-border px-6 py-3 sticky top-0 z-50">
      <div className="flex items-center justify-between max-w-7xl mx-auto">
        <div className="flex items-center space-x-4">
          <div className="bg-primary-50 p-2 rounded-lg">
             <Mic className="w-5 h-5 text-primary-600" />
          </div>
          <div>
            <h1 className="text-lg font-bold text-gray-900 dark:text-dark-text leading-none">Flash-Input</h1>
            <p className="text-xs text-gray-500 dark:text-dark-muted mt-0.5">Tauri Client v1.0.0</p>
          </div>
          <div className="hidden sm:block h-6 w-px bg-gray-200 dark:bg-dark-border mx-2" />
          <div className="hidden sm:block">
            <StatusIndicator status={appStatus} />
          </div>
        </div>

        <div className="flex items-center space-x-2">
          <ThemeToggle />
          <Button
            variant="ghost"
            size="sm"
            icon={<Settings className="w-4 h-4"/>}
            onClick={() => setCurrentView(currentView === 'dashboard' ? 'settings' : 'dashboard')}
          >
            {currentView === 'dashboard' ? 'Settings' : 'Dashboard'}
          </Button>
        </div>
      </div>
    </header>
  );
});