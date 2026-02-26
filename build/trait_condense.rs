#![allow(clippy::all, clippy::pedantic, clippy::restriction)]
/// Condense a `pub trait` block to fit within 99 lines by:
///   1. Removing doc comment lines from the trait body.
///   2. Joining each multi-line `async fn` signature into a single line.
///
/// The trait header (attributes, `pub trait NAME...`) and closing `}` are
/// preserved as-is.  Only the method signatures inside the body are affected.
pub fn condense_trait_block(lines: &[String]) -> Vec<String> {
    let refs: Vec<&str> = lines.iter().map(String::as_str).collect();

    // Find the opening `{` of the trait body.
    let open_idx = match refs.iter().position(|l| l.trim().ends_with('{')) {
        Some(i) => i,
        None => return lines.to_vec(),
    };

    let prefix: Vec<String> = refs[..=open_idx].iter().map(|l| (*l).to_string()).collect();

    // Find the closing `}` (last line).
    let close_idx = refs.len().saturating_sub(1);
    let suffix = refs[close_idx].to_string();

    // Process the body lines (between open and close).
    let body = &refs[open_idx + 1..close_idx];
    let mut result: Vec<String> = prefix;
    let mut method_buf: Vec<String> = Vec::new();
    let mut in_method = false;

    for line in body {
        let trimmed = line.trim();
        if trimmed.starts_with("///") || trimmed.starts_with("//") {
            continue; // drop doc comments
        }
        if !in_method && trimmed.starts_with("async fn ") {
            in_method = true;
            method_buf.push(trimmed.to_string());
            if trimmed.ends_with(';') {
                result.push(format!("    {}", method_buf.join(" ")));
                method_buf.clear();
                in_method = false;
            }
        } else if in_method {
            method_buf.push(trimmed.to_string());
            if trimmed.ends_with(';') {
                result.push(format!("    {}", method_buf.join(" ")));
                method_buf.clear();
                in_method = false;
            }
        } else {
            result.push((*line).to_string());
        }
    }

    result.push(suffix);
    result
}
