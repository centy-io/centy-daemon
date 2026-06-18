use super::super::config::{HookDefinition, HookOperation, HooksFile, ParsedPattern, Phase};
use crate::utils::get_centy_path;
use std::path::Path;
/// Load hooks configuration from the project's hooks.yaml.
/// Returns an empty vec if the file does not exist or has no hooks.
pub async fn load_hooks_config(project_path: &Path) -> Vec<HookDefinition> {
    let hooks_path = get_centy_path(project_path).join("hooks.yaml");
    let Ok(content) = tokio::fs::read_to_string(&hooks_path).await else {
        return Vec::new();
    };
    serde_yaml::from_str::<HooksFile>(&content)
        .map(|f| f.hooks)
        .unwrap_or_default()
}
/// Find matching hooks for the given phase, `item_type`, and operation.
/// Returns enabled hooks sorted by specificity descending (most-specific-first).
#[must_use]
pub fn find_matching_hooks<'hook>(
    hooks: &'hook [HookDefinition],
    phase: Phase,
    item_type: &str,
    operation: HookOperation,
) -> Vec<&'hook HookDefinition> {
    let mut matching: Vec<(&HookDefinition, u8)> = hooks
        .iter()
        .filter(|h| h.enabled)
        .filter_map(|h| {
            ParsedPattern::parse(&h.pattern)
                .ok()
                .filter(|p| p.matches(phase, item_type, operation))
                .map(|p| (h, p.specificity()))
        })
        .collect();
    matching.sort_by(|a, b| b.1.cmp(&a.1));
    matching.into_iter().map(|(h, _)| h).collect()
}
