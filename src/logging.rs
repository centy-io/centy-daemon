use std::path::PathBuf;
use tracing::Level;
use tracing_appender::rolling::{RollingFileAppender, Rotation};
use tracing_subscriber::{
    fmt::{self, format::FmtSpan},
    layer::SubscriberExt,
    util::SubscriberInitExt,
    EnvFilter, Layer,
};

/// Configuration for the logging system.
pub struct LogConfig {
    /// Directory where log files will be written.
    pub log_dir: PathBuf,
    /// Default log level when RUST_LOG is not set.
    pub log_level: Level,
    /// Whether to use JSON format for logs.
    pub json_format: bool,
    /// Log rotation period.
    pub rotation: Rotation,
}

impl Default for LogConfig {
    fn default() -> Self {
        let log_dir = dirs::home_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join(".centy")
            .join("logs");

        Self {
            log_dir,
            log_level: Level::INFO,
            json_format: false,
            rotation: Rotation::DAILY,
        }
    }
}

/// Initialize the logging system with the given configuration.
///
/// This sets up dual output to both files and stdout, with support for:
/// - Runtime log level configuration via RUST_LOG environment variable
/// - JSON or human-readable format
/// - Log file rotation (daily, hourly, or never)
///
/// # Errors
///
/// Returns an error if the log directory cannot be created.
pub fn init_logging(config: LogConfig) -> anyhow::Result<()> {
    // Create log directory if it doesn't exist
    std::fs::create_dir_all(&config.log_dir)?;

    // File appender with rotation
    let file_appender = RollingFileAppender::new(
        config.rotation,
        &config.log_dir,
        "centy-daemon.log",
    );

    // Build env filter (runtime configurable via RUST_LOG)
    // Default to the configured level for centy_daemon
    let env_filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| {
        EnvFilter::new(format!("centy_daemon={}", config.log_level))
    });

    if config.json_format {
        // JSON format for production/log aggregation
        let json_file_layer = fmt::layer()
            .json()
            .with_writer(file_appender)
            .with_span_events(FmtSpan::CLOSE)
            .with_current_span(true)
            .with_target(true)
            .with_filter(EnvFilter::try_from_default_env().unwrap_or_else(|_| {
                EnvFilter::new(format!("centy_daemon={}", config.log_level))
            }));

        let json_stdout_layer = fmt::layer()
            .json()
            .with_writer(std::io::stdout)
            .with_span_events(FmtSpan::CLOSE)
            .with_current_span(true)
            .with_target(true)
            .with_filter(env_filter);

        tracing_subscriber::registry()
            .with(json_file_layer)
            .with(json_stdout_layer)
            .init();
    } else {
        // Human-readable format for development
        let file_layer = fmt::layer()
            .with_writer(file_appender)
            .with_span_events(FmtSpan::CLOSE)
            .with_target(true)
            .with_ansi(false) // No ANSI colors in files
            .with_filter(EnvFilter::try_from_default_env().unwrap_or_else(|_| {
                EnvFilter::new(format!("centy_daemon={}", config.log_level))
            }));

        let stdout_layer = fmt::layer()
            .with_writer(std::io::stdout)
            .with_span_events(FmtSpan::CLOSE)
            .with_ansi(true) // Colors for terminal
            .with_filter(env_filter);

        tracing_subscriber::registry()
            .with(file_layer)
            .with(stdout_layer)
            .init();
    }

    Ok(())
}

/// Parse rotation period from string.
pub fn parse_rotation(s: &str) -> Rotation {
    match s.to_lowercase().as_str() {
        "hourly" => Rotation::HOURLY,
        "never" => Rotation::NEVER,
        _ => Rotation::DAILY,
    }
}
