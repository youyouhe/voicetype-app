# Code Style & Conventions

No explicit linters/formatters (eslint, prettier) or configs found (.eslintrc, .prettierrc absent).

## Rust
- Standard Rustfmt (cargo fmt).
- Commands use `#[tauri::command]`, serde derive.

## TypeScript
- Standard TSC strict mode (tsconfig.json).
- Vanilla JS/TS: DOM queries, async invoke.

## General
- Naming: camelCase (TS), snake_case (Rust).
- No tests in codebase.
- Chinese strings in greet (user-localized).
- Follow Tauri v2 patterns: invoke_handler! in lib.rs::run().