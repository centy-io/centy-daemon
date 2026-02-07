use crate::config::{
    CentyConfig, CustomFieldDefinition as InternalCustomFieldDef, LlmConfig as InternalLlmConfig,
};
use crate::hooks::HookDefinition as InternalHookDefinition;

use super::proto::Config;

pub fn proto_to_config(proto: &Config) -> CentyConfig {
    let llm_config = proto
        .llm
        .as_ref()
        .map(|l| InternalLlmConfig {
            auto_close_on_complete: l.auto_close_on_complete,
            update_status_on_start: l.update_status_on_start,
            allow_direct_edits: l.allow_direct_edits,
            default_workspace_mode: l.default_workspace_mode,
        })
        .unwrap_or_default();

    CentyConfig {
        version: if proto.version.is_empty() {
            None
        } else {
            Some(proto.version.clone())
        },
        priority_levels: proto.priority_levels as u32,
        custom_fields: proto
            .custom_fields
            .iter()
            .map(|f| InternalCustomFieldDef {
                name: f.name.clone(),
                field_type: f.field_type.clone(),
                required: f.required,
                default_value: if f.default_value.is_empty() {
                    None
                } else {
                    Some(f.default_value.clone())
                },
                enum_values: f.enum_values.clone(),
            })
            .collect(),
        defaults: proto.defaults.clone(),
        allowed_states: proto.allowed_states.clone(),
        default_state: proto.default_state.clone(),
        state_colors: proto.state_colors.clone(),
        priority_colors: proto.priority_colors.clone(),
        llm: llm_config,
        custom_link_types: proto
            .custom_link_types
            .iter()
            .map(|lt| crate::link::CustomLinkTypeDefinition {
                name: lt.name.clone(),
                inverse: lt.inverse.clone(),
                description: if lt.description.is_empty() {
                    None
                } else {
                    Some(lt.description.clone())
                },
            })
            .collect(),
        default_editor: if proto.default_editor.is_empty() {
            None
        } else {
            Some(proto.default_editor.clone())
        },
        hooks: proto
            .hooks
            .iter()
            .map(|h| InternalHookDefinition {
                pattern: h.pattern.clone(),
                command: h.command.clone(),
                is_async: h.run_async,
                timeout: if h.timeout == 0 { 30 } else { h.timeout },
                enabled: h.enabled,
            })
            .collect(),
    }
}
