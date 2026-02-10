use crate::server::proto::{CleanupExpiredWorkspacesRequest, CleanupExpiredWorkspacesResponse};
use crate::server::structured_error::to_error_json;
use crate::workspace::cleanup_expired_workspaces as internal_cleanup_expired;
use tonic::{Response, Status};

pub async fn cleanup_expired_workspaces(
    _req: CleanupExpiredWorkspacesRequest,
) -> Result<Response<CleanupExpiredWorkspacesResponse>, Status> {
    match internal_cleanup_expired().await {
        Ok(results) => {
            let cleaned_count = results.iter().filter(|r| r.error.is_none()).count() as u32;
            let cleaned_paths: Vec<String> = results
                .iter()
                .filter(|r| r.error.is_none())
                .map(|r| r.workspace_path.clone())
                .collect();
            let failed_paths: Vec<String> = results
                .iter()
                .filter(|r| r.error.is_some())
                .map(|r| r.workspace_path.clone())
                .collect();

            Ok(Response::new(CleanupExpiredWorkspacesResponse {
                success: true,
                error: String::new(),
                cleaned_count,
                cleaned_paths,
                failed_paths,
            }))
        }
        Err(e) => Ok(Response::new(CleanupExpiredWorkspacesResponse {
            success: false,
            error: to_error_json("", &e),
            cleaned_count: 0,
            cleaned_paths: vec![],
            failed_paths: vec![],
        })),
    }
}
