mod catalog;
mod grpc;
mod server;

use crate::server::ShutdownSignal;
use color_eyre::eyre::{bail, Result, WrapErr as _};
use std::net::SocketAddr;
use tokio::sync::watch;

pub const DEFAULT_ADDR: &str = "127.0.0.1:50052";

fn router(grpc_addr: SocketAddr) -> Result<axum::Router> {
    let service = server::McpServer::new(grpc::endpoint_for(grpc_addr))?;
    Ok(axum::Router::new().nest_service("/mcp", service.into_service()))
}

pub async fn spawn(
    bind_addr: &str,
    grpc_addr: SocketAddr,
    mut shutdown: watch::Receiver<ShutdownSignal>,
) -> Result<tokio::task::JoinHandle<Result<()>>> {
    let addr: SocketAddr = bind_addr.parse().wrap_err("invalid MCP bind address")?;
    if !addr.ip().is_loopback() {
        bail!("MCP endpoint must bind to a loopback address, got {addr}");
    }
    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .wrap_err_with(|| format!("failed to bind MCP endpoint on {addr}"))?;
    let router = router(grpc_addr)?;
    Ok(tokio::spawn(async move {
        axum::serve(listener, router)
            .with_graceful_shutdown(async move {
                loop {
                    if shutdown.changed().await.is_err()
                        || *shutdown.borrow() != ShutdownSignal::None
                    {
                        break;
                    }
                }
            })
            .await
            .wrap_err("MCP HTTP server failed")
    }))
}

#[cfg(test)]
mod tests {
    use super::router;
    use crate::server::proto::centy_daemon_server::CentyDaemonServer;
    use crate::server::{CentyDaemonService, ShutdownSignal};
    use std::net::SocketAddr;
    use std::sync::Arc;
    use tokio::io::{AsyncReadExt as _, AsyncWriteExt as _};
    use tokio::sync::watch;
    use tonic::transport::Server;

    const BODY_TOOLS_LIST: &str = r#"{"jsonrpc":"2.0","id":1,"method":"tools/list"}"#;
    const BODY_GET_DAEMON_INFO: &str = r#"{"jsonrpc":"2.0","id":2,"method":"tools/call","params":{"name":"centy_v1_CentyDaemon_GetDaemonInfo","arguments":{}}}"#;
    const BODY_INITIALIZE: &str = r#"{"jsonrpc":"2.0","id":0,"method":"initialize","params":{"protocolVersion":"2025-06-18","capabilities":{},"clientInfo":{"name":"test","version":"0"}}}"#;

    async fn post(addr: SocketAddr, body: &str) -> String {
        let mut stream = tokio::net::TcpStream::connect(addr).await.unwrap();
        let request = format!(
            "POST /mcp HTTP/1.1\r\nHost: 127.0.0.1\r\ncontent-type: application/json\r\naccept: application/json, text/event-stream\r\ncontent-length: {}\r\nconnection: close\r\n\r\n{body}",
            body.len()
        );
        stream.write_all(request.as_bytes()).await.unwrap();
        let mut response = String::new();
        stream.read_to_string(&mut response).await.unwrap();
        response
    }

    /// Starts a real daemon gRPC server on 127.0.0.1:0 and returns its address.
    async fn spawn_grpc() -> SocketAddr {
        let probe = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = probe.local_addr().unwrap();
        drop(probe);
        let (tx, _rx) = watch::channel(ShutdownSignal::None);
        let service = CentyDaemonService::new_for_tests(Arc::new(tx));
        tokio::spawn(async move {
            Server::builder()
                .add_service(CentyDaemonServer::new(service))
                .serve(addr)
                .await
        });
        // Give the server a moment to bind before the first call.
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;
        addr
    }

    #[tokio::test]
    async fn stateless_post_works_without_session_header() {
        let grpc_addr = spawn_grpc().await;
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        let app = router(grpc_addr).unwrap();
        tokio::spawn(async move { axum::serve(listener, app).await });

        let init = post(addr, BODY_INITIALIZE).await;
        assert!(
            init.starts_with("HTTP/1.1 200"),
            "initialize failed: {init}"
        );
        assert!(!init.to_ascii_lowercase().contains("mcp-session-id"));

        // Two independent requests, no session header, both must return tools.
        for () in [(); 2] {
            let list = post(addr, BODY_TOOLS_LIST).await;
            assert!(
                list.starts_with("HTTP/1.1 200"),
                "tools/list failed: {list}"
            );
            assert!(list.contains("\"tools\""), "no tools in response: {list}");
        }

        // tools/call goes through the real gRPC server and must not hang.
        let call = tokio::time::timeout(
            std::time::Duration::from_secs(10),
            post(addr, BODY_GET_DAEMON_INFO),
        )
        .await
        .expect("tools/call timed out");
        assert!(
            call.starts_with("HTTP/1.1 200"),
            "tools/call failed: {call}"
        );
        assert!(call.contains("\"version\""), "no version in result: {call}");
    }
}
