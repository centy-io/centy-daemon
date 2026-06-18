use super::helpers::applescript_quoted;
use super::launch_agent::launch_agent_plist_content;
use super::APPLET_ICNS;
#[test]
fn test_applet_icns_is_valid_icns() {
    // ICNS files begin with the 4-byte magic "icns"
    assert!(
        APPLET_ICNS.starts_with(b"icns"),
        "APPLET_ICNS does not start with ICNS magic bytes"
    );
    assert!(!APPLET_ICNS.is_empty(), "APPLET_ICNS must not be empty");
}

#[test]
fn test_applescript_quoted_no_special() {
    assert_eq!(
        applescript_quoted("/usr/bin/worktree"),
        "\"/usr/bin/worktree\""
    );
}
#[test]
fn test_applescript_quoted_backslash() {
    assert_eq!(applescript_quoted("C:\\foo"), "\"C:\\\\foo\"");
}
#[test]
fn test_applescript_quoted_double_quote() {
    assert_eq!(applescript_quoted("say \"hi\""), "\"say \\\"hi\\\"\"");
}
#[test]
fn test_launch_agent_plist_content_label() {
    let app = std::path::Path::new("/Users/test/Applications/WorktreeRunner.app");
    let content = launch_agent_plist_content(app);
    assert!(content.contains("io.worktree.runner"));
}

#[test]
fn test_launch_agent_plist_content_lsregister_flag() {
    let app = std::path::Path::new("/Users/test/Applications/WorktreeRunner.app");
    let content = launch_agent_plist_content(app);
    assert!(content.contains("-f"));
    assert!(content.contains("lsregister"));
}

#[test]
fn test_launch_agent_plist_content_run_at_load() {
    let app = std::path::Path::new("/Users/test/Applications/WorktreeRunner.app");
    let content = launch_agent_plist_content(app);
    assert!(content.contains("<true/>"));
    assert!(content.contains("RunAtLoad"));
}

#[test]
fn test_launch_agent_plist_content_app_path() {
    let app = std::path::Path::new("/Users/test/Applications/WorktreeRunner.app");
    let content = launch_agent_plist_content(app);
    assert!(content.contains("/Users/test/Applications/WorktreeRunner.app"));
}

#[test]
fn test_launch_agent_plist_content_start_interval() {
    let app = std::path::Path::new("/Users/test/Applications/WorktreeRunner.app");
    let content = launch_agent_plist_content(app);
    assert!(
        content.contains("StartInterval"),
        "plist must contain StartInterval"
    );
    assert!(
        content.contains("3600"),
        "StartInterval must be 3600 seconds"
    );
}
