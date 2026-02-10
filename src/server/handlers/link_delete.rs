use std::path::Path;

use crate::config::read_config;
use crate::hooks::{HookItemType, HookOperation};
use crate::link::DeleteLinkOptions;
use crate::registry::track_project_async;
use crate::server::convert_link::proto_link_target_to_internal;
use crate::server::helpers::nonempty;
use crate::server::hooks_helper::{maybe_run_post_hooks, maybe_run_pre_hooks};
use crate::server::proto::{DeleteLinkRequest, DeleteLinkResponse};
use crate::server::structured_error::to_error_json;
use tonic::{Response, Status};

pub async fn delete_link(req: DeleteLinkRequest) -> Result<Response<DeleteLinkResponse>, Status> {
    track_project_async(req.project_path.clone());
    let project_path = Path::new(&req.project_path);
    let hook_project_path = req.project_path.clone();
    let hook_item_id = req.source_id.clone();
    let hook_request_data = serde_json::json!({
        "source_id": &req.source_id,
        "target_id": &req.target_id,
        "link_type": &req.link_type,
    });
    if let Err(e) = maybe_run_pre_hooks(
        project_path,
        HookItemType::Link,
        HookOperation::Delete,
        &hook_project_path,
        Some(&hook_item_id),
        Some(hook_request_data.clone()),
    )
    .await
    {
        return Ok(Response::new(DeleteLinkResponse {
            success: false,
            error: to_error_json(&req.project_path, &e),
            ..Default::default()
        }));
    }

    let source_type = proto_link_target_to_internal(req.source_type());
    let target_type = proto_link_target_to_internal(req.target_type());
    let custom_types = match read_config(project_path).await {
        Ok(Some(config)) => config.custom_link_types,
        Ok(None) | Err(_) => vec![],
    };

    let options = DeleteLinkOptions {
        source_id: req.source_id,
        source_type,
        target_id: req.target_id,
        target_type,
        link_type: nonempty(req.link_type),
    };

    match crate::link::delete_link(project_path, options, &custom_types).await {
        Ok(result) => {
            maybe_run_post_hooks(
                project_path,
                HookItemType::Link,
                HookOperation::Delete,
                &hook_project_path,
                Some(&hook_item_id),
                Some(hook_request_data),
                true,
            )
            .await;
            Ok(Response::new(DeleteLinkResponse {
                success: true,
                error: String::new(),
                deleted_count: result.deleted_count,
            }))
        }
        Err(e) => {
            maybe_run_post_hooks(
                project_path,
                HookItemType::Link,
                HookOperation::Delete,
                &hook_project_path,
                Some(&hook_item_id),
                Some(hook_request_data),
                false,
            )
            .await;
            Ok(Response::new(DeleteLinkResponse {
                success: false,
                error: to_error_json(&req.project_path, &e),
                deleted_count: 0,
            }))
        }
    }
}
