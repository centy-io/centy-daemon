use std::path::Path;

/// The commented-only scaffold written to `.worktree.toml` when the file is
/// absent. Every line is a comment so the file has no effect until edited.
pub const SCAFFOLD: &str = crate::templates::WORKTREE_TOML;

/// Write [`SCAFFOLD`] to `<worktree_path>/.worktree.toml` if the file does not
/// already exist.
///
/// Returns `true` when the file was created, `false` when it was already
/// present (no write is performed in that case).
///
/// # Errors
///
/// Returns an error if the file cannot be written.
pub fn scaffold_if_missing(worktree_path: &Path) -> std::io::Result<bool> {
    let path = worktree_path.join(".worktree.toml");
    if path.exists() {
        return Ok(false);
    }
    std::fs::write(path, SCAFFOLD)?;
    Ok(true)
}

#[cfg(test)]
#[path = "repo_hooks_scaffold_tests.rs"]
mod tests;
