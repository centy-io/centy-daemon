use std::path::PathBuf;
use std::process::Command;
use std::sync::Arc;

use crate::server::proto::{RestartRequest, RestartResponse};
use crate::server::structured_error::StructuredError;
use crate::server::ShutdownSignal;
use tokio::sync::watch;
use tonic::{Response, Status};
use tracing::info;

#[allow(clippy::cognitive_complexity)]
async fn perform_restart(
    delay: u32,
    exe_path: PathBuf,
    shutdown_tx: Arc<watch::Sender<ShutdownSignal>>,
) {
    if delay > 0 {
        tokio::time::sleep(tokio::time::Duration::from_secs(u64::from(delay))).await;
    } else {
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
    }
    info!("Spawning new daemon process: {:?}", exe_path);
    match Command::new(&exe_path).spawn() {
        Ok(_) => {
            info!("New daemon process spawned successfully");
            let _ = shutdown_tx.send(ShutdownSignal::Restart);
        }
        Err(e) => {
            info!("Failed to spawn new daemon process: {}", e);
        }
    }
}

pub fn restart(
    req: RestartRequest,
    shutdown_tx: &Arc<watch::Sender<ShutdownSignal>>,
    exe_path: Option<&PathBuf>,
) -> Result<Response<RestartResponse>, Status> {
    let delay = req.delay_seconds;
    info!("Restart requested with delay: {} seconds", delay);
    let Some(exe_path) = exe_path.cloned() else {
        return Ok(Response::new(RestartResponse {
            success: false,
            message: StructuredError::new(
                "",
                "RESTART_ERROR",
                "Cannot restart: unable to determine executable path".to_string(),
            )
            .to_json(),
        }));
    };
    tokio::spawn(perform_restart(delay, exe_path, Arc::clone(shutdown_tx)));
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
