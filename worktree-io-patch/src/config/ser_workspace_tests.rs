use super::*;

#[test]
fn test_workspace_section_present_in_default() {
    let s = Config::default().to_toml_with_comments();
    assert!(s.contains("[workspace]"));
    assert!(s.contains("# Workspace lifecycle configuration."));
    assert!(!s.contains("ttl ="));
    assert!(!s.contains("auto_prune ="));
}

#[test]
fn test_workspace_section_with_ttl() {
    let mut c = Config::default();
    c.set_value("workspace.ttl", "7days").unwrap();
    let s = c.to_toml_with_comments();
    assert!(s.contains("[workspace]"));
    assert!(s.contains("ttl ="));
    assert!(s.contains("# Maximum age of a workspace before it is considered expired."));
}

#[test]
fn test_workspace_ttl_round_trips() {
    let mut c = Config::default();
    c.set_value("workspace.ttl", "7days").unwrap();
    let s = c.to_toml_with_comments();
    let parsed: Config = toml::from_str(&s).unwrap();
    assert!(parsed.workspace.ttl.is_some());
}

#[test]
fn test_auto_prune_absent_by_default() {
    let s = Config::default().to_toml_with_comments();
    assert!(!s.contains("auto_prune ="));
}

#[test]
fn test_auto_prune_present_when_true() {
    let mut c = Config::default();
    c.set_value("workspace.auto_prune", "true").unwrap();
    let s = c.to_toml_with_comments();
    assert!(s.contains("auto_prune = true"));
    assert!(s.contains("# When true, expired worktrees are pruned each time `open` is invoked."));
}

#[test]
fn test_auto_prune_round_trips() {
    let mut c = Config::default();
    c.set_value("workspace.auto_prune", "true").unwrap();
    let s = c.to_toml_with_comments();
    let parsed: Config = toml::from_str(&s).unwrap();
    assert!(parsed.workspace.auto_prune);
}

#[test]
fn test_temp_absent_by_default() {
    let s = Config::default().to_toml_with_comments();
    assert!(!s.contains("temp ="));
}

#[test]
fn test_temp_present_when_true() {
    let mut c = Config::default();
    c.set_value("workspace.temp", "true").unwrap();
    let s = c.to_toml_with_comments();
    assert!(s.contains("temp = true"));
    assert!(s.contains("# When true, worktrees are stored under the OS temp directory."));
}

#[test]
fn test_temp_round_trips() {
    let mut c = Config::default();
    c.set_value("workspace.temp", "true").unwrap();
    let s = c.to_toml_with_comments();
    let parsed: Config = toml::from_str(&s).unwrap();
    assert!(parsed.workspace.temp);
}
