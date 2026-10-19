#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use std::net::{SocketAddr, TcpListener, TcpStream};
use std::process::Command;
use std::time::{Duration, Instant};

fn free_loopback_addr() -> SocketAddr {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    listener.local_addr().unwrap()
}

/// Runs the real daemon, waits for its MCP port, then sends SIGTERM.
/// The daemon must shut down on its own and exit with status 0.
#[cfg(unix)]
#[test]
fn daemon_exits_zero_on_sigterm() {
    use std::os::unix::process::ExitStatusExt as _;

    let grpc_addr = free_loopback_addr();
    let mcp_addr = free_loopback_addr();
    let log_dir = tempfile::tempdir().unwrap();
    let mut child = Command::new(env!("CARGO_BIN_EXE_centy-daemon"))
        .args([
            "--addr",
            &grpc_addr.to_string(),
            "--mcp-addr",
            &mcp_addr.to_string(),
            "--log-dir",
            log_dir.path().to_str().unwrap(),
        ])
        .stdout(std::fs::File::create(log_dir.path().join("stdout.txt")).unwrap())
        .stderr(std::fs::File::create(log_dir.path().join("stderr.txt")).unwrap())
        .spawn()
        .unwrap();

    let ready_deadline = Instant::now() + Duration::from_secs(20);
    while TcpStream::connect(mcp_addr).is_err() {
        if Instant::now() > ready_deadline {
            child.kill().unwrap();
            panic!("MCP port {mcp_addr} did not answer within 20s");
        }
        std::thread::sleep(Duration::from_millis(100));
    }

    let kill = Command::new("kill")
        .args(["-TERM", &child.id().to_string()])
        .status()
        .unwrap();
    assert!(kill.success(), "failed to send SIGTERM");

    let exit_deadline = Instant::now() + Duration::from_secs(10);
    let status = loop {
        if let Some(status) = child.try_wait().unwrap() {
            break status;
        }
        if Instant::now() > exit_deadline {
            child.kill().unwrap();
            panic!("daemon still running 10s after SIGTERM");
        }
        std::thread::sleep(Duration::from_millis(50));
    };

    assert_eq!(
        status.signal(),
        None,
        "daemon was killed by signal {:?} instead of shutting down",
        status.signal()
    );
    assert!(
        status.success(),
        "daemon exited with {status:?} after SIGTERM"
    );
}
