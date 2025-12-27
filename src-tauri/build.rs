fn main() {
    // Fix SQLite linking on Windows
    #[cfg(target_os = "windows")]
    {
        // Tell libsqlite3-sys to use the bundled SQLite
        println!("cargo:rustc-cfg=libsqlite3_sys_bundled");
        // Link Windows system libraries that SQLite may need
        println!("cargo:rustc-link-lib=shell32");
        println!("cargo:rustc-link-lib=ole32");
    }

    tauri_build::build()
}
