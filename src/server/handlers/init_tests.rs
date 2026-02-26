use crate::config::{CentyConfig, ProjectMetadata, WorkspaceConfig};

/// Compile-time exhaustiveness check: if `CentyConfig`, `WorkspaceConfig`, or
/// `ProjectMetadata` gains a new field this test will fail to compile, forcing
/// the developer to explicitly handle or acknowledge the new field in `apply_init_config`.
///
/// Fields not configurable at init time (complex types, deprecated, or internal) should be
/// listed here with a comment explaining why they are intentionally excluded from the init API.
#[test]
fn test_all_config_fields_acknowledged_in_init() {
    let CentyConfig {
        version: _,           // internal: set by daemon, not exposed as init flag
        priority_levels: _,   // ✓ exposed via InitRequest.init_config (Config.priority_levels)
        custom_fields: _,     // ✓ exposed via InitRequest.init_config (Config.custom_fields)
        defaults: _,          // ✓ exposed via InitRequest.init_config (Config.defaults)
        allowed_states: _,    // deprecated: migrated to per-item-type config.yaml
        state_colors: _,      // ✓ exposed via InitRequest.init_config (Config.state_colors)
        priority_colors: _,   // ✓ exposed via InitRequest.init_config (Config.priority_colors)
        custom_link_types: _, // ✓ exposed via InitRequest.init_config (Config.custom_link_types)
        default_editor: _,    // ✓ exposed via InitRequest.init_config (Config.default_editor)
        hooks: _,             // ✓ exposed via InitRequest.init_config (Config.hooks)
        workspace,            // ✓ exposed via InitRequest.init_config (Config.workspace)
    } = CentyConfig::default();

    let WorkspaceConfig {
        update_status_on_open: _, // ✓ exposed via InitRequest.init_config (Config.workspace.update_status_on_open)
    } = workspace;

    // ProjectMetadata fields are exposed directly on InitRequest (not inside init_config).
    let ProjectMetadata {
        title: _, // ✓ exposed via InitRequest.title
    } = ProjectMetadata::default();
}
