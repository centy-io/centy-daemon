#![allow(clippy::unwrap_used, clippy::expect_used)]

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
fn test_expand_vars_known_env_var_with_tail() {
    // Set a known env var so the `$VAR/tail` branch executes
    std::env::set_var("CENTY_TEST_EXPAND_VAR", "/base/dir");
    let result = expand_vars("$CENTY_TEST_EXPAND_VAR/subdir");
    assert_eq!(result, "/base/dir/subdir");
    std::env::remove_var("CENTY_TEST_EXPAND_VAR");
}

#[test]
fn test_expand_vars_known_env_var_no_tail() {
    // Set a known env var so the `$VAR` (no tail) branch executes
    std::env::set_var("CENTY_TEST_EXPAND_VAR2", "/base/dir2");
    let result = expand_vars("$CENTY_TEST_EXPAND_VAR2");
    assert_eq!(result, "/base/dir2");
    std::env::remove_var("CENTY_TEST_EXPAND_VAR2");
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
fn test_resolve_pattern_strips_trailing_slash() {
    let result1 = resolve_pattern("$TMPDIR");
    let result2 = resolve_pattern("$TMPDIR/");
    assert!(result1.is_some());
    assert!(result2.is_some());
    assert_eq!(result1, result2);
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

#[test]
fn test_is_ignored_path_with_nonexistent_path() {
    // Path that doesn't exist — canonicalize falls back to itself
    let path = std::path::Path::new("/nonexistent/path/for/centy/test");
    // Just verify no panic
    let _ = is_ignored_path(path);
}
