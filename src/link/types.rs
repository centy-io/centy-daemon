use serde::{Deserialize, Serialize};
/// Target entity type for links — stored as a plain string so any configured item type can be linked.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(transparent)]
pub struct TargetType(String);
impl TargetType {
    #[must_use]
    pub fn new(s: impl Into<String>) -> Self {
        Self(s.into())
    }
    #[must_use]
    pub fn issue() -> Self {
        Self::new("issue")
    }
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
    #[must_use]
    pub fn folder_name(&self) -> String {
        format!("{}s", self.0)
    }
}
impl std::str::FromStr for TargetType {
    type Err = std::convert::Infallible;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self(s.to_lowercase()))
    }
}
impl std::fmt::Display for TargetType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
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
        Self {
            target_id,
            target_type,
            link_type,
            created_at: crate::utils::now_iso(),
        }
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
