use std::path::{Path, PathBuf};
use std::time::SystemTime;

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

/// A workspace entry stored in the registry.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkspaceRecord {
    /// Absolute path to the worktree directory.
    pub path: PathBuf,
    /// When this workspace was first created.
    #[serde(with = "humantime_serde")]
    pub created_at: SystemTime,
}

/// Persistent registry of all known workspaces and their creation timestamps.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct WorkspaceRegistry {
    /// Registered workspaces.
    #[serde(rename = "workspace")]
    pub workspace: Vec<WorkspaceRecord>,
}

impl WorkspaceRegistry {
    /// Return the path to the workspace registry file
    /// (`~/.config/worktree/workspaces.toml`).
    ///
    /// # Errors
    ///
    /// Returns an error if the home directory cannot be determined.
    pub fn path() -> Result<PathBuf> {
        let home = dirs::home_dir().context("Could not determine home directory")?;
        Ok(home
            .join(".config")
            .join("worktree")
            .join("workspaces.toml"))
    }

    fn load_from(path: &Path) -> Result<Self> {
        if !path.exists() {
            return Ok(Self::default());
        }
        let content = std::fs::read_to_string(path)
            .context(format!("Failed to read registry from {}", path.display()))?;
        toml::from_str(&content).context(format!("Failed to parse registry at {}", path.display()))
    }

    fn write_to(&self, path: &Path) -> Result<()> {
        let parent = path.parent().context("registry path has no parent")?;
        std::fs::create_dir_all(parent)
            .context(format!("Failed to create config dir {}", parent.display()))?;
        let content = toml::to_string(self).context("Failed to serialize workspace registry")?;
        std::fs::write(path, content)
            .context(format!("Failed to write registry to {}", path.display()))
    }

    /// Load the registry from disk; returns an empty registry when absent.
    ///
    /// # Errors
    ///
    /// Returns an error if the file cannot be read or parsed.
    pub fn load() -> Result<Self> {
        Self::load_from(&Self::path()?)
    }

    /// Persist the registry to disk.
    ///
    /// # Errors
    ///
    /// Returns an error if the directory cannot be created or the file cannot
    /// be written.
    pub fn save(&self) -> Result<()> {
        self.write_to(&Self::path()?)
    }

    /// Register a workspace path with the current timestamp.
    ///
    /// Idempotent: if `path` is already present, the existing entry is left
    /// unchanged.
    pub fn register(&mut self, path: PathBuf) {
        if !self.workspace.iter().any(|r| r.path == path) {
            self.workspace.push(WorkspaceRecord {
                path,
                created_at: SystemTime::now(),
            });
        }
    }
}

#[cfg(test)]
#[path = "registry_tests.rs"]
mod registry_tests;
