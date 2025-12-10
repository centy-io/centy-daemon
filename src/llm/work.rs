use serde::{Deserialize, Serialize};
use std::path::Path;
use thiserror::Error;
use tokio::fs;

use super::prompt::LlmAction;
use crate::utils::{get_centy_path, now_iso};

#[derive(Error, Debug)]
pub enum WorkTrackingError {
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("JSON error: {0}")]
    JsonError(#[from] serde_json::Error),
}

/// Stored work session info
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LlmWorkSession {
    pub issue_id: String,
    pub display_number: u32,
    pub issue_title: String,
    pub agent_name: String,
    pub action: String, // "plan" or "implement"
    pub started_at: String,
    pub pid: Option<u32>,
}

const WORK_FILE: &str = "llm-work.json";

/// Get path to work tracking file
fn get_work_file_path(project_path: &Path) -> std::path::PathBuf {
    get_centy_path(project_path).join(WORK_FILE)
}

/// Read current work session (if any)
pub async fn read_work_session(
    project_path: &Path,
) -> Result<Option<LlmWorkSession>, WorkTrackingError> {
    let path = get_work_file_path(project_path);

    if !path.exists() {
        return Ok(None);
    }

    let content = fs::read_to_string(&path).await?;
    let session: LlmWorkSession = serde_json::from_str(&content)?;
    Ok(Some(session))
}

/// Record a new work session
pub async fn record_work_session(
    project_path: &Path,
    issue_id: &str,
    display_number: u32,
    issue_title: &str,
    agent_name: &str,
    action: LlmAction,
    pid: Option<u32>,
) -> Result<LlmWorkSession, WorkTrackingError> {
    let session = LlmWorkSession {
        issue_id: issue_id.to_string(),
        display_number,
        issue_title: issue_title.to_string(),
        agent_name: agent_name.to_string(),
        action: action.as_str().to_string(),
        started_at: now_iso(),
        pid,
    };

    let path = get_work_file_path(project_path);
    let content = serde_json::to_string_pretty(&session)?;
    fs::write(&path, content).await?;

    Ok(session)
}

/// Clear the work session
pub async fn clear_work_session(project_path: &Path) -> Result<(), WorkTrackingError> {
    let path = get_work_file_path(project_path);

    if path.exists() {
        fs::remove_file(&path).await?;
    }

    Ok(())
}

/// Check if a PID is still running (Unix)
#[cfg(unix)]
pub fn is_process_running(pid: u32) -> bool {
    use std::process::Command;

    Command::new("kill")
        .args(["-0", &pid.to_string()])
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

/// Check if a PID is still running (Windows)
#[cfg(windows)]
pub fn is_process_running(pid: u32) -> bool {
    use std::process::Command;

    Command::new("tasklist")
        .args(["/FI", &format!("PID eq {}", pid)])
        .output()
        .map(|o| {
            let output = String::from_utf8_lossy(&o.stdout);
            output.contains(&pid.to_string())
        })
        .unwrap_or(false)
}

/// Fallback for other platforms
#[cfg(not(any(unix, windows)))]
pub fn is_process_running(_pid: u32) -> bool {
    // On other platforms, assume the process might be running
    true
}

/// Check if there's active work and if the process is still running
pub async fn get_active_work_status(
    project_path: &Path,
) -> Result<Option<(LlmWorkSession, bool)>, WorkTrackingError> {
    match read_work_session(project_path).await? {
        Some(session) => {
            let is_running = session.pid.map(is_process_running).unwrap_or(false);
            Ok(Some((session, is_running)))
        }
        None => Ok(None),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_work_session_lifecycle() {
        let temp_dir = TempDir::new().unwrap();
        let project_path = temp_dir.path();

        // Create .centy directory
        fs::create_dir_all(get_centy_path(project_path))
            .await
            .unwrap();

        // Initially, no work session
        let session = read_work_session(project_path).await.unwrap();
        assert!(session.is_none());

        // Record a work session
        let recorded = record_work_session(
            project_path,
            "test-uuid-123",
            42,
            "Fix auth bug",
            "claude",
            LlmAction::Implement,
            Some(12345),
        )
        .await
        .unwrap();

        assert_eq!(recorded.issue_id, "test-uuid-123");
        assert_eq!(recorded.display_number, 42);
        assert_eq!(recorded.issue_title, "Fix auth bug");
        assert_eq!(recorded.agent_name, "claude");
        assert_eq!(recorded.action, "implement");
        assert_eq!(recorded.pid, Some(12345));

        // Read it back
        let read_session = read_work_session(project_path).await.unwrap().unwrap();
        assert_eq!(read_session.issue_id, "test-uuid-123");
        assert_eq!(read_session.agent_name, "claude");

        // Clear the session
        clear_work_session(project_path).await.unwrap();

        // Should be gone now
        let session = read_work_session(project_path).await.unwrap();
        assert!(session.is_none());
    }

    #[tokio::test]
    async fn test_clear_nonexistent_session() {
        let temp_dir = TempDir::new().unwrap();
        let project_path = temp_dir.path();

        // Create .centy directory
        fs::create_dir_all(get_centy_path(project_path))
            .await
            .unwrap();

        // Clearing a non-existent session should not error
        let result = clear_work_session(project_path).await;
        assert!(result.is_ok());
    }

    #[test]
    fn test_work_session_serialization() {
        let session = LlmWorkSession {
            issue_id: "test-id".to_string(),
            display_number: 1,
            issue_title: "Test Issue".to_string(),
            agent_name: "claude".to_string(),
            action: "plan".to_string(),
            started_at: "2025-01-15T10:00:00Z".to_string(),
            pid: Some(12345),
        };

        let json = serde_json::to_string(&session).unwrap();
        assert!(json.contains("\"issueId\":"));
        assert!(json.contains("\"displayNumber\":"));
        assert!(json.contains("\"agentName\":"));

        let deserialized: LlmWorkSession = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.issue_id, "test-id");
        assert_eq!(deserialized.pid, Some(12345));
    }

    #[cfg(unix)]
    #[test]
    fn test_is_process_running_invalid_pid() {
        // PID 0 is special and should not be "running" in the usual sense
        // A very high PID is likely not running
        assert!(!is_process_running(999999999));
    }

    #[cfg(unix)]
    #[test]
    fn test_is_process_running_current_process() {
        // Current process should be running
        let current_pid = std::process::id();
        assert!(is_process_running(current_pid));
    }
}
