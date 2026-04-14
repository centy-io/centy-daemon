use anyhow::Result;
use std::process::Command;

use crate::opener::augmented_path;

/// Template variables available to hook scripts.
pub struct HookContext {
    /// GitHub owner / organization name.
    pub owner: String,
    /// Repository name.
    pub repo: String,
    /// Issue number or Linear UUID as a string.
    pub issue: String,
    /// Git branch name for the worktree.
    pub branch: String,
    /// Absolute path to the worktree directory.
    pub worktree_path: String,
}

impl HookContext {
    /// Expand `{{owner}}`, `{{repo}}`, `{{issue}}`, `{{branch}}`, and
    /// `{{worktree_path}}` placeholders in `template`.
    #[must_use]
    pub fn render(&self, template: &str) -> String {
        template
            .replace("{{owner}}", &self.owner)
            .replace("{{repo}}", &self.repo)
            .replace("{{issue}}", &self.issue)
            .replace("{{branch}}", &self.branch)
            .replace("{{worktree_path}}", &self.worktree_path)
    }
}

/// Render `script` with `ctx`, write to a temp file, and execute it.
/// Stdout and stderr are forwarded to the caller's terminal.
/// A non-zero exit code prints a warning but does not return an error.
///
/// # Errors
///
/// Returns an error if the temp file cannot be written or its permissions
/// cannot be set.
pub fn run_hook(script: &str, ctx: &HookContext) -> Result<()> {
    let rendered = ctx.render(script);

    let ext = if cfg!(windows) { ".bat" } else { ".sh" };
    let tmp_path =
        std::env::temp_dir().join(format!("worktree-hook-{}{ext}", uuid::Uuid::new_v4()));
    std::fs::write(&tmp_path, rendered.as_bytes())?;

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&tmp_path, std::fs::Permissions::from_mode(0o755))?;
    }

    #[cfg(windows)]
    let result = Command::new("cmd")
        .args([std::ffi::OsStr::new("/C"), tmp_path.as_os_str()])
        .env("PATH", augmented_path())
        .status();
    #[cfg(not(windows))]
    let result = Command::new("sh")
        .arg(&tmp_path)
        .env("PATH", augmented_path())
        .status();
    let _ = std::fs::remove_file(&tmp_path);

    match result {
        Ok(status) if !status.success() => {
            eprintln!("Warning: hook exited with status {:?}", status.code());
        }
        // LLVM_COV_EXCL_START
        Err(e) => {
            eprintln!("Warning: failed to run hook: {e}");
        }
        // LLVM_COV_EXCL_STOP
        _ => {}
    }

    Ok(())
}

#[cfg(test)]
#[path = "hooks_tests.rs"]
mod tests;

#[cfg(test)]
#[path = "hooks_multiline_tests.rs"]
mod multiline_tests;
