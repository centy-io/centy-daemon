//! gwq CLI client wrapper.
//!
//! This module provides a Rust wrapper for the gwq CLI tool, which manages
//! git worktrees. gwq is bundled with centy and located relative to the
//! centy executable.
//!
//! See: <https://github.com/d-kuro/gwq>

use serde::Deserialize;
use std::path::{Path, PathBuf};
use std::process::Command;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum GwqError {
    #[error("gwq binary not found at expected location")]
    BinaryNotFound,

    #[error("Failed to execute gwq command: {0}")]
    CommandError(String),

    #[error("gwq command failed: {0}")]
    #[allow(dead_code)] // Used by list_worktrees
    ExecutionError(String),

    #[error("Failed to parse gwq output: {0}")]
    #[allow(dead_code)] // Used by list_worktrees
    ParseError(String),

    #[error("Not a git repository")]
    NotGitRepository,

    #[error("Worktree error: {0}")]
    WorktreeError(String),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
}

/// Represents a worktree as returned by gwq list --json
#[derive(Debug, Clone, Deserialize)]
#[allow(dead_code)] // Part of public API
pub struct GwqWorktree {
    /// Absolute path to the worktree
    pub path: PathBuf,

    /// Branch name (empty for detached HEAD)
    #[serde(default)]
    pub branch: String,

    /// HEAD commit SHA
    #[serde(default)]
    pub head: String,

    /// Whether this is the main worktree
    #[serde(default, rename = "isMain")]
    pub is_main: bool,

    /// Whether the worktree is bare
    #[serde(default, rename = "isBare")]
    pub is_bare: bool,

    /// Repository URL (for gwq-managed worktrees)
    #[serde(default)]
    pub url: String,
}

/// Client for interacting with the gwq CLI tool
pub struct GwqClient {
    gwq_path: PathBuf,
}

impl GwqClient {
    /// Create a new gwq client.
    ///
    /// Attempts to find the gwq binary in the following locations:
    /// 1. Bundled with centy (relative to executable)
    /// 2. In the system PATH
    pub fn new() -> Result<Self, GwqError> {
        // First, try to find bundled gwq relative to centy executable
        if let Ok(exe_path) = std::env::current_exe() {
            if let Some(exe_dir) = exe_path.parent() {
                let bundled_gwq = exe_dir.join("gwq");
                if bundled_gwq.exists() {
                    return Ok(Self {
                        gwq_path: bundled_gwq,
                    });
                }

                // Also check in bin subdirectory
                let bin_gwq = exe_dir.join("bin").join("gwq");
                if bin_gwq.exists() {
                    return Ok(Self { gwq_path: bin_gwq });
                }
            }
        }

        // Fall back to system PATH
        if let Ok(path) = which::which("gwq") {
            return Ok(Self { gwq_path: path });
        }

        Err(GwqError::BinaryNotFound)
    }

    /// Create a new gwq client with a specific gwq binary path.
    ///
    /// This is useful for testing or when the gwq binary is in a non-standard location.
    #[must_use]
    #[allow(dead_code)] // Part of public API
    pub fn with_path(gwq_path: PathBuf) -> Self {
        Self { gwq_path }
    }

    /// Check if gwq is available.
    #[allow(dead_code)] // Part of public API
    pub fn is_available() -> bool {
        Self::new().is_ok()
    }

    /// Get the path to the gwq binary.
    #[must_use]
    #[allow(dead_code)] // Part of public API
    pub fn gwq_path(&self) -> &Path {
        &self.gwq_path
    }

    /// Create a git worktree at the specified path.
    ///
    /// This uses `git worktree add` directly since gwq's add command creates
    /// worktrees in its own managed directory structure. We want to keep
    /// centy's temp directory pattern.
    ///
    /// # Arguments
    /// * `repo_path` - Path to the source git repository
    /// * `worktree_path` - Path where the worktree will be created
    /// * `git_ref` - Git ref to check out (usually "HEAD")
    pub fn add_worktree_at_path(
        &self,
        repo_path: &Path,
        worktree_path: &Path,
        git_ref: &str,
    ) -> Result<(), GwqError> {
        // Verify source is a git repo
        if !self.is_git_repository(repo_path) {
            return Err(GwqError::NotGitRepository);
        }

        // Use git worktree add directly to create at specific path
        let output = Command::new("git")
            .args([
                "worktree",
                "add",
                "--detach",
                &worktree_path.to_string_lossy(),
                git_ref,
            ])
            .current_dir(repo_path)
            .env_remove("GIT_DIR")
            .env_remove("GIT_WORK_TREE")
            .output()
            .map_err(|e| GwqError::CommandError(e.to_string()))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(GwqError::WorktreeError(stderr.to_string()));
        }

