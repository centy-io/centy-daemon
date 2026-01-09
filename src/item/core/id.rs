//! Item identifier types supporting both UUID and slug-based IDs.

use std::fmt;

/// Unified item identifier supporting both UUID and slug-based IDs.
///
/// - Issues and PRs use UUID-based identifiers for conflict-free distributed creation
/// - Docs use slug-based identifiers for human-readable URLs
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ItemId {
    /// UUID-based identifier (used by Issue, PR)
    Uuid(uuid::Uuid),
    /// Slug-based identifier (used by Doc)
    Slug(String),
}

impl ItemId {
    /// Create a new UUID-based ItemId
    #[must_use]
    pub fn new_uuid() -> Self {
        ItemId::Uuid(uuid::Uuid::new_v4())
    }

    /// Create an ItemId from a UUID
    #[must_use]
    pub fn from_uuid(uuid: uuid::Uuid) -> Self {
        ItemId::Uuid(uuid)
    }

    /// Create an ItemId from a slug string
    #[must_use]
    pub fn from_slug(slug: impl Into<String>) -> Self {
        ItemId::Slug(slug.into())
    }

    /// Parse a string as an ItemId, attempting UUID first, then falling back to slug
    #[must_use]
    pub fn parse(s: &str) -> Self {
        match uuid::Uuid::parse_str(s) {
            Ok(uuid) => ItemId::Uuid(uuid),
            Err(_) => ItemId::Slug(s.to_string()),
        }
    }

    /// Get the UUID if this is a UUID-based ID
    #[must_use]
    pub fn as_uuid(&self) -> Option<&uuid::Uuid> {
        match self {
            ItemId::Uuid(uuid) => Some(uuid),
            ItemId::Slug(_) => None,
        }
    }

    /// Get the slug if this is a slug-based ID
    #[must_use]
    pub fn as_slug(&self) -> Option<&str> {
        match self {
            ItemId::Uuid(_) => None,
            ItemId::Slug(slug) => Some(slug),
        }
    }

    /// Check if this is a UUID-based ID
    #[must_use]
    pub fn is_uuid(&self) -> bool {
        matches!(self, ItemId::Uuid(_))
    }

    /// Check if this is a slug-based ID
    #[must_use]
    pub fn is_slug(&self) -> bool {
        matches!(self, ItemId::Slug(_))
    }

    /// Get the string representation suitable for folder/file names
    #[must_use]
    pub fn to_storage_name(&self) -> String {
        match self {
            ItemId::Uuid(uuid) => uuid.to_string(),
            ItemId::Slug(slug) => slug.clone(),
        }
    }
}

impl fmt::Display for ItemId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ItemId::Uuid(uuid) => write!(f, "{uuid}"),
            ItemId::Slug(slug) => write!(f, "{slug}"),
        }
    }
}

impl From<uuid::Uuid> for ItemId {
    fn from(uuid: uuid::Uuid) -> Self {
        ItemId::Uuid(uuid)
    }
}

impl From<String> for ItemId {
    fn from(s: String) -> Self {
        ItemId::parse(&s)
    }
}

impl From<&str> for ItemId {
    fn from(s: &str) -> Self {
        ItemId::parse(s)
    }
}

impl serde::Serialize for ItemId {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

impl<'de> serde::Deserialize<'de> for ItemId {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        Ok(ItemId::parse(&s))
    }
}

/// Trait for entities that have an identifier
pub trait Identifiable {
    /// Get the item's identifier
    fn id(&self) -> ItemId;

    /// Get the item's identifier as a string reference
    fn id_str(&self) -> &str;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_item_id_uuid() {
        let uuid = uuid::Uuid::new_v4();
        let id = ItemId::from_uuid(uuid);
        assert!(id.is_uuid());
        assert!(!id.is_slug());
        assert_eq!(id.as_uuid(), Some(&uuid));
        assert_eq!(id.as_slug(), None);
    }

    #[test]
    fn test_item_id_slug() {
        let id = ItemId::from_slug("my-doc");
        assert!(id.is_slug());
        assert!(!id.is_uuid());
        assert_eq!(id.as_slug(), Some("my-doc"));
        assert_eq!(id.as_uuid(), None);
    }

    #[test]
    fn test_item_id_parse_uuid() {
        let uuid = uuid::Uuid::new_v4();
        let id = ItemId::parse(&uuid.to_string());
        assert!(id.is_uuid());
        assert_eq!(id.as_uuid(), Some(&uuid));
    }

    #[test]
    fn test_item_id_parse_slug() {
        let id = ItemId::parse("not-a-uuid");
        assert!(id.is_slug());
        assert_eq!(id.as_slug(), Some("not-a-uuid"));
    }

    #[test]
    fn test_item_id_storage_name() {
        let uuid = uuid::Uuid::parse_str("550e8400-e29b-41d4-a716-446655440000").unwrap();
        let id = ItemId::from_uuid(uuid);
        assert_eq!(id.to_storage_name(), "550e8400-e29b-41d4-a716-446655440000");

        let id = ItemId::from_slug("my-document");
        assert_eq!(id.to_storage_name(), "my-document");
    }

    #[test]
    fn test_item_id_serde() {
        let uuid = uuid::Uuid::new_v4();
        let id = ItemId::from_uuid(uuid);
        let json = serde_json::to_string(&id).unwrap();
        let parsed: ItemId = serde_json::from_str(&json).unwrap();
        assert_eq!(id, parsed);

        let id = ItemId::from_slug("test-slug");
        let json = serde_json::to_string(&id).unwrap();
        let parsed: ItemId = serde_json::from_str(&json).unwrap();
        assert_eq!(id, parsed);
    }
}
