use std::path::Path;

use crate::config::item_type_config::{write_item_type_config, ItemTypeRegistry};
use crate::manifest::{read_manifest, update_manifest, write_manifest};
use crate::registry::track_project_async;
use crate::server::proto::{CreateItemTypeRequest, CreateItemTypeResponse, ItemTypeConfigProto};
use crate::server::structured_error::StructuredError;
use mdstore::{CustomFieldDef, IdStrategy, TypeConfig, TypeFeatures};
use tonic::{Response, Status};

/// Validate the plural field: must be lowercase alphanumeric + hyphens, non-empty.
fn is_valid_plural(s: &str) -> bool {
    !s.is_empty()
        && s.chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
        && !s.starts_with('-')
        && !s.ends_with('-')
}

fn error_response(cwd: &str, code: &str, message: String) -> Response<CreateItemTypeResponse> {
    let se = StructuredError::new(cwd, code, message);
    Response::new(CreateItemTypeResponse {
        success: false,
        error: se.to_json(),
        config: None,
    })
}

fn config_to_proto(folder: &str, config: &TypeConfig) -> ItemTypeConfigProto {
    ItemTypeConfigProto {
        name: config.name.clone(),
        plural: folder.to_string(),
        identifier: config.identifier.to_string(),
        features: Some(crate::server::proto::ItemTypeFeatures {
            display_number: config.features.display_number,
            status: config.features.status,
            priority: config.features.priority,
            assets: config.features.assets,
            org_sync: config.features.org_sync,
            r#move: config.features.move_item,
            duplicate: config.features.duplicate,
        }),
        statuses: config.statuses.clone(),
        default_status: config.default_status.clone().unwrap_or_default(),
        priority_levels: config.priority_levels.unwrap_or(0),
        custom_fields: config
            .custom_fields
            .iter()
            .map(|f| crate::server::proto::CustomFieldDefinition {
                name: f.name.clone(),
                field_type: f.field_type.clone(),
                required: f.required,
                default_value: f.default_value.clone().unwrap_or_default(),
                enum_values: f.enum_values.clone(),
            })
            .collect(),
    }
}

/// Validate the request fields. Returns an error message on failure, or Ok(()) on success.
fn validate_request(req: &CreateItemTypeRequest) -> Result<(), (String, String)> {
    if req.name.trim().is_empty() {
        return Err(("VALIDATION_ERROR".into(), "name must not be empty".into()));
    }
    if !is_valid_plural(&req.plural) {
        return Err((
            "VALIDATION_ERROR".into(),
            "plural must be lowercase alphanumeric with hyphens (e.g., \"bugs\", \"epics\")".into(),
        ));
    }
    if req.identifier != "uuid" && req.identifier != "slug" {
        return Err((
            "VALIDATION_ERROR".into(),
            "identifier must be \"uuid\" or \"slug\"".into(),
        ));
    }
    if !req.default_status.is_empty() {
        if req.statuses.is_empty() {
            return Err((
                "VALIDATION_ERROR".into(),
                "default_status provided but statuses list is empty".into(),
            ));
        }
        if !req.statuses.contains(&req.default_status) {
            return Err((
                "VALIDATION_ERROR".into(),
                format!(
                    "default_status \"{}\" must be in statuses list {:?}",
                    req.default_status, req.statuses
                ),
            ));
        }
    }
    Ok(())
}

/// Build a `TypeConfig` from the validated request.
fn build_config(req: CreateItemTypeRequest) -> TypeConfig {
    let identifier = if req.identifier == "slug" {
        IdStrategy::Slug
    } else {
        IdStrategy::Uuid
    };
    let features = req.features.unwrap_or_default();
    TypeConfig {
        name: req.name,
        identifier,
        features: TypeFeatures {
            display_number: features.display_number,
            status: features.status,
            priority: features.priority,
            assets: features.assets,
            org_sync: features.org_sync,
            move_item: features.r#move,
            duplicate: features.duplicate,
        },
        statuses: req.statuses,
        default_status: if req.default_status.is_empty() {
            None
        } else {
            Some(req.default_status)
        },
        priority_levels: if req.priority_levels == 0 {
            None
        } else {
            Some(req.priority_levels)
        },
        custom_fields: req
            .custom_fields
            .into_iter()
            .map(|f| CustomFieldDef {
                name: f.name,
                field_type: f.field_type,
                required: f.required,
                default_value: if f.default_value.is_empty() {
                    None
                } else {
                    Some(f.default_value)
                },
                enum_values: f.enum_values,
            })
            .collect(),
    }
}