        Ok(())
    }

    /// Remove a git worktree.
    ///
    /// # Arguments
    /// * `worktree_path` - Path to the worktree to remove
    /// * `force` - If true, force removal even with uncommitted changes
    pub fn remove_worktree(&self, worktree_path: &Path, force: bool) -> Result<(), GwqError> {
        let mut args = vec!["remove".to_string()];
        if force {
            args.push("--force".to_string());
        }
        args.push(worktree_path.to_string_lossy().to_string());

        let output = self.run_gwq(&args, None)?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            // If worktree doesn't exist, that's okay for cleanup purposes
            if !stderr.contains("is not a working tree")
                && !stderr.contains("is not a worktree")
                && !stderr.contains("does not exist")
            {
                return Err(GwqError::WorktreeError(stderr.to_string()));
            }
        }

        Ok(())
    }

    /// Remove a git worktree using git directly.
    ///
    /// This is used when we know the source repository path.
    ///
    /// # Arguments
    /// * `repo_path` - Path to the source git repository
    /// * `worktree_path` - Path to the worktree to remove
    /// * `force` - If true, force removal even with uncommitted changes
    pub fn remove_worktree_from_repo(
        &self,
        repo_path: &Path,
        worktree_path: &Path,
        force: bool,
    ) -> Result<(), GwqError> {
        let worktree_str = worktree_path.to_string_lossy().to_string();
        let mut args = vec!["worktree", "remove"];
        if force {
            args.push("--force");
        }
        args.push(&worktree_str);

        let output = Command::new("git")
            .args(&args)
            .current_dir(repo_path)
            .env_remove("GIT_DIR")
            .env_remove("GIT_WORK_TREE")
            .output()
            .map_err(|e| GwqError::CommandError(e.to_string()))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            // If worktree doesn't exist, that's okay for cleanup purposes
            if !stderr.contains("is not a working tree") {
                return Err(GwqError::WorktreeError(stderr.to_string()));
            }
        }

        Ok(())
    }

    /// List worktrees using gwq.
    ///
    /// # Arguments
    /// * `repo_path` - Optional path to a specific repository. If None, lists global worktrees.
    /// * `global` - If true, list all worktrees across all repositories
    #[allow(dead_code)] // Part of public API
    pub fn list_worktrees(
        &self,
        repo_path: Option<&Path>,
        global: bool,
    ) -> Result<Vec<GwqWorktree>, GwqError> {
        let mut args = vec!["list".to_string(), "--json".to_string()];
        if global {
            args.push("-g".to_string());
        }

        let output = self.run_gwq(&args, repo_path)?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(GwqError::ExecutionError(stderr.to_string()));
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        if stdout.trim().is_empty() {
            return Ok(Vec::new());
        }

        // gwq list --json returns an array of worktrees
        serde_json::from_str(&stdout).map_err(|e| GwqError::ParseError(e.to_string()))
    }

    /// List worktrees using git directly.
    ///
    /// This is useful when gwq is not available or for getting worktrees
    /// from a specific repository.
    #[allow(dead_code)] // Part of public API
    pub fn list_worktrees_git(&self, repo_path: &Path) -> Result<Vec<GwqWorktree>, GwqError> {
        let output = Command::new("git")
            .args(["worktree", "list", "--porcelain"])
            .current_dir(repo_path)
            .env_remove("GIT_DIR")
            .env_remove("GIT_WORK_TREE")
            .output()
            .map_err(|e| GwqError::CommandError(e.to_string()))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(GwqError::ExecutionError(stderr.to_string()));
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        Ok(parse_git_worktree_list(&stdout))
    }

    /// Prune stale worktree references.
    ///
    /// Call this after manually deleting worktree directories to clean up
    /// git's internal tracking.
    pub fn prune(&self, repo_path: &Path) -> Result<(), GwqError> {
        let output = Command::new("git")
            .args(["worktree", "prune"])
            .current_dir(repo_path)
            .env_remove("GIT_DIR")
            .env_remove("GIT_WORK_TREE")
            .output()
            .map_err(|e| GwqError::CommandError(e.to_string()))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(GwqError::WorktreeError(stderr.to_string()));
        }

        Ok(())
    }

    /// Check if a path is a git worktree.
    pub fn is_worktree(&self, path: &Path) -> bool {
        // Check for .git file (worktrees have a .git file, not directory)
        let git_path = path.join(".git");
        if git_path.is_file() {
            return true;
        }

        // Also check if it's a regular git repo (main worktree)
        if git_path.is_dir() {
            return true;
        }

        false
    }

    /// Check if a path exists as a worktree in the given repository.
    #[allow(dead_code)] // Part of public API
    pub fn worktree_exists(&self, repo_path: &Path, worktree_path: &Path) -> bool {
        match self.list_worktrees_git(repo_path) {
            Ok(worktrees) => worktrees.iter().any(|wt| wt.path == worktree_path),
            Err(_) => false,
        }
    }

    /// Check if a path is a git repository.
    fn is_git_repository(&self, path: &Path) -> bool {
        Command::new("git")
            .args(["rev-parse", "--git-dir"])
            .current_dir(path)
            .env_remove("GIT_DIR")
            .env_remove("GIT_WORK_TREE")
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false)
    }

    /// Run a gwq command with the given arguments.
    fn run_gwq(
        &self,
        args: &[String],
        working_dir: Option<&Path>,
    ) -> Result<std::process::Output, GwqError> {
        let mut cmd = Command::new(&self.gwq_path);
        cmd.args(args);

        if let Some(dir) = working_dir {
            cmd.current_dir(dir);
        }

        // Clear git environment variables
        cmd.env_remove("GIT_DIR");
        cmd.env_remove("GIT_WORK_TREE");

        cmd.output()
            .map_err(|e| GwqError::CommandError(e.to_string()))
    }
}

