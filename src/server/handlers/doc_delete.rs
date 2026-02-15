use std::path::Path;

use crate::hooks::{HookItemType, HookOperation};
use crate::item::generic::storage::generic_delete;
use crate::manifest::read_manifest;
use crate::registry::track_project_async;
use crate::server::convert_infra::manifest_to_proto;
use crate::server::handlers::item_type_resolve::resolve_item_type_config;
use crate::server::hooks_helper::{maybe_run_post_hooks, maybe_run_pre_hooks};
use crate::server::proto::{DeleteDocRequest, DeleteDocResponse};
use crate::server::structured_error::to_error_json;
use tonic::{Response, Status};

pub async fn delete_doc_handler(
    req: DeleteDocRequest,
) -> Result<Response<DeleteDocResponse>, Status> {
    track_project_async(req.project_path.clone());
    let project_path = Path::new(&req.project_path);

    // Pre-hook
    let hook_project_path = req.project_path.clone();
    let hook_item_id = req.slug.clone();
    let hook_request_data = serde_json::json!({
        "slug": &req.slug,
        "force": req.force,
    });
    if let Err(e) = maybe_run_pre_hooks(
        project_path,
        HookItemType::Doc,
        HookOperation::Delete,
        &hook_project_path,
        Some(&hook_item_id),
        Some(hook_request_data.clone()),
    )
    .await
    {
        return Ok(Response::new(DeleteDocResponse {
            success: false,
            error: to_error_json(&req.project_path, &e),
            ..Default::default()
        }));
    }

    let config = match resolve_item_type_config(project_path, "docs").await {
        Ok(c) => c,
        Err(e) => {
            return Ok(Response::new(DeleteDocResponse {
                success: false,
                error: to_error_json(&req.project_path, &e),
                ..Default::default()
            }))
        }
    };

    match generic_delete(project_path, &config, &req.slug, req.force).await {
        Ok(()) => {
            maybe_run_post_hooks(
                project_path,
                HookItemType::Doc,
                HookOperation::Delete,
                &hook_project_path,
                Some(&hook_item_id),
                Some(hook_request_data),
                true,
            )
            .await;

            let manifest = read_manifest(project_path).await.ok().flatten();
            Ok(Response::new(DeleteDocResponse {
                success: true,
                error: String::new(),
                manifest: manifest.map(|m| manifest_to_proto(&m)),
            }))
        }
        Err(e) => {
            maybe_run_post_hooks(
                project_path,
                HookItemType::Doc,
                HookOperation::Delete,
                &hook_project_path,
                Some(&hook_item_id),
                Some(hook_request_data),
                false,
            )
            .await;

            Ok(Response::new(DeleteDocResponse {
                success: false,
                error: to_error_json(&req.project_path, &e),
                manifest: None,
            }))
        }
    }
}
