mod common;
mod config;
mod docs;
mod features;
mod issue;
mod link;
mod llm;
mod logging;
mod manifest;
mod metrics;
mod migration;
mod pr;
mod reconciliation;
mod registry;
mod search;
mod server;
mod source_control;
mod template;
mod user;
mod utils;
mod version;
mod workspace;

use clap::Parser;
use http::Method;
use logging::{init_logging, parse_rotation, LogConfig};
use server::proto::centy_daemon_server::CentyDaemonServer;
use server::{CentyDaemonService, ShutdownSignal};
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::watch;
use tonic::transport::Server;
use tower_http::cors::{AllowOrigin, Any, CorsLayer};
use tracing::info;

const DEFAULT_ADDR: &str = "127.0.0.1:50051";
const DEFAULT_CORS_ORIGINS: &str = "http://localhost,https://localhost,http://127.0.0.1,https://127.0.0.1,tauri://localhost,https://tauri.localhost";

/// Centy Daemon - Local-first issue and documentation tracker service
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Address to bind the server to
    #[arg(short, long, env = "CENTY_DAEMON_ADDR", default_value = DEFAULT_ADDR)]
    addr: String,

    /// Comma-separated list of allowed CORS origins.
    /// Use "*" to allow all origins (not recommended for production).
    /// Example: --cors-origins=https://app.centy.io,http://localhost:5180
    #[arg(
        long,
        env = "CENTY_CORS_ORIGINS",
        default_value = DEFAULT_CORS_ORIGINS,
        value_delimiter = ','
    )]
    cors_origins: Vec<String>,

    /// Enable JSON log format (for production/log aggregation)
    #[arg(long, env = "CENTY_LOG_JSON", default_value = "false")]
    log_json: bool,

    /// Log rotation period: daily, hourly, or never
    #[arg(long, env = "CENTY_LOG_ROTATION", default_value = "daily")]
    log_rotation: String,

    /// Custom log directory (default: ~/.centy/logs)
    #[arg(long, env = "CENTY_LOG_DIR")]
    log_dir: Option<String>,
}

// Include the file descriptor set for gRPC reflection
pub const FILE_DESCRIPTOR_SET: &[u8] = tonic::include_file_descriptor_set!("centy_descriptor");

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Parse CLI arguments first (before logging, so we can use log config)
    let args = Args::parse();

    // Configure and initialize logging
    let log_dir = args.log_dir.map(PathBuf::from).unwrap_or_else(|| {
        dirs::home_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join(".centy")
            .join("logs")
    });

    let log_config = LogConfig {
        log_dir,
        json_format: args.log_json,
        rotation: parse_rotation(&args.log_rotation),
        ..Default::default()
    };

    init_logging(log_config)?;

    // Parse address
    let addr = args.addr.parse()?;

    // Process CORS origins
    let cors_origins: Vec<String> = args
        .cors_origins
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

    // Create shutdown signal channel
    let (shutdown_tx, mut shutdown_rx) = watch::channel(ShutdownSignal::None);
    let shutdown_tx = Arc::new(shutdown_tx);

    // Get the current executable path for restart
    let exe_path = std::env::current_exe().ok();

    let service = CentyDaemonService::new(shutdown_tx.clone(), exe_path);

    // Create reflection service
    let reflection_service = tonic_reflection::server::Builder::configure()
        .register_encoded_file_descriptor_set(FILE_DESCRIPTOR_SET)
        .build_v1()?;

    // Configure CORS for gRPC-Web
    // Always allow *.centy.io origins, plus any configured origins
    let cors = CorsLayer::new()
        .allow_origin(AllowOrigin::predicate(move |origin, _| {
            if allow_all_origins {
                return true;
            }

            if let Ok(origin_str) = origin.to_str() {
                // Always allow *.centy.io
                if origin_str.ends_with(".centy.io")
                    || origin_str == "https://centy.io"
                    || origin_str == "http://centy.io"
                {
                    return true;
                }

                // Check configured origins
                cors_origins
                    .iter()
                    .any(|allowed| origin_str.starts_with(allowed))
            } else {
                false
            }
        }))
        .allow_methods([Method::GET, Method::POST, Method::OPTIONS])
        .allow_headers(Any)
        .expose_headers(Any);

    info!("Starting Centy daemon on {} (gRPC + gRPC-Web)", addr);

    Server::builder()
        .accept_http1(true) // Required for gRPC-Web
        .layer(cors)
        .layer(tonic_web::GrpcWebLayer::new())
        .add_service(reflection_service)
        .add_service(CentyDaemonServer::new(service))
        .serve_with_shutdown(addr, async move {
            // Wait for shutdown signal
            loop {
                shutdown_rx.changed().await.ok();
                match *shutdown_rx.borrow() {
                    ShutdownSignal::Shutdown => {
                        info!("Received shutdown signal, stopping server...");
                        break;
                    }
                    ShutdownSignal::Restart => {
                        info!("Received restart signal, stopping server...");
                        break;
                    }
                    ShutdownSignal::None => {}
                }
            }
        })
        .await?;

    info!("Centy daemon stopped");
    Ok(())
}
