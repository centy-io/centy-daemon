use std::path::Path;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::process::Command;

use super::context::HookContext;
use super::error::HookError;

/// Result of executing a hook command
#[derive(Debug)]
#[allow(dead_code)]
pub struct HookExecResult {
    pub exit_code: i32,
    pub stdout: String,
    pub stderr: String,
}

/// Execute a single hook command
#[allow(unknown_lints, max_lines_per_function)]
pub async fn execute_hook(
    command: &str,
    context: &HookContext,
    project_path: &Path,
    timeout_secs: u64,
    pattern: &str,
) -> Result<HookExecResult, HookError> {
    let env_vars = context.to_env_vars();
    let json_input = context.to_json()?;

    let mut child = Command::new("bash")
        .arg("-c")
        .arg(command)
        .current_dir(project_path.join(".centy"))
        .envs(env_vars)
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()?;

    // Write JSON to stdin
    if let Some(mut stdin) = child.stdin.take() {
        // Ignore write errors - the process may have exited
        let _ = stdin.write_all(json_input.as_bytes()).await;
        drop(stdin);
    }

    // Take stdout/stderr handles before waiting
    let mut stdout_handle = child.stdout.take();
    let mut stderr_handle = child.stderr.take();

    // Wait with timeout
    let wait_result =
        tokio::time::timeout(std::time::Duration::from_secs(timeout_secs), child.wait()).await;

    match wait_result {
        Ok(Ok(status)) => {
            // Read stdout and stderr
            let mut stdout_buf = Vec::new();
            let mut stderr_buf = Vec::new();
            if let Some(ref mut stdout) = stdout_handle {
                let _ = stdout.read_to_end(&mut stdout_buf).await;
            }
            if let Some(ref mut stderr) = stderr_handle {
                let _ = stderr.read_to_end(&mut stderr_buf).await;
            }

            Ok(HookExecResult {
                exit_code: status.code().unwrap_or(-1),
                stdout: String::from_utf8_lossy(&stdout_buf).to_string(),
                stderr: String::from_utf8_lossy(&stderr_buf).to_string(),
            })
        }
        Ok(Err(e)) => Err(HookError::ExecutionError(format!(
            "Failed to execute hook command: {e}"
        ))),
        Err(_) => {
            // Timeout - try to kill the process
            let _ = child.kill().await;
            Err(HookError::Timeout {
                pattern: pattern.to_string(),
                timeout_secs,
            })
        }
    }
}
