use super::super::config::{HookDefinition, HookOperation, ParsedPattern, Phase};
use std::path::Path;
/// Load hooks configuration from the project's config.json.
/// Returns an empty vec if no config exists or no hooks are configured.
pub async fn load_hooks_config(project_path: &Path) -> Vec<HookDefinition> {
    match crate::config::read_config(project_path).await {
        Ok(Some(config)) => config.hooks,
        _ => Vec::new(),
    }
}
/// Find matching hooks for the given phase, item_type, and operation.
/// Returns enabled hooks sorted by specificity descending (most-specific-first).
pub fn find_matching_hooks<'a>(
    hooks: &'a [HookDefinition],
    phase: Phase,
    item_type: &str,
    operation: HookOperation,
) -> Vec<&'a HookDefinition> {
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