pub async fn create_item_type(
    req: CreateItemTypeRequest,
) -> Result<Response<CreateItemTypeResponse>, Status> {
    track_project_async(req.project_path.clone());
    let cwd = req.project_path.clone();
    let project_path = Path::new(&cwd);

    if let Err((code, msg)) = validate_request(&req) {
        return Ok(error_response(&cwd, &code, msg));
    }

    // Check for duplicates against existing types
    match ItemTypeRegistry::build(project_path).await {
        Ok(registry) => {
            for folder in registry.folders() {
                if folder.eq_ignore_ascii_case(&req.plural) {
                    return Ok(error_response(
                        &cwd,
                        "ALREADY_EXISTS",
                        format!("Item type with plural \"{}\" already exists", req.plural),
                    ));
                }
            }
            for config in registry.all().values() {
                if config.name.eq_ignore_ascii_case(&req.name) {
                    return Ok(error_response(
                        &cwd,
                        "ALREADY_EXISTS",
                        format!("Item type with name \"{}\" already exists", req.name),
                    ));
                }
            }
        }
        Err(e) => {
            return Ok(error_response(
                &cwd,
                "IO_ERROR",
                format!("Failed to discover existing item types: {e}"),
            ));
        }
    }

    let plural = req.plural.clone();
    let config = build_config(req);

    // Write config.yaml to disk (also creates the directory)
    if let Err(e) = write_item_type_config(project_path, &plural, &config).await {
        return Ok(error_response(
            &cwd,
            "IO_ERROR",
            format!("Failed to write item type config: {e}"),
        ));
    }

    // Update manifest timestamp
    if let Ok(Some(mut manifest)) = read_manifest(project_path).await {
        update_manifest(&mut manifest);
        let _ = write_manifest(project_path, &manifest).await;
    }

    Ok(Response::new(CreateItemTypeResponse {
        success: true,
        error: String::new(),
        config: Some(config_to_proto(&plural, &config)),
    }))
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn test_is_valid_plural() {
        assert!(is_valid_plural("bugs"));
        assert!(is_valid_plural("user-stories"));
        assert!(is_valid_plural("epics123"));
        assert!(!is_valid_plural(""));
        assert!(!is_valid_plural("Bugs")); // uppercase
        assert!(!is_valid_plural("bug_reports")); // underscore
        assert!(!is_valid_plural("-bugs")); // leading hyphen
        assert!(!is_valid_plural("bugs-")); // trailing hyphen
        assert!(!is_valid_plural("my bugs")); // space
    }

    #[test]
    fn test_config_to_proto_roundtrip() {
        let config = TypeConfig {
            name: "Bug".to_string(),
            identifier: IdStrategy::Uuid,
            features: TypeFeatures {
                display_number: true,
                status: true,
                priority: true,
                assets: false,
                org_sync: false,
                move_item: true,
                duplicate: true,
            },
            statuses: vec!["open".to_string(), "closed".to_string()],
            default_status: Some("open".to_string()),
            priority_levels: Some(3),
            custom_fields: vec![],
        };

        let proto = config_to_proto("bugs", &config);
        assert_eq!(proto.name, "Bug");
        assert_eq!(proto.plural, "bugs");
        assert_eq!(proto.identifier, "uuid");
        assert_eq!(proto.statuses, vec!["open", "closed"]);
        assert_eq!(proto.default_status, "open");
        assert_eq!(proto.priority_levels, 3);

        let f = proto.features.unwrap();
        assert!(f.display_number);
        assert!(f.status);
        assert!(f.priority);
        assert!(!f.assets);
        assert!(f.r#move);
        assert!(f.duplicate);
    }
}
