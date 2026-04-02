mod init;
pub use init::{cleanup_old_log_files, init_logging, parse_rotation};
use std::path::PathBuf;
use std::sync::OnceLock;
use tracing::Level;
use tracing_appender::rolling::Rotation;
/// Log filename used by the daemon.
pub const LOG_FILENAME: &str = "centy-daemon.log";
/// Default maximum number of log files to retain.
pub const DEFAULT_MAX_LOG_FILES: usize = 7;
/// Global log file path, set once at startup.
static LOG_FILE_PATH: OnceLock<String> = OnceLock::new();
/// Store the log file path for later retrieval (e.g., in structured error responses).
pub fn set_log_file_path(path: String) {
    drop(LOG_FILE_PATH.set(path));
}
/// Get the log file path set at startup.
pub fn get_log_file_path() -> &'static str {
    LOG_FILE_PATH.get().map_or("", |s| s.as_str())
}
/// Configuration for the logging system.
pub struct LogConfig {
    pub log_dir: PathBuf,
    pub log_level: Level,
    pub json_format: bool,
    pub rotation: Rotation,
    /// Maximum number of log files to retain. Older files are deleted at startup.
    pub max_log_files: usize,
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
            max_log_files: DEFAULT_MAX_LOG_FILES,
        }
    }
}
#[cfg(test)]
#[path = "../logging_tests.rs"]
mod logging_tests;
