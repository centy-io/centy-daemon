use std::path::Path;

use crate::hooks::{HookItemType, HookOperation};
use crate::item::entities::doc::get_doc;
use crate::item::generic::storage::generic_soft_delete;
use crate::manifest::read_manifest;
use crate::registry::track_project_async;
use crate::server::convert_entity::doc_to_proto;
use crate::server::convert_infra::manifest_to_proto;
use crate::server::handlers::item_type_resolve::resolve_item_type_config;
use crate::server::hooks_helper::{maybe_run_post_hooks, maybe_run_pre_hooks};
use crate::server::proto::{SoftDeleteDocRequest, SoftDeleteDocResponse};
use crate::server::structured_error::to_error_json;
use tonic::{Response, Status};

pub async fn soft_delete_doc_handler(
    req: SoftDeleteDocRequest,
) -> Result<Response<SoftDeleteDocResponse>, Status> {
    track_project_async(req.project_path.clone());
    let project_path = Path::new(&req.project_path);

    // Pre-hook
    let hook_project_path = req.project_path.clone();
    let hook_item_id = req.slug.clone();
    let hook_request_data = serde_json::json!({
        "slug": &req.slug,
    });
    if let Err(e) = maybe_run_pre_hooks(
        project_path,
        HookItemType::Doc,
        HookOperation::SoftDelete,
        &hook_project_path,
        Some(&hook_item_id),
        Some(hook_request_data.clone()),
    )
    .await
    {
        return Ok(Response::new(SoftDeleteDocResponse {
            success: false,
            error: to_error_json(&req.project_path, &e),
            ..Default::default()
        }));
    }

    let config = match resolve_item_type_config(project_path, "docs").await {
        Ok(c) => c,
        Err(e) => {
            return Ok(Response::new(SoftDeleteDocResponse {
                success: false,
                error: to_error_json(&req.project_path, &e),
                ..Default::default()
            }))
        }
    };

    match generic_soft_delete(project_path, &config, &req.slug).await {
        Ok(()) => {
            maybe_run_post_hooks(
                project_path,
                HookItemType::Doc,
                HookOperation::SoftDelete,
                &hook_project_path,
                Some(&hook_item_id),
                Some(hook_request_data),
                true,
            )
            .await;

            // Read the doc back for the type-specific response
            let doc = get_doc(project_path, &req.slug).await.ok();
            let manifest = read_manifest(project_path).await.ok().flatten();

            Ok(Response::new(SoftDeleteDocResponse {
                success: true,
                error: String::new(),
                doc: doc.map(|d| doc_to_proto(&d)),
                manifest: manifest.map(|m| manifest_to_proto(&m)),
            }))
        }
        Err(e) => {
            maybe_run_post_hooks(
                project_path,
                HookItemType::Doc,
                HookOperation::SoftDelete,
                &hook_project_path,
                Some(&hook_item_id),
                Some(hook_request_data),
                false,
            )
            .await;

            Ok(Response::new(SoftDeleteDocResponse {
                success: false,
                error: to_error_json(&req.project_path, &e),
                doc: None,
                manifest: None,
            }))
        }
    }
}