/// Parse git worktree list --porcelain output into `GwqWorktree` structs.
#[allow(dead_code)] // Used by list_worktrees_git
fn parse_git_worktree_list(output: &str) -> Vec<GwqWorktree> {
    let mut worktrees = Vec::new();
    let mut current_path: Option<PathBuf> = None;
    let mut current_head = String::new();
    let mut current_branch = String::new();
    let mut is_bare = false;

    for line in output.lines() {
        if let Some(path_str) = line.strip_prefix("worktree ") {
            // Save previous worktree if exists
            if let Some(path) = current_path.take() {
                worktrees.push(GwqWorktree {
                    path,
                    branch: std::mem::take(&mut current_branch),
                    head: std::mem::take(&mut current_head),
                    is_main: worktrees.is_empty(), // First worktree is main
                    is_bare,
                    url: String::new(),
                });
                is_bare = false;
            }
            current_path = Some(PathBuf::from(path_str));
        } else if let Some(head) = line.strip_prefix("HEAD ") {
            current_head = head.to_string();
        } else if let Some(branch) = line.strip_prefix("branch refs/heads/") {
            current_branch = branch.to_string();
        } else if line == "bare" {
            is_bare = true;
        } else if line == "detached" {
            // Detached HEAD, no branch name
            current_branch = String::new();
        }
    }

    // Don't forget the last worktree
    if let Some(path) = current_path {
        worktrees.push(GwqWorktree {
            path,
            branch: current_branch,
            head: current_head,
            is_main: worktrees.is_empty(),
            is_bare,
            url: String::new(),
        });
    }

    worktrees
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_git_worktree_list_empty() {
        let output = "";
        let worktrees = parse_git_worktree_list(output);
        assert!(worktrees.is_empty());
    }

    #[test]
    fn test_parse_git_worktree_list_single() {
        let output = "worktree /path/to/repo\nHEAD abc123\nbranch refs/heads/main\n";
        let worktrees = parse_git_worktree_list(output);
        assert_eq!(worktrees.len(), 1);
        assert_eq!(worktrees[0].path, PathBuf::from("/path/to/repo"));
        assert_eq!(worktrees[0].branch, "main");
        assert_eq!(worktrees[0].head, "abc123");
        assert!(worktrees[0].is_main);
    }

    #[test]
    fn test_parse_git_worktree_list_multiple() {
        let output = "worktree /main/repo\nHEAD abc123\nbranch refs/heads/main\n\nworktree /tmp/workspace\nHEAD def456\ndetached\n";
        let worktrees = parse_git_worktree_list(output);
        assert_eq!(worktrees.len(), 2);

        assert_eq!(worktrees[0].path, PathBuf::from("/main/repo"));
        assert_eq!(worktrees[0].branch, "main");
        assert!(worktrees[0].is_main);

        assert_eq!(worktrees[1].path, PathBuf::from("/tmp/workspace"));
        assert!(worktrees[1].branch.is_empty()); // Detached HEAD
        assert!(!worktrees[1].is_main);
    }

    #[test]
    fn test_gwq_client_new_finds_system_gwq() {
        // This test will pass if gwq is installed system-wide
        // Otherwise it will return BinaryNotFound which is expected
        let result = GwqClient::new();
        match result {
            Ok(client) => assert!(client.gwq_path().exists()),
            Err(GwqError::BinaryNotFound) => {
                // Expected if gwq is not installed
            }
            Err(e) => panic!("Unexpected error: {e}"),
        }
    }

    #[test]
    fn test_gwq_client_with_path() {
        let path = PathBuf::from("/test/gwq");
        let client = GwqClient::with_path(path.clone());
        assert_eq!(client.gwq_path(), &path);
    }
}
