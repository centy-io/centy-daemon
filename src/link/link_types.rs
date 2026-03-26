use super::types::CustomLinkTypeDefinition;
/// All built-in link type names (each is a valid link type from the source's perspective).
pub const BUILTIN_LINK_TYPES: &[&str] = &[
    "blocks",
    "blocked-by",
    "parent-of",
    "child-of",
    "relates-to",
    "related-from",
    "duplicates",
    "duplicated-by",
];
/// Check if a link type is valid (either built-in or custom).
#[must_use]
pub fn is_valid_link_type(link_type: &str, custom_types: &[CustomLinkTypeDefinition]) -> bool {
    if BUILTIN_LINK_TYPES.contains(&link_type) {
        return true;
    }
    custom_types.iter().any(|c| c.name == link_type)
}
