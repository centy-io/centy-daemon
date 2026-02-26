#![allow(clippy::all, clippy::pedantic, clippy::restriction)]
use super::service_extract_helpers::{build_condensed, extract_rpc_arms};
use std::path::Path;

/// Transform an oversized `impl<T, B> Service<...>` block.
/// Writes helper files for each match arm.
/// Returns (condensed_impl_lines, include_lines).
pub fn transform_service_impl(
    out_dir: &Path,
    stem: &str,
    lines: &[String],
) -> Result<(Vec<String>, Vec<String>), Box<dyn std::error::Error>> {
    let refs: Vec<&str> = lines.iter().map(String::as_str).collect();
    let call_s = match refs.iter().position(|l| l.trim().starts_with("fn call(")) {
        Some(i) => i,
        None => return Ok((lines.to_vec(), vec![])),
    };
    let match_s = match refs[call_s..]
        .iter()
        .position(|l| l.trim().starts_with("match req"))
    {
        Some(i) => call_s + i,
        None => return Ok((lines.to_vec(), vec![])),
    };
    let (arms, includes, def) = extract_rpc_arms(out_dir, stem, &refs, match_s)?;
    Ok((
        build_condensed(&refs, call_s, match_s, &arms, &def),
        includes,
    ))
}
