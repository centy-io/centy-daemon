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

/// Format an issue file: normalize markdown and insert the managed-by comment
/// as the first line inside the YAML frontmatter block if not already present.
/// This keeps `---` at position 0, allowing frontmatter parsers to work normally.
///
/// Note: pulldown-cmark-to-cmark escapes `#` to `\#` inside YAML metadata
/// blocks.  We detect both the plain and escaped forms of the header and
/// normalise back to the plain form so the function is idempotent.
pub fn format_issue_file(content: &str) -> String {
    let formatted = format_markdown(content);
    if let Some(after_dashes) = formatted.strip_prefix("---\n") {
        // pulldown-cmark-to-cmark escapes '#' inside YAML blocks → "\# ..."
        let escaped = format!("\\{CENTY_HEADER_YAML}");
        if after_dashes.starts_with(CENTY_HEADER_YAML) {
            // Already has the comment in plain form – nothing to do.
            formatted
        } else if after_dashes.starts_with(escaped.as_str()) {
            // Comment is present but was escaped by format_markdown. Un-escape it.
            format!(
                "---\n{CENTY_HEADER_YAML}\n{}",
                &after_dashes[escaped.len()..]
            )
        } else {
            // Comment not present – insert it.
            format!("---\n{CENTY_HEADER_YAML}\n{after_dashes}")
        }
    } else {
        formatted
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
