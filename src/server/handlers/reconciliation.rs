use std::path::Path;

use crate::reconciliation::{
    build_reconciliation_plan, execute_reconciliation, ReconciliationDecisions,
};
use crate::registry::{
    get_project_info, infer_organization_from_remote, set_project_organization, track_project_async,
};
use crate::server::convert_infra::{file_info_to_proto, manifest_to_proto, org_inference_to_proto};
use crate::server::proto::{
    ExecuteReconciliationRequest, GetReconciliationPlanRequest, InitResponse, ReconciliationPlan,
};
use tonic::{Response, Status};

pub async fn get_reconciliation_plan(
    req: GetReconciliationPlanRequest,
) -> Result<Response<ReconciliationPlan>, Status> {
    track_project_async(req.project_path.clone());
    let project_path = Path::new(&req.project_path);

    match build_reconciliation_plan(project_path).await {
        Ok(plan) => {
            let needs_decisions = plan.needs_decisions();
            Ok(Response::new(ReconciliationPlan {
                to_create: plan.to_create.into_iter().map(file_info_to_proto).collect(),
                to_restore: plan
                    .to_restore
                    .into_iter()
                    .map(file_info_to_proto)
                    .collect(),
                to_reset: plan.to_reset.into_iter().map(file_info_to_proto).collect(),
                up_to_date: plan
                    .up_to_date
                    .into_iter()
                    .map(file_info_to_proto)
                    .collect(),
                user_files: plan
                    .user_files
                    .into_iter()
                    .map(file_info_to_proto)
                    .collect(),
                needs_decisions,
            }))
        }
        Err(e) => Err(Status::internal(e.to_string())),
    }
}
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
