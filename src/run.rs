use crate::app::{self, report_server_error};
use crate::cors::build_cors_layer;
use crate::logging::{init_logging, parse_rotation, LogConfig, LOG_FILENAME};
use crate::server::proto::centy_daemon_server::CentyDaemonServer;
use crate::server::{CentyDaemonService, ShutdownSignal};
use color_eyre::eyre::Result;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::watch;
use tonic::transport::Server;
use tower_http::trace::{DefaultMakeSpan, TraceLayer};
use tracing::{info, warn, Level};

pub async fn run(args: app::Args) -> Result<()> {
    let log_dir = args.log_dir.map(PathBuf::from).unwrap_or_else(|| {
        dirs::home_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join(".centy")
            .join("logs")
    });
    let log_file = log_dir.join(LOG_FILENAME);
    crate::logging::set_log_file_path(log_file.to_string_lossy().to_string());
    let log_config = LogConfig {
        log_dir,
        json_format: args.log_json,
        rotation: parse_rotation(&args.log_rotation),
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
    let user_cfg = crate::user_config::load_user_config().unwrap_or_else(|e| {
        warn!("Failed to load user config, using defaults: {e}");
        crate::user_config::UserConfig::default()
    });
    crate::registry::init_ignore_paths(&user_cfg.registry.ignore_paths);
    let addr = args.addr.parse()?;
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
    let cors = build_cors_layer(cors_origins);
    let (shutdown_tx, mut shutdown_rx) = watch::channel(ShutdownSignal::None);
    let shutdown_tx = Arc::new(shutdown_tx);
    let exe_path = std::env::current_exe().ok();
    crate::cleanup::spawn_cleanup_task();
    let service = CentyDaemonService::new(shutdown_tx.clone(), exe_path, user_cfg);
    let reflection_service = tonic_reflection::server::Builder::configure()
        .register_encoded_file_descriptor_set(app::FILE_DESCRIPTOR_SET)
        .build_v1()?;
    info!("Starting Centy daemon on {} (gRPC + gRPC-Web)", addr);
    let server_result = Server::builder()
        .accept_http1(true)
        .layer(TraceLayer::new_for_http().make_span_with(DefaultMakeSpan::new().level(Level::INFO)))
        .layer(cors)
        .layer(tonic_web::GrpcWebLayer::new())
        .add_service(reflection_service)
        .add_service(CentyDaemonServer::new(service))
        .serve_with_shutdown(addr, async move {
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
        .await;
    if let Err(e) = server_result {
        report_server_error(addr, &log_file, &e);
        return Err(e.into());
    }
    info!("Centy daemon stopped");
    Ok(())
}
