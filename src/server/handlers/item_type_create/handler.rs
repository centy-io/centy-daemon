use std::path::Path;
use crate::config::item_type_config::ItemTypeRegistry;
use crate::config::item_type_config::write_item_type_config;
use crate::manifest::{read_manifest, update_manifest, write_manifest};
use crate::registry::track_project_async;
use crate::server::assert_service::assert_initialized;
use crate::server::convert_entity::config_to_proto;
use crate::server::proto::{CreateItemTypeRequest, CreateItemTypeResponse};
use crate::server::structured_error::StructuredError;
use tonic::{Response, Status};
use super::build::build_config;
use super::validate::validate_request;
fn error_response(cwd: &str, code: &str, message: String) -> Response<CreateItemTypeResponse> {
    let se = StructuredError::new(cwd, code, message);
    Response::new(CreateItemTypeResponse { success: false, error: se.to_json(), config: None })
}
#[allow(unknown_lints, max_lines_per_function, clippy::too_many_lines)]
pub async fn create_item_type(
    req: CreateItemTypeRequest,
) -> Result<Response<CreateItemTypeResponse>, Status> {
    track_project_async(req.project_path.clone());
    let cwd = req.project_path.clone();
    let project_path = Path::new(&cwd);
    if let Err(e) = assert_initialized(project_path).await {
        return Ok(error_response(&cwd, "NOT_INITIALIZED", e.to_string()));
    }
    if let Err((code, msg)) = validate_request(&req) {
        return Ok(error_response(&cwd, &code, msg));
    }
    match ItemTypeRegistry::build(project_path).await {
        Ok(registry) => {
            for folder in registry.folders() {
                if folder.eq_ignore_ascii_case(&req.plural) {
                    return Ok(error_response(&cwd, "ALREADY_EXISTS",
                        format!("Item type with plural \"{}\" already exists", req.plural)));
                }
            }
            for config in registry.all().values() {
                if config.name.eq_ignore_ascii_case(&req.name) {
                    return Ok(error_response(&cwd, "ALREADY_EXISTS",
                        format!("Item type with name \"{}\" already exists", req.name)));
                }
            }
        }
        Err(e) => {
            return Ok(error_response(&cwd, "IO_ERROR",
                format!("Failed to discover existing item types: {e}")));
        }
    }
    let plural = req.plural.clone();
    let config = build_config(req);
    if let Err(e) = write_item_type_config(project_path, &plural, &config).await {
        return Ok(error_response(&cwd, "IO_ERROR",
            format!("Failed to write item type config: {e}")));
    }
    if let Ok(Some(mut manifest)) = read_manifest(project_path).await {
        update_manifest(&mut manifest);
        let _ = write_manifest(project_path, &manifest).await;
    }
    Ok(Response::new(CreateItemTypeResponse {
        success: true, error: String::new(),
        config: Some(config_to_proto(&plural, &config)),
    }))
}
#[cfg(test)]
#[path = "tests.rs"]
mod tests;
