use std::path::Path;
use crate::reconciliation::{execute_reconciliation, ReconciliationDecisions};
use crate::registry::{
    get_project_info, infer_organization_from_remote, set_project_organization, track_project_async,
};
use crate::server::convert_infra::{manifest_to_proto, org_inference_to_proto};
use crate::server::proto::{ExecuteReconciliationRequest, InitResponse};
use crate::server::structured_error::to_error_json;
use tonic::{Response, Status};
#[allow(unknown_lints, max_nesting_depth)]
pub async fn execute_reconciliation_handler(
    req: ExecuteReconciliationRequest,
) -> Result<Response<InitResponse>, Status> {
    track_project_async(req.project_path.clone());
    let project_path = Path::new(&req.project_path);
    let decisions = req
        .decisions
        .map(|d| ReconciliationDecisions {
            restore: d.restore.into_iter().collect(),
            reset: d.reset.into_iter().collect(),
        })
        .unwrap_or_default();
    match execute_reconciliation(project_path, decisions, false).await {
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
