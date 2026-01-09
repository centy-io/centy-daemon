use std::path::Path;
use std::process::{Command, Stdio};
use thiserror::Error;
use tracing::info;

use super::config::{AgentConfig, LocalLlmConfig};
use super::prompt::{LlmAction, PromptBuilder};
use crate::item::entities::issue::Issue;

/// Mode for starting the agent process
#[derive(Debug, Clone, Copy, Default)]
pub enum AgentSpawnMode {
    /// Spawn as background process, return PID (daemon mode)
    #[default]
    Background,
    /// Replace current process with agent (CLI mode)
    /// Unix: uses exec(), never returns on success
    /// Windows: falls back to waiting for process completion
    #[allow(dead_code)] // Reserved for CLI use cases
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
        None => config
            .get_default_agent()
            .ok_or(AgentError::NoDefaultAgent)?,
    };

    // Build the prompt
    let prompt_builder = PromptBuilder::new();
    let user_template = match action {
        LlmAction::Plan => agent.plan_template.as_deref(),
        LlmAction::Implement => agent.implement_template.as_deref(),
        LlmAction::Deepdive => None, // Deepdive uses default prompt only
    };
    let prompt = prompt_builder
        .build_prompt(project_path, issue, action, user_template, priority_levels)
        .await?;

    // Create preview (first 500 chars)
    let prompt_preview = PromptBuilder::preview(&prompt, 500);

    info!(
        "Starting agent '{}' for issue #{} ({}) in {:?} mode",
        agent.name,
        issue.metadata.display_number,
        action.as_str(),
        mode
    );

    // Handle stdin_prompt mode differently - pipe prompt via stdin
    if agent.stdin_prompt {
        return start_agent_with_stdin(
            project_path,
            config,
            agent,
            &prompt,
            &prompt_preview,
            extra_args,
            mode,
        );
    }

    // Build command for agents that accept prompt as argument
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
    cmd.arg(&prompt);

    // Configure stdio
    cmd.stdin(Stdio::null())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit());

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

