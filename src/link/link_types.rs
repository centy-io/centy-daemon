use super::types::CustomLinkTypeDefinition;
/// Built-in link types and their inverses
pub const BUILTIN_LINK_TYPES: &[(&str, &str)] = &[
    ("blocks", "blocked-by"),
    ("blocked-by", "blocks"),
    ("parent-of", "child-of"),
    ("child-of", "parent-of"),
    ("relates-to", "related-from"),
    ("related-from", "relates-to"),
    ("duplicates", "duplicated-by"),
    ("duplicated-by", "duplicates"),
];
/// Get the inverse of a link type.
/// Returns `None` if the link type is not found.
pub fn get_inverse_link_type(
    link_type: &str,
    custom_types: &[CustomLinkTypeDefinition],
) -> Option<String> {
    for (name, inverse) in BUILTIN_LINK_TYPES {
        if *name == link_type { return Some((*inverse).to_string()); }
    }
    for custom in custom_types {
        if custom.name == link_type { return Some(custom.inverse.clone()); }
        if custom.inverse == link_type { return Some(custom.name.clone()); }
    }
    None
}
/// Check if a link type is valid (either built-in or custom)
pub fn is_valid_link_type(link_type: &str, custom_types: &[CustomLinkTypeDefinition]) -> bool {
    for (name, _) in BUILTIN_LINK_TYPES {
        if *name == link_type { return true; }
    }
    for custom in custom_types {
        if custom.name == link_type || custom.inverse == link_type { return true; }
    }
    false
}
