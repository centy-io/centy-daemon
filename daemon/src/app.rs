use clap::Parser;
pub const FILE_DESCRIPTOR_SET: &[u8] = tonic::include_file_descriptor_set!("centy_descriptor");
pub const DEFAULT_ADDR: &str = "127.0.0.1:50051";
/// Address for the built-in MCP Streamable HTTP endpoint.
pub const DEFAULT_MCP_ADDR: &str = crate::mcp::DEFAULT_ADDR;
/// Centy Daemon - Local-first issue and documentation tracker service
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    /// Address to bind the server to
    #[arg(short, long, env = "CENTY_DAEMON_ADDR", default_value = DEFAULT_ADDR)]
    pub addr: String,
    /// Address to bind the built-in MCP Streamable HTTP endpoint (always loopback-only)
    #[arg(long, env = "CENTY_MCP_ADDR", default_value = DEFAULT_MCP_ADDR)]
    pub mcp_addr: String,
    /// Comma-separated list of allowed CORS origins.
    /// Use "*" to allow all origins (not recommended for production).
    /// Example: --cors-origins=https://app.centy.io,http://localhost:5180
    #[arg(
        long,
        env = "CENTY_CORS_ORIGINS",
        default_value = crate::cors::DEFAULT_CORS_ORIGINS,
        value_delimiter = ','
    )]
    pub cors_origins: Vec<String>,
    /// Launch the bundled Centy web app on localhost.
    #[arg(long, env = "CENTY_SERVE_WEB", default_value_t = false)]
    pub serve_web: bool,
    /// Address for the bundled Centy web app.
    #[arg(long, env = "CENTY_WEB_ADDR", default_value = "127.0.0.1:5180")]
    pub web_addr: String,
    /// Enable JSON log format (for production/log aggregation)
    #[arg(long, env = "CENTY_LOG_JSON", default_value = "false")]
    pub log_json: bool,
    /// Log rotation period: daily, hourly, or never
    #[arg(long, env = "CENTY_LOG_ROTATION", default_value = "daily")]
    pub log_rotation: String,
    /// Custom log directory (default: ~/.centy/logs)
    #[arg(long, env = "CENTY_LOG_DIR")]
    pub log_dir: Option<String>,
}
pub fn report_server_error(
    addr: std::net::SocketAddr,
    log_file: &std::path::Path,
    e: &tonic::transport::Error,
) {
    let err_string = format!("{e:?}");
    if err_string.contains("AddrInUse") {
        eprintln!();
        eprintln!("Error: Failed to start server - address {addr} is already in use");
        eprintln!();
        eprintln!("Another instance of centy-daemon may already be running.");
        eprintln!();
        eprintln!("Options:");
        eprintln!("  1. Kill the existing process:   pkill centy-daemon");
        eprintln!("  2. Use a different port:        centy-daemon --addr 127.0.0.1:50052");
        eprintln!("  3. Check what's using the port: lsof -i :{}", addr.port());
        eprintln!();
        eprintln!("Logs: {}", log_file.display());
        eprintln!();
    }
    eprintln!();
    eprintln!("Error: Failed to start server: {e}");
    eprintln!();
    eprintln!("Logs: {}", log_file.display());
    eprintln!();
}
