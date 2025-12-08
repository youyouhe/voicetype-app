use tauri::{AppHandle, WebviewWindow, tray::TrayIconBuilder, Size, PhysicalSize};
use std::sync::{Arc, Mutex};

pub struct SystemTrayManager {
    app_handle: AppHandle,
    overlay_window: Arc<Mutex<Option<WebviewWindow>>>,
}

impl SystemTrayManager {
    pub fn new(app_handle: AppHandle) -> Self {
        Self {
            app_handle,
            overlay_window: Arc::new(Mutex::new(None)),
        }
    }

    pub fn create_tray_icon() -> Result<TrayIconBuilder<tauri::Wry>, Box<dyn std::error::Error>> {
        let tray = TrayIconBuilder::new()
            .title("Flash-Input")
            .tooltip("Flash-Input 语音输入助手");

        Ok(tray)
    }

    pub fn create_overlay_window(&self) -> Result<WebviewWindow, Box<dyn std::error::Error>> {
        // Always use a fresh label to avoid conflicts
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        let window_label = format!("overlay_{}", timestamp);

        println!("🔄 Creating new overlay window with label: {}", window_label);

        // Create overlay window with transparent background and always on top
        let window = tauri::WebviewWindowBuilder::new(
            &self.app_handle,
            window_label,
            tauri::WebviewUrl::App("/overlay.html".into())
        )
        .title("麦克风悬浮窗")
        .transparent(true)
        .decorations(false)
        .always_on_top(true)
        .skip_taskbar(true)
        .visible(true)
        .inner_size(200.0, 200.0)
        .resizable(false)
        .shadow(false)
        .build()?;

        // Store reference to overlay window
        let mut overlay = self.overlay_window.lock().unwrap();
        *overlay = Some(window.clone());

        // Set window position and size with retry logic
        let window_clone = window.clone();
        let screen_width = 1920;
        let screen_height = 1280;
        let margin_bottom = 80;

        let bottom_x = screen_width / 2; // 960 (center horizontally)
        let bottom_y = screen_height - margin_bottom; // 1280 - 80 = 1200
        let target_x = bottom_x - 100; // 860 (center 200px window)
        let target_y = bottom_y - 100; // 1100 (position 200px window)

        println!("  - Attempting to set window to 200x200 at position ({}, {})", target_x, target_y);

        // Try multiple times to set size and position
        for attempt in 1..=3 {
            std::thread::sleep(std::time::Duration::from_millis(100));

            // Force set size
            if let Ok(_) = window_clone.set_size(Size::Physical(PhysicalSize {
                width: 200,
                height: 200,
            })) {
                println!("  - Size set attempt {}: OK", attempt);
            } else {
                println!("  - Size set attempt {}: FAILED", attempt);
            }

            // Force set position
            if let Ok(_) = window_clone.set_position(tauri::Position::Physical(tauri::PhysicalPosition {
                x: target_x,
                y: target_y,
            })) {
                println!("  - Position set attempt {}: OK", attempt);
            } else {
                println!("  - Position set attempt {}: FAILED", attempt);
            }
        }

        // Debug: Print final window info
        println!("🔍 Final window state after corrections:");
        if let Ok(is_visible) = window.is_visible() {
            println!("  - Window visible: {}", is_visible);
        }

        if let Ok(position) = window.outer_position() {
            println!("  - Final window position: {:?}", position);
        }

        if let Ok(size) = window.outer_size() {
            println!("  - Final window size: {:?}", size);
        }

        if let Ok(inner_size) = window.inner_size() {
            println!("  - Final window inner size: {:?}", inner_size);
        }

        println!("  - Target position: ({}, {})", target_x, target_y);
        println!("  - Target size: 200x200");
        println!("  - Always on top: true");
        println!("  - Transparent: true");

        Ok(window)
    }

    pub fn toggle_overlay_window(&self) {
        let overlay = self.overlay_window.lock().unwrap();
        
        if let Some(ref window) = *overlay {
            if window.is_visible().unwrap_or(false) {
                let _ = window.hide();
            } else {
                self.position_overlay_at_cursor(window);
                let _ = window.show();
                let _ = window.set_focus();
            }
        } else {
            // Create overlay window if it doesn't exist
            drop(overlay); // Release the lock before creating new window
            if let Ok(window) = self.create_overlay_window() {
                self.position_overlay_at_cursor(&window);
                let _ = window.show();
                let _ = window.set_focus();
            }
        }
    }

    fn position_overlay_at_cursor(&self, window: &WebviewWindow) {
        // Get current cursor position
        if let Ok((x, y)) = self.get_cursor_position() {
            let _ = window.set_position(tauri::Position::Physical(tauri::PhysicalPosition {
                x: x - 30, // Center the 60x60 window on cursor
                y: y - 30,
            }));
        }
    }

    fn get_cursor_position(&self) -> Result<(i32, i32), Box<dyn std::error::Error>> {
        // This is a simplified implementation
        // In a real implementation, you would use platform-specific APIs
        // to get the current cursor position
        use std::process::Command;
        
        if cfg!(target_os = "linux") {
            let output = Command::new("xdotool")
                .args(&["getmouselocation"])
                .output()?;
            
            let output_str = String::from_utf8(output.stdout)?;
            let parts: Vec<&str> = output_str.split_whitespace().collect();
            
            if parts.len() >= 2 {
                let x: i32 = parts[0].trim_start_matches("x:").parse()?;
                let y: i32 = parts[1].trim_start_matches("y:").parse()?;
                return Ok((x, y));
            }
        }
        
        // Fallback to center of screen
        Ok((800, 600))
    }

    pub fn show_overlay_at_cursor(&self) {
        let overlay = self.overlay_window.lock().unwrap();
        
        if let Some(ref window) = *overlay {
            self.position_overlay_at_cursor(window);
            let _ = window.show();
            let _ = window.set_focus();
        } else {
            // Create overlay window if it doesn't exist
            drop(overlay); // Release the lock before creating new window
            if let Ok(window) = self.create_overlay_window() {
                self.position_overlay_at_cursor(&window);
                let _ = window.show();
                let _ = window.set_focus();
            }
        }
    }

    pub fn hide_overlay_window(&self) {
        let overlay = self.overlay_window.lock().unwrap();
        
        if let Some(ref window) = *overlay {
            let _ = window.hide();
        }
    }
}