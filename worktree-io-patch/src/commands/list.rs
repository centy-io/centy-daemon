use std::time::SystemTime;

use anyhow::Result;
use worktree_io::config::Config;
use worktree_io::ttl::{self, Ttl, WorkspaceRecord, WorkspaceRegistry};

pub fn cmd_list(json: bool) -> Result<()> {
    let config = Config::load()?;
    let ttl = config.workspace.ttl;
    let now = SystemTime::now();
    let registry = WorkspaceRegistry::load()?;
    if json {
        print_json(&registry.workspace, ttl.as_ref(), now);
    } else {
        print_human(&registry.workspace, ttl.as_ref(), now);
    }
    Ok(())
}

fn print_json(workspaces: &[WorkspaceRecord], ttl: Option<&Ttl>, now: SystemTime) {
    let ttl_str = ttl.map_or_else(|| "null".to_owned(), |t| format!("\"{t}\""));
    let entries: Vec<String> = workspaces
        .iter()
        .map(|r| {
            let path_str = r.path.display().to_string();
            let path_esc = path_str.replace('\\', "\\\\").replace('"', "\\\"");
            let created = humantime::format_rfc3339(r.created_at);
            let expired = ttl.is_some_and(|t| ttl::is_expired(r, t, now));
            format!(
                "{{\"path\":\"{path_esc}\",\"created_at\":\"{created}\",\"expired\":{expired}}}"
            )
        })
        .collect();
    println!(
        "{{\"ttl\":{ttl_str},\"workspaces\":[{}]}}",
        entries.join(",")
    );
}

fn print_human(workspaces: &[WorkspaceRecord], ttl: Option<&Ttl>, now: SystemTime) {
    if workspaces.is_empty() {
        eprintln!("No workspaces registered.");
        return;
    }
    match ttl {
        Some(t) => eprintln!("{} workspace(s) registered (TTL: {t}):", workspaces.len()),
        None => eprintln!("{} workspace(s) registered:", workspaces.len()),
    }
    for r in workspaces {
        let age = now.duration_since(r.created_at).unwrap_or_default();
        let age_str = humantime::format_duration(age);
        match ttl {
            Some(t) if ttl::is_expired(r, t, now) => {
                eprintln!("  {}  created {}  EXPIRED", r.path.display(), age_str);
            }
            Some(t) => {
                let remaining = t.duration().saturating_sub(age);
                eprintln!(
                    "  {}  created {}  expires in {}",
                    r.path.display(),
                    age_str,
                    humantime::format_duration(remaining),
                );
            }
            None => {
                eprintln!("  {}  created {}", r.path.display(), age_str);
            }
        }
    }
}
