#![allow(clippy::unwrap_used, clippy::expect_used, clippy::indexing_slicing)]

mod common;

use centy_daemon::server::handlers::check_for_updates::check_for_updates;
use centy_daemon::server::proto::CheckForUpdatesRequest;
use common::create_test_dir;
use tokio::io::{AsyncReadExt as _, AsyncWriteExt as _};
use tokio::net::TcpListener;

/// Serve `body` once as JSON over HTTP on a loopback port; return its URL.
async fn mock_releases(body: &'static str) -> String {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let url = format!("http://{}/releases", listener.local_addr().unwrap());
    tokio::spawn(async move {
        let (mut sock, _) = listener.accept().await.unwrap();
        let mut buf = [0u8; 4096];
        let _read = sock.read(&mut buf).await.unwrap();
        let resp = format!(
            "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
            body.len()
        );
        sock.write_all(resp.as_bytes()).await.unwrap();
    });
    url
}

// One test fn: CENTY_RELEASES_URL is process-global, so cases run in sequence.
#[tokio::test]
async fn test_check_for_updates_against_mock_github() {
    let _home = create_test_dir(); // isolates CENTY_HOME

    // Newer stable release behind a newer prerelease: prerelease is skipped.
    let url = mock_releases(
        r#"[
          {"tag_name":"v100.0.0-rc.1","name":"RC","published_at":"2099-02-01T00:00:00Z","prerelease":true,"html_url":"https://example.com/rc"},
          {"tag_name":"v99.0.0","name":"Big","published_at":"2099-01-01T00:00:00Z","prerelease":false,"html_url":"https://example.com/99"}
        ]"#,
    )
    .await;
    std::env::set_var("CENTY_RELEASES_URL", &url);
    let resp = check_for_updates(CheckForUpdatesRequest {})
        .await
        .unwrap()
        .into_inner();
    assert!(resp.success, "error: {}", resp.error);
    assert_eq!(resp.current_version, env!("CARGO_PKG_VERSION"));
    assert_eq!(resp.latest_version, "99.0.0");
    assert!(resp.update_available);
    assert_eq!(resp.releases.len(), 2);
    assert_eq!(resp.releases[0].tag_name, "v100.0.0-rc.1");
    assert!(resp.releases[0].prerelease);
    assert_eq!(resp.releases[1].html_url, "https://example.com/99");

    // Only an older release: no update.
    let url = mock_releases(
        r#"[{"tag_name":"v0.0.1","name":null,"published_at":null,"prerelease":false,"html_url":"https://example.com/old"}]"#,
    )
    .await;
    std::env::set_var("CENTY_RELEASES_URL", &url);
    let resp = check_for_updates(CheckForUpdatesRequest {})
        .await
        .unwrap()
        .into_inner();
    assert!(resp.success);
    assert_eq!(resp.latest_version, "0.0.1");
    assert!(!resp.update_available);

    // Unreachable endpoint: success=false with a structured error.
    let dead = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let dead_url = format!("http://{}/releases", dead.local_addr().unwrap());
    drop(dead);
    std::env::set_var("CENTY_RELEASES_URL", dead_url);
    let resp = check_for_updates(CheckForUpdatesRequest {})
        .await
        .unwrap()
        .into_inner();
    assert!(!resp.success);
    assert!(resp.error.contains("GITHUB_API_ERROR"), "{}", resp.error);
    assert!(!resp.update_available);
}
