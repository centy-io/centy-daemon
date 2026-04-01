/// All camelCase field names managed by the system in config.json.
const SYSTEM_KEYS: &[&str] = &[
    "version",
    "priorityLevels",
    "customFields",
    "defaults",
    "stateColors",
    "priorityColors",
    "customLinkTypes",
    "defaultEditor",
    "workspace",
    "cleanup",
];

/// Key prefixes that belong to system-managed sections.
const SECTION_PREFIXES: &[&str] = &["workspace."];

/// Returns `true` if `key` is a system-managed config key (exact match or section prefix).
#[must_use]
pub fn is_system_key(key: &str) -> bool {
    SYSTEM_KEYS.contains(&key)
        || SECTION_PREFIXES
            .iter()
            .any(|prefix| key.starts_with(prefix))
}
