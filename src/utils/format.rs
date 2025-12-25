use pulldown_cmark::{Options, Parser};
use pulldown_cmark_to_cmark::cmark;

/// Format markdown content using pulldown-cmark for consistent formatting.
/// Ensures the output ends with a trailing newline.
pub fn format_markdown(input: &str) -> String {
    let options = Options::all();
    let parser = Parser::new_ext(input, options);
    let mut output = String::new();
    cmark(parser, &mut output).expect("markdown formatting should not fail");
    // Ensure trailing newline
    if !output.ends_with('\n') {
        output.push('\n');
    }
    output
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_markdown_basic() {
        let input = "# Hello\n\nWorld";
        let output = format_markdown(input);
        assert!(output.ends_with('\n'));
        assert!(output.contains("# Hello"));
    }

    #[test]
    fn test_format_markdown_trailing_newline() {
        let input = "# Test";
        let output = format_markdown(input);
        assert!(output.ends_with('\n'));
    }

    #[test]
    fn test_format_markdown_already_has_newline() {
        let input = "# Test\n";
        let output = format_markdown(input);
        assert!(output.ends_with('\n'));
        // Should not have double newline
        assert!(!output.ends_with("\n\n"));
    }
}
