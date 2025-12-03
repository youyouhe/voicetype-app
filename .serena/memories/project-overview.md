# Tauri v2 Desktop App Overview

## Purpose
Standard \"Hello Tauri\" template for cross-platform desktop app. Frontend (web) calls Rust backend via Tauri IPC commands like `greet` and `add`.

## Tech Stack
- **Frontend**: Vite + Vanilla TypeScript (no framework). Entry: `index.html` → `src/main.ts`.
- **Backend**: Rust with Tauri v2, `tauri_plugin_opener`. Commands in `src-tauri/src/lib.rs`.
- **Build**: Vite bundles to `dist/`, Tauri embeds it.
- **Separate**: `ui/` is unrelated React/Vite project.

## Structure
```
.
├── src/              # Frontend TS/CSS/assets
├── src-tauri/        # Rust backend + config
├── ui/               # Unrelated React app
├── package.json      # NPM scripts/deps
├── vite.config.ts    # Tauri-optimized Vite
└── CLAUDE.md         # Dev guidance
```
Key integration: Dev server at localhost:1420, Tauri devUrl points here.