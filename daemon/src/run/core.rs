use crate::cors::build_cors_layer;
use crate::logging::{init_logging, parse_rotation, LogConfig, LOG_FILENAME};
use color_eyre::eyre::{eyre, Result};
use std::path::PathBuf;
use std::process::Child;
use tower_http::cors::CorsLayer;
use tracing::{info, warn};

pub fn setup_logging(
    log_dir_opt: Option<String>,
    log_json: bool,
    log_rotation: &str,
) -> Result<PathBuf> {
    let log_dir = log_dir_opt.map_or_else(
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
    if let Err(error) = init_logging(log_config) {
        eprintln!("\nError: Failed to initialize logging: {error}\n\nNote: Logging could not be set up.\nLogs: {}\n", log_file.display());
        return Err(error);
    }
    Ok(log_file)
}

pub fn build_cors(origins: &[String]) -> CorsLayer {
    let cors_origins: Vec<String> = origins
        .iter()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect();
    let allow_all_origins = cors_origins.iter().any(|origin| origin == "*");
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

pub fn launch_web(enabled: bool, addr: &str) -> Result<Option<Child>> {
    if !enabled {
        return Ok(None);
    }
    let (host, port) = addr
        .rsplit_once(':')
        .ok_or_else(|| eyre!("CENTY_WEB_ADDR must be host:port"))?;
    let child = std::process::Command::new("pnpm")
        .args(["--dir", "web", "dev", "--hostname", host, "--port", port])
        .spawn()
        .map_err(|error| eyre!("failed to launch bundled web app: {error}"))?;
    info!(address = %addr, "Starting bundled Centy web app");
    Ok(Some(child))
}

pub fn stop_web(web_process: Option<Child>) {
    if let Some(mut child) = web_process {
        if let Err(error) = child.kill() {
            warn!(%error, "Failed to stop bundled Centy web app");
        }
    }
}
