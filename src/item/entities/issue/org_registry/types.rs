use crate::utils::now_iso;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum OrgIssueRegistryError {
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
    #[error("JSON error: {0}")]
    JsonError(#[from] serde_json::Error),
    #[error("Failed to determine home directory")]
    HomeDirNotFound,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OrgIssueRegistry {
    #[serde(default)]
    pub next_display_number: HashMap<String, u32>,
    pub updated_at: String,
}

impl Default for OrgIssueRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl OrgIssueRegistry {
    #[must_use]
    pub fn new() -> Self {
        Self {
            next_display_number: HashMap::new(),
            updated_at: now_iso(),
        }
    }
}
