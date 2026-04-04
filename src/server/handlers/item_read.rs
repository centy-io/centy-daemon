use std::path::Path;

use crate::item::core::error::ItemError;
use crate::item::generic::storage::{generic_get, generic_get_by_display_number};
use crate::registry::org_repo::find_org_repo;
use crate::registry::track_project_async;
use crate::server::convert_entity::{generic_item_to_proto, user_to_generic_item_proto};
use crate::server::proto::{GetItemRequest, GetItemResponse};
use crate::server::structured_error::to_error_json;
use crate::user::get_user;
use mdstore::{Filters, TypeConfig};
use tonic::{Response, Status};

use super::item_type_resolve::{resolve_item_id, resolve_item_type_config};

pub async fn get_item(req: GetItemRequest) -> Result<Response<GetItemResponse>, Status> {
    track_project_async(req.project_path.clone());
    let project_path = Path::new(&req.project_path);

    // Route user type to user-specific handler
    let lower = req.item_type.to_lowercase();
    if lower == "user" || lower == "users" {
        return match get_user(project_path, &req.item_id).await {
            Ok(user) => Ok(Response::new(GetItemResponse {
                success: true,
                error: String::new(),
                item: Some(user_to_generic_item_proto(&user)),
                source: String::new(),
            })),
            Err(e) => Ok(Response::new(GetItemResponse {
                success: false,
                error: to_error_json(&req.project_path, &e),
                item: None,
                source: String::new(),
            })),
        };
    }

    let (item_type, config) = match resolve_item_type_config(project_path, &req.item_type).await {
        Ok(pair) => pair,
        Err(e) => {
            return Ok(Response::new(GetItemResponse {
                success: false,
                error: to_error_json(&req.project_path, &e),
                item: None,
                source: String::new(),
            }));
        }
    };

    // Dispatch: if display_number is specified and > 0, look up by display number
    let result = match req.display_number {
        Some(dn) if dn > 0 => {
            generic_get_by_display_number(project_path, &item_type, &config, dn).await
        }
        _ => {
            let resolved_id =
                resolve_item_id(project_path, &item_type, &config, &req.item_id).await;
            match resolved_id {
                Ok(id) => generic_get(project_path, &item_type, &id).await,
                Err(e) => Err(e),
            }
        }
    };

    match result {
        Ok(item) => Ok(Response::new(GetItemResponse {
            success: true,
            error: String::new(),
            item: Some(generic_item_to_proto(&item, &item_type)),
            source: String::new(),
        })),
        Err(e) if matches!(e, ItemError::NotFound(_)) => {
            // Not found in project — try org repo fallback.
            match find_org_repo(&req.project_path).await {
                Ok(Some(org_repo_path)) => {
                    let org_path = Path::new(&org_repo_path);
                    match get_from_org_repo(
                        org_path,
                        &item_type,
                        &config,
                        req.display_number,
                        &req.item_id,
                    )
                    .await
                    {
                        Ok(item) => {
                            let mut item_proto = generic_item_to_proto(&item, &item_type);
                            item_proto.source = "org".to_string();
                            Ok(Response::new(GetItemResponse {
                                success: true,
                                error: String::new(),
                                item: Some(item_proto),
                                source: "org".to_string(),
                            }))
                        }
                        Err(org_err) => Ok(Response::new(GetItemResponse {
                            success: false,
                            error: to_error_json(&req.project_path, &org_err),
                            item: None,
                            source: String::new(),
                        })),
                    }
                }
                // No org repo tracked or error discovering it — return the original not-found.
                _ => Ok(Response::new(GetItemResponse {
                    success: false,
                    error: to_error_json(&req.project_path, &e),
                    item: None,
                    source: String::new(),
                })),
            }
        }
        Err(e) => Ok(Response::new(GetItemResponse {
            success: false,
            error: to_error_json(&req.project_path, &e),
            item: None,
            source: String::new(),
        })),
    }
}

/// Look up an item in the org repo.
///
/// The org repo path IS the `.centy` directory, so item storage dirs are
/// `org_path/<item_type>/` rather than `org_path/.centy/<item_type>/`.
async fn get_from_org_repo(
    org_path: &Path,
    item_type: &str,
    config: &TypeConfig,
    display_number: Option<u32>,
    item_id: &str,
) -> Result<mdstore::Item, ItemError> {
    // The org repo path is already the .centy dir; do not append ".centy".
    let type_dir = org_path.join(item_type);

    match display_number {
        Some(dn) if dn > 0 => lookup_by_display_number_in_dir(&type_dir, config, dn).await,
        _ => {
            // If item_id looks like a display number, search by it.
            if config.features.display_number {
                if let Ok(num) = item_id.parse::<u32>() {
                    if num > 0 {
                        return lookup_by_display_number_in_dir(&type_dir, config, num).await;
                    }
                }
            }
            Ok(mdstore::get(&type_dir, item_id).await?)
        }
    }
}

/// Scan `type_dir` for an item with the given display number.
async fn lookup_by_display_number_in_dir(
    type_dir: &Path,
    config: &TypeConfig,
    display_number: u32,
) -> Result<mdstore::Item, ItemError> {
    if !config.features.display_number {
        return Err(ItemError::FeatureNotEnabled(
            "display_number is not enabled for this item type".to_string(),
        ));
    }
    let items = mdstore::list(type_dir, Filters::new().include_deleted()).await?;
    for item in items {
        if item.frontmatter.display_number == Some(display_number) {
            return Ok(item);
        }
    }
    Err(ItemError::NotFound(format!(
        "display_number {display_number}"
    )))
}
