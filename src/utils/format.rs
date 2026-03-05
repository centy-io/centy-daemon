pub const CENTY_HEADER_MD: &str =
    "<!-- This file is managed by Centy. Use the Centy CLI to modify it. -->";

pub const CENTY_HEADER_YAML: &str =
    "# This file is managed by Centy. Use the Centy CLI to modify it.";

/// Prepend the Centy-managed YAML header comment if not already present.
pub fn with_yaml_header(content: &str) -> String {
    if content.starts_with(CENTY_HEADER_YAML) {
        content.to_string()
    } else {
        format!("{CENTY_HEADER_YAML}\n{content}")
    }
}

/// Strip the Centy-managed markdown header comment from the start of content.
/// Used before passing file content to frontmatter parsers.
pub fn strip_centy_md_header(content: &str) -> &str {
    content
        .strip_prefix(CENTY_HEADER_MD)
        .and_then(|s| s.strip_prefix('\n'))
        .unwrap_or(content)
}

#[cfg(test)]
#[path = "format_tests.rs"]
mod tests;
