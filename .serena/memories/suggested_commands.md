# Essential Development Commands (Linux)

## Run/Dev
- `npm run dev` - Vite dev server (localhost:1420)
- `npm run tauri` or `npm run tauri dev` - Full Tauri dev (web + Rust)
- `npm run preview` - Preview prod build

## Build
- `npm run build` - TS check + Vite build to dist/
- `npm run tauri build` - Full Tauri production build

## Check/Test
- `cargo check` - Fast Rust check
- `cargo test` - Rust unit tests
- `tsc` - TS type check (part of npm run build)

## Utils
- `git status/diff/log`
- `ls -la`, `cd`, `grep -r "pattern" src/`
- No custom lint/format; standard editorconfig/VSCode if configured.