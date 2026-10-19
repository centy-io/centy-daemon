use crate::app::{self, report_server_error};
use crate::server::proto::centy_daemon_server::CentyDaemonServer;
use crate::server::{CentyDaemonService, ShutdownSignal};
use color_eyre::eyre::{eyre, Result, WrapErr as _};
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::sync::watch;
use tonic::transport::server::TcpIncoming;
use tonic::transport::Server;
use tonic_health::server::health_reporter;
use tower_http::trace::{DefaultMakeSpan, TraceLayer};
use tracing::{info, warn, Level};

mod core;

pub async fn run(args: app::Args) -> Result<()> {
    core::setup_logging(args.log_dir, args.log_json, &args.log_rotation)?;
    let user_cfg = crate::user_config::load_user_config().unwrap_or_else(|e| {
        warn!("Failed to load user config, using defaults: {e}");
        crate::user_config::UserConfig::default()
    });
    crate::registry::init_ignore_paths(&user_cfg.registry.ignore_paths);
    let addr: SocketAddr = args.addr.parse()?;
    let listener = match tokio::net::TcpListener::bind(addr).await {
        Ok(listener) => listener,
        Err(e) => {
            report_server_error(addr, &e);
            return Err(e).wrap_err_with(|| format!("failed to bind gRPC endpoint on {addr}"));
        }
    };
    let incoming = TcpIncoming::from_listener(listener, true, None)
        .map_err(|e| eyre!("failed to set up gRPC listener on {addr}: {e}"))?;
    let cors = core::build_cors(&args.cors_origins);
    let web_process = core::launch_web(args.serve_web, &args.web_addr)?;
    let (tx_raw, mut shutdown_rx) = watch::channel(ShutdownSignal::None);
    let shutdown_tx = Arc::new(tx_raw);
    let signal_tx = Arc::clone(&shutdown_tx);
    tokio::spawn(async move {
        termination_signal().await;
        info!("Received termination signal, stopping server...");
        let _res = signal_tx.send(ShutdownSignal::Shutdown);
    });
    let mcp_task = crate::mcp::spawn(&args.mcp_addr, addr, shutdown_tx.subscribe()).await?;
    let exe_path = std::env::current_exe().ok();
    crate::cleanup::spawn_cleanup_task();
    let service = CentyDaemonService::new(Arc::clone(&shutdown_tx), exe_path, user_cfg);
    let reflection_service = tonic_reflection::server::Builder::configure()
        .register_encoded_file_descriptor_set(app::FILE_DESCRIPTOR_SET)
        .register_encoded_file_descriptor_set(tonic_health::pb::FILE_DESCRIPTOR_SET)
        .build_v1()?;
    let (mut health_reporter, health_service) = health_reporter();
    health_reporter
        .set_serving::<CentyDaemonServer<CentyDaemonService>>()
        .await;
    info!("Starting Centy daemon on {} (gRPC + gRPC-Web)", addr);
    let server_result = Server::builder()
        .accept_http1(true)
        .layer(TraceLayer::new_for_http().make_span_with(DefaultMakeSpan::new().level(Level::INFO)))
        .layer(cors)
        .layer(tonic_web::GrpcWebLayer::new())
        .add_service(reflection_service)
        .add_service(health_service)
        .add_service(CentyDaemonServer::new(service))
        .serve_with_incoming_shutdown(incoming, async move {
            loop {
                drop(shutdown_rx.changed().await);
                let signal = *shutdown_rx.borrow();
                match signal {
                    ShutdownSignal::Shutdown => {
                        info!("Received shutdown signal, stopping server...");
                        health_reporter
                            .set_not_serving::<CentyDaemonServer<CentyDaemonService>>()
                            .await;
                        break;
                    }
                    ShutdownSignal::Restart => {
                        info!("Received restart signal, stopping server...");
                        health_reporter
                            .set_not_serving::<CentyDaemonServer<CentyDaemonService>>()
                            .await;
                        break;
                    }
                    ShutdownSignal::None => {}
                }
            }
        })
        .await;
    mcp_task.await.map_err(color_eyre::eyre::Report::from)??;
    core::stop_web(web_process);
    if let Err(e) = server_result {
        report_server_error(addr, &e);
        return Err(e.into());
    }
    info!("Centy daemon stopped");
    Ok(())
}

/// Waits for SIGINT (ctrl-c). A failed listener logs a warning and waits forever.
async fn wait_ctrl_c() {
    match tokio::signal::ctrl_c().await {
        Ok(()) => {}
        Err(e) => {
            warn!("Failed to listen for SIGINT, ignoring it: {e}");
            std::future::pending::<()>().await;
        }
    }
}

/// Waits for SIGTERM. A failed listener logs a warning and waits forever.
#[cfg(unix)]
async fn wait_sigterm() {
    match tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate()) {
        Ok(mut term) => {
            if term.recv().await.is_none() {
                warn!("SIGTERM listener closed, ignoring it");
                std::future::pending::<()>().await;
            }
        }
        Err(e) => {
            warn!("Failed to listen for SIGTERM, ignoring it: {e}");
            std::future::pending::<()>().await;
        }
    }
}

/// Resolves on the first SIGINT (ctrl-c) or, on unix, SIGTERM.
async fn termination_signal() {
    #[cfg(unix)]
    drop(futures::future::select(Box::pin(wait_ctrl_c()), Box::pin(wait_sigterm())).await);
    #[cfg(not(unix))]
    wait_ctrl_c().await;
}
