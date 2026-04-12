use http::Method;
use tower_http::cors::{AllowOrigin, Any, CorsLayer};

pub const DEFAULT_CORS_ORIGINS: &str = "http://localhost,https://localhost,http://127.0.0.1,https://127.0.0.1,tauri://localhost,https://tauri.localhost";

/// Build a CORS layer for gRPC-Web with the given allowed origins.
///
/// Always allows *.centy.io origins, plus any configured origins.
/// Pass "*" in the origins list to allow all origins (not recommended for production).
pub fn build_cors_layer(cors_origins: Vec<String>) -> CorsLayer {
    let allow_all_origins = cors_origins.iter().any(|o| o == "*");

    CorsLayer::new()
        .allow_origin(AllowOrigin::predicate(move |origin, _| {
            if allow_all_origins {
                return true;
            }

            if let Ok(origin_str) = origin.to_str() {
                // Always allow *.centy.io
                if origin_str.ends_with(".centy.io")
                    || origin_str == "https://centy.io"
                    || origin_str == "http://centy.io"
                {
                    return true;
                }

                // Check configured origins
                cors_origins
                    .iter()
                    .any(|allowed| origin_str.starts_with(allowed))
            } else {
                false
            }
        }))
        .allow_methods([Method::GET, Method::POST, Method::OPTIONS])
        .allow_headers(Any)
        .expose_headers(Any)
}
