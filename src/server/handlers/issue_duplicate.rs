use std::path::{Path, PathBuf};

use crate::config::item_type_config::default_issue_config;
use crate::config::read_config;
use crate::hooks::HookOperation;
use crate::item::entities::issue::priority_label;
use crate::item::generic::storage::generic_duplicate;
use crate::item::generic::types::DuplicateGenericItemOptions;
use crate::manifest::read_manifest;
use crate::registry::track_project_async;
use crate::server::convert_infra::manifest_to_proto;
use crate::server::helpers::nonempty;
use crate::server::hooks_helper::{maybe_run_post_hooks, maybe_run_pre_hooks};
use crate::server::proto::{DuplicateIssueRequest, DuplicateIssueResponse, Issue, IssueMetadata};
use crate::server::structured_error::to_error_json;
use tonic::{Response, Status};

pub async fn duplicate_issue(
    req: DuplicateIssueRequest,
) -> Result<Response<DuplicateIssueResponse>, Status> {
    track_project_async(req.source_project_path.clone());
    track_project_async(req.target_project_path.clone());
    let hook_project_path = req.source_project_path.clone();
    let hook_item_id = req.issue_id.clone();
    let hook_request_data = serde_json::json!({
        "source_project_path": &req.source_project_path,
        "target_project_path": &req.target_project_path,
        "issue_id": &req.issue_id,
    });
    if let Err(e) = maybe_run_pre_hooks(
        Path::new(&hook_project_path),
        "issue",
        HookOperation::Duplicate,
        &hook_project_path,
        Some(&hook_item_id),
        Some(hook_request_data.clone()),
    )
    .await
    {
        return Ok(Response::new(DuplicateIssueResponse {
            success: false,
            error: to_error_json(&req.source_project_path, &e),
            ..Default::default()
        }));
    }

    let target_config = read_config(Path::new(&req.target_project_path))
        .await
        .ok()
        .flatten()
        .unwrap_or_default();
    let priority_levels = target_config.priority_levels;
    let item_type_config = default_issue_config(&target_config);

    let options = DuplicateGenericItemOptions {
        source_project_path: PathBuf::from(&req.source_project_path),
        target_project_path: PathBuf::from(&req.target_project_path),
        item_id: req.issue_id,
        new_id: None, // Issues always get a new UUID
        new_title: nonempty(req.new_title),
    };

    match generic_duplicate(&item_type_config, options).await {
        Ok(result) => {
            maybe_run_post_hooks(
                Path::new(&hook_project_path),
                "issue",
                HookOperation::Duplicate,
                &hook_project_path,
                Some(&hook_item_id),
                Some(hook_request_data),
                true,
            )
            .await;

            // Read updated manifest from target project
            let manifest = read_manifest(Path::new(&req.target_project_path))
                .await
                .ok()
                .flatten();

            // Convert GenericItem to Issue proto
            let display_number = result.item.frontmatter.display_number.unwrap_or(0);
            let priority = result.item.frontmatter.priority.unwrap_or(0);
            let issue = Issue {
                id: result.item.id.clone(),
                display_number,
                issue_number: result.item.id.clone(),
                title: result.item.title.clone(),
                description: result.item.body.clone(),
                metadata: Some(IssueMetadata {
                    display_number,
                    status: result.item.frontmatter.status.clone().unwrap_or_default(),
                    priority: priority as i32,
                    created_at: result.item.frontmatter.created_at.clone(),
                    updated_at: result.item.frontmatter.updated_at.clone(),
                    custom_fields: result
                        .item
                        .frontmatter
                        .custom_fields
                        .iter()
                        .map(|(k, v)| (k.clone(), v.to_string()))
                        .collect(),
                    priority_label: priority_label(priority, priority_levels),
                    draft: false,
                    deleted_at: String::new(),
                    is_org_issue: false,
                    org_slug: String::new(),
                    org_display_number: 0,
                }),
            };

            Ok(Response::new(DuplicateIssueResponse {
                success: true,
                error: String::new(),
                issue: Some(issue),
                original_issue_id: result.original_id,
                manifest: manifest.as_ref().map(manifest_to_proto),
            }))
        }
        Err(e) => {
            maybe_run_post_hooks(
                Path::new(&hook_project_path),
                "issue",
                HookOperation::Duplicate,
                &hook_project_path,
                Some(&hook_item_id),
                Some(hook_request_data),
                false,
            )
            .await;
            Ok(Response::new(DuplicateIssueResponse {
                success: false,
                error: to_error_json(&req.source_project_path, &e),
                issue: None,
                original_issue_id: String::new(),
                manifest: None,
            }))
        }
    }
}
