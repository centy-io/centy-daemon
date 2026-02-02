//! Feature types (WIP - not yet integrated)

use serde::{Deserialize, Serialize};

/// Status of the features system in a project
#[derive(Debug, Clone)]
pub struct FeatureStatus {
    /// Whether features/ folder exists
    pub initialized: bool,
    /// Whether compact.md exists
    pub has_compact: bool,
    /// Whether instruction.md exists
    pub has_instruction: bool,
    /// Number of uncompacted issues
    pub uncompacted_count: u32,
}

/// Reference to an issue that was compacted
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CompactedIssueRef {
    pub id: String,
    pub display_number: u32,
    pub title: String,
}
