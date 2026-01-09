//! Title generation using LLM agents
//!
//! This module provides synchronous LLM text generation with stdout capture
//! for generating issue titles from descriptions.

use serde::Deserialize;
use std::path::Path;
use std::process::Stdio;
use std::time::Duration;
use thiserror::Error;
use tokio::process::Command;
use tokio::time::timeout;
use tracing::info;

use super::agent::check_agent_available;
use super::config::{get_effective_local_config, AgentConfig, LocalConfigError, LocalLlmConfig};

/// Default timeout for title generation (30 seconds)
const TITLE_GENERATION_TIMEOUT_SECS: u64 = 30;

/// Prompt template for title generation
const TITLE_GENERATION_PROMPT: &str = r#"Generate a concise, descriptive issue title based on the following description.

IMPORTANT: Respond ONLY with valid JSON in this exact format:
{"title": "Your generated title here"}

Rules for the title:
- Keep it under 80 characters
- Use imperative mood (e.g., "Add", "Fix", "Update", "Implement")
- Be specific and actionable
- Do not include issue numbers or prefixes

Description:
{description}

Respond with JSON only:"#;

#[derive(Error, Debug)]
pub enum TitleGenerationError {
    #[error("Title is required when LLM is not configured")]
    NoLlmConfigured,

    #[error("LLM agent '{0}' not found or not installed")]
    AgentNotAvailable(String),

    #[error("Failed to spawn LLM process: {0}")]
    SpawnError(String),

    #[error("LLM process timed out after {0} seconds")]
    Timeout(u64),

    #[error("LLM process failed: {0}")]
    ProcessFailed(String),

    #[error("Failed to parse LLM response: {0}")]
    ParseError(String),

    #[error("LLM returned empty title")]
    EmptyTitle,

    #[error("Config error: {0}")]
    ConfigError(#[from] LocalConfigError),
}

/// Response structure expected from LLM
#[derive(Debug, Deserialize)]
struct LlmTitleResponse {
    title: String,
}

/// Result of title generation
#[derive(Debug, Clone)]
pub struct GeneratedTitle {
    pub title: String,
}

/// Generate an issue title using the configured LLM agent
///
/// This function:
/// 1. Loads effective LLM config
/// 2. Checks if default agent is available
/// 3. Spawns the agent with a title generation prompt
/// 4. Captures stdout and parses JSON response
/// 5. Returns the generated title
pub async fn generate_title(
    project_path: &Path,
    description: &str,
) -> Result<GeneratedTitle, TitleGenerationError> {
    // Load effective config
    let config = get_effective_local_config(Some(project_path)).await?;

    // Get default agent
    let agent = config
        .get_default_agent()
        .ok_or(TitleGenerationError::NoLlmConfigured)?;

    // Check if agent is available
    if !check_agent_available(agent) {
        return Err(TitleGenerationError::AgentNotAvailable(agent.name.clone()));
    }

    // Build prompt
    let prompt = TITLE_GENERATION_PROMPT.replace("{description}", description);

    info!(
        "Generating title for issue using agent '{}'",
        agent.name
    );

    // Execute command and get title
    let title = execute_title_generation(&config, agent, &prompt, project_path).await?;

    info!("Generated title: {}", title);

    Ok(GeneratedTitle {
        title,
    })
}

/// Execute the LLM command and capture output
async fn execute_title_generation(
    config: &LocalLlmConfig,
    agent: &AgentConfig,
    prompt: &str,
    project_path: &Path,
) -> Result<String, TitleGenerationError> {
    let mut cmd = Command::new(&agent.command);

    // Add default args (e.g., --print for Claude)
    cmd.args(&agent.default_args);

    // Set working directory
    cmd.current_dir(project_path);

    // Set environment variables from config
    for (key, value) in &config.env_vars {
        cmd.env(key, value);
    }

    // Pass prompt as argument
    cmd.arg(prompt);

    // Configure stdio for capture
    cmd.stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());

