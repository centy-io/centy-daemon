use std::path::Path;

use crate::hooks::{HookItemType, HookOperation};
use crate::item::entities::doc::{update_doc, UpdateDocOptions};
use crate::registry::track_project_async;
use crate::server::convert_entity::doc_to_proto;
use crate::server::convert_infra::manifest_to_proto;
use crate::server::helpers::{doc_sync_results_to_proto, nonempty};
use crate::server::hooks_helper::{maybe_run_post_hooks, maybe_run_pre_hooks};
use crate::server::proto::{UpdateDocRequest, UpdateDocResponse};
use crate::server::structured_error::to_error_json;
use tonic::{Response, Status};

pub async fn update_doc_handler(
    req: UpdateDocRequest,
) -> Result<Response<UpdateDocResponse>, Status> {
    track_project_async(req.project_path.clone());
    let project_path = Path::new(&req.project_path);

    // Pre-hook
    let hook_project_path = req.project_path.clone();
    let hook_item_id = req.slug.clone();
    let hook_request_data = serde_json::json!({
        "slug": &req.slug,
        "title": &req.title,
        "content": &req.content,
    });
    if let Err(e) = maybe_run_pre_hooks(
        project_path,
        HookItemType::Doc,
        HookOperation::Update,
        &hook_project_path,
        Some(&hook_item_id),
        Some(hook_request_data.clone()),
    )
    .await
    {
        return Ok(Response::new(UpdateDocResponse {
            success: false,
            error: to_error_json(&req.project_path, &e),
            ..Default::default()
        }));
    }

    let options = UpdateDocOptions {
        title: nonempty(req.title),
        content: nonempty(req.content),
        new_slug: nonempty(req.new_slug),
    };

    match update_doc(project_path, &req.slug, options).await {
        Ok(result) => {
            maybe_run_post_hooks(
                project_path,
                HookItemType::Doc,
                HookOperation::Update,
                &hook_project_path,
                Some(&hook_item_id),
                Some(hook_request_data),
                true,
            )
            .await;

            let sync_results = doc_sync_results_to_proto(result.sync_results);

            Ok(Response::new(UpdateDocResponse {
                success: true,
                error: String::new(),
                doc: Some(doc_to_proto(&result.doc)),
                manifest: Some(manifest_to_proto(&result.manifest)),
                sync_results,
            }))
        }
        Err(e) => {
            maybe_run_post_hooks(
                project_path,
                HookItemType::Doc,
                HookOperation::Update,
                &hook_project_path,
                Some(&hook_item_id),
                Some(hook_request_data),
                false,
            )
            .await;

            Ok(Response::new(UpdateDocResponse {
                success: false,
                error: to_error_json(&req.project_path, &e),
                doc: None,
                manifest: None,
                sync_results: Vec::new(),
            }))
        }
    }
}
