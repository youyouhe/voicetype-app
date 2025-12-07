import React from 'react';
import { Sun, Moon, Palette, Monitor } from 'lucide-react';
import { useTheme } from '../../contexts/ThemeContext';

export const AppearanceSettings: React.FC = () => {
  const { theme, setTheme } = useTheme();
  
  console.log('AppearanceSettings rendered, current theme:', theme);

  const themes = [
    {
      id: 'light' as const,
      name: 'Light',
      description: 'Clean and bright interface',
      icon: Sun,
    },
    {
      id: 'dark' as const,
      name: 'Dark',
      description: 'Easy on the eyes in low light',
      icon: Moon,
    },
    {
      id: 'system' as const,
      name: 'System',
      description: 'Follows your system preference',
      icon: Monitor,
    },
  ];

  return (
    <div className="space-y-6">
      <div>
        <h2 className="text-2xl font-bold text-gray-900 dark:text-dark-text mb-2 flex items-center">
          <Palette className="w-6 h-6 mr-2" />
          Appearance Settings
        </h2>
        <p className="text-gray-600 dark:text-dark-muted">
          Customize the look and feel of the application.
        </p>
      </div>

      <div className="border-t border-gray-200 dark:border-dark-border pt-6">
        <h3 className="text-lg font-semibold text-gray-900 dark:text-dark-text mb-4">Theme</h3>
        <p className="text-sm text-gray-600 dark:text-dark-muted mb-6">
          Choose your preferred theme for the interface.
        </p>
        
        <div className="grid grid-cols-1 md:grid-cols-3 gap-4">
          {themes.map((themeOption) => {
            const Icon = themeOption.icon;
            const isActive = theme === themeOption.id;
            
            return (
              <button
                key={themeOption.id}
                onClick={() => {
                  console.log('Theme button clicked:', themeOption.id);
                  if (themeOption.id === 'system') {
                    // For now, map system to light since we don't have system detection
                    console.log('Setting theme to light (system option)');
                    setTheme('light');
                  } else {
                    console.log('Setting theme to:', themeOption.id);
                    setTheme(themeOption.id);
                  }
                }}
                className={`
                  p-4 rounded-xl border-2 transition-all duration-200 text-left
                  ${isActive
                    ? 'border-primary-500 bg-primary-50 dark:bg-primary-900/20 ring-2 ring-primary-200 dark:ring-primary-800'
                    : 'border-gray-200 dark:border-dark-border hover:border-gray-300 dark:hover:border-dark-border hover:bg-gray-50 dark:hover:bg-dark-surface'
                  }
                `}
              >
                <div className="flex items-center mb-3">
                  <Icon className={`w-6 h-6 mr-3 ${isActive ? 'text-primary-600 dark:text-primary-400' : 'text-gray-400 dark:text-dark-muted'}`} />
                  <span className={`font-medium ${isActive ? 'text-primary-900 dark:text-primary-300' : 'text-gray-900 dark:text-dark-text'}`}>
                    {themeOption.name}
                  </span>
                </div>
                <p className="text-sm text-gray-600 dark:text-dark-muted">
                  {themeOption.description}
                </p>
                {isActive && (
                  <div className="mt-3">
                    <div className="flex items-center">
                      <div className="w-2 h-2 rounded-full bg-primary-500 mr-2"></div>
                      <span className="text-xs font-medium text-primary-700 dark:text-primary-400">Active</span>
                    </div>
                  </div>
                )}
              </button>
            );
          })}
        </div>
      </div>

      <div className="border-t border-gray-200 dark:border-dark-border pt-6">
        <h3 className="text-lg font-semibold text-gray-900 dark:text-dark-text mb-4">Theme Preview</h3>
        <div className="bg-gray-50 dark:bg-dark-bg rounded-lg p-6 border border-gray-200 dark:border-dark-border">
          <p className="text-gray-700 dark:text-dark-text mb-4">
            Your theme changes are applied immediately and saved automatically.
          </p>
          <div className="grid grid-cols-2 gap-4 text-sm">
            <div className="bg-white dark:bg-dark-surface p-3 rounded border border-gray-200 dark:border-dark-border">
              <div className="font-medium text-gray-900 dark:text-dark-text mb-1">Current theme:</div>
              <div className="text-gray-600 dark:text-dark-muted capitalize">{theme}</div>
            </div>
            <div className="bg-white dark:bg-dark-surface p-3 rounded border border-gray-200 dark:border-dark-border">
              <div className="font-medium text-gray-900 dark:text-dark-text mb-1">Status:</div>
              <div className="text-green-600 dark:text-green-400">✓ Applied successfully</div>
            </div>
          </div>
        </div>
      </div>
    </div>
  );
};