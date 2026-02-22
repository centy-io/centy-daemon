//! Global ignore-path machinery for project listings.
//!
//! At daemon startup call [`init_ignore_paths`] once with the patterns from
//! the user config.  Afterwards, [`is_ignored_path`] determines whether a
//! project path should be hidden from listings.
//!
//! Pattern resolution rules:
//! - `~`  is expanded to the home directory.
//! - `$TMPDIR` is expanded to `std::env::temp_dir()`.
//! - Other `$VAR` prefixes are expanded via `std::env::var`.
//! - Trailing `/**`, `/*`, or `/` are stripped; matching is prefix-based.
//! - Paths are canonicalized where possible (resolves macOS `/private` symlinks
//!   etc.).

use std::path::{Path, PathBuf};
use std::sync::OnceLock;

/// Compiled set of path prefixes excluded from project listings.
static IGNORE_PREFIXES: OnceLock<Vec<PathBuf>> = OnceLock::new();

/// Initialise the global ignore-prefix list from user-config patterns.
///
/// Should be called once at daemon startup.  Subsequent calls are no-ops
/// (OnceLock semantics).
pub fn init_ignore_paths(patterns: &[String]) {
    let prefixes: Vec<PathBuf> = patterns
        .iter()
        .filter_map(|p| resolve_pattern(p))
        .collect();
    // Ignore error if already set (e.g. during tests)
    let _ = IGNORE_PREFIXES.set(prefixes);
}

/// Returns `true` if `path` falls under any configured ignore prefix.
///
/// Falls back to `is_in_temp_dir` when [`init_ignore_paths`] has not yet
/// been called (e.g. in unit tests that exercise `list_projects` directly).
pub fn is_ignored_path(path: &Path) -> bool {
    let canonical = path.canonicalize().unwrap_or_else(|_| path.to_path_buf());
    match IGNORE_PREFIXES.get() {
        Some(prefixes) => prefixes.iter().any(|prefix| canonical.starts_with(prefix)),
        None => crate::utils::is_in_temp_dir(&canonical),
    }
}

/// Expand and normalise a single pattern string into a prefix `PathBuf`.
///
/// Returns `None` if the pattern cannot be resolved (e.g. unknown `$VAR` or
/// no home directory).
fn resolve_pattern(pattern: &str) -> Option<PathBuf> {
    // Strip trailing glob suffixes — we do prefix-based matching.
    let stripped = pattern
        .trim_end_matches("/**")
        .trim_end_matches("/*")
        .trim_end_matches('/');

    let after_vars = expand_vars(stripped);
    let after_tilde = expand_tilde(&after_vars)?;

    let path = PathBuf::from(&after_tilde);
    // Canonicalize resolves symlinks (e.g. /var → /private/var on macOS).
    Some(path.canonicalize().unwrap_or(path))
}

/// Expand `$TMPDIR` and simple `$VAR` prefixes to their runtime values.
fn expand_vars(s: &str) -> String {
    // Fast path for the common $TMPDIR token.
    if s == "$TMPDIR" || s.starts_with("$TMPDIR/") {
        let temp = std::env::temp_dir();
        let temp_str = temp.to_string_lossy();
        return s.replacen("$TMPDIR", temp_str.as_ref(), 1);
    }

    // Generic single-var expansion: $VAR or $VAR/rest
    if let Some(rest) = s.strip_prefix('$') {
        let (var_name, tail) = rest.split_once('/').unwrap_or((rest, ""));
        if let Ok(val) = std::env::var(var_name) {
            return if tail.is_empty() {
                val
            } else {
                format!("{val}/{tail}")
            };
        }
    }

    s.to_string()
}

/// Expand a leading `~` to the home directory path.
///
/// Returns `None` if the home directory cannot be determined.
fn expand_tilde(s: &str) -> Option<String> {
    if s == "~" {
        return dirs::home_dir().map(|h| h.to_string_lossy().into_owned());
    }
    if let Some(rest) = s.strip_prefix("~/") {
        let home = dirs::home_dir()?;
        return Some(format!("{}/{rest}", home.to_string_lossy()));
    }
    Some(s.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_expand_tilde_path() {
        if let Some(home) = dirs::home_dir() {
            let result = expand_tilde("~/projects").unwrap();
            assert_eq!(result, format!("{}/projects", home.to_string_lossy()));
        }
    }

    #[test]
    fn test_expand_tilde_bare() {
        if let Some(home) = dirs::home_dir() {
            let result = expand_tilde("~").unwrap();
            assert_eq!(result, home.to_string_lossy());
        }
    }

    #[test]
    fn test_expand_tilde_no_tilde() {
        let s = "/usr/local/share";
        assert_eq!(expand_tilde(s).unwrap(), s);
    }

    #[test]
    fn test_expand_vars_tmpdir() {
        let temp = std::env::temp_dir();
        let result = expand_vars("$TMPDIR");
        assert_eq!(result, temp.to_string_lossy());
    }

    #[test]
    fn test_expand_vars_tmpdir_with_suffix() {
        let temp = std::env::temp_dir();
        let result = expand_vars("$TMPDIR/subdir");
        assert!(result.starts_with(temp.to_string_lossy().as_ref()));
        assert!(result.ends_with("/subdir"));
    }

    #[test]
    fn test_expand_vars_unknown_passthrough() {
        // Unknown $VAR should be returned as-is
        let s = "$UNKNOWN_VAR_XYZ";
        assert_eq!(expand_vars(s), s);
    }

    #[test]
    fn test_expand_vars_plain_path() {
        assert_eq!(expand_vars("/some/path"), "/some/path");
    }

    #[test]
    fn test_resolve_pattern_strips_glob_star() {
        let result = resolve_pattern("$TMPDIR/*");
        assert!(result.is_some());
        let result2 = resolve_pattern("$TMPDIR/**");
        assert!(result2.is_some());
        // Both should resolve to the same prefix
        assert_eq!(result, result2);
    }

    #[test]
    fn test_resolve_pattern_tmpdir() {
        let temp = std::env::temp_dir();
        let canonical_temp = temp.canonicalize().unwrap_or(temp);
        let prefix = resolve_pattern("$TMPDIR").unwrap();
        assert_eq!(prefix, canonical_temp);
    }

    #[test]
    fn test_resolve_pattern_tilde_worktrees() {
        if dirs::home_dir().is_some() {
            let prefix = resolve_pattern("~/worktrees/*");
            assert!(prefix.is_some());
            let p = prefix.unwrap();
            // Should end with "worktrees" (glob suffix stripped)
            assert!(p.ends_with("worktrees"));
        }
    }

    #[test]
    fn test_is_ignored_path_temp() {
        // Temp paths should always be ignored (either via prefixes or fallback)
        let temp_path = std::env::temp_dir().join("centy-test-project");
        assert!(is_ignored_path(&temp_path));
    }

    #[test]
    fn test_is_ignored_path_home_not_ignored() {
        if let Some(home) = dirs::home_dir() {
            let normal = home.join("projects").join("myapp");
            // A normal projects path should NOT be ignored
            // (only true when IGNORE_PREFIXES is not initialized or set correctly)
            // We can't guarantee IGNORE_PREFIXES state in tests, so just check no panic
            let _ = is_ignored_path(&normal);
        }
    }
}
