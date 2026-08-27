mod catalog;
mod grpc;
mod server;

use crate::server::ShutdownSignal;
use color_eyre::eyre::{bail, Result, WrapErr};
use std::net::SocketAddr;
use tokio::sync::watch;

pub const DEFAULT_ADDR: &str = "127.0.0.1:50052";

pub async fn spawn(
    addr: &str,
    grpc_addr: SocketAddr,
    mut shutdown: watch::Receiver<ShutdownSignal>,
) -> Result<tokio::task::JoinHandle<Result<()>>> {
    let addr: SocketAddr = addr.parse().wrap_err("invalid MCP bind address")?;
    if !addr.ip().is_loopback() {
        bail!("MCP endpoint must bind to a loopback address, got {addr}");
    }
    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .wrap_err_with(|| format!("failed to bind MCP endpoint on {addr}"))?;
    let service = server::McpServer::new(grpc::endpoint_for(grpc_addr))?;
    let router = axum::Router::new().nest_service("/mcp", service.into_service());
    Ok(tokio::spawn(async move {
        axum::serve(listener, router)
            .with_graceful_shutdown(async move {
                loop {
                    if shutdown.changed().await.is_err()
                        || *shutdown.borrow() != ShutdownSignal::None
                    {
                        break;
                    }
                }
            })
            .await
            .wrap_err("MCP HTTP server failed")
    }))
}
