# After Code Changes

1. `cargo check` - Verify Rust compiles.
2. `npm run build` - TS check + frontend build.
3. `npm run tauri dev` - Test full app.
4. If building: `npm run tauri build`.

No auto-format/lint; manual git commit after verification.
Commit message: Conventional (feat/fix/refactor), no empty commits.