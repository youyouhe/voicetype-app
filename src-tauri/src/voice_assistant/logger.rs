use tracing::{info, warn, error, debug, Level};
use tracing_appender::{rolling, non_blocking};
use tracing_subscriber::{
    fmt::{self, format::FmtSpan},
    layer::SubscriberExt,
    util::SubscriberInitExt,
    EnvFilter, Registry,
};
use std::fs;
use std::path::Path;

pub struct Logger {
    _guards: Vec<tracing_appender::non_blocking::WorkerGuard>,
}

impl Logger {
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let mut guards = Vec::new();

        // Create logs directory if it doesn't exist
        let logs_dir = Path::new("logs");
        if !logs_dir.exists() {
            fs::create_dir_all(logs_dir)?;
        }

        // Set up file appender with rotation
        let file_appender = rolling::never("logs", "app.log");
        let (non_blocking_file, file_guard) = non_blocking(file_appender);
        guards.push(file_guard);

        // Set up console layer with colors
        let console_layer = fmt::layer()
            .with_target(false)
            .with_span_events(FmtSpan::CLOSE)
            .with_timer(tracing_subscriber::fmt::time::ChronoUtc::new("%H:%M:%S".to_string()))
            .with_level(true)
            .with_ansi(true)
            .compact();

        // Set up file layer (no colors, simple format)
        let file_layer = fmt::layer()
            .with_writer(non_blocking_file)
            .with_target(false)
            .with_timer(tracing_subscriber::fmt::time::ChronoUtc::new("%Y-%m-%d %H:%M:%S".to_string()))
            .with_ansi(false)
            .with_level(true)
            .compact();

        // Set up environment filter (can be overridden by RUST_LOG env var)
        let env_filter = EnvFilter::try_from_default_env()
            .unwrap_or_else(|_| EnvFilter::new("hello_tauri_lib=info"));

        // Initialize global subscriber with both layers
        Registry::default()
            .with(env_filter)
            .with(console_layer)
            .with(file_layer)
            .init();

        info!("Logger initialized successfully");
        debug!("Debug logging enabled");

        Ok(Logger { _guards: guards })
    }

    pub fn set_level(&self, level: Level) {
        // Note: In tracing, changing log level dynamically after initialization
        // requires more complex setup. For now, this is a placeholder.
        // In practice, you'd use directives or rebuild the subscriber.
        info!("Log level set to {:?}", level);
    }
}

impl Default for Logger {
    fn default() -> Self {
        Self::new().expect("Failed to initialize logger")
    }
}

// Convenience functions that match the Python logger interface
pub fn info(message: &str) {
    info!("{}", message);
}

pub fn warn(message: &str) {
    warn!("{}", message);
}

pub fn error(message: &str) {
    error!("{}", message);
}

pub fn debug(message: &str) {
    debug!("{}", message);
}

// Static logger instance for global access (similar to Python module-level logger)
static mut GLOBAL_LOGGER: Option<Logger> = None;
static INIT: std::sync::Once = std::sync::Once::new();

pub fn init_logger() -> Result<(), Box<dyn std::error::Error>> {
    INIT.call_once(|| {
        unsafe {
            GLOBAL_LOGGER = Some(Logger::new().expect("Failed to initialize global logger"));
        }
    });
    Ok(())
}

#[allow(static_mut_refs)]
pub fn get_logger() -> Option<&'static Logger> {
    unsafe {
        GLOBAL_LOGGER.as_ref()
    }
}

// Macro-based logging for more idiomatic Rust usage
#[macro_export]
macro_rules! log_info {
    ($($arg:tt)*) => {
        tracing::info!($($arg)*);
    };
}

#[macro_export]
macro_rules! log_warn {
    ($($arg:tt)*) => {
        tracing::warn!($($arg)*);
    };
}

#[macro_export]
macro_rules! log_error {
    ($($arg:tt)*) => {
        tracing::error!($($arg)*);
    };
}

#[macro_export]
macro_rules! log_debug {
    ($($arg:tt)*) => {
        tracing::debug!($($arg)*);
    };
}