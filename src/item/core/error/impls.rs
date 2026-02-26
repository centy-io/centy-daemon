use super::ItemError;
impl From<mdstore::StoreError> for ItemError {
    fn from(err: mdstore::StoreError) -> Self {
        match err {
            mdstore::StoreError::IoError(e) => ItemError::IoError(e),
            mdstore::StoreError::NotFound(id) => ItemError::NotFound(id),
            mdstore::StoreError::ValidationError(msg) => ItemError::ValidationError(msg),
            mdstore::StoreError::JsonError(e) => ItemError::JsonError(e),
            mdstore::StoreError::YamlError(msg) => ItemError::YamlError(msg),
            mdstore::StoreError::FrontmatterError(msg) => ItemError::FrontmatterError(msg),
            mdstore::StoreError::ItemTypeNotFound(msg) => ItemError::ItemTypeNotFound(msg),
            mdstore::StoreError::FeatureNotEnabled(msg) => ItemError::FeatureNotEnabled(msg),
            mdstore::StoreError::AlreadyDeleted(id) => ItemError::AlreadyDeleted(id),
            mdstore::StoreError::NotDeleted(id) => ItemError::NotDeleted(id),
            mdstore::StoreError::InvalidStatus { status, allowed } => {
                ItemError::InvalidStatus { status, allowed }
            }
            mdstore::StoreError::InvalidPriority { priority, max } => {
                ItemError::InvalidPriority { priority, max }
            }
            mdstore::StoreError::AlreadyExists(id) => ItemError::AlreadyExists(id),
            mdstore::StoreError::IsDeleted(id) => ItemError::IsDeleted(id),
            mdstore::StoreError::SameLocation => ItemError::SameProject,
            mdstore::StoreError::Custom(msg) => ItemError::Custom(msg),
        }
    }
}
impl From<mdstore::ConfigError> for ItemError {
    fn from(err: mdstore::ConfigError) -> Self {
        match err {
            mdstore::ConfigError::IoError(e) => ItemError::IoError(e),
            mdstore::ConfigError::YamlError(e) => ItemError::YamlError(e.to_string()),
            mdstore::ConfigError::JsonError(e) => ItemError::JsonError(e),
        }
    }
}
impl ItemError {
    /// Create a custom error with a message
    pub fn custom(msg: impl Into<String>) -> Self { ItemError::Custom(msg.into()) }
    /// Create a not found error
    pub fn not_found(id: impl Into<String>) -> Self { ItemError::NotFound(id.into()) }
    /// Create a validation error
    pub fn validation(msg: impl Into<String>) -> Self { ItemError::ValidationError(msg.into()) }
}
