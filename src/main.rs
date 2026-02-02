// Deny panic/unwrap/expect in production code, allow in tests
#![cfg_attr(
    not(test),
    deny(
        clippy::panic,
        clippy::unwrap_used,
        clippy::expect_used,
        clippy::panic_in_result_fn,
        clippy::unwrap_in_result
    )
)]
#![cfg_attr(
    test,
    allow(
        clippy::panic,
        clippy::unwrap_used,
        clippy::expect_used,
        clippy::panic_in_result_fn,
        clippy::unwrap_in_result
    )
)]

mod common;
mod config;
mod cors;
mod features;
mod grpc_logging;
mod item;
mod link;
mod logging;
mod manifest;
mod metrics;
mod reconciliation;
mod registry;
mod search;
mod server;
mod template;
mod user;
mod utils;
mod workspace;

use clap::Parser;
use color_eyre::eyre::Result;
use cors::{build_cors_layer, DEFAULT_CORS_ORIGINS};
use grpc_logging::GrpcLoggingLayer;
use logging::{init_logging, parse_rotation, LogConfig};
use server::proto::centy_daemon_server::CentyDaemonServer;
use server::{CentyDaemonService, ShutdownSignal};
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::watch;
use tonic::transport::Server;
use tracing::info;

const DEFAULT_ADDR: &str = "127.0.0.1:50051";

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
async fn main() -> Result<()> {
    // Install color-eyre error hooks for colored error output
    color_eyre::install()?;

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

    // Configure CORS for gRPC-Web
    let cors = build_cors_layer(cors_origins);

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

    info!("Starting Centy daemon on {} (gRPC + gRPC-Web)", addr);

    let server_result = Server::builder()
        .accept_http1(true) // Required for gRPC-Web
        .layer(cors)
        .layer(GrpcLoggingLayer)
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
        .await;

    if let Err(e) = server_result {
        let err_string = format!("{e:?}");
        if err_string.contains("AddrInUse") {
            eprintln!();
            eprintln!("Error: Failed to start server - address {addr} is already in use");
            eprintln!();
            eprintln!("Another instance of centy-daemon may already be running.");
            eprintln!();
            eprintln!("Options:");
            eprintln!("  1. Kill the existing process:   pkill centy-daemon");
            eprintln!("  2. Use a different port:        centy-daemon --addr 127.0.0.1:50052");
            eprintln!("  3. Check what's using the port: lsof -i :{}", addr.port());
            eprintln!();
            std::process::exit(1);
        }
        return Err(e.into());
    }

    info!("Centy daemon stopped");
    Ok(())
}