    // Spawn with timeout
    let output = timeout(
        Duration::from_secs(TITLE_GENERATION_TIMEOUT_SECS),
        cmd.output(),
    )
    .await
    .map_err(|_| TitleGenerationError::Timeout(TITLE_GENERATION_TIMEOUT_SECS))?
    .map_err(|e| TitleGenerationError::SpawnError(e.to_string()))?;

    // Check if process succeeded
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(TitleGenerationError::ProcessFailed(stderr.to_string()));
    }

    // Parse stdout
    let stdout = String::from_utf8_lossy(&output.stdout);
    parse_title_response(&stdout)
}

/// Parse the JSON response from LLM to extract title
fn parse_title_response(response: &str) -> Result<String, TitleGenerationError> {
    // Try to find JSON in the response (LLM might add extra text)
    let json_start = response.find('{');
    let json_end = response.rfind('}');

    match (json_start, json_end) {
        (Some(start), Some(end)) if end > start => {
            let json_str = &response[start..=end];
            let parsed: LlmTitleResponse = serde_json::from_str(json_str)
                .map_err(|e| TitleGenerationError::ParseError(e.to_string()))?;

            let title = parsed.title.trim().to_string();
            if title.is_empty() {
                return Err(TitleGenerationError::EmptyTitle);
            }

            Ok(title)
        }
        _ => Err(TitleGenerationError::ParseError(
            "No valid JSON found in response".to_string(),
        )),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_title_response_valid_json() {
        let response = r#"{"title": "Add user authentication"}"#;
        let result = parse_title_response(response);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "Add user authentication");
    }

    #[test]
    fn test_parse_title_response_with_preamble() {
        let response = r#"Here is the title:
{"title": "Fix login bug"}
That should work!"#;
        let result = parse_title_response(response);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "Fix login bug");
    }

    #[test]
    fn test_parse_title_response_with_whitespace() {
        let response = r#"{"title": "  Update dependencies  "}"#;
        let result = parse_title_response(response);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "Update dependencies");
    }

    #[test]
    fn test_parse_title_response_empty_title() {
        let response = r#"{"title": ""}"#;
        let result = parse_title_response(response);
        assert!(matches!(result, Err(TitleGenerationError::EmptyTitle)));
    }

    #[test]
    fn test_parse_title_response_whitespace_only_title() {
        let response = r#"{"title": "   "}"#;
        let result = parse_title_response(response);
        assert!(matches!(result, Err(TitleGenerationError::EmptyTitle)));
    }

    #[test]
    fn test_parse_title_response_no_json() {
        let response = "Just some text without JSON";
        let result = parse_title_response(response);
        assert!(matches!(result, Err(TitleGenerationError::ParseError(_))));
    }

    #[test]
    fn test_parse_title_response_invalid_json() {
        let response = r#"{title: "missing quotes"}"#;
        let result = parse_title_response(response);
        assert!(matches!(result, Err(TitleGenerationError::ParseError(_))));
    }

    #[test]
    fn test_parse_title_response_missing_title_field() {
        let response = r#"{"name": "wrong field"}"#;
        let result = parse_title_response(response);
        assert!(matches!(result, Err(TitleGenerationError::ParseError(_))));
    }

    #[test]
    fn test_error_display() {
        let err = TitleGenerationError::NoLlmConfigured;
        assert!(err.to_string().contains("LLM is not configured"));

        let err = TitleGenerationError::AgentNotAvailable("claude".to_string());
        assert!(err.to_string().contains("claude"));
        assert!(err.to_string().contains("not found"));

        let err = TitleGenerationError::Timeout(30);
        assert!(err.to_string().contains("30"));
        assert!(err.to_string().contains("timed out"));

        let err = TitleGenerationError::EmptyTitle;
        assert!(err.to_string().contains("empty"));
    }
}
