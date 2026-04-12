#![allow(clippy::all, clippy::pedantic, clippy::restriction)]
/// Collect leading `#![...]` inner-attribute blocks from `lines`, convert them
/// to outer `#[...]` attributes, and return both the converted attributes and
/// the index of the first non-attribute, non-blank line.
///
/// Multi-line attribute blocks (where `#![` opens on one line and `)]` closes
/// on a later line) are handled correctly.
pub fn strip_leading_inner_attrs(lines: &[&str]) -> (Vec<String>, usize) {
    let mut outer_attrs: Vec<String> = Vec::new();
    let mut body_start = 0;
    let mut inside_attr = false;
    let mut attr_buf: Vec<String> = Vec::new();
    for line in lines {
        let trimmed = line.trim();
        if inside_attr {
            attr_buf.push((*line).to_string());
            body_start += 1;
            if trimmed.ends_with(")]") || trimmed == "]" {
                let joined = attr_buf.join("\n");
                outer_attrs.push(joined.replacen("#![", "#[", 1));
                attr_buf.clear();
                inside_attr = false;
            }
        } else if trimmed.is_empty() {
            body_start += 1;
        } else if trimmed.starts_with("#![") {
            inside_attr = true;
            attr_buf.push((*line).to_string());
            body_start += 1;
            if trimmed.ends_with(']') {
                outer_attrs.push(trimmed.replacen("#![", "#[", 1));
                attr_buf.clear();
                inside_attr = false;
            }
        } else {
            break;
        }
    }
    (outer_attrs, body_start)
}
