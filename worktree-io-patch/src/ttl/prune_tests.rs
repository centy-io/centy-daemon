use std::path::PathBuf;
use std::time::{Duration, SystemTime};

use super::*;

fn past(secs: u64) -> SystemTime {
    SystemTime::now()
        .checked_sub(Duration::from_secs(secs))
        .expect("time subtraction overflowed")
}

fn record(path: PathBuf, created_at: SystemTime) -> WorkspaceRecord {
    WorkspaceRecord { path, created_at }
}

#[test]
fn test_prune_returns_expired_with_existing_path() {
    let dir = tempfile::tempdir().unwrap();
    let ttl = Ttl::new(Duration::from_secs(60));
    let records = vec![record(dir.path().to_path_buf(), past(120))];
    let expired = prune(&records, &ttl, SystemTime::now());
    assert_eq!(expired.len(), 1);
    assert_eq!(expired[0].path, dir.path());
}

#[test]
fn test_prune_skips_missing_paths() {
    let ttl = Ttl::new(Duration::from_secs(60));
    let records = vec![record(PathBuf::from("/nonexistent/path/xyz"), past(120))];
    assert!(prune(&records, &ttl, SystemTime::now()).is_empty());
}

#[test]
fn test_prune_skips_unexpired() {
    let dir = tempfile::tempdir().unwrap();
    let ttl = Ttl::new(Duration::from_secs(3600));
    let records = vec![record(dir.path().to_path_buf(), past(60))];
    assert!(prune(&records, &ttl, SystemTime::now()).is_empty());
}

#[test]
fn test_prune_mixed_records() {
    let expired_dir = tempfile::tempdir().unwrap();
    let fresh_dir = tempfile::tempdir().unwrap();
    let ttl = Ttl::new(Duration::from_secs(600));
    let records = vec![
        record(expired_dir.path().to_path_buf(), past(1200)),
        record(fresh_dir.path().to_path_buf(), past(60)),
        record(PathBuf::from("/no/such/path"), past(1200)),
    ];
    let expired = prune(&records, &ttl, SystemTime::now());
    assert_eq!(expired.len(), 1);
    assert_eq!(expired[0].path, expired_dir.path());
}
