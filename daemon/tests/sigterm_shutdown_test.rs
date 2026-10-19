#![cfg(unix)]
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use std::net::{SocketAddr, TcpListener, TcpStream};
use std::os::unix::process::ExitStatusExt as _;
use std::process::{Child, Command};
use std::time::{Duration, Instant};

fn free_loopback_addr() -> SocketAddr {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    listener.local_addr().unwrap()
}

/// Kills and reaps the daemon when the test ends, including on panic.
struct ChildGuard(Child);

impl Drop for ChildGuard {
    fn drop(&mut self) {
        // The child may have exited already; a failed kill or wait is expected then.
        drop(self.0.kill());
        drop(self.0.wait());
    }
}

/// Runs the real daemon, waits for its MCP port, then sends SIGTERM.
/// The daemon must shut down on its own and exit with status 0.
#[test]
fn daemon_exits_zero_on_sigterm() {
    let grpc_addr = free_loopback_addr();
    let mcp_addr = free_loopback_addr();
    let log_dir = tempfile::tempdir().unwrap();
    let home_dir = tempfile::tempdir().unwrap();
    let child = Command::new(env!("CARGO_BIN_EXE_centy-daemon"))
        .args([
            "--addr",
            &grpc_addr.to_string(),
            "--mcp-addr",
            &mcp_addr.to_string(),
            "--log-dir",
            log_dir.path().to_str().unwrap(),
        ])
        .env("HOME", home_dir.path())
        .stdout(std::fs::File::create(log_dir.path().join("stdout.txt")).unwrap())
        .stderr(std::fs::File::create(log_dir.path().join("stderr.txt")).unwrap())
        .spawn()
        .unwrap();
    let mut daemon = ChildGuard(child);

    let ready_deadline = Instant::now() + Duration::from_secs(20);
    loop {
        if let Some(status) = daemon.0.try_wait().unwrap() {
            panic!("daemon exited with {status:?} before MCP port {mcp_addr} answered");
        }
        if TcpStream::connect(mcp_addr).is_ok() {
            break;
        }
        assert!(
            Instant::now() <= ready_deadline,
            "MCP port {mcp_addr} did not answer within 20s"
        );
        std::thread::sleep(Duration::from_millis(100));
    }

    let kill = Command::new("kill")
        .args(["-TERM", &daemon.0.id().to_string()])
        .status()
        .unwrap();
    assert!(kill.success(), "failed to send SIGTERM");

    let exit_deadline = Instant::now() + Duration::from_secs(10);
    let status = loop {
        if let Some(status) = daemon.0.try_wait().unwrap() {
            break status;
        }
        assert!(
            Instant::now() <= exit_deadline,
            "daemon still running 10s after SIGTERM"
        );
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
