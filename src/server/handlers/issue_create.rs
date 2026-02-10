use std::path::Path;

use crate::hooks::{HookItemType, HookOperation};
use crate::item::entities::issue::CreateIssueOptions;
use crate::registry::track_project_async;
use crate::server::convert_infra::manifest_to_proto;
use crate::server::helpers::{nonempty, nonzero_u32, sync_results_to_proto};
use crate::server::hooks_helper::{maybe_run_post_hooks, maybe_run_pre_hooks};
use crate::server::proto::{CreateIssueRequest, CreateIssueResponse};
use crate::server::structured_error::to_error_json;
use tonic::{Response, Status};

pub async fn create_issue(
    req: CreateIssueRequest,
) -> Result<Response<CreateIssueResponse>, Status> {
    track_project_async(req.project_path.clone());
    let project_path = Path::new(&req.project_path);

    // Pre-hook
    let hook_project_path = req.project_path.clone();
    let hook_request_data = serde_json::json!({
        "title": &req.title,
        "description": &req.description,
        "priority": req.priority,
        "status": &req.status,
    });
    if let Err(e) = maybe_run_pre_hooks(
        project_path,
        HookItemType::Issue,
        HookOperation::Create,
        &hook_project_path,
        None,
        Some(hook_request_data.clone()),
    )
    .await
    {
        return Ok(Response::new(CreateIssueResponse {
            success: false,
            error: to_error_json(&req.project_path, &e),
            ..Default::default()
        }));
    }

    // Convert int32 priority: 0 means use default, otherwise use the value
    let options = CreateIssueOptions {
        title: req.title,
        description: req.description,
        priority: nonzero_u32(req.priority),
        status: nonempty(req.status),
        custom_fields: req.custom_fields,
        template: nonempty(req.template),
        draft: Some(req.draft),
        is_org_issue: req.is_org_issue,
    };

    match crate::item::entities::issue::create_issue(project_path, options).await {
        #[allow(deprecated)]
        Ok(result) => {
            maybe_run_post_hooks(
                project_path,
                HookItemType::Issue,
                HookOperation::Create,
                &hook_project_path,
                Some(&result.id),
                Some(hook_request_data),
                true,
            )
            .await;
            let sync_results = sync_results_to_proto(result.sync_results);
            Ok(Response::new(CreateIssueResponse {
                success: true,
                error: String::new(),
                id: result.id.clone(),
                display_number: result.display_number,
                issue_number: result.issue_number, // Legacy
                created_files: result.created_files,
                manifest: Some(manifest_to_proto(&result.manifest)),
                org_display_number: result.org_display_number.unwrap_or(0),
                sync_results,
            }))
        }
        Err(e) => {
            maybe_run_post_hooks(
                project_path,
                HookItemType::Issue,
                HookOperation::Create,
                &hook_project_path,
                None,
                Some(hook_request_data),
                false,
            )
            .await;
            Ok(Response::new(CreateIssueResponse {
                success: false,
                error: to_error_json(&req.project_path, &e),
                id: String::new(),
                display_number: 0,
                issue_number: String::new(),
                created_files: vec![],
                manifest: None,
                org_display_number: 0,
                sync_results: vec![],
            }))
        }
    }
}
