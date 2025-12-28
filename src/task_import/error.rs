use thiserror::Error;

#[derive(Debug, Error)]
pub enum TaskProviderError {
    #[error("Authentication failed: {0}")]
    AuthenticationFailed(String),

    #[error("Resource not found: {0}")]
    NotFound(String),

    #[error("Permission denied: {0}")]
    PermissionDenied(String),

    #[error("Rate limit exceeded: {0}")]
    RateLimitExceeded(String),

    #[error("Network error: {0}")]
    NetworkError(String),

    #[error("Invalid configuration: {0}")]
    InvalidConfig(String),

    #[error("Provider error: {0}")]
    ProviderError(String),
}

#[derive(Debug, Error)]
pub enum TaskImportError {
    #[error("Provider error: {0}")]
    ProviderError(#[from] TaskProviderError),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Issue CRUD error: {0}")]
    IssueCrudError(String),

    #[error("Configuration error: {0}")]
    ConfigError(String),

    #[error("Mapping error: {0}")]
    MappingError(String),
}

#[derive(Debug, Error)]
pub enum MapperError {
    #[error("Invalid mapping: {0}")]
    InvalidMapping(String),

    #[error("Missing required field: {0}")]
    MissingField(String),
}

#[derive(Debug, Error)]
pub enum AuthError {
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("JSON error: {0}")]
    JsonError(#[from] serde_json::Error),

    #[error("Provider not configured: {0}")]
    ProviderNotConfigured(String),
}

#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("JSON error: {0}")]
    JsonError(#[from] serde_json::Error),

    #[error("Provider not found: {0}")]
    ProviderNotFound(String),
}
