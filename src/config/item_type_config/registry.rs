use super::io::discover_item_types_map;
use super::types::ItemTypeConfig;
use crate::utils::get_centy_path;
use std::collections::HashMap;
use std::path::Path;
use tracing::{info, warn};

/// In-memory registry of item types keyed by folder name.
#[derive(Debug, Clone)]
pub struct ItemTypeRegistry {
    types: HashMap<String, ItemTypeConfig>,
}

impl ItemTypeRegistry {
    /// Build the registry by scanning `.centy/*/config.yaml` files.
    /// Skips malformed/missing configs. Deduplicates by type name.
    pub async fn build(project_path: &Path) -> Result<Self, mdstore::ConfigError> {
        let centy_path = get_centy_path(project_path);
        let types = discover_item_types_map(&centy_path).await?;

        // Duplicate name detection (centy-specific logic)
        let mut seen_names: HashMap<String, String> = HashMap::new();
        let mut deduped = HashMap::new();
        for (folder, config) in types {
            if let Some(existing) = seen_names.get(&config.name) {
                warn!(
                    name = %config.name,
                    folder = %folder,
                    existing_folder = %existing,
                    "Duplicate type name detected, skipping"
                );
                continue;
            }
            seen_names.insert(config.name.clone(), folder.clone());
            deduped.insert(folder, config);
        }

        let type_names: Vec<&str> = deduped.values().map(|c| c.name.as_str()).collect();
        info!(count = deduped.len(), types = ?type_names, "Item type registry built");

        Ok(Self { types: deduped })
    }

    /// Get a config by folder name (e.g. `"issues"`, `"docs"`).
    #[must_use]
    pub fn get(&self, folder: &str) -> Option<&ItemTypeConfig> {
        self.types.get(folder)
    }

    /// Get a config by type name (e.g. `"Issue"`, `"Doc"`).
    #[must_use]
    pub fn get_by_name(&self, name: &str) -> Option<(&String, &ItemTypeConfig)> {
        self.types.iter().find(|(_, c)| c.name == name)
    }

    /// Get all registered item types.
    #[must_use]
    pub fn all(&self) -> &HashMap<String, ItemTypeConfig> {
        &self.types
    }

    /// Get the number of registered types.
    #[must_use]
    pub fn len(&self) -> usize {
        self.types.len()
    }

    /// Check if the registry is empty.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.types.is_empty()
    }

    /// Get folder names of all registered types.
    #[must_use]
    pub fn folders(&self) -> Vec<&String> {
        self.types.keys().collect()
    }

    /// Resolve an input string to a `(folder_name, config)` pair.
    /// Tries exact folder, case-insensitive name, then case-insensitive folder.
    #[must_use]
    pub fn resolve(&self, input: &str) -> Option<(&String, &ItemTypeConfig)> {
        if let Some((key, config)) = self.types.get_key_value(input) {
            return Some((key, config));
        }
        let input_lower = input.to_lowercase();
        if let Some(pair) = self
            .types
            .iter()
            .find(|(_, c)| c.name.to_lowercase() == input_lower)
        {
            return Some(pair);
        }
        self.types
            .iter()
            .find(|(k, _)| k.to_lowercase() == input_lower)
    }
}
