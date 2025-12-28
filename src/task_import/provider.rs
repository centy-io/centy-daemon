use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::error::TaskProviderError;

/// External task/issue from a provider
#[derive(Debug, Clone)]
pub struct ExternalTask {
    /// External task ID (e.g., "123" for GitHub issue #123)
    pub external_id: String,
    /// Task title
    pub title: String,
    /// Task description/body
    pub description: String,
    /// Provider's status (e.g., "open", "closed")
    pub status: String,
    /// Labels/tags associated with the task
    pub labels: Vec<String>,
    /// ISO timestamp when created
    pub created_at: String,
    /// ISO timestamp when last updated
    pub updated_at: String,
    /// Task author/creator
    pub author: Option<String>,
    /// URL to the original task
    pub url: String,
    /// Provider-specific metadata
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Provider-agnostic authentication credentials
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum AuthCredentials {
    /// Personal Access Token
    PersonalAccessToken { token: String },
    /// OAuth token
    OAuthToken { token: String },
    /// CLI tool command to get token (e.g., "gh auth token")
    CliTool { command: String },
    /// No authentication (for public access)
    None,
}

/// Result of connection test
#[derive(Debug)]
pub struct ConnectionTestResult {
    pub success: bool,
    pub message: String,
    pub permissions: Vec<String>,
}

/// Generic task provider trait
///
/// This trait defines the interface for importing tasks from external providers
/// (GitHub, GitLab, Jira, etc.)
#[async_trait]
pub trait TaskProvider: Send + Sync {
    /// Provider name (e.g., "github", "gitlab")
    fn provider_name(&self) -> &str;

    /// Authenticate with the provider
    ///
    /// # Arguments
    /// * `credentials` - Authentication credentials
    ///
    /// # Returns
    /// * `Ok(())` if authentication successful
    /// * `Err(TaskProviderError)` if authentication fails
    async fn authenticate(&self, credentials: &AuthCredentials) -> Result<(), TaskProviderError>;

    /// Test connection and validate permissions
    ///
    /// # Returns
    /// * `Ok(ConnectionTestResult)` with connection status
    /// * `Err(TaskProviderError)` if test fails
    async fn test_connection(&self) -> Result<ConnectionTestResult, TaskProviderError>;

    /// List all tasks from a specific source
    ///
    /// # Arguments
    /// * `source_id` - Source identifier (e.g., "owner/repo" for GitHub)
    ///
    /// # Returns
    /// * `Ok(Vec<ExternalTask>)` with all tasks
    /// * `Err(TaskProviderError)` if listing fails
    async fn list_tasks(&self, source_id: &str) -> Result<Vec<ExternalTask>, TaskProviderError>;

    /// Fetch a single task by ID
    ///
    /// # Arguments
    /// * `source_id` - Source identifier
    /// * `task_id` - Task ID within the source
    ///
    /// # Returns
    /// * `Ok(ExternalTask)` if found
    /// * `Err(TaskProviderError)` if not found or fetch fails
    async fn get_task(
        &self,
        source_id: &str,
        task_id: &str,
    ) -> Result<ExternalTask, TaskProviderError>;
}
