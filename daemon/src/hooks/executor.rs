use std::path::Path;
use std::time::{Duration, Instant};
use tokio::io::{AsyncReadExt as _, AsyncWriteExt as _};
use tokio::process::Command;

use super::context::HookContext;
use super::error::HookError;

/// Captured stdout/stderr is cut to this many characters before it is stored.
const MAX_CAPTURE_CHARS: usize = 64 * 1024;

/// How long to wait for the output pipes after the hook exits (a background
/// grandchild can keep them open).
const PIPE_DRAIN_GRACE: Duration = Duration::from_millis(500);

/// Result of executing a hook command
#[derive(Debug)]
pub struct HookExecResult {
    pub exit_code: i32,
    pub stdout: String,
    pub stderr: String,
    pub duration_ms: u64,
}

/// Read a pipe to the end on its own task so a chatty hook cannot fill the pipe and stall.
fn spawn_reader<R>(pipe: Option<R>) -> tokio::task::JoinHandle<String>
where
    R: tokio::io::AsyncRead + Unpin + Send + 'static,
{
    tokio::spawn(async move {
        let mut buf = Vec::new();
        if let Some(mut p) = pipe {
            drop(p.read_to_end(&mut buf).await);
        }
        String::from_utf8_lossy(&buf)
            .chars()
            .take(MAX_CAPTURE_CHARS)
            .collect()
    })
}

async fn drain(reader: tokio::task::JoinHandle<String>) -> String {
    tokio::time::timeout(PIPE_DRAIN_GRACE, reader)
        .await
        .ok()
        .and_then(Result::ok)
        .unwrap_or_default()
}

/// Execute a single hook command
pub async fn execute_hook(
    command: &str,
    context: &HookContext,
    project_path: &Path,
    timeout_secs: u64,
    pattern: &str,
) -> Result<HookExecResult, HookError> {
    let env_vars = context.to_env_vars();
    let json_input = context.to_json()?;
    let start = Instant::now();

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
        drop(stdin.write_all(json_input.as_bytes()).await);
        drop(stdin);
    }

    let stdout_reader = spawn_reader(child.stdout.take());
    let stderr_reader = spawn_reader(child.stderr.take());

    // Wait with timeout
    let wait_result = tokio::time::timeout(Duration::from_secs(timeout_secs), child.wait()).await;

    match wait_result {
        Ok(Ok(status)) => Ok(HookExecResult {
            exit_code: status.code().unwrap_or(-1i32),
            stdout: drain(stdout_reader).await,
            stderr: drain(stderr_reader).await,
            duration_ms: u64::try_from(start.elapsed().as_millis()).unwrap_or(u64::MAX),
        }),
        Ok(Err(e)) => Err(HookError::ExecutionError(format!(
            "Failed to execute hook command: {e}"
        ))),
        Err(_) => {
            // Timeout - try to kill the process
            drop(child.kill().await);
            stdout_reader.abort();
            stderr_reader.abort();
            Err(HookError::Timeout {
                pattern: pattern.to_string(),
                timeout_secs,
            })
        }
    }
}
