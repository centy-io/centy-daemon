use anyhow::{Context, Result};
use std::path::PathBuf;

use super::Config;

impl Config {
    /// Return the path to the config file (`~/.config/worktree/config.toml`).
    ///
    /// # Errors
    ///
    /// Returns an error if the home directory cannot be determined.
    pub fn path() -> Result<PathBuf> {
        let home = dirs::home_dir().context("Could not determine home directory")?;
        Ok(home.join(".config").join("worktree").join("config.toml"))
    }

    /// Load config from disk, returning `Default` if the file does not yet exist.
    ///
    /// # Errors
    ///
    /// Returns an error if the file cannot be read or parsed.
    pub fn load() -> Result<Self> {
        let path = Self::path()?;
        if !path.exists() {
            return Ok(Self::default());
        }
        // LLVM_COV_EXCL_START
        let content = std::fs::read_to_string(&path)
            .with_context(|| format!("Failed to read config from {}", path.display()))?;
        let config: Self = toml::from_str(&content)
            .with_context(|| format!("Failed to parse config at {}", path.display()))?;
        Ok(config)
        // LLVM_COV_EXCL_STOP
    }

    /// Persist the current config to disk.
    ///
    /// # Errors
    ///
    /// Returns an error if the config directory cannot be created or the file
    /// cannot be written.
    pub fn save(&self) -> Result<()> {
        // LLVM_COV_EXCL_LINE
        // LLVM_COV_EXCL_START
        let path = Self::path()?;
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)
                .with_context(|| format!("Failed to create config dir {}", parent.display()))?;
        }
        let content = self.to_toml_with_comments();
        std::fs::write(&path, content)
            .with_context(|| format!("Failed to write config to {}", path.display()))?;
        Ok(())
        // LLVM_COV_EXCL_STOP
    }
}
