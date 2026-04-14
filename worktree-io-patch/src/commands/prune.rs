use std::path::{Path, PathBuf};
use std::time::SystemTime;

use anyhow::{bail, Result};
use worktree_io::ttl::{Ttl, WorkspaceRecord, WorkspaceRegistry};
use worktree_io::{config::Config, ttl};

pub fn cmd_prune(json: bool) -> Result<()> {
    let config = Config::load()?;
    let Some(ttl) = config.workspace.ttl else {
        bail!("No workspace TTL configured. Set workspace.ttl in your config (e.g. \"7days\").");
    };
    let now = SystemTime::now();
    let mut registry = WorkspaceRegistry::load()?;
    let checked = registry.workspace.len();
    let expired = ttl::prune(&registry.workspace, &ttl, now);
    if !json && expired.is_empty() {
        eprintln!("Checked {checked} workspace(s), none expired (TTL: {ttl}).");
        return Ok(());
    }
    let expired_paths: Vec<PathBuf> = expired.iter().map(|r| r.path.clone()).collect();
    let mut total_freed: u64 = 0;
    let mut json_entries: Vec<String> = Vec::new();
    for record in &expired {
        let freed = dir_size(&record.path);
        total_freed += freed;
        if json {
            json_entries.push(entry_json(record, &ttl, freed));
        } else {
            let age = now.duration_since(record.created_at).unwrap_or_default();
            eprintln!(
                "Removing {} (age: {}, TTL: {ttl}, freed: {})",
                record.path.display(),
                humantime::format_duration(age),
                format_bytes(freed)
            );
        }
        if let Err(e) = std::fs::remove_dir_all(&record.path) {
            eprintln!("Warning: failed to remove {}: {e}", record.path.display());
        }
    }
    let ws = &mut registry.workspace;
    ws.retain(|r| !expired_paths.contains(&r.path));
    registry.save()?;
    if json {
        let entries = json_entries.join(",");
        println!("{{\"checked\":{checked},\"pruned\":[{entries}],\"total_freed_bytes\":{total_freed},\"ttl\":\"{ttl}\"}}");
    } else {
        eprintln!(
            "Pruned {} expired workspace(s). Total freed: {}.",
            expired_paths.len(),
            format_bytes(total_freed)
        );
    }
    Ok(())
}

fn entry_json(r: &WorkspaceRecord, ttl: &Ttl, freed_bytes: u64) -> String {
    let path_str = r.path.display().to_string();
    let path_esc = path_str.replace('\\', "\\\\").replace('"', "\\\"");
    let expires = r.created_at.checked_add(ttl.duration());
    let ea = expires.unwrap_or(r.created_at);
    format!(
        "{{\"path\":\"{path_esc}\",\"expired_at\":\"{}\",\"freed_bytes\":{freed_bytes}}}",
        humantime::format_rfc3339(ea)
    )
}

fn format_bytes(n: u64) -> String {
    const KB: u64 = 1_024;
    const MB: u64 = 1_024 * KB;
    const GB: u64 = 1_024 * MB;
    let (whole, frac, unit) = if n >= GB {
        (n / GB, (n % GB) * 10 / GB, "GB")
    } else if n >= MB {
        (n / MB, (n % MB) * 10 / MB, "MB")
    } else if n >= KB {
        (n / KB, (n % KB) * 10 / KB, "KB")
    } else {
        return format!("{n} B");
    };
    format!("{whole}.{frac} {unit}")
}

fn entry_size(e: &std::fs::DirEntry) -> u64 {
    let p = e.path();
    e.metadata()
        .map_or(0, |m| if m.is_dir() { dir_size(&p) } else { m.len() })
}

fn dir_size(path: &Path) -> u64 {
    std::fs::read_dir(path).map_or(0, |rd| rd.flatten().map(|e| entry_size(&e)).sum())
}

#[cfg(test)]
#[path = "prune_tests.rs"]
mod prune_tests;
