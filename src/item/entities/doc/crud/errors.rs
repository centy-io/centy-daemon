use thiserror::Error;

use crate::template::TemplateError;

#[derive(Error, Debug)]
pub enum DocError {
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Manifest error: {0}")]
    ManifestError(#[from] crate::manifest::ManifestError),

    #[error("JSON error: {0}")]
    JsonError(#[from] serde_json::Error),

    #[error("Centy not initialized. Run 'centy init' first.")]
    NotInitialized,

    #[error("Doc '{0}' not found")]
    DocNotFound(String),

    #[error("Title is required")]
    TitleRequired,

    #[error("Doc with slug '{0}' already exists")]
    SlugAlreadyExists(String),

    #[error("Invalid slug: {0}")]
    InvalidSlug(String),

    #[error("Doc '{0}' is not soft-deleted")]
    DocNotDeleted(String),

    #[error("Doc '{0}' is already soft-deleted")]
    DocAlreadyDeleted(String),

    #[error("Template error: {0}")]
    TemplateError(#[from] TemplateError),

    #[error("Target project not initialized")]
    TargetNotInitialized,

    #[error("Cannot move doc to same project")]
    SameProjectMove,

    #[error("Cannot create org doc: project has no organization")]
    NoOrganization,

    #[error("Registry error: {0}")]
    RegistryError(String),
}
