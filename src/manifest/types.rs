use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
#[serde(rename_all = "camelCase")]
pub struct CentyManifest {
    pub schema_version: u32,
    pub centy_version: String,
    pub created_at: String,
    pub updated_at: String,
}

/// Type of managed file (file or directory)
/// Used for reconciliation and file type distinction
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum ManagedFileType {
    File,
    Directory,
}
