use async_trait::async_trait;
use octocrab::Octocrab;
use std::collections::HashMap;
use std::sync::RwLock;

use super::error::TaskProviderError;
use super::provider::{AuthCredentials, ConnectionTestResult, ExternalTask, TaskProvider};

pub struct GitHubProvider {
    client: RwLock<Option<Octocrab>>,
}

impl GitHubProvider {
    #[must_use]
    pub fn new() -> Self {
        Self {
            client: RwLock::new(None),
        }
    }

    /// Parse "owner/repo" format
    fn parse_source_id(source_id: &str) -> Result<(String, String), TaskProviderError> {
        let parts: Vec<&str> = source_id.split('/').collect();
        if parts.len() != 2 {
            return Err(TaskProviderError::InvalidConfig(format!(
                "Invalid GitHub source_id: expected 'owner/repo', got '{source_id}'"
            )));
        }
        Ok((parts[0].to_string(), parts[1].to_string()))
    }
}

impl Default for GitHubProvider {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl TaskProvider for GitHubProvider {
    fn provider_name(&self) -> &str {
        "github"
    }

    async fn authenticate(&self, credentials: &AuthCredentials) -> Result<(), TaskProviderError> {
        match credentials {
            AuthCredentials::PersonalAccessToken { token } => {
                let client = Octocrab::builder()
                    .personal_token(token.clone())
                    .build()
                    .map_err(|e| TaskProviderError::AuthenticationFailed(e.to_string()))?;

                *self.client.write().unwrap() = Some(client);
                Ok(())
            }
            AuthCredentials::CliTool { command } => {
                // Execute command to get token
                let output = tokio::process::Command::new("sh")
                    .arg("-c")
                    .arg(command)
                    .output()
                    .await
                    .map_err(|e| TaskProviderError::AuthenticationFailed(e.to_string()))?;

                if !output.status.success() {
                    return Err(TaskProviderError::AuthenticationFailed(
                        String::from_utf8_lossy(&output.stderr).to_string(),
                    ));
                }

                let token = String::from_utf8_lossy(&output.stdout).trim().to_string();

                // Recurse with extracted token
                self.authenticate(&AuthCredentials::PersonalAccessToken { token })
                    .await
            }
            AuthCredentials::None => {
                // Public API (unauthenticated)
                let client = Octocrab::builder()
                    .build()
                    .map_err(|e| TaskProviderError::AuthenticationFailed(e.to_string()))?;
                *self.client.write().unwrap() = Some(client);
                Ok(())
            }
            _ => Err(TaskProviderError::InvalidConfig(
                "Unsupported auth type for GitHub".to_string(),
            )),
        }
    }

    async fn test_connection(&self) -> Result<ConnectionTestResult, TaskProviderError> {
        let client = {
            let client_opt = self.client.read().unwrap();
            client_opt
                .as_ref()
                .ok_or_else(|| TaskProviderError::AuthenticationFailed("Not authenticated".to_string()))?
                .clone()
        };

        // Test by fetching current user (if authenticated)
        match client.current().user().await {
            Ok(user) => Ok(ConnectionTestResult {
                success: true,
                message: format!("Authenticated as {}", user.login),
                permissions: vec!["read:issues".to_string()],
            }),
            Err(e) => Ok(ConnectionTestResult {
                success: false,
                message: e.to_string(),
                permissions: vec![],
            }),
        }
    }

    async fn list_tasks(&self, source_id: &str) -> Result<Vec<ExternalTask>, TaskProviderError> {
        let client = {
            let client_opt = self.client.read().unwrap();
            client_opt
                .as_ref()
                .ok_or_else(|| TaskProviderError::AuthenticationFailed("Not authenticated".to_string()))?
                .clone()
        };

        let (owner, repo) = Self::parse_source_id(source_id)?;

        // Fetch issues using octocrab
        let issues = client
            .issues(&owner, &repo)
            .list()
            .state(octocrab::params::State::All)
            .send()
            .await
            .map_err(|e| TaskProviderError::ProviderError(e.to_string()))?;

        // Convert to ExternalTask
        let tasks = issues
            .items
            .into_iter()
            .map(|issue| {
                // Convert IssueState to string
                let status = match issue.state {
                    octocrab::models::IssueState::Open => "open",
                    octocrab::models::IssueState::Closed => "closed",
                    _ => "open", // fallback
                }.to_string();

                ExternalTask {
                    external_id: issue.number.to_string(),
                    title: issue.title,
                    description: issue.body.unwrap_or_default(),
                    status,
                    labels: issue.labels.iter().map(|l| l.name.clone()).collect(),
                    created_at: issue.created_at.to_rfc3339(),
                    updated_at: issue.updated_at.to_rfc3339(),
                    author: Some(issue.user.login.clone()),
                    url: issue.html_url.to_string(),
                    metadata: HashMap::new(),
                }
            })
            .collect();

        Ok(tasks)
    }

    async fn get_task(
        &self,
        source_id: &str,
        task_id: &str,
    ) -> Result<ExternalTask, TaskProviderError> {
        let client = {
            let client_opt = self.client.read().unwrap();
            client_opt
                .as_ref()
                .ok_or_else(|| TaskProviderError::AuthenticationFailed("Not authenticated".to_string()))?
                .clone()
        };

        let (owner, repo) = Self::parse_source_id(source_id)?;
        let issue_number: u64 = task_id
            .parse()
            .map_err(|_| TaskProviderError::InvalidConfig(format!("Invalid issue number: {task_id}")))?;

        let issue = client
            .issues(&owner, &repo)
            .get(issue_number)
            .await
            .map_err(|e| TaskProviderError::NotFound(e.to_string()))?;

        // Convert IssueState to string
        let status = match issue.state {
            octocrab::models::IssueState::Open => "open",
            octocrab::models::IssueState::Closed => "closed",
            _ => "open", // fallback
        }.to_string();

        Ok(ExternalTask {
            external_id: issue.number.to_string(),
            title: issue.title,
            description: issue.body.unwrap_or_default(),
            status,
            labels: issue.labels.iter().map(|l| l.name.clone()).collect(),
            created_at: issue.created_at.to_rfc3339(),
            updated_at: issue.updated_at.to_rfc3339(),
            author: Some(issue.user.login.clone()),
            url: issue.html_url.to_string(),
            metadata: HashMap::new(),
        })
    }
}
