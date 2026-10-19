use std::path::{Path, PathBuf};

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use tokio::io::AsyncWriteExt as _;

use crate::utils::get_centy_path;

/// The `.local.` infix matches the managed `.centy/.gitignore` pattern `*.local.*`, so hook
/// output (which may hold secrets) is never committed.
pub const HOOK_EXECUTIONS_FILE: &str = "hook_executions.local.jsonl";

/// The log keeps only the newest this-many records.
const MAX_RECORDS: usize = 1000;

/// A single hook execution record, persisted as one JSON line.
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
    /// True when a pre-hook failed or timed out and so blocked the operation.
    pub blocked_operation: bool,
    pub phase: String,
    pub item_type: String,
    pub operation: String,
    pub item_id: Option<String>,
    pub timed_out: bool,
}

/// Optional filters for listing hook executions. Empty strings mean "no filter".
#[derive(Debug, Default)]
pub struct HookExecutionFilter {
    pub phase: String,
    pub item_type: String,
    pub operation: String,
    pub item_id: String,
    /// 0 means no limit.
    pub limit: usize,
}

impl HookExecutionFilter {
    fn matches(&self, r: &HookExecutionRecord) -> bool {
        let eq = |want: &str, have: &str| want.is_empty() || want == have;
        eq(&self.phase, &r.phase)
            && eq(&self.item_type, &r.item_type)
            && eq(&self.operation, &r.operation)
            && eq(&self.item_id, r.item_id.as_deref().unwrap_or(""))
    }
}

fn log_path(project_path: &Path) -> PathBuf {
    get_centy_path(project_path).join(HOOK_EXECUTIONS_FILE)
}

/// Append a record to `<project>/.centy/hook_executions.local.jsonl`. Failures are logged, never raised:
/// history must not break a hook run.
pub async fn append_hook_execution(project_path: &Path, record: &HookExecutionRecord) {
    let Ok(mut line) = serde_json::to_string(record) else {
        return;
    };
    line.push('\n');
    // One write_all of a whole line: O_APPEND keeps concurrent async hooks from interleaving.
    let result = async {
        let mut file = tokio::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(log_path(project_path))
            .await?;
        file.write_all(line.as_bytes()).await?;
        drop(file);
        trim_to_newest(&log_path(project_path)).await
    }
    .await;
    if let Err(e) = result {
        tracing::warn!("Failed to write hook execution log: {e}");
    }
}

/// Keep only the newest `MAX_RECORDS` lines.
// ponytail: re-reads the whole file on every append (bounded by MAX_RECORDS lines of capped
// output) and a concurrent append during the rewrite can be lost; add a size check or a lock
// if hook volume makes either matter.
async fn trim_to_newest(path: &Path) -> std::io::Result<()> {
    let content = tokio::fs::read_to_string(path).await?;
    let lines: Vec<&str> = content.lines().collect();
    let Some(excess) = lines.len().checked_sub(MAX_RECORDS).filter(|n| *n > 0) else {
        return Ok(());
    };
    let kept = lines.iter().skip(excess).fold(String::new(), |mut acc, l| {
        acc.push_str(l);
        acc.push('\n');
        acc
    });
    let tmp = path.with_extension("jsonl.tmp");
    tokio::fs::write(&tmp, kept).await?;
    tokio::fs::rename(&tmp, path).await
}

async fn read_all(project_path: &Path) -> Vec<HookExecutionRecord> {
    let Ok(content) = tokio::fs::read_to_string(log_path(project_path)).await else {
        return Vec::new();
    };
    content
        .lines()
        .filter_map(|l| serde_json::from_str(l).ok())
        .collect()
}

/// Matching records, newest first.
pub async fn list_hook_executions(
    project_path: &Path,
    filter: &HookExecutionFilter,
) -> Vec<HookExecutionRecord> {
    let mut records: Vec<_> = read_all(project_path)
        .await
        .into_iter()
        .filter(|r| filter.matches(r))
        .collect();
    records.sort_by_key(|r| std::cmp::Reverse(r.timestamp));
    if filter.limit > 0 {
        records.truncate(filter.limit);
    }
    records
}

pub async fn get_hook_execution(
    project_path: &Path,
    execution_id: &str,
) -> Option<HookExecutionRecord> {
    read_all(project_path)
        .await
        .into_iter()
        .find(|r| r.id == execution_id)
}
