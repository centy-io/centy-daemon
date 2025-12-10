use std::path::Path;
use std::process::{Command, Stdio};
use thiserror::Error;
use tracing::info;

use super::config::{AgentConfig, LocalLlmConfig};
use super::prompt::{LlmAction, PromptBuilder};
use crate::issue::Issue;

#[derive(Error, Debug)]
pub enum AgentError {
    #[error("Agent '{0}' not found in configuration")]
    AgentNotFound(String),

    #[error("No default agent configured")]
    NoDefaultAgent,

    #[error("Failed to spawn agent process: {0}")]
    SpawnError(String),

    #[error("Prompt error: {0}")]
    PromptError(#[from] super::prompt::PromptError),
}

/// Result of spawning an agent
#[derive(Debug, Clone)]
pub struct SpawnResult {
    pub agent_name: String,
    pub pid: Option<u32>,
    pub prompt_preview: String,
}

/// Spawn an LLM agent to work on an issue
pub async fn spawn_agent(
    project_path: &Path,
    config: &LocalLlmConfig,
    issue: &Issue,
    action: LlmAction,
    agent_name: Option<&str>,
    extra_args: Vec<String>,
    priority_levels: u32,
) -> Result<SpawnResult, AgentError> {
    // Resolve which agent to use
    let agent = match agent_name {
        Some(name) => config
            .get_agent(name)
            .ok_or_else(|| AgentError::AgentNotFound(name.to_string()))?,
        None => config.get_default_agent().ok_or(AgentError::NoDefaultAgent)?,
    };

    // Build the prompt
    let prompt_builder = PromptBuilder::new();
    let user_template = match action {
        LlmAction::Plan => agent.plan_template.as_deref(),
        LlmAction::Implement => agent.implement_template.as_deref(),
    };
    let prompt = prompt_builder
        .build_prompt(project_path, issue, action, user_template, priority_levels)
        .await?;

    // Build command
    let mut cmd = Command::new(&agent.command);

    // Add default args
    cmd.args(&agent.default_args);

    // Add extra args
    cmd.args(&extra_args);

    // Set working directory
    cmd.current_dir(project_path);

    // Set environment variables from config
    for (key, value) in &config.env_vars {
        cmd.env(key, value);
    }

    // Pass prompt via argument
    // Most CLI agents accept prompt as last positional arg
    cmd.arg(&prompt);

    // Configure stdio for fire-and-forget
    cmd.stdin(Stdio::null())
        .stdout(Stdio::inherit()) // Let output go to terminal
        .stderr(Stdio::inherit());

    info!(
        "Spawning agent '{}' for issue #{} ({})",
        agent.name,
        issue.metadata.display_number,
        action.as_str()
    );

    // Spawn the process
    let child = cmd
        .spawn()
        .map_err(|e| AgentError::SpawnError(e.to_string()))?;

    let pid = child.id();
    info!("Agent process spawned with PID: {:?}", pid);

    // Create preview (first 500 chars)
    let prompt_preview = PromptBuilder::preview(&prompt, 500);

    Ok(SpawnResult {
        agent_name: agent.name.clone(),
        pid: Some(pid),
        prompt_preview,
    })
}

/// Check if an agent command exists and is executable
#[must_use] 
pub fn check_agent_available(agent: &AgentConfig) -> bool {
    // Try to find the command
    which::which(&agent.command).is_ok()
}

/// Get list of available agents (those whose commands exist)
#[must_use] 
pub fn get_available_agents(config: &LocalLlmConfig) -> Vec<&AgentConfig> {
    config
        .agents
        .iter()
        .filter(|agent| check_agent_available(agent))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::llm::config::{AgentType, LocalLlmConfig};

    #[test]
    fn test_check_agent_not_found() {
        let agent = AgentConfig {
            agent_type: AgentType::Custom,
            name: "nonexistent-agent-xyz".to_string(),
            command: "nonexistent-command-xyz".to_string(),
            default_args: vec![],
            plan_template: None,
            implement_template: None,
        };

        assert!(!check_agent_available(&agent));
    }

    #[test]
    fn test_get_available_agents() {
        let config = LocalLlmConfig {
            default_agent: Some("nonexistent".to_string()),
            agents: vec![
                AgentConfig {
                    agent_type: AgentType::Custom,
                    name: "nonexistent".to_string(),
                    command: "nonexistent-command-xyz".to_string(),
                    default_args: vec![],
                    plan_template: None,
                    implement_template: None,
                },
                // This one might exist on the system
                AgentConfig {
                    agent_type: AgentType::Custom,
                    name: "ls-test".to_string(),
                    command: "ls".to_string(), // ls should exist on most systems
                    default_args: vec![],
                    plan_template: None,
                    implement_template: None,
                },
            ],
            env_vars: Default::default(),
        };

        let available = get_available_agents(&config);
        // At least ls should be available on most systems
        // The nonexistent command should not be available
        assert!(available.iter().all(|a| a.name != "nonexistent"));
    }

    #[test]
    fn test_agent_error_display() {
        let err = AgentError::AgentNotFound("my-agent".to_string());
        assert!(err.to_string().contains("my-agent"));
        assert!(err.to_string().contains("not found"));

        let err = AgentError::NoDefaultAgent;
        assert!(err.to_string().contains("default"));

        let err = AgentError::SpawnError("permission denied".to_string());
        assert!(err.to_string().contains("permission denied"));
    }
}
