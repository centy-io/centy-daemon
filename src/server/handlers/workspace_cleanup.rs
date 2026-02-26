use crate::server::proto::{CleanupExpiredWorkspacesRequest, CleanupExpiredWorkspacesResponse};
use tonic::{Response, Status};

/// TTL-based cleanup has been removed. Returns success with zero cleaned workspaces.
#[allow(unknown_lints, clippy::unused_async)]
pub async fn cleanup_expired_workspaces(
    _req: CleanupExpiredWorkspacesRequest,
) -> Result<Response<CleanupExpiredWorkspacesResponse>, Status> {
    Ok(Response::new(CleanupExpiredWorkspacesResponse {
        success: true,
        error: String::new(),
        cleaned_count: 0,
        cleaned_paths: vec![],
        failed_paths: vec![],
    }))
}
