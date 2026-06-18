use anyhow::{Context, Result};

use super::Config;

impl Config {
    /// Get a config value by dot-separated key path.
    ///
    /// # Errors
    ///
    /// Returns an error if `key` is not a recognised config key.
    pub fn get_value(&self, key: &str) -> Result<String> {
        match key {
            "editor" | "editor.command" => Ok(self.editor.command.clone().unwrap_or_default()),
            "editor.background" => Ok(self.editor.background.to_string()),
            "open.editor" => Ok(self.open.editor.to_string()),
            "workspace.ttl" => Ok(self
                .workspace
                .ttl
                .map_or_else(String::new, |t| t.to_string())),
            "workspace.auto_prune" => Ok(self.workspace.auto_prune.to_string()),
            "workspace.temp" => Ok(self.workspace.temp.to_string()),
            _ => anyhow::bail!("Unknown config key: {key}"),
        }
    }

    /// Set a config value by dot-separated key path.
    ///
    /// # Errors
    ///
    /// Returns an error if `key` is not a recognised config key or if the
    /// value cannot be parsed (e.g. a non-boolean for `open.editor`).
    pub fn set_value(&mut self, key: &str, value: &str) -> Result<()> {
        match key {
            "editor" | "editor.command" => {
                self.editor.command = (!value.is_empty()).then(|| value.to_string());
            }
            "editor.background" => {
                self.editor.background = value
                    .parse::<bool>()
                    .with_context(|| format!("Invalid boolean value: {value}"))?;
            }
            "open.editor" => {
                self.open.editor = value
                    .parse::<bool>()
                    .with_context(|| format!("Invalid boolean value: {value}"))?;
            }
            "workspace.ttl" => {
                self.workspace.ttl = (!value.is_empty())
                    .then(|| {
                        value
                            .parse()
                            .map_err(|e| anyhow::anyhow!("Invalid duration {value:?}: {e}"))
                    })
                    .transpose()?;
            }
            "workspace.auto_prune" => {
                self.workspace.auto_prune = value
                    .parse::<bool>()
                    .with_context(|| format!("Invalid boolean value: {value}"))?;
            }
            "workspace.temp" => {
                self.workspace.temp = value
                    .parse::<bool>()
                    .with_context(|| format!("Invalid boolean value: {value}"))?;
            }
            _ => anyhow::bail!("Unknown config key: {key}"),
        }
        Ok(())
    }
}
