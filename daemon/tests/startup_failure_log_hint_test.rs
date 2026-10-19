#![allow(clippy::unwrap_used, clippy::expect_used)]

use std::process::Command;

/// A startup failure before the server binds (here: an unparsable `--addr`)
/// must still tell the user where the log file is.
#[test]
fn startup_failure_prints_log_file_location() {
    let log_dir = tempfile::tempdir().unwrap();
    let out = Command::new(env!("CARGO_BIN_EXE_centy-daemon"))
        .args([
            "--addr",
            "not-an-address",
            "--log-dir",
            log_dir.path().to_str().unwrap(),
        ])
        .output()
        .unwrap();
    assert!(!out.status.success());
    let stderr = String::from_utf8_lossy(&out.stderr);
    let expected = format!(
        "Logs: {}",
        log_dir.path().join("centy-daemon.log").display()
    );
    assert!(stderr.contains(&expected), "stderr: {stderr}");
    assert_eq!(stderr.matches("Logs:").count(), 1, "stderr: {stderr}");
}
