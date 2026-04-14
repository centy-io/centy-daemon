use std::path::PathBuf;
use std::time::{Duration, SystemTime};

use super::*;

fn past(secs: u64) -> SystemTime {
    SystemTime::now()
        .checked_sub(Duration::from_secs(secs))
        .expect("time subtraction overflowed")
}

fn future(secs: u64) -> SystemTime {
    SystemTime::now()
        .checked_add(Duration::from_secs(secs))
        .expect("time addition overflowed")
}

// ── Ttl ──────────────────────────────────────────────────────────────────────

#[test]
fn test_ttl_new_and_duration() {
    let d = Duration::from_secs(3600);
    assert_eq!(Ttl::new(d).duration(), d);
}

#[test]
fn test_ttl_display_round_trips() {
    let ttl = Ttl::new(Duration::from_secs(7 * 24 * 3600));
    let parsed: Ttl = ttl.to_string().parse().unwrap();
    assert_eq!(parsed.duration(), ttl.duration());
}

#[test]
fn test_ttl_parse_valid() {
    let ttl: Ttl = "1day".parse().unwrap();
    assert_eq!(ttl.duration(), Duration::from_secs(86_400));
}

#[test]
fn test_ttl_parse_invalid() {
    assert!("not-a-duration".parse::<Ttl>().is_err());
}

// ── is_expired ───────────────────────────────────────────────────────────────

#[test]
fn test_is_expired_when_age_exceeds_ttl() {
    let ttl = Ttl::new(Duration::from_secs(60));
    let r = WorkspaceRecord {
        path: PathBuf::new(),
        created_at: past(120),
    };
    assert!(is_expired(&r, &ttl, SystemTime::now()));
}

#[test]
fn test_is_not_expired_when_young() {
    let ttl = Ttl::new(Duration::from_secs(3600));
    let r = WorkspaceRecord {
        path: PathBuf::new(),
        created_at: past(60),
    };
    assert!(!is_expired(&r, &ttl, SystemTime::now()));
}

#[test]
fn test_is_not_expired_when_created_in_future() {
    let ttl = Ttl::new(Duration::from_secs(60));
    let r = WorkspaceRecord {
        path: PathBuf::new(),
        created_at: future(30),
    };
    assert!(!is_expired(&r, &ttl, SystemTime::now()));
}

#[test]
fn test_is_expired_at_exact_boundary() {
    let ttl = Ttl::new(Duration::from_secs(100));
    let now = SystemTime::now();
    let r = WorkspaceRecord {
        path: PathBuf::new(),
        created_at: now.checked_sub(Duration::from_secs(100)).unwrap(),
    };
    assert!(is_expired(&r, &ttl, now));
}
