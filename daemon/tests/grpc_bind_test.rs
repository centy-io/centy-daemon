#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use std::process::Command;
use std::time::{Duration, Instant};

/// Runs the real daemon with a gRPC address that is already taken.
/// The daemon must exit non-zero and its stderr must name the address.
#[test]
fn daemon_exits_with_error_when_grpc_port_is_in_use() {
    let holder = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
    let grpc_addr = holder.local_addr().unwrap();
    let log_dir = tempfile::tempdir().unwrap();
    let stderr_path = log_dir.path().join("stderr.txt");
    let mut child = Command::new(env!("CARGO_BIN_EXE_centy-daemon"))
        .args([
            "--addr",
            &grpc_addr.to_string(),
            "--mcp-addr",
            "127.0.0.1:0",
            "--log-dir",
            log_dir.path().to_str().unwrap(),
        ])
        .stdout(std::fs::File::create(log_dir.path().join("stdout.txt")).unwrap())
        .stderr(std::fs::File::create(&stderr_path).unwrap())
        .spawn()
        .unwrap();

    let deadline = Instant::now() + Duration::from_secs(5);
    let status = loop {
        if let Some(status) = child.try_wait().unwrap() {
            break status;
        }
        if Instant::now() > deadline {
            child.kill().unwrap();
            panic!("daemon still running after 5s with gRPC port {grpc_addr} in use");
        }
        std::thread::sleep(Duration::from_millis(50));
    };

    let stderr = std::fs::read_to_string(&stderr_path).unwrap();
    assert!(!status.success(), "daemon exited 0 with gRPC port in use");
    assert!(
        stderr.contains(&grpc_addr.to_string()),
        "stderr does not name the gRPC address {grpc_addr}: {stderr}"
    );
}
