use crate::config::CentyConfig;

use super::proto::{
    Config, CustomFieldDefinition, LinkTypeDefinition, WorkspaceConfig as ProtoWorkspaceConfig,
};

pub fn config_to_proto(config: &CentyConfig) -> Config {
    Config {
        custom_fields: config
            .custom_fields
            .iter()
            .map(|f| CustomFieldDefinition {
                name: f.name.clone(),
                field_type: f.field_type.clone(),
                required: f.required,
                default_value: f.default_value.clone().unwrap_or_default(),
                enum_values: f.enum_values.clone(),
            })
            .collect(),
        defaults: config.defaults.clone(),
        priority_levels: i32::try_from(config.priority_levels).unwrap_or(i32::MAX),
        version: config.effective_version(),
        state_colors: config.state_colors.clone(),
        priority_colors: config.priority_colors.clone(),
        custom_link_types: config
            .custom_link_types
            .iter()
            .map(|lt| LinkTypeDefinition {
                name: lt.name.clone(),
                description: lt.description.clone().unwrap_or_default(),
            })
            .collect(),
        default_editor: config.default_editor.clone().unwrap_or_default(),
        workspace: Some(ProtoWorkspaceConfig {
            update_status_on_open: config.workspace.update_status_on_open,
        }),
        user_values: config
            .extra
            .iter()
            .map(|(k, v)| {
                let s = match v {
                    serde_json::Value::String(s) => s.clone(),
                    serde_json::Value::Null
                    | serde_json::Value::Bool(_)
                    | serde_json::Value::Number(_)
                    | serde_json::Value::Array(_)
                    | serde_json::Value::Object(_) => v.to_string(),
                };
                (k.clone(), s)
            })
            .collect(),
    }
}
