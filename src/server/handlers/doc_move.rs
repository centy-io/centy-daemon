use std::path::{Path, PathBuf};

use crate::hooks::HookOperation;
use crate::item::entities::doc::{move_doc, MoveDocOptions};
use crate::registry::track_project_async;
use crate::server::convert_entity::doc_to_proto;
use crate::server::convert_infra::manifest_to_proto;
use crate::server::hooks_helper::{maybe_run_post_hooks, maybe_run_pre_hooks};
use crate::server::proto::{MoveDocRequest, MoveDocResponse};
use crate::server::structured_error::to_error_json;
use tonic::{Response, Status};

pub async fn move_doc_handler(req: MoveDocRequest) -> Result<Response<MoveDocResponse>, Status> {
    track_project_async(req.source_project_path.clone());
    track_project_async(req.target_project_path.clone());

    // Pre-hook
    let hook_project_path = req.source_project_path.clone();
    let hook_item_id = req.slug.clone();
    let hook_request_data = serde_json::json!({
        "source_project_path": &req.source_project_path,
        "target_project_path": &req.target_project_path,
        "slug": &req.slug,
    });
    if let Err(e) = maybe_run_pre_hooks(
        Path::new(&hook_project_path),
        "doc",
        HookOperation::Move,
        &hook_project_path,
        Some(&hook_item_id),
        Some(hook_request_data.clone()),
    )
    .await
    {
        return Ok(Response::new(MoveDocResponse {
            success: false,
            error: to_error_json(&req.source_project_path, &e),
            ..Default::default()
        }));
    }

    let options = MoveDocOptions {
        source_project_path: PathBuf::from(&req.source_project_path),
        target_project_path: PathBuf::from(&req.target_project_path),
        slug: req.slug.clone(),
        new_slug: if req.new_slug.is_empty() {
            None
        } else {
            Some(req.new_slug)
        },
    };

    match move_doc(options).await {
        Ok(result) => {
            maybe_run_post_hooks(
                Path::new(&hook_project_path),
                "doc",
                HookOperation::Move,
                &hook_project_path,
                Some(&hook_item_id),
                Some(hook_request_data),
                true,
            )
            .await;

            Ok(Response::new(MoveDocResponse {
                success: true,
                error: String::new(),
                doc: Some(doc_to_proto(&result.doc)),
                old_slug: result.old_slug,
                source_manifest: Some(manifest_to_proto(&result.source_manifest)),
                target_manifest: Some(manifest_to_proto(&result.target_manifest)),
            }))
        }
        Err(e) => {
            maybe_run_post_hooks(
                Path::new(&hook_project_path),
                "doc",
                HookOperation::Move,
                &hook_project_path,
                Some(&hook_item_id),
                Some(hook_request_data),
                false,
            )
            .await;

            Ok(Response::new(MoveDocResponse {
                success: false,
                error: to_error_json(&req.source_project_path, &e),
                doc: None,
                old_slug: req.slug,
                source_manifest: None,
                target_manifest: None,
            }))
        }
    }
}
