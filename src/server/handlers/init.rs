use std::path::Path;

use crate::reconciliation::{execute_reconciliation, ReconciliationDecisions};
use crate::registry::{
    get_project_info, infer_organization_from_remote, set_project_organization, track_project_async,
};
use crate::server::convert_infra::{manifest_to_proto, org_inference_to_proto};
use crate::server::proto::{
    InitRequest, InitResponse, IsInitializedRequest, IsInitializedResponse,
};
use crate::utils::get_centy_path;
use tonic::{Response, Status};

pub async fn init(req: InitRequest) -> Result<Response<InitResponse>, Status> {
    track_project_async(req.project_path.clone());
    let project_path = Path::new(&req.project_path);

    let decisions = req
        .decisions
        .map(|d| ReconciliationDecisions {
            restore: d.restore.into_iter().collect(),
            reset: d.reset.into_iter().collect(),
        })
        .unwrap_or_default();

    match execute_reconciliation(project_path, decisions, req.force).await {
        Ok(result) => {
            let existing_org = get_project_info(&req.project_path)
                .await
                .ok()
                .flatten()
                .and_then(|info| info.organization_slug);
            let inference =
                infer_organization_from_remote(project_path, existing_org.as_deref()).await;

            if existing_org.is_none() && !inference.has_mismatch {
                if let Some(ref slug) = inference.inferred_org_slug {
                    let _ = set_project_organization(&req.project_path, Some(slug)).await;
                }
            }

            Ok(Response::new(InitResponse {
                success: true,
                error: String::new(),
                created: result.created,
                restored: result.restored,
                reset: result.reset,
                skipped: result.skipped,
                manifest: Some(manifest_to_proto(&result.manifest)),
                org_inference: Some(org_inference_to_proto(&inference)),
            }))
        }
        Err(e) => Ok(Response::new(InitResponse {
            success: false,
            error: e.to_string(),
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
