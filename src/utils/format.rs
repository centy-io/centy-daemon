use pulldown_cmark::{Options, Parser};
use pulldown_cmark_to_cmark::cmark;

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

#[cfg(test)]
#[path = "format_tests.rs"]
mod tests;
