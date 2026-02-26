use crate::config::write_config;
use crate::server::proto::Config as ProtoConfig;
use crate::server::proto_to_config::proto_to_config;
use std::path::Path;
/// Merge non-zero/non-empty fields from `proto_config` over the existing project config.
pub(super) async fn apply_init_config(
    project_path: &Path,
    proto_config: &ProtoConfig,
) -> Result<(), mdstore::ConfigError> {
    let mut config = crate::config::read_config(project_path)
        .await?
        .unwrap_or_default();
    let overrides = proto_to_config(proto_config);
    if overrides.priority_levels != 0 {
        config.priority_levels = overrides.priority_levels;
    }
    if overrides.version.is_some() {
        config.version = overrides.version;
    }
    if overrides.default_editor.is_some() {
        config.default_editor = overrides.default_editor;
    }
    if !overrides.custom_fields.is_empty() {
        config.custom_fields = overrides.custom_fields;
    }
    if !overrides.defaults.is_empty() {
        config.defaults = overrides.defaults;
    }
    if !overrides.state_colors.is_empty() {
        config.state_colors = overrides.state_colors;
    }
    if !overrides.priority_colors.is_empty() {
        config.priority_colors = overrides.priority_colors;
    }
    if !overrides.custom_link_types.is_empty() {
        config.custom_link_types = overrides.custom_link_types;
    }
    if !overrides.hooks.is_empty() {
        config.hooks = overrides.hooks;
    }
    if let Some(ref ws) = proto_config.workspace {
        if let Some(v) = ws.update_status_on_open {
            config.workspace.update_status_on_open = Some(v);
        }
    }
    write_config(project_path, &config).await
}
