#![allow(clippy::all, clippy::pedantic, clippy::restriction)]
/// Split `lines` into chunks.  A new chunk is flushed whenever brace depth
/// has returned to 0 and one of two conditions holds:
///
///   A) The current line is exactly `}` (closing a top-level block item) and
///      the chunk has accumulated at least `max_size` lines.
///
///   B) The current line ends with `;` at depth 0 (a simple item: `use`,
///      `type`, `const`, etc.).  These are always flushed immediately so that
///      they never end up in the same chunk as a following large block item.
pub fn split_into_chunks(lines: &[&str], max_size: usize) -> Vec<Vec<String>> {
    let mut chunks: Vec<Vec<String>> = Vec::new();
    let mut current_chunk: Vec<String> = Vec::new();
    let mut brace_depth: i32 = 0;
    for line in lines {
        current_chunk.push((*line).to_string());
        for ch in line.chars() {
            match ch {
                '{' => brace_depth += 1,
                '}' => brace_depth -= 1,
                _ => {}
            }
        }
        if brace_depth == 0 {
            let trimmed = line.trim();
            let flush = (trimmed == "}" && current_chunk.len() >= max_size)
                || trimmed.ends_with(';');
            if flush {
                chunks.push(current_chunk.clone());
                current_chunk = Vec::new();
            }
        }
    }
    if !current_chunk.is_empty() {
        chunks.push(current_chunk);
    }
    chunks
}
