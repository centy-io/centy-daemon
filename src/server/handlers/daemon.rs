use std::sync::Arc;

use crate::server::proto::{DaemonInfo, GetDaemonInfoRequest, ShutdownRequest, ShutdownResponse};
use crate::server::ShutdownSignal;
use crate::utils::{format_display_path, CENTY_VERSION};
use tokio::sync::watch;
use tonic::{Response, Status};
use tracing::info;

#[allow(
    renamed_and_removed_lints,
    unknown_lints,
    unused_async,
    clippy::unused_async
)]
pub async fn get_daemon_info(_req: GetDaemonInfoRequest) -> Result<Response<DaemonInfo>, Status> {
    let binary_path = std::env::current_exe()
        .map(|p| format_display_path(&p.to_string_lossy()))
        .unwrap_or_default();

    Ok(Response::new(DaemonInfo {
        version: CENTY_VERSION.to_string(),
        binary_path,
        vscode_available: which::which("code").is_ok(),
    }))
}

#[allow(
    renamed_and_removed_lints,
    unknown_lints,
    unused_async,
    clippy::unused_async
)]
pub async fn shutdown(
    req: ShutdownRequest,
    shutdown_tx: &Arc<watch::Sender<ShutdownSignal>>,
) -> Result<Response<ShutdownResponse>, Status> {
    let delay = req.delay_seconds;

    info!("Shutdown requested with delay: {} seconds", delay);

    // Clone the sender for use in the spawned task
    let shutdown_tx = shutdown_tx.clone();

    // Spawn a task to handle the delayed shutdown
    // Always wait a small amount of time to ensure the response is sent before shutting down
    tokio::spawn(async move {
        if delay > 0 {
            tokio::time::sleep(tokio::time::Duration::from_secs(u64::from(delay))).await;
        } else {
            // Small delay to ensure the RPC response is fully sent before shutdown
            tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
        }
        let _ = shutdown_tx.send(ShutdownSignal::Shutdown);
    });

    let message = if delay > 0 {
        format!("Daemon will shutdown in {delay} seconds")
    } else {
        "Daemon shutting down".to_string()
    };

    Ok(Response::new(ShutdownResponse {
        success: true,
        message,
    }))
}
