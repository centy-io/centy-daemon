use crate::config::read_config;
use crate::link::{CreateLinkOptions, TargetType};
use crate::registry::track_project_async;
use crate::server::error_mapping::ToStructuredError;
use crate::server::proto::{CreateLinkRequest, CreateLinkResponse};
use crate::server::structured_error::to_error_json;
use std::path::Path;
use tonic::{Response, Status};

mod core;
mod hooks;
mod resolution;
mod validation;

fn err_resp(
    cwd: &str,
    e: &(impl std::fmt::Display + ToStructuredError),
) -> Response<CreateLinkResponse> {
    Response::new(CreateLinkResponse {
        success: false,
        error: to_error_json(cwd, e),
        ..Default::default()
    })
}

pub async fn create_link(req: CreateLinkRequest) -> Result<Response<CreateLinkResponse>, Status> {
    track_project_async(req.project_path.clone());
    let project_path = Path::new(&req.project_path);
    if let Err(e) = validation::check_initialized(project_path) {
        return Ok(err_resp(&req.project_path, &e));
    }
    let hook_project_path = req.project_path.clone();
    let hook_item_id = req.source_id.clone();
    let hook_request_data = serde_json::json!({
        "source_id": &req.source_id, "target_id": &req.target_id, "link_type": &req.link_type,
    });
    if let Err(e) = hooks::run_pre_hooks(
        project_path,
        &hook_project_path,
        &hook_item_id,
        hook_request_data.clone(),
    )
    .await
    {
        return Ok(err_resp(&req.project_path, &e));
    }
    let (source_type_str, source_id_arg) = split_type_prefix(&req.source_item_type, &req.source_id);
    let (target_type_str, target_id_arg) = split_type_prefix(&req.target_item_type, &req.target_id);
    let source_type = TargetType::new(source_type_str.to_lowercase());
    let target_type = TargetType::new(target_type_str.to_lowercase());
    let (source_id, target_id) = match resolution::resolve_link_ids(
        project_path,
        &source_type,
        &target_type,
        &source_id_arg,
        &target_id_arg,
    )
    .await
    {
        Ok(ids) => ids,
        Err(e) => return Ok(err_resp(&req.project_path, &e)),
    };
    let custom_types = match read_config(project_path).await {
        Ok(Some(config)) => config.custom_link_types,
        Ok(None) | Err(_) => vec![],
    };
    let options = CreateLinkOptions {
        source_id,
        source_type,
        target_id,
        target_type,
        link_type: req.link_type,
    };
    core::run_create_link(
        project_path,
        options,
        custom_types,
        hook_project_path,
        hook_item_id,
        hook_request_data,
        &req.project_path,
    )
    .await
}

/// Parse an optional `type:id` prefix from a link source/target argument.
///
/// Link sources and targets may be given as `type:id` (e.g. `plan:1` or
/// `plan:<uuid>`) so items can be linked across types. When a prefix is
/// present the embedded type overrides `default_type` and the remaining `id`
/// is returned. UUIDs and display numbers never contain a colon, so a `:`
/// unambiguously marks a type prefix. Returns `(type, id)`.
fn split_type_prefix(default_type: &str, raw: &str) -> (String, String) {
    match raw.split_once(':') {
        Some((ty, id)) if !ty.is_empty() && !id.is_empty() => (ty.to_string(), id.to_string()),
        _ => (default_type.to_string(), raw.to_string()),
    }
}

#[cfg(test)]
mod split_type_prefix_tests {
    use super::split_type_prefix;

    #[test]
    fn no_prefix_keeps_default_type() {
        assert_eq!(
            split_type_prefix("issue", "1"),
            ("issue".to_string(), "1".to_string())
        );
        let uuid = "aaaaaaaa-bbbb-cccc-dddd-eeeeeeeeeeee";
        assert_eq!(
            split_type_prefix("issue", uuid),
            ("issue".to_string(), uuid.to_string())
        );
    }

    #[test]
    fn prefix_overrides_default_type() {
        assert_eq!(
            split_type_prefix("issue", "plan:1"),
            ("plan".to_string(), "1".to_string())
        );
        assert_eq!(
            split_type_prefix("issue", "plan:bbbbbbbb-1111-2222-3333-444444444444"),
            (
                "plan".to_string(),
                "bbbbbbbb-1111-2222-3333-444444444444".to_string()
            )
        );
    }

    #[test]
    fn malformed_prefix_falls_back_to_default() {
        assert_eq!(
            split_type_prefix("issue", ":1"),
            ("issue".to_string(), ":1".to_string())
        );
        assert_eq!(
            split_type_prefix("issue", "plan:"),
            ("issue".to_string(), "plan:".to_string())
        );
    }
}
