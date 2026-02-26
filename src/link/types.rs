use serde::{Deserialize, Serialize};
/// Target entity type for links
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum TargetType { Issue, Doc }
impl TargetType {
    #[must_use]
    pub fn as_str(&self) -> &'static str {
        match self { Self::Issue => "issue", Self::Doc => "doc" }
    }
    #[must_use]
    pub fn folder_name(&self) -> &'static str {
        match self { Self::Issue => "issues", Self::Doc => "docs" }
    }
}
impl std::str::FromStr for TargetType {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "issue" => Ok(Self::Issue),
            "doc" => Ok(Self::Doc),
            _ => Err(format!("Invalid target type: {s}")),
        }
    }
}
impl std::fmt::Display for TargetType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}
/// A link between two entities
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Link {
    pub target_id: String,
    pub target_type: TargetType,
    pub link_type: String,
    pub created_at: String,
}
impl Link {
    #[must_use]
    pub fn new(target_id: String, target_type: TargetType, link_type: String) -> Self {
        Self { target_id, target_type, link_type, created_at: crate::utils::now_iso() }
    }
}
/// Custom link type definition (for config.json)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CustomLinkTypeDefinition {
    pub name: String,
    pub inverse: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}
