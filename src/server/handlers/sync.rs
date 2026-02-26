use crate::server::proto;
use crate::server::structured_error::StructuredError;
use tonic::{Response, Status};

#[allow(unknown_lints, clippy::unused_async)]
pub async fn list_sync_conflicts(
    _req: proto::ListSyncConflictsRequest,
) -> Result<Response<proto::ListSyncConflictsResponse>, Status> {
    // Sync feature removed - return empty list
    Ok(Response::new(proto::ListSyncConflictsResponse {
        conflicts: vec![],
        success: true,
        error: String::new(),
    }))
}

#[allow(unknown_lints, clippy::unused_async)]
pub async fn get_sync_conflict(
    req: proto::GetSyncConflictRequest,
) -> Result<Response<proto::GetSyncConflictResponse>, Status> {
    // Sync feature removed - conflict not found
    Ok(Response::new(proto::GetSyncConflictResponse {
        conflict: None,
        success: false,
        error: StructuredError::new(
            "",
            "SYNC_DISABLED",
            format!(
                "Sync feature disabled. Conflict not found: {}",
                req.conflict_id
            ),
        )
        .to_json(),
    }))
}

#[allow(unknown_lints, clippy::unused_async)]
pub async fn resolve_sync_conflict(
    _req: proto::ResolveSyncConflictRequest,
) -> Result<Response<proto::ResolveSyncConflictResponse>, Status> {
    // Sync feature removed - cannot resolve conflicts
    Ok(Response::new(proto::ResolveSyncConflictResponse {
        success: false,
        error: StructuredError::new("", "SYNC_DISABLED", "Sync feature is disabled".to_string())
            .to_json(),
    }))
}

#[allow(unknown_lints, clippy::unused_async)]
pub async fn get_sync_status(
    _req: proto::GetSyncStatusRequest,
) -> Result<Response<proto::GetSyncStatusResponse>, Status> {
    // Sync feature removed - return disabled status
    Ok(Response::new(proto::GetSyncStatusResponse {
        mode: proto::SyncMode::Disabled as i32,
        has_pending_changes: false,
        has_pending_push: false,
        conflict_count: 0,
        last_sync_time: String::new(),
        success: true,
        error: String::new(),
    }))
}

#[allow(unknown_lints, clippy::unused_async)]
pub async fn sync_pull(
    _req: proto::SyncPullRequest,
) -> Result<Response<proto::SyncPullResponse>, Status> {
    // Sync feature removed - no-op success
    Ok(Response::new(proto::SyncPullResponse {
        success: true,
        error: String::new(),
        had_changes: false,
        conflict_files: vec![],
    }))
}

#[allow(unknown_lints, clippy::unused_async)]
pub async fn sync_push(
    _req: proto::SyncPushRequest,
) -> Result<Response<proto::SyncPushResponse>, Status> {
    // Sync feature removed - no-op success
    Ok(Response::new(proto::SyncPushResponse {
        success: true,
        error: String::new(),
        had_changes: false,
    }))
}
