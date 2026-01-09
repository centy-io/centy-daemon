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
    /// Number of migration files
    pub migration_count: u32,
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

/// YAML frontmatter for migration files
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MigrationFrontmatter {
    pub timestamp: String,
    pub compacted_issues: Vec<CompactedIssueRef>,
}

impl MigrationFrontmatter {
    /// Parse frontmatter from markdown content
    #[allow(dead_code)] // Part of WIP features module
    pub fn parse(content: &str) -> Option<Self> {
        let lines: Vec<&str> = content.lines().collect();

        // Check for frontmatter
        if lines.first() != Some(&"---") {
            return None;
        }

        // Find closing ---
        let end_idx = lines.iter().skip(1).position(|&line| line == "---")?;
        let frontmatter_yaml: String = lines[1..=end_idx].join("\n");

        serde_yaml::from_str(&frontmatter_yaml).ok()
    }

    /// Generate YAML frontmatter string
    #[allow(dead_code)] // Part of WIP features module
    pub fn to_yaml(&self) -> String {
        serde_yaml::to_string(self).unwrap_or_default()
    }
}
