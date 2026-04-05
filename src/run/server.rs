use crate::app::{self, report_server_error};
use crate::server::proto::centy_daemon_server::CentyDaemonServer;
use crate::server::{CentyDaemonService, ShutdownSignal};
use color_eyre::eyre::Result;
use std::sync::Arc;
use tokio::sync::watch;
use tonic::transport::Server;
use tower_http::trace::{DefaultMakeSpan, TraceLayer};
use tracing::{info, warn, Level};

pub async fn start(args: app::Args) -> Result<()> {
    let user_cfg = crate::user_config::load_user_config().unwrap_or_else(|e| {
        warn!("Failed to load user config, using defaults: {e}");
        crate::user_config::UserConfig::default()
    });
    crate::registry::init_ignore_paths(&user_cfg.registry.ignore_paths);
    let addr = args.addr.parse()?;
    let cors = super::core::build_cors(&args.cors_origins);
    let (tx_raw, mut shutdown_rx) = watch::channel(ShutdownSignal::None);
    let shutdown_tx = Arc::new(tx_raw);
    let exe_path = std::env::current_exe().ok();
    crate::cleanup::spawn_cleanup_task();
    let service = CentyDaemonService::new(Arc::clone(&shutdown_tx), exe_path, user_cfg);
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
                drop(shutdown_rx.changed().await);
                let signal = *shutdown_rx.borrow();
                match signal {
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
        report_server_error(addr, &e);
        return Err(e.into());
    }
    info!("Centy daemon stopped");
    Ok(())
}
