use crate::hooks::HookOperation;
use crate::link::{CustomLinkTypeDefinition, UpdateLinkOptions};
use crate::server::convert_link::link_view_to_proto;
use crate::server::hooks_helper::maybe_run_post_hooks;
use crate::server::proto::UpdateLinkResponse;
use crate::server::structured_error::to_error_json;
use std::path::Path;
use tonic::{Response, Status};

pub async fn run_update_link(
    project_path: &Path,
    options: UpdateLinkOptions,
    custom_types: Vec<CustomLinkTypeDefinition>,
    hook_project_path: String,
    hook_item_id: String,
    hook_request_data: serde_json::Value,
    cwd: &str,
) -> Result<Response<UpdateLinkResponse>, Status> {
    match crate::link::update_link(project_path, options, &custom_types).await {
        Ok(record) => {
            maybe_run_post_hooks(
                project_path,
                "link",
                HookOperation::Update,
                &hook_project_path,
                Some(&hook_item_id),
                Some(hook_request_data),
                true,
            )
            .await;
            Ok(Response::new(UpdateLinkResponse {
                success: true,
                error: String::new(),
                updated_link: Some(link_view_to_proto(&record.source_view())),
                inverse_link: Some(link_view_to_proto(&record.target_view())),
            }))
        }
        Err(e) => {
            maybe_run_post_hooks(
                project_path,
                "link",
                HookOperation::Update,
                &hook_project_path,
                Some(&hook_item_id),
                Some(hook_request_data),
                false,
            )
            .await;
            Ok(Response::new(UpdateLinkResponse {
                success: false,
                error: to_error_json(cwd, &e),
                updated_link: None,
                inverse_link: None,
            }))
        }
    }
}
