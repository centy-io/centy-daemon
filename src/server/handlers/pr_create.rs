use std::path::Path;

use crate::hooks::{HookItemType, HookOperation};
use crate::item::entities::pr::CreatePrOptions;
use crate::registry::track_project_async;
use crate::server::convert_infra::manifest_to_proto;
use crate::server::helpers::{nonempty, nonzero_u32};
use crate::server::hooks_helper::{maybe_run_post_hooks, maybe_run_pre_hooks};
use crate::server::proto::{CreatePrRequest, CreatePrResponse};
use crate::server::structured_error::to_error_json;
use tonic::{Response, Status};

pub async fn create_pr(req: CreatePrRequest) -> Result<Response<CreatePrResponse>, Status> {
    track_project_async(req.project_path.clone());
    let project_path = Path::new(&req.project_path);

    // Pre-hook
    let hook_project_path = req.project_path.clone();
    let hook_request_data = serde_json::json!({
        "title": &req.title,
        "description": &req.description,
        "source_branch": &req.source_branch,
        "target_branch": &req.target_branch,
    });
    if let Err(e) = maybe_run_pre_hooks(
        project_path,
        HookItemType::Pr,
        HookOperation::Create,
        &hook_project_path,
        None,
        Some(hook_request_data.clone()),
    )
    .await
    {
        return Ok(Response::new(CreatePrResponse {
            success: false,
            error: to_error_json(&req.project_path, &e),
            ..Default::default()
        }));
    }

    let options = CreatePrOptions {
        title: req.title,
        description: req.description,
        source_branch: nonempty(req.source_branch),
        target_branch: nonempty(req.target_branch),
        reviewers: req.reviewers,
        priority: nonzero_u32(req.priority),
        status: nonempty(req.status),
        custom_fields: req.custom_fields,
        template: nonempty(req.template),
    };

    match crate::item::entities::pr::create_pr(project_path, options).await {
        Ok(result) => {
            maybe_run_post_hooks(
                project_path,
                HookItemType::Pr,
                HookOperation::Create,
                &hook_project_path,
                Some(&result.id),
                Some(hook_request_data),
                true,
            )
            .await;

            Ok(Response::new(CreatePrResponse {
                success: true,
                error: String::new(),
                id: result.id,
                display_number: result.display_number,
                created_files: result.created_files,
                manifest: Some(manifest_to_proto(&result.manifest)),
                detected_source_branch: result.detected_source_branch,
            }))
        }
        Err(e) => {
            maybe_run_post_hooks(
                project_path,
                HookItemType::Pr,
                HookOperation::Create,
                &hook_project_path,
                None,
                Some(hook_request_data),
                false,
            )
            .await;

            Ok(Response::new(CreatePrResponse {
                success: false,
                error: to_error_json(&req.project_path, &e),
                id: String::new(),
                display_number: 0,
                created_files: vec![],
                manifest: None,
                detected_source_branch: String::new(),
            }))
        }
    }
}
