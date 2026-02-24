use std::path::Path;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use tokio::io::AsyncWriteExt;

const HOOK_EXECUTIONS_FILE: &str = "hook_executions.jsonl";

/// A single hook execution record, persisted to disk.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HookExecutionRecord {
    pub id: String,
    pub timestamp: DateTime<Utc>,
    pub hook_pattern: String,
    pub command: String,
    /// None when the hook timed out before exiting.
    pub exit_code: Option<i32>,
    pub stdout: String,
    pub stderr: String,
    pub duration_ms: u64,
    /// True when this was a pre-hook that returned a non-zero exit code and
    /// therefore blocked the operation.
    pub blocked_operation: bool,
    pub phase: String,
    pub item_type: String,
    pub operation: String,
    pub item_id: Option<String>,
    pub timed_out: bool,
}

/// Append a single record to the project's hook execution log.
///
/// The file lives at `<project_path>/.centy/hook_executions.jsonl`.
pub async fn append_hook_execution(
    project_path: &Path,
    record: &HookExecutionRecord,
) {
    let file_path = project_path.join(".centy").join(HOOK_EXECUTIONS_FILE);
    match tokio::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&file_path)
        .await
    {
        Ok(mut file) => {
            if let Ok(line) = serde_json::to_string(record) {
                let _ = file.write_all(format!("{line}\n").as_bytes()).await;
            }
        }
        Err(e) => {
            tracing::warn!("Failed to open hook execution log: {e}");
        }
    }
}

/// Read all hook execution records for a project, optionally filtered.
pub async fn list_hook_executions(
    project_path: &Path,
    filter: &HookExecutionFilter,
) -> Vec<HookExecutionRecord> {
    let file_path = project_path.join(".centy").join(HOOK_EXECUTIONS_FILE);
    let content = match tokio::fs::read_to_string(&file_path).await {
        Ok(c) => c,
        Err(_) => return vec![],
    };

    let mut records: Vec<HookExecutionRecord> = content
        .lines()
        .filter(|l| !l.is_empty())
        .filter_map(|line| serde_json::from_str(line).ok())
        .filter(|r: &HookExecutionRecord| filter.matches(r))
        .collect();

    // Most-recent first
    records.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));

    if let Some(limit) = filter.limit {
        records.truncate(limit as usize);
    }

    records
}

/// Find a specific execution by ID.
pub async fn get_hook_execution(
    project_path: &Path,
    execution_id: &str,
) -> Option<HookExecutionRecord> {
    let file_path = project_path.join(".centy").join(HOOK_EXECUTIONS_FILE);
    let content = tokio::fs::read_to_string(&file_path).await.ok()?;

    content
        .lines()
        .filter(|l| !l.is_empty())
        .filter_map(|line| serde_json::from_str::<HookExecutionRecord>(line).ok())
        .find(|r| r.id == execution_id)
}

/// Optional filters for listing hook executions.
#[derive(Debug, Default)]
pub struct HookExecutionFilter {
    pub phase: Option<String>,
    pub item_type: Option<String>,
    pub operation: Option<String>,
    pub item_id: Option<String>,
    pub limit: Option<u32>,
}

impl HookExecutionFilter {
    fn matches(&self, record: &HookExecutionRecord) -> bool {
        if let Some(ref phase) = self.phase {
            if !phase.is_empty() && record.phase != *phase {
                return false;
            }
        }
        if let Some(ref item_type) = self.item_type {
            if !item_type.is_empty() && record.item_type != *item_type {
                return false;
            }
        }
        if let Some(ref operation) = self.operation {
            if !operation.is_empty() && record.operation != *operation {
                return false;
            }
        }
        if let Some(ref item_id) = self.item_id {
            if !item_id.is_empty() {
                match &record.item_id {
                    Some(id) if id == item_id => {}
                    _ => return false,
                }
            }
        }
        true
    }
}
