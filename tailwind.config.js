/** @type {import('tailwindcss').Config} */
export default {
  content: [
    "./index.html",
    "./src/**/*.{js,ts,jsx,tsx}",
  ],
  darkMode: 'class',
  theme: {
    extend: {
      colors: {
        primary: {
          50: '#eff6ff',
          100: '#dbeafe',
          200: '#bfdbfe',
          300: '#93c5fd',
          400: '#60a5fa',
          500: '#3b82f6',
          600: '#2563eb',
          700: '#1d4ed8',
          800: '#1e40af',
          900: '#1e3a8a',
        },
        // Override slate colors for pure black dark mode
        slate: {
          50: '#f8fafc',
          100: '#f1f5f9',
          200: '#e2e8f0',
          300: '#cbd5e1',
          400: '#94a3b8',
          500: '#64748b',
          600: '#475569',
          700: '#334155',
          750: '#1a1a1a',
          800: '#0a0a0a',
          850: '#050505',
          900: '#000000',
          950: '#000000',
        },
        // Custom dark theme colors (with 'dark-' prefix to use in classes)
        'dark-bg': '#000000',
        'dark-surface': '#0a0a0a',
        'dark-border': '#1a1a1a',
        'dark-text': '#ffffff',
        'dark-muted': '#a0a0a0',
      },
    },
  },
  plugins: [],
}