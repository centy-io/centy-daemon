use std::path::Path;
use std::process::{Command, Stdio};
use thiserror::Error;
use tracing::info;

use super::config::{AgentConfig, LocalLlmConfig};
use super::prompt::{LlmAction, PromptBuilder};
use crate::issue::Issue;

/// Mode for starting the agent process
#[derive(Debug, Clone, Copy, Default)]
pub enum AgentSpawnMode {
    /// Spawn as background process, return PID (daemon mode)
    #[default]
    Background,
    /// Replace current process with agent (CLI mode)
    /// Unix: uses exec(), never returns on success
    /// Windows: falls back to waiting for process completion
    ReplaceProcess,
}

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

/// Start an LLM agent to work on an issue
///
/// This is the core implementation that supports both spawn() and exec() modes.
///
/// **Important**: When using `ReplaceProcess` mode:
/// - On Unix: This function uses exec() and NEVER RETURNS on success
/// - On Windows: This function waits for the child process and returns when it exits
/// - The work session MUST be recorded BEFORE calling this function with ReplaceProcess
pub async fn start_agent(
    project_path: &Path,
    config: &LocalLlmConfig,
    issue: &Issue,
    action: LlmAction,
    agent_name: Option<&str>,
    extra_args: Vec<String>,
    priority_levels: u32,
    mode: AgentSpawnMode,
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

    // Configure stdio
    cmd.stdin(Stdio::null())
        .stdout(Stdio::inherit()) // Let output go to terminal
        .stderr(Stdio::inherit());

    // Create preview (first 500 chars)
    let prompt_preview = PromptBuilder::preview(&prompt, 500);

    info!(
        "Starting agent '{}' for issue #{} ({}) in {:?} mode",
        agent.name,
        issue.metadata.display_number,
        action.as_str(),
        mode
    );

    match mode {
        AgentSpawnMode::Background => {
            // Spawn and return immediately (daemon behavior)
            let child = cmd
                .spawn()
                .map_err(|e| AgentError::SpawnError(e.to_string()))?;

            let pid = child.id();
            info!("Agent process spawned with PID: {:?}", pid);

            Ok(SpawnResult {
                agent_name: agent.name.clone(),
                pid: Some(pid),
                prompt_preview,
            })
        }
        AgentSpawnMode::ReplaceProcess => {
            // Platform-specific exec behavior
            exec_agent_process(cmd, &agent.name, &prompt_preview)
        }
    }
}

/// Spawn an LLM agent to work on an issue (backward-compatible wrapper)
///
/// This is the original fire-and-forget API. For CLI use cases that need
/// exec() behavior, use `start_agent` with `AgentSpawnMode::ReplaceProcess`.
pub async fn spawn_agent(
    project_path: &Path,
    config: &LocalLlmConfig,
    issue: &Issue,
    action: LlmAction,
    agent_name: Option<&str>,
    extra_args: Vec<String>,
    priority_levels: u32,
) -> Result<SpawnResult, AgentError> {
    start_agent(
        project_path,
        config,
        issue,
        action,
        agent_name,
        extra_args,
        priority_levels,
        AgentSpawnMode::Background,
    )
    .await
}

/// Unix implementation: exec() replaces current process
#[cfg(unix)]
fn exec_agent_process(
    mut cmd: Command,
    _agent_name: &str,
    _prompt_preview: &str,
) -> Result<SpawnResult, AgentError> {
    use std::os::unix::process::CommandExt;

    info!("Executing agent via exec() - replacing current process");

    // exec() never returns on success - it replaces the current process
    let err = cmd.exec();

    // If we get here, exec() failed
    Err(AgentError::SpawnError(format!("exec() failed: {err}")))
}

/// Windows implementation: spawn and wait (foreground process)
#[cfg(windows)]
fn exec_agent_process(
    mut cmd: Command,
    agent_name: &str,
    prompt_preview: &str,
) -> Result<SpawnResult, AgentError> {
    info!("Windows: spawning agent in foreground (waiting for completion)");

    let mut child = cmd
        .spawn()
        .map_err(|e| AgentError::SpawnError(e.to_string()))?;

    let pid = child.id();

    // Wait for the process to complete
    let status = child
        .wait()
        .map_err(|e| AgentError::SpawnError(format!("Failed to wait for process: {}", e)))?;

    info!("Agent process exited with status: {:?}", status);

    Ok(SpawnResult {
        agent_name: agent_name.to_string(),
        pid: Some(pid),
        prompt_preview: prompt_preview.to_string(),
    })
}

/// Fallback for other platforms: spawn and wait
#[cfg(not(any(unix, windows)))]
fn exec_agent_process(
    mut cmd: Command,
    agent_name: &str,
    prompt_preview: &str,
) -> Result<SpawnResult, AgentError> {
    info!("Unsupported platform: spawning agent in foreground");

    let mut child = cmd
        .spawn()
        .map_err(|e| AgentError::SpawnError(e.to_string()))?;

    let pid = child.id();
    let status = child
        .wait()
        .map_err(|e| AgentError::SpawnError(format!("Failed to wait for process: {}", e)))?;

    info!("Agent process exited with status: {:?}", status);

    Ok(SpawnResult {
        agent_name: agent_name.to_string(),
        pid: Some(pid),
        prompt_preview: prompt_preview.to_string(),
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
