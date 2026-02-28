use pulldown_cmark::{Options, Parser};
use pulldown_cmark_to_cmark::cmark;

pub const CENTY_HEADER_MD: &str =
    "<!-- This file is managed by Centy. Use the Centy CLI to modify it. -->";

pub const CENTY_HEADER_YAML: &str =
    "# This file is managed by Centy. Use the Centy CLI to modify it.";

/// Format markdown content using pulldown-cmark for consistent formatting.
/// Ensures the output ends with a trailing newline.
pub fn format_markdown(input: &str) -> String {
    let options = Options::all();
    let parser = Parser::new_ext(input, options);
    let mut output = String::new();
    let _ = cmark(parser, &mut output);
    if !output.ends_with('\n') {
        output.push('\n');
    }
    output
}

/// Format an issue file: normalize markdown and prepend the managed-by
/// header comment if not already present.
pub fn format_issue_file(content: &str) -> String {
    let formatted = format_markdown(content);
    if formatted.starts_with(CENTY_HEADER_MD) {
        formatted
    } else {
        format!("{CENTY_HEADER_MD}\n{formatted}")
    }
}

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
