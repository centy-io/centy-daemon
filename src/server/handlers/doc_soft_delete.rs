use std::path::Path;

use crate::hooks::{HookItemType, HookOperation};
use crate::item::entities::doc::soft_delete_doc;
use crate::registry::track_project_async;
use crate::server::convert_entity::doc_to_proto;
use crate::server::convert_infra::manifest_to_proto;
use crate::server::hooks_helper::{maybe_run_post_hooks, maybe_run_pre_hooks};
use crate::server::proto::{SoftDeleteDocRequest, SoftDeleteDocResponse};
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
            error: e,
            ..Default::default()
        }));
    }

    match soft_delete_doc(project_path, &req.slug).await {
        Ok(result) => {
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

            Ok(Response::new(SoftDeleteDocResponse {
                success: true,
                error: String::new(),
                doc: Some(doc_to_proto(&result.doc)),
                manifest: Some(manifest_to_proto(&result.manifest)),
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
                error: e.to_string(),
                doc: None,
                manifest: None,
            }))
        }
    }
}
