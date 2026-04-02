use super::{LogConfig, LOG_FILENAME};
use color_eyre::eyre::Result;
use std::path::Path;
use tracing::Level;
use tracing_appender::rolling::{RollingFileAppender, Rotation};
use tracing_error::ErrorLayer;
use tracing_subscriber::{
    fmt::{self, format::FmtSpan},
    layer::SubscriberExt as _,
    util::SubscriberInitExt as _,
    EnvFilter, Layer as _,
};
fn make_env_filter(log_level: Level) -> EnvFilter {
    EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new(format!("centy_daemon={log_level}")))
}
fn init_json_layers(env_filter: EnvFilter, file_appender: RollingFileAppender, log_level: Level) {
    let json_file_layer = fmt::layer()
        .json()
        .with_writer(file_appender)
        .with_span_events(FmtSpan::CLOSE)
        .with_current_span(true)
        .with_target(true)
        .with_filter(make_env_filter(log_level));
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
}
fn init_text_layers(env_filter: EnvFilter, file_appender: RollingFileAppender, log_level: Level) {
    let file_layer = fmt::layer()
        .with_writer(file_appender)
        .with_span_events(FmtSpan::CLOSE)
        .with_target(true)
        .with_ansi(false)
        .with_filter(make_env_filter(log_level));
    let stdout_layer = fmt::layer()
        .with_writer(std::io::stdout)
        .with_span_events(FmtSpan::CLOSE)
        .with_ansi(true)
        .with_filter(env_filter);
    tracing_subscriber::registry()
        .with(file_layer)
        .with(stdout_layer)
        .with(ErrorLayer::default())
        .init();
}
/// Remove old log files from `log_dir` that start with `LOG_FILENAME`, keeping only the
/// `max_files` most recently modified. Files that cannot be read or removed are silently skipped.
pub fn cleanup_old_log_files(log_dir: &Path, max_files: usize) {
    let Ok(entries) = std::fs::read_dir(log_dir) else {
        return;
    };
    let mut log_files: Vec<(std::time::SystemTime, std::path::PathBuf)> = entries
        .flatten()
        .filter(|e| e.file_name().to_string_lossy().starts_with(LOG_FILENAME))
        .filter_map(|e| {
            let modified = e.metadata().ok()?.modified().ok()?;
            Some((modified, e.path()))
        })
        .collect();
    if log_files.len() <= max_files {
        return;
    }
    // Sort oldest first so we delete from the front.
    log_files.sort_by_key(|(mtime, _)| *mtime);
    let to_delete = log_files.len().saturating_sub(max_files);
    for (_, path) in log_files.into_iter().take(to_delete) {
        drop(std::fs::remove_file(&path));
    }
}

/// Initialize the logging system with the given configuration.
pub fn init_logging(config: LogConfig) -> Result<()> {
    std::fs::create_dir_all(&config.log_dir)?;
    cleanup_old_log_files(&config.log_dir, config.max_log_files);
    let file_appender = RollingFileAppender::new(config.rotation, &config.log_dir, LOG_FILENAME);
    let env_filter = make_env_filter(config.log_level);
    if config.json_format {
        init_json_layers(env_filter, file_appender, config.log_level);
    } else {
        init_text_layers(env_filter, file_appender, config.log_level);
    }
    Ok(())
}
/// Parse rotation period from string.
#[must_use]
pub fn parse_rotation(s: &str) -> Rotation {
    match s.to_lowercase().as_str() {
        "hourly" => Rotation::HOURLY,
        "never" => Rotation::NEVER,
        _ => Rotation::DAILY,
    }
}
