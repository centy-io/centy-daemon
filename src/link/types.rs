use serde::{Deserialize, Serialize};
/// Target entity type for links — stored as a plain string so any configured item type can be linked.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(transparent)]
pub struct TargetType(String);
impl TargetType {
    #[must_use]
    pub fn new<T: Into<String>>(s: T) -> Self {
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
/// Which side of a link a queried entity is on.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LinkDirection {
    /// The queried entity is the source of the link.
    Source,
    /// The queried entity is the target of the link.
    Target,
}
impl LinkDirection {
    #[must_use]
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Source => "source",
            Self::Target => "target",
        }
    }
}
/// Full bidirectional link record stored as one markdown file in `.centy/links/`.
#[derive(Debug, Clone)]
pub struct LinkRecord {
    /// UUID of the link file — use for deletion.
    pub id: String,
    pub source_id: String,
    pub source_type: TargetType,
    pub target_id: String,
    pub target_type: TargetType,
    /// Link type from the source's perspective (e.g. "blocks").
    pub link_type: String,
    pub created_at: String,
    pub updated_at: String,
}
impl LinkRecord {
    /// View from the source entity's perspective.
    #[must_use]
    pub fn source_view(&self) -> LinkView {
        LinkView {
            id: self.id.clone(),
            target_id: self.target_id.clone(),
            target_type: self.target_type.clone(),
            link_type: self.link_type.clone(),
            direction: LinkDirection::Source,
            created_at: self.created_at.clone(),
        }
    }
    /// View from the target entity's perspective.
    #[must_use]
    pub fn target_view(&self) -> LinkView {
        LinkView {
            id: self.id.clone(),
            target_id: self.source_id.clone(),
            target_type: self.source_type.clone(),
            link_type: self.link_type.clone(),
            direction: LinkDirection::Target,
            created_at: self.created_at.clone(),
        }
    }
}
/// Entity-centric view of a link (what gets returned in list / create responses).
#[derive(Debug, Clone)]
pub struct LinkView {
    /// UUID of the underlying link file.
    pub id: String,
    /// The other entity's ID.
    pub target_id: String,
    /// The other entity's type.
    pub target_type: TargetType,
    /// Link type from the source's perspective (always).
    pub link_type: String,
    /// Which side the queried entity is on.
    pub direction: LinkDirection,
    pub created_at: String,
}
/// Custom link type definition (for config.json).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CustomLinkTypeDefinition {
    pub name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}
