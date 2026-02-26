#![allow(clippy::all, clippy::pedantic, clippy::restriction)]
/// Apply variable renaming to a single line to replace `self.xxx` references
/// with function parameters.
fn rename_vars(s: &str) -> String {
    s.replace("accept_compression_encodings", "ace")
        .replace("send_compression_encodings", "sce")
        .replace("max_decoding_message_size", "mdms")
        .replace("max_encoding_message_size", "mems")
}

/// Return true if the line is one of the self.xxx binding lines we remove.
fn is_self_binding(trimmed: &str) -> bool {
    trimmed.starts_with("let accept_compression_encodings")
        || trimmed.starts_with("let send_compression_encodings")
        || trimmed.starts_with("let max_decoding_message_size")
        || trimmed.starts_with("let max_encoding_message_size")
        || trimmed.starts_with("let inner = self.inner.clone()")
}

/// Build a free RPC handler function from a match arm body.
/// `arm_body` includes the opening `{` (first) and closing `}` (last) lines.
pub fn build_handler_fn(fn_name: &str, arm_body: &[String]) -> Vec<String> {
    let mut lines = Vec::new();
    lines.push(format!(
        "pub fn {fn_name}<T: CentyDaemon, B: Body + std::marker::Send + 'static>(\
inner: Arc<T>, ace: EnabledCompressionEncodings, sce: EnabledCompressionEncodings, \
mdms: Option<usize>, mems: Option<usize>, req: http::Request<B>,\
) -> BoxFuture<http::Response<tonic::body::BoxBody>, std::convert::Infallible>"
    ));
    lines.push("where B::Error: Into<StdError> + std::marker::Send + 'static {".to_string());
    for raw in &arm_body[1..arm_body.len().saturating_sub(1)] {
        let t = raw.trim();
        if is_self_binding(t) {
            continue;
        }
        lines.push(rename_vars(raw));
    }
    lines.push("}".to_string());
    lines
}

/// Build the default RPC handler function.
pub fn build_default_fn(fn_name: &str, arm_body: &[String]) -> Vec<String> {
    let mut lines = Vec::new();
    lines.push(format!(
        "pub fn {fn_name}<B: Body + std::marker::Send + 'static>(\
req: http::Request<B>,\
) -> BoxFuture<http::Response<tonic::body::BoxBody>, std::convert::Infallible>"
    ));
    lines.push("where B::Error: Into<StdError> + std::marker::Send + 'static {".to_string());
    for raw in &arm_body[1..arm_body.len().saturating_sub(1)] {
        lines.push(raw.clone());
    }
    lines.push("}".to_string());
    lines
}

/// Convert an RPC path like `/centy.v1.CentyDaemon/Init` to snake_case.
pub fn path_to_fn_name(path: &str, idx: usize) -> String {
    let method = path.split('/').last().unwrap_or("unknown");
    let snake: String = method
        .chars()
        .enumerate()
        .flat_map(|(i, c)| {
            if c.is_uppercase() && i > 0 {
                vec!['_', c.to_ascii_lowercase()]
            } else {
                vec![c.to_ascii_lowercase()]
            }
        })
        .collect();
    format!("rpc_handler_{idx}_{snake}")
}
