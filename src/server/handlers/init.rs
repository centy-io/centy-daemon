use std::path::Path;

use crate::reconciliation::{execute_reconciliation, ReconciliationDecisions};
use crate::registry::{
    get_project_info, infer_organization_from_remote, set_project_organization, track_project_async,
};
use crate::server::convert_infra::{manifest_to_proto, org_inference_to_proto};
use crate::server::proto::{
    InitRequest, InitResponse, IsInitializedRequest, IsInitializedResponse,
};
use crate::server::structured_error::to_error_json;
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
mod tests {
    use crate::config::{CentyConfig, ProjectMetadata, WorkspaceConfig};

    /// Compile-time exhaustiveness check: if `CentyConfig`, `WorkspaceConfig`, or
    /// `ProjectMetadata` gains a new field this test will fail to compile, forcing
    /// the developer to explicitly handle or acknowledge the new field in `apply_init_config`.
    ///
    /// Fields not configurable at init time (complex types, deprecated, or internal) should be
    /// listed here with a comment explaining why they are intentionally excluded from the init API.
    #[test]
    fn test_all_config_fields_acknowledged_in_init() {
        let CentyConfig {
            version: _,           // internal: set by daemon, not exposed as init flag
            priority_levels: _,   // ✓ exposed via InitRequest.init_config (Config.priority_levels)
            custom_fields: _,     // ✓ exposed via InitRequest.init_config (Config.custom_fields)
            defaults: _,          // ✓ exposed via InitRequest.init_config (Config.defaults)
            allowed_states: _,    // deprecated: migrated to per-item-type config.yaml
            state_colors: _,      // ✓ exposed via InitRequest.init_config (Config.state_colors)
            priority_colors: _,   // ✓ exposed via InitRequest.init_config (Config.priority_colors)
            custom_link_types: _, // ✓ exposed via InitRequest.init_config (Config.custom_link_types)
            default_editor: _,    // ✓ exposed via InitRequest.init_config (Config.default_editor)
            hooks: _,             // ✓ exposed via InitRequest.init_config (Config.hooks)
            workspace,            // ✓ exposed via InitRequest.init_config (Config.workspace)
        } = CentyConfig::default();

        let WorkspaceConfig {
            update_status_on_open: _, // ✓ exposed via InitRequest.init_config (Config.workspace.update_status_on_open)
        } = workspace;

        // ProjectMetadata fields are exposed directly on InitRequest (not inside init_config).
        let ProjectMetadata {
            title: _, // ✓ exposed via InitRequest.title
        } = ProjectMetadata::default();
    }
}
