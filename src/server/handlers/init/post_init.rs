use super::config_merge::apply_init_config;
use super::mcp_io::ensure_mcp_json;
use crate::config::set_project_title;
use crate::reconciliation::ReconciliationResult;
use crate::registry::{get_project_info, infer_organization_from_remote, set_project_organization};
use crate::server::convert_infra::{manifest_to_proto, org_inference_to_proto};
use crate::server::proto::{InitRequest, InitResponse};
use std::path::Path;
use tonic::{Response, Status};

pub(super) async fn post_reconcile(
    req: &InitRequest,
    project_path: &Path,
    result: ReconciliationResult,
) -> Result<Response<InitResponse>, Status> {
    let existing_org = get_project_info(&req.project_path)
        .await
        .ok()
        .flatten()
        .and_then(|info| info.organization_slug);
    let inference = infer_organization_from_remote(project_path, existing_org.as_deref()).await;
    if existing_org.is_none() && !inference.has_mismatch {
        if let Some(slug) = &inference.inferred_org_slug {
            drop(set_project_organization(&req.project_path, Some(slug)).await);
        }
    }
    if let Some(proto_config) = &req.init_config {
        if let Err(e) = apply_init_config(project_path, proto_config).await {
            return Ok(Response::new(InitResponse {
                success: false,
                error: format!("Failed to write init config: {e}"),
                created: vec![],
                restored: vec![],
                reset: vec![],
                skipped: vec![],
                manifest: None,
                org_inference: None,
            }));
        }
    }
    if !req.title.is_empty() {
        if let Err(e) = set_project_title(project_path, Some(req.title.clone())).await {
            return Ok(Response::new(InitResponse {
                success: false,
                error: format!("Failed to write project title: {e}"),
                created: vec![],
                restored: vec![],
                reset: vec![],
                skipped: vec![],
                manifest: None,
                org_inference: None,
            }));
        }
    }
    if let Err(e) = ensure_mcp_json(project_path).await {
        return Ok(Response::new(InitResponse {
            success: false,
            error: e,
            created: vec![],
            restored: vec![],
            reset: vec![],
            skipped: vec![],
            manifest: None,
            org_inference: None,
        }));
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
