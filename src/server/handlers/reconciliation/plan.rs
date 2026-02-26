use std::path::Path;
use crate::reconciliation::build_reconciliation_plan;
use crate::registry::track_project_async;
use crate::server::convert_infra::file_info_to_proto;
use crate::server::proto::{GetReconciliationPlanRequest, ReconciliationPlan};
use crate::server::structured_error::to_error_json;
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
                success: true,
                error: String::new(),
            }))
        }
        Err(e) => Ok(Response::new(ReconciliationPlan {
            success: false,
            error: to_error_json(&req.project_path, &e),
            ..Default::default()
        })),
    }
}
