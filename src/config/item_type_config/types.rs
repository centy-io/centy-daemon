use mdstore::{CustomFieldDef, IdStrategy};
use serde::{Deserialize, Serialize};

/// Features that can be toggled per item type.
///
/// Stored as a nested object inside `config.yaml` under the `features` key.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ItemTypeFeatures {
    /// Enable display numbers (1, 2, 3…) for items.
    #[serde(default)]
    pub display_number: bool,
    /// Enable status tracking (e.g. `open`, `in-progress`, `closed`).
    #[serde(default)]
    pub status: bool,
    /// Enable priority levels.
    #[serde(default)]
    pub priority: bool,
    /// Enable soft-deletion (items can be deleted and restored).
    #[serde(default)]
    pub soft_delete: bool,
    /// Enable file attachments.
    #[serde(default)]
    pub assets: bool,
    /// Enable organization sync.
    #[serde(default)]
    pub org_sync: bool,
    /// Enable moving items between projects.
    #[serde(rename = "move", default)]
    pub move_item: bool,
    /// Enable duplication of items.
    #[serde(default)]
    pub duplicate: bool,
}

/// Full configuration for an item type, parsed from `.centy/<folder>/config.yaml`.
///
/// This is the canonical schema for item-type configuration in centy-daemon.
/// Both built-in types (`issues`, `docs`) and custom types share this format.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ItemTypeConfig {
    /// Human-readable singular name (e.g. `"Issue"`, `"Doc"`, `"Epic"`).
    pub name: String,
    /// Optional icon identifier (e.g. `"clipboard"`, `"document"`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub icon: Option<String>,
    /// ID generation strategy: `uuid` (distributed) or `slug` (title-derived).
    pub identifier: IdStrategy,
    /// Toggle-able features for this item type.
    pub features: ItemTypeFeatures,
    /// Allowed status values (e.g. `["open", "in-progress", "closed"]`).
    /// Omitted when empty.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub statuses: Vec<String>,
    /// Default status for newly created items. Must appear in `statuses`.
    /// Omitted when not set.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub default_status: Option<String>,
    /// Number of priority levels (must be > 0 when present).
    /// Omitted when not set.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub priority_levels: Option<u32>,
    /// Custom field definitions for this item type.
    /// Omitted when empty.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub custom_fields: Vec<CustomFieldDef>,
    /// Path to the Handlebars template file, relative to `.centy/<folder>/`.
    /// Omitted when not set.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub template: Option<String>,
}
