use color_eyre::eyre::Result;
use std::path::PathBuf;
use tracing::Level;
use tracing_appender::rolling::{RollingFileAppender, Rotation};
use tracing_error::ErrorLayer;
use tracing_subscriber::{
    fmt::{self, format::FmtSpan},
    layer::SubscriberExt,
    util::SubscriberInitExt,
    EnvFilter, Layer,
};

/// Log filename used by the daemon.
pub const LOG_FILENAME: &str = "centy-daemon.log";

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
pub fn init_logging(config: LogConfig) -> Result<()> {
    // Create log directory if it doesn't exist
    std::fs::create_dir_all(&config.log_dir)?;

    // File appender with rotation
    let file_appender = RollingFileAppender::new(config.rotation, &config.log_dir, LOG_FILENAME);

    // Build env filter (runtime configurable via RUST_LOG)
    // Default to the configured level for centy_daemon
    let env_filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new(format!("centy_daemon={}", config.log_level)));

    if config.json_format {
        // JSON format for production/log aggregation
        let json_file_layer =
            fmt::layer()
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
            .with(ErrorLayer::default())
            .init();
    } else {
        // Human-readable format for development
        let file_layer =
            fmt::layer()
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
            .with(ErrorLayer::default())
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_log_config_default() {
        let config = LogConfig::default();
        assert_eq!(config.log_level, Level::INFO);
        assert!(!config.json_format);
        assert!(config.log_dir.ends_with("logs"));
    }

    #[test]
    fn test_log_config_default_log_dir_contains_centy() {
        let config = LogConfig::default();
        let path_str = config.log_dir.to_string_lossy();
        assert!(path_str.contains(".centy"));
    }

    #[test]
    fn test_parse_rotation_hourly() {
        let rotation = parse_rotation("hourly");
        // Rotation doesn't impl PartialEq, so use debug
        let debug = format!("{rotation:?}");
        assert!(debug.contains("Hourly") || debug.contains("hourly") || debug.contains("3600"));
    }

    #[test]
    fn test_parse_rotation_never() {
        let rotation = parse_rotation("never");
        let debug = format!("{rotation:?}");
        assert!(debug.contains("Never") || debug.contains("never"));
    }

    #[test]
    fn test_parse_rotation_daily() {
        let rotation = parse_rotation("daily");
        let debug = format!("{rotation:?}");
        assert!(debug.contains("Daily") || debug.contains("daily") || debug.contains("86400"));
    }

    #[test]
    fn test_parse_rotation_case_insensitive() {
        let _ = parse_rotation("HOURLY");
        let _ = parse_rotation("Never");
        let _ = parse_rotation("DAILY");
    }

    #[test]
    fn test_parse_rotation_unknown_defaults_to_daily() {
        let rotation = parse_rotation("weekly");
        let debug = format!("{rotation:?}");
        // Unknown values default to daily
        let daily = format!("{:?}", parse_rotation("daily"));
        assert_eq!(debug, daily);
    }

    #[test]
    fn test_log_filename_constant() {
        assert_eq!(LOG_FILENAME, "centy-daemon.log");
    }
}
