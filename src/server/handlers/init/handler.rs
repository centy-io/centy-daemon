use super::post_init::post_reconcile;
use crate::reconciliation::{execute_reconciliation, ReconciliationDecisions};
use crate::registry::{track_project, track_project_async};
use crate::server::proto::{InitRequest, InitResponse, IsInitializedRequest, IsInitializedResponse};
use crate::server::structured_error::to_error_json;
use crate::utils::get_centy_path;
use std::path::Path;
use tonic::{Response, Status};
pub async fn init(req: InitRequest) -> Result<Response<InitResponse>, Status> {
    let _ = track_project(&req.project_path).await;
    let mut req = req;
    let project_path_str = req.project_path.clone();
    let project_path = Path::new(&project_path_str);
    let decisions = req
        .decisions
        .take()
        .map(|d| ReconciliationDecisions {
            restore: d.restore.into_iter().collect(),
            reset: d.reset.into_iter().collect(),
        })
        .unwrap_or_default();
    match execute_reconciliation(project_path, decisions, req.force).await {
        Ok(result) => post_reconcile(&req, project_path, result).await,
        Err(e) => Ok(Response::new(InitResponse {
            success: false,
            error: to_error_json(&req.project_path, &e),
            created: vec![],
            restored: vec![],
            reset: vec![],
            skipped: vec![],
            manifest: None,
            org_inference: None,
        })),
    }
}
pub async fn is_initialized(
    req: IsInitializedRequest,
) -> Result<Response<IsInitializedResponse>, Status> {
    track_project_async(req.project_path.clone());
    let project_path = Path::new(&req.project_path);
    let centy_path = get_centy_path(project_path);
    let manifest_path = centy_path.join(".centy-manifest.json");
    let initialized = manifest_path.exists();
    let centy_path_str = if initialized {
        centy_path.to_string_lossy().to_string()
    } else {
        String::new()
    };
    Ok(Response::new(IsInitializedResponse {
        initialized,
        centy_path: centy_path_str,
    }))
}
#[cfg(test)]
#[path = "../init_tests.rs"]
mod init_tests;
