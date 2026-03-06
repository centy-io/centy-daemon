use crate::cors::build_cors_layer;
use crate::logging::{init_logging, parse_rotation, LogConfig, LOG_FILENAME};
use color_eyre::eyre::Result;
use std::path::PathBuf;
use tower_http::cors::CorsLayer;
use tracing::info;

pub fn setup_logging(
    log_dir: Option<String>,
    log_json: bool,
    log_rotation: &str,
) -> Result<PathBuf> {
    let log_dir = log_dir.map_or_else(
        || {
            dirs::home_dir()
                .unwrap_or_else(|| PathBuf::from("."))
                .join(".centy")
                .join("logs")
        },
        PathBuf::from,
    );
    let log_file = log_dir.join(LOG_FILENAME);
    crate::logging::set_log_file_path(log_file.to_string_lossy().to_string());
    let log_config = LogConfig {
        log_dir,
        json_format: log_json,
        rotation: parse_rotation(log_rotation),
        ..Default::default()
    };
    if let Err(e) = init_logging(log_config) {
        eprintln!();
        eprintln!("Error: Failed to initialize logging: {e}");
        eprintln!();
        eprintln!("Note: Logging could not be set up.");
        eprintln!("Logs: {}", log_file.display());
        eprintln!();
        return Err(e);
    }
    Ok(log_file)
}

pub fn build_cors(origins: &[String]) -> CorsLayer {
    let cors_origins: Vec<String> = origins
        .iter()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect();
    let allow_all_origins = cors_origins.iter().any(|o| o == "*");
    info!(
        "CORS origins: {}",
        if allow_all_origins {
            "*".to_string()
        } else {
            cors_origins.join(", ")
        }
    );
    build_cors_layer(cors_origins)
}
