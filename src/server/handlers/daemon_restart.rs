use std::path::PathBuf;
use std::process::Command;
use std::sync::Arc;

use crate::server::proto::{RestartRequest, RestartResponse};
use crate::server::structured_error::StructuredError;
use crate::server::ShutdownSignal;
use tokio::sync::watch;
use tonic::{Response, Status};
use tracing::info;

pub async fn restart(
    req: RestartRequest,
    shutdown_tx: &Arc<watch::Sender<ShutdownSignal>>,
    exe_path: Option<&PathBuf>,
) -> Result<Response<RestartResponse>, Status> {
    let delay = req.delay_seconds;

    info!("Restart requested with delay: {} seconds", delay);

    // Check if we have the executable path
    let exe_path = match exe_path {
        Some(path) => path.clone(),
        None => {
            return Ok(Response::new(RestartResponse {
                success: false,
                message: StructuredError::new(
                    "",
                    "RESTART_ERROR",
                    "Cannot restart: unable to determine executable path".to_string(),
                )
                .to_json(),
            }));
        }
    };

    // Clone what we need for the spawned task
    let shutdown_tx = shutdown_tx.clone();

    // Spawn a task to handle the delayed restart
    // Always wait a small amount of time to ensure the response is sent before restarting
    tokio::spawn(async move {
        if delay > 0 {
            tokio::time::sleep(tokio::time::Duration::from_secs(u64::from(delay))).await;
        } else {
            // Small delay to ensure the RPC response is fully sent before restart
            tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
        }

        // Spawn a new daemon process before shutting down
        info!("Spawning new daemon process: {:?}", exe_path);
        match Command::new(&exe_path).spawn() {
            Ok(_) => {
                info!("New daemon process spawned successfully");
                // Signal the current server to shutdown
                let _ = shutdown_tx.send(ShutdownSignal::Restart);
            }
            Err(e) => {
                info!("Failed to spawn new daemon process: {}", e);
            }
        }
    });

    let message = if delay > 0 {
        format!("Daemon will restart in {delay} seconds")
    } else {
        "Daemon restarting".to_string()
    };

    Ok(Response::new(RestartResponse {
        success: true,
        message,
    }))
}
