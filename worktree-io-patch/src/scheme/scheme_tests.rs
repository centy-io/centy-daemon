use super::*;
#[test]
fn test_display_installed() {
    let s = SchemeStatus::Installed {
        path: "/Applications/Foo.app".into(),
    };
    assert_eq!(s.to_string(), "Installed at /Applications/Foo.app");
}
#[test]
fn test_display_not_installed() {
    assert_eq!(SchemeStatus::NotInstalled.to_string(), "Not installed");
}
#[test]
fn test_status_does_not_panic() {
    let _ = status();
}
