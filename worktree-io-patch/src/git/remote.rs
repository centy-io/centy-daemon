use anyhow::{bail, Context, Result};

/// Get the URL of a named remote from a git repository.
///
/// # Errors
///
/// Returns an error if `git remote get-url` fails or the output is not valid UTF-8.
pub fn get_remote_url(repo: &std::path::Path, remote: &str) -> Result<String> {
    let output = super::git_cmd()
        .args(["-C"])
        .arg(repo)
        .args(["remote", "get-url", remote])
        .output()
        .context("Failed to run `git remote get-url`")?; // LLVM_COV_EXCL_LINE

    if !output.status.success() {
        bail!(
            "git remote get-url {remote} failed: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        );
    }

    Ok(String::from_utf8(output.stdout)
        .context("Remote URL is not valid UTF-8")? // LLVM_COV_EXCL_LINE
        .trim()
        .to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::process::Command;

    fn git(dir: &std::path::Path, args: &[&str]) {
        Command::new("git")
            .args(["-C"])
            .arg(dir)
            .args(args)
            .env_remove("GIT_DIR")
            .status()
            .unwrap();
    }

    #[test]
    fn get_remote_url_success() {
        let dir = tempfile::tempdir().unwrap();
        git(dir.path(), &["init"]);
        git(
            dir.path(),
            &["remote", "add", "origin", "https://github.com/acme/api.git"],
        );
        let url = get_remote_url(dir.path(), "origin").unwrap();
        assert_eq!(url, "https://github.com/acme/api.git");
    }

    #[test]
    fn get_remote_url_no_remote() {
        let dir = tempfile::tempdir().unwrap();
        git(dir.path(), &["init"]);
        let err = get_remote_url(dir.path(), "origin").unwrap_err();
        assert!(err.to_string().contains("git remote get-url origin failed"));
    }
}