/// Start an agent that requires stdin input (like `claude --print`)
///
/// For ReplaceProcess mode, we use a shell to pipe the prompt since exec() replaces
/// the process and we can't write to stdin afterward.
fn start_agent_with_stdin(
    project_path: &Path,
    config: &LocalLlmConfig,
    agent: &AgentConfig,
    prompt: &str,
    prompt_preview: &str,
    extra_args: Vec<String>,
    mode: AgentSpawnMode,
) -> Result<SpawnResult, AgentError> {
    use std::io::Write;

    match mode {
        AgentSpawnMode::Background => {
            // For background mode, we can spawn with piped stdin and write to it
            let mut cmd = Command::new(&agent.command);
            cmd.args(&agent.default_args);
            cmd.args(&extra_args);
            cmd.current_dir(project_path);

            for (key, value) in &config.env_vars {
                cmd.env(key, value);
            }

            cmd.stdin(Stdio::piped())
                .stdout(Stdio::inherit())
                .stderr(Stdio::inherit());

            let mut child = cmd
                .spawn()
                .map_err(|e| AgentError::SpawnError(e.to_string()))?;

            // Write prompt to stdin
            if let Some(mut stdin) = child.stdin.take() {
                stdin.write_all(prompt.as_bytes()).map_err(|e| {
                    AgentError::SpawnError(format!("Failed to write to stdin: {e}"))
                })?;
                // stdin is dropped here, closing the pipe
            }

            let pid = child.id();
            info!("Agent process spawned with PID: {:?}", pid);

            Ok(SpawnResult {
                agent_name: agent.name.clone(),
                pid: Some(pid),
                prompt_preview: prompt_preview.to_string(),
            })
        }
        AgentSpawnMode::ReplaceProcess => {
            // For exec mode, we need to use a shell to pipe stdin since we can't
            // write to stdin after exec() replaces the process.
            // We use `cat <<'PROMPT_EOF' | command args...` to avoid shell escaping issues.
            exec_agent_with_stdin(
                project_path,
                config,
                agent,
                prompt,
                prompt_preview,
                extra_args,
            )
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

/// Unix implementation: exec a shell command that pipes stdin to the agent
#[cfg(unix)]
fn exec_agent_with_stdin(
    project_path: &Path,
    config: &LocalLlmConfig,
    agent: &AgentConfig,
    prompt: &str,
    _prompt_preview: &str,
    extra_args: Vec<String>,
) -> Result<SpawnResult, AgentError> {
    use std::os::unix::process::CommandExt;

    info!("Executing agent with stdin via shell - replacing current process");

    // Build the command string for the shell
    // We use a heredoc to avoid escaping issues with the prompt
    let mut all_args = agent.default_args.clone();
    all_args.extend(extra_args);

    // Shell-escape the command and args
    let escaped_command = shell_escape::escape(agent.command.clone().into());
    let escaped_args: Vec<_> = all_args
        .iter()
        .map(|a| shell_escape::escape(a.clone().into()).to_string())
        .collect();

    // Build the shell command with a heredoc
    // Using <<'EOF' (quoted) prevents variable expansion in the heredoc
    let shell_cmd = format!(
        "cat <<'CENTY_PROMPT_EOF' | {} {}\n{}\nCENTY_PROMPT_EOF",
        escaped_command,
        escaped_args.join(" "),
        prompt
    );

    let mut cmd = Command::new("sh");
    cmd.args(["-c", &shell_cmd]);
    cmd.current_dir(project_path);

    // Set environment variables
    for (key, value) in &config.env_vars {
        cmd.env(key, value);
    }

    cmd.stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit());

    // exec() never returns on success
    let err = cmd.exec();
    Err(AgentError::SpawnError(format!("exec() failed: {err}")))
}

/// Windows implementation: spawn shell with piped stdin and wait
#[cfg(windows)]
fn exec_agent_with_stdin(
    project_path: &Path,
    config: &LocalLlmConfig,
    agent: &AgentConfig,
    prompt: &str,
    _prompt_preview: &str,
    extra_args: Vec<String>,
) -> Result<SpawnResult, AgentError> {
    use std::io::Write;

    info!("Windows: spawning agent with stdin in foreground");

    let mut cmd = Command::new(&agent.command);
    cmd.args(&agent.default_args);
    cmd.args(&extra_args);
    cmd.current_dir(project_path);

    for (key, value) in &config.env_vars {
        cmd.env(key, value);
    }

    cmd.stdin(Stdio::piped())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit());

    let mut child = cmd
        .spawn()
        .map_err(|e| AgentError::SpawnError(e.to_string()))?;

    let pid = child.id();

    // Write prompt to stdin
    if let Some(mut stdin) = child.stdin.take() {
        stdin
            .write_all(prompt.as_bytes())
            .map_err(|e| AgentError::SpawnError(format!("Failed to write to stdin: {e}")))?;
    }

    // Wait for completion
    let status = child
        .wait()
        .map_err(|e| AgentError::SpawnError(format!("Failed to wait for process: {e}")))?;

    info!("Agent process exited with status: {:?}", status);

    Ok(SpawnResult {
        agent_name: agent.name.clone(),
        pid: Some(pid),
        prompt_preview: prompt_preview.to_string(),
    })
}

/// Fallback for other platforms
#[cfg(not(any(unix, windows)))]
fn exec_agent_with_stdin(
    project_path: &Path,
    config: &LocalLlmConfig,
    agent: &AgentConfig,
    prompt: &str,
    _prompt_preview: &str,
    extra_args: Vec<String>,
) -> Result<SpawnResult, AgentError> {
    use std::io::Write;

    info!("Unsupported platform: spawning agent with stdin in foreground");

    let mut cmd = Command::new(&agent.command);
    cmd.args(&agent.default_args);
    cmd.args(&extra_args);
    cmd.current_dir(project_path);

    for (key, value) in &config.env_vars {
        cmd.env(key, value);
    }

    cmd.stdin(Stdio::piped())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit());

    let mut child = cmd
        .spawn()
        .map_err(|e| AgentError::SpawnError(e.to_string()))?;

    let pid = child.id();

    if let Some(mut stdin) = child.stdin.take() {
        stdin
            .write_all(prompt.as_bytes())
            .map_err(|e| AgentError::SpawnError(format!("Failed to write to stdin: {e}")))?;
    }

    let status = child
        .wait()
        .map_err(|e| AgentError::SpawnError(format!("Failed to wait for process: {e}")))?;

    info!("Agent process exited with status: {:?}", status);

    Ok(SpawnResult {
        agent_name: agent.name.clone(),
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::llm::config::AgentType;

    #[test]
    fn test_check_agent_not_found() {
        let agent = AgentConfig {
            agent_type: AgentType::Custom,
            name: "nonexistent-agent-xyz".to_string(),
            command: "nonexistent-command-xyz".to_string(),
            default_args: vec![],
            stdin_prompt: false,
            plan_template: None,
            implement_template: None,
        };

        assert!(!check_agent_available(&agent));
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
