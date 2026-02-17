use crate::config::CentyConfig;

use super::proto::{
    Config, CustomFieldDefinition, HookDefinition as ProtoHookDefinition, LinkTypeDefinition,
    WorkspaceConfig as ProtoWorkspaceConfig,
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
        priority_levels: config.priority_levels as i32,
        version: config.effective_version(),
        state_colors: config.state_colors.clone(),
        priority_colors: config.priority_colors.clone(),
        custom_link_types: config
            .custom_link_types
            .iter()
            .map(|lt| LinkTypeDefinition {
                name: lt.name.clone(),
                inverse: lt.inverse.clone(),
                description: lt.description.clone().unwrap_or_default(),
            })
            .collect(),
        default_editor: config.default_editor.clone().unwrap_or_default(),
        hooks: config
            .hooks
            .iter()
            .map(|h| ProtoHookDefinition {
                pattern: h.pattern.clone(),
                command: h.command.clone(),
                run_async: h.is_async,
                timeout: h.timeout,
                enabled: h.enabled,
            })
            .collect(),
        workspace: Some(ProtoWorkspaceConfig {
            update_status_on_open: config.workspace.update_status_on_open,
        }),
    }
}
