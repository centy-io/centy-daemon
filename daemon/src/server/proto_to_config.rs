use crate::config::{is_system_key, CentyConfig, WorkspaceConfig};
use crate::server::helpers::nonempty;
use mdstore::CustomFieldDef as InternalCustomFieldDef;

use super::proto::Config;

pub fn proto_to_config(proto: &Config) -> CentyConfig {
    CentyConfig {
        version: nonempty(proto.version.clone()),
        priority_levels: u32::try_from(proto.priority_levels).unwrap_or(0),
        custom_fields: proto
            .custom_fields
            .iter()
            .map(|f| InternalCustomFieldDef {
                name: f.name.clone(),
                field_type: f.field_type.clone(),
                required: f.required,
                default_value: nonempty(f.default_value.clone()),
                enum_values: f.enum_values.clone(),
            })
            .collect(),
        defaults: proto.defaults.clone(),
        state_colors: proto.state_colors.clone(),
        priority_colors: proto.priority_colors.clone(),
        custom_link_types: proto
            .custom_link_types
            .iter()
            .map(|lt| crate::link::CustomLinkTypeDefinition {
                name: lt.name.clone(),
                description: nonempty(lt.description.clone()),
            })
            .collect(),
        default_editor: nonempty(proto.default_editor.clone()),
        workspace: proto
            .workspace
            .as_ref()
            .map(|w| WorkspaceConfig {
                update_status_on_open: w.update_status_on_open,
            })
            .unwrap_or_default(),
        cleanup: crate::config::CleanupConfig::default(),
        extra: proto
            .user_values
            .iter()
            .filter(|(k, _)| !is_system_key(k))
            .map(|(k, v)| (k.clone(), serde_json::Value::String(v.clone())))
            .collect(),
    }
}
