use crate::config::item_type_config::{ItemTypeConfig, ItemTypeFeatures};
use crate::server::proto::CreateItemTypeRequest;
use mdstore::{CustomFieldDef, IdStrategy};
/// Build an `ItemTypeConfig` from the validated request.
pub(super) fn build_config(req: CreateItemTypeRequest) -> ItemTypeConfig {
    let identifier = if req.identifier == "slug" {
        IdStrategy::Slug
    } else {
        IdStrategy::Uuid
    };
    let features = req.features.unwrap_or_default();
    ItemTypeConfig {
        name: req.name,
        icon: None,
        identifier,
        features: ItemTypeFeatures {
            display_number: features.display_number,
            status: features.status,
            priority: features.priority,
            soft_delete: features.soft_delete,
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
        template: None,
    }
}
