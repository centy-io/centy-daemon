use std::path::Path;

use crate::config::read_config;
use crate::hooks::{HookItemType, HookOperation};
use crate::link::CreateLinkOptions;
use crate::registry::track_project_async;
use crate::server::convert_link::{internal_link_to_proto, proto_link_target_to_internal};
use crate::server::hooks_helper::{maybe_run_post_hooks, maybe_run_pre_hooks};
use crate::server::proto::{CreateLinkRequest, CreateLinkResponse};
use tonic::{Response, Status};

pub async fn create_link(req: CreateLinkRequest) -> Result<Response<CreateLinkResponse>, Status> {
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
        HookOperation::Create,
        &hook_project_path,
        Some(&hook_item_id),
        Some(hook_request_data.clone()),
    )
    .await
    {
        return Ok(Response::new(CreateLinkResponse {
            success: false,
            error: e,
            ..Default::default()
        }));
    }

    let source_type = proto_link_target_to_internal(req.source_type());
    let target_type = proto_link_target_to_internal(req.target_type());
    let custom_types = match read_config(project_path).await {
        Ok(Some(config)) => config.custom_link_types,
        Ok(None) | Err(_) => vec![],
    };

    let options = CreateLinkOptions {
        source_id: req.source_id,
        source_type,
        target_id: req.target_id,
        target_type,
        link_type: req.link_type,
    };

    match crate::link::create_link(project_path, options, &custom_types).await {
        Ok(result) => {
            maybe_run_post_hooks(
                project_path,
                HookItemType::Link,
                HookOperation::Create,
                &hook_project_path,
                Some(&hook_item_id),
                Some(hook_request_data),
                true,
            )
            .await;
            Ok(Response::new(CreateLinkResponse {
                success: true,
                error: String::new(),
                created_link: Some(internal_link_to_proto(&result.created_link)),
                inverse_link: Some(internal_link_to_proto(&result.inverse_link)),
            }))
        }
        Err(e) => {
            maybe_run_post_hooks(
                project_path,
                HookItemType::Link,
                HookOperation::Create,
                &hook_project_path,
                Some(&hook_item_id),
                Some(hook_request_data),
                false,
            )
            .await;
            Ok(Response::new(CreateLinkResponse {
                success: false,
                error: e.to_string(),
                created_link: None,
                inverse_link: None,
            }))
        }
    }
}
