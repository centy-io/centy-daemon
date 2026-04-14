use std::path::PathBuf;
use std::time::{Duration, SystemTime};

use serde::{Deserialize, Serialize};

use super::*;

#[derive(Serialize, Deserialize)]
struct TtlHelper {
    ttl: Ttl,
}

#[derive(Serialize, Deserialize)]
struct RecordHelper {
    workspace: Vec<WorkspaceRecord>,
}

#[test]
fn test_ttl_serde_round_trip() {
    let original = Ttl::new(Duration::from_secs(7 * 24 * 3600));
    let s = toml::to_string(&TtlHelper { ttl: original }).unwrap();
    let parsed: TtlHelper = toml::from_str(&s).unwrap();
    assert_eq!(parsed.ttl.duration(), original.duration());
}

#[test]
fn test_workspace_registry_serde_empty() {
    let r = WorkspaceRegistry::default();
    let s = toml::to_string(&r).unwrap();
    let parsed: WorkspaceRegistry = toml::from_str(&s).unwrap();
    assert!(parsed.workspace.is_empty());
}

#[test]
fn test_workspace_registry_serde_with_entries() {
    let mut r = WorkspaceRegistry::default();
    r.register(PathBuf::from("/tmp/ws1"));
    r.register(PathBuf::from("/tmp/ws2"));
    let s = toml::to_string(&r).unwrap();
    let parsed: WorkspaceRegistry = toml::from_str(&s).unwrap();
    assert_eq!(parsed.workspace.len(), 2);
}

#[test]
fn test_workspace_record_serde_round_trip() {
    let created_at = SystemTime::now()
        .checked_sub(Duration::from_secs(3600))
        .unwrap();
    let h = RecordHelper {
        workspace: vec![WorkspaceRecord {
            path: PathBuf::from("/tmp/my-workspace"),
            created_at,
        }],
    };
    let s = toml::to_string(&h).unwrap();
    let parsed: RecordHelper = toml::from_str(&s).unwrap();
    assert_eq!(parsed.workspace[0].path, PathBuf::from("/tmp/my-workspace"));
    // Timestamps round-trip to second precision via RFC3339
    let to_secs = |t: SystemTime| t.duration_since(SystemTime::UNIX_EPOCH).unwrap().as_secs();
    assert_eq!(
        to_secs(h.workspace[0].created_at),
        to_secs(parsed.workspace[0].created_at),
    );
}
