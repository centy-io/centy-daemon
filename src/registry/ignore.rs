//! Global ignore-path machinery for project listings.
//!
//! At daemon startup call [`init_ignore_paths`] once with the patterns from
//! the user config. Afterwards, [`is_ignored_path`] determines whether a
//! project path should be hidden from listings.
//!
//! Pattern resolution: `~` expands to home dir, `$TMPDIR`/`$VAR` expand to env vars,
//! trailing `/**`, `/*`, or `/` are stripped, paths are canonicalized.

use std::path::{Path, PathBuf};
use std::sync::OnceLock;

static IGNORE_PREFIXES: OnceLock<Vec<PathBuf>> = OnceLock::new();

/// Initialise the global ignore-prefix list from user-config patterns. Call once at daemon startup.
pub fn init_ignore_paths(patterns: &[String]) {
    let prefixes: Vec<PathBuf> = patterns.iter().filter_map(|p| resolve_pattern(p)).collect();
    let _ = IGNORE_PREFIXES.set(prefixes);
}

/// Returns `true` if `path` falls under any configured ignore prefix.
/// Falls back to `is_in_temp_dir` when [`init_ignore_paths`] has not yet been called.
pub fn is_ignored_path(path: &Path) -> bool {
    let canonical = path.canonicalize().unwrap_or_else(|_| path.to_path_buf());
    match IGNORE_PREFIXES.get() {
        Some(prefixes) => prefixes.iter().any(|prefix| canonical.starts_with(prefix)),
        None => crate::utils::is_in_temp_dir(&canonical),
    }
}

/// Expand and normalise a single pattern string into a prefix `PathBuf`.
fn resolve_pattern(pattern: &str) -> Option<PathBuf> {
    let stripped = pattern.trim_end_matches("/**").trim_end_matches("/*").trim_end_matches('/');
    let after_vars = expand_vars(stripped);
    let after_tilde = expand_tilde(&after_vars)?;
    let path = PathBuf::from(&after_tilde);
    Some(path.canonicalize().unwrap_or(path))
}

/// Expand `$TMPDIR` and simple `$VAR` prefixes to their runtime values.
fn expand_vars(s: &str) -> String {
    if s == "$TMPDIR" || s.starts_with("$TMPDIR/") {
        let temp = std::env::temp_dir();
        return s.replacen("$TMPDIR", temp.to_string_lossy().as_ref(), 1);
    }
    if let Some(rest) = s.strip_prefix('$') {
        let (var_name, tail) = rest.split_once('/').unwrap_or((rest, ""));
        if let Ok(val) = std::env::var(var_name) {
            return if tail.is_empty() { val } else { format!("{val}/{tail}") };
        }
    }
    s.to_string()
}

/// Expand a leading `~` to the home directory path.
fn expand_tilde(s: &str) -> Option<String> {
    if s == "~" { return dirs::home_dir().map(|h| h.to_string_lossy().into_owned()); }
    if let Some(rest) = s.strip_prefix("~/") {
        let home = dirs::home_dir()?;
        return Some(format!("{}/{rest}", home.to_string_lossy()));
    }
    Some(s.to_string())
}

#[cfg(test)]
#[path = "ignore_tests.rs"]
mod tests;
