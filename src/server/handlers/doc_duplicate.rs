use std::path::{Path, PathBuf};

use crate::hooks::{HookItemType, HookOperation};
use crate::item::entities::doc::{duplicate_doc, DuplicateDocOptions};
use crate::registry::track_project_async;
use crate::server::convert_entity::doc_to_proto;
use crate::server::convert_infra::manifest_to_proto;
use crate::server::helpers::nonempty;
use crate::server::hooks_helper::{maybe_run_post_hooks, maybe_run_pre_hooks};
use crate::server::proto::{DuplicateDocRequest, DuplicateDocResponse};
use crate::server::structured_error::to_error_json;
use tonic::{Response, Status};

pub async fn duplicate_doc_handler(
    req: DuplicateDocRequest,
) -> Result<Response<DuplicateDocResponse>, Status> {
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
        HookItemType::Doc,
        HookOperation::Duplicate,
        &hook_project_path,
        Some(&hook_item_id),
        Some(hook_request_data.clone()),
    )
    .await
    {
        return Ok(Response::new(DuplicateDocResponse {
            success: false,
            error: to_error_json(&req.source_project_path, &e),
            ..Default::default()
        }));
    }

    let options = DuplicateDocOptions {
        source_project_path: PathBuf::from(&req.source_project_path),
        target_project_path: PathBuf::from(&req.target_project_path),
        slug: req.slug.clone(),
        new_slug: nonempty(req.new_slug),
        new_title: nonempty(req.new_title),
    };

    match duplicate_doc(options).await {
        Ok(result) => {
            maybe_run_post_hooks(
                Path::new(&hook_project_path),
                HookItemType::Doc,
                HookOperation::Duplicate,
                &hook_project_path,
                Some(&hook_item_id),
                Some(hook_request_data),
                true,
            )
            .await;

            Ok(Response::new(DuplicateDocResponse {
                success: true,
                error: String::new(),
                doc: Some(doc_to_proto(&result.doc)),
                original_slug: result.original_slug,
                manifest: Some(manifest_to_proto(&result.manifest)),
            }))
        }
        Err(e) => {
            maybe_run_post_hooks(
                Path::new(&hook_project_path),
                HookItemType::Doc,
                HookOperation::Duplicate,
                &hook_project_path,
                Some(&hook_item_id),
                Some(hook_request_data),
                false,
            )
            .await;

            Ok(Response::new(DuplicateDocResponse {
                success: false,
                error: to_error_json(&req.source_project_path, &e),
                doc: None,
                original_slug: req.slug,
                manifest: None,
            }))
        }
    }
}
