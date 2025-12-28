pub mod auth;
pub mod config;
pub mod error;
pub mod github;
pub mod import;
pub mod mapper;
pub mod provider;

// Re-export commonly used types
pub use auth::{get_provider_credentials, read_auth_config, write_auth_config, TaskImportAuth};
pub use config::{
    get_provider_config, read_config, update_provider_config, write_config, FieldMappings,
    ProviderConfig, TaskImportConfig,
};
pub use error::{AuthError, ConfigError, MapperError, TaskImportError, TaskProviderError};
pub use github::GitHubProvider;
pub use import::{import_tasks, ImportError, ImportFilter, ImportOptions, ImportResult};
pub use mapper::{map_external_task_to_create, map_external_task_to_update};
pub use provider::{AuthCredentials, ConnectionTestResult, ExternalTask, TaskProvider};
