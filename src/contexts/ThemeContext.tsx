import React, { createContext, useContext, useState, useEffect, ReactNode } from 'react';
import { StorageService } from '../services/storageService';

export type Theme = 'light' | 'dark';

interface ThemeContextType {
  theme: Theme;
  toggleTheme: () => void;
  setTheme: (theme: Theme) => void;
}

const ThemeContext = createContext<ThemeContextType | undefined>(undefined);

export const useTheme = () => {
  const context = useContext(ThemeContext);
  if (context === undefined) {
    throw new Error('useTheme must be used within a ThemeProvider');
  }
  return context;
};

interface ThemeProviderProps {
  children: ReactNode;
}

export const ThemeProvider: React.FC<ThemeProviderProps> = ({ children }) => {
  const [theme, setThemeState] = useState<Theme>('light');

  // Load theme from localStorage on mount
  useEffect(() => {
    console.log('ThemeProvider: Loading theme from localStorage');
    const saved = localStorage.getItem('theme');
    console.log('ThemeProvider: Found saved theme:', saved);
    if (saved && (saved === 'light' || saved === 'dark')) {
      setThemeState(saved as Theme);
    }
  }, []);

  // Save theme to localStorage and apply to DOM whenever it changes
  useEffect(() => {
    console.log('ThemeProvider: Theme changed to:', theme);
    // Save to localStorage
    localStorage.setItem('theme', theme);
    console.log('ThemeProvider: Theme saved to localStorage');
    
    // Apply theme to document element immediately
    const root = document.documentElement;
    console.log('ThemeProvider: Applying theme to DOM, current classes:', root.className);
    if (theme === 'dark') {
      root.classList.add('dark');
      console.log('ThemeProvider: Added dark class, new classes:', root.className);
    } else {
      root.classList.remove('dark');
      console.log('ThemeProvider: Removed dark class, new classes:', root.className);
    }
  }, [theme]);

  const setTheme = (newTheme: Theme) => {
    setThemeState(newTheme);
  };

  const toggleTheme = () => {
    setThemeState(prev => prev === 'light' ? 'dark' : 'light');
  };

  return (
    <ThemeContext.Provider value={{ theme, toggleTheme, setTheme }}>
      {children}
    </ThemeContext.Provider>
  );
};