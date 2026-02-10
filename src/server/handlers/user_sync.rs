use std::path::Path;

use crate::registry::track_project_async;
use crate::server::convert_infra::manifest_to_proto;
use crate::server::proto::{
    GitContributor as ProtoGitContributor, SyncUsersRequest, SyncUsersResponse,
};
use crate::server::structured_error::to_error_json;
use crate::user::sync_users as internal_sync_users;
use tonic::{Response, Status};

pub async fn sync_users(req: SyncUsersRequest) -> Result<Response<SyncUsersResponse>, Status> {
    track_project_async(req.project_path.clone());
    let project_path = Path::new(&req.project_path);

    match internal_sync_users(project_path, req.dry_run).await {
        Ok(full_result) => {
            let result = full_result.result;
            Ok(Response::new(SyncUsersResponse {
                success: true,
                error: String::new(),
                created: result.created,
                skipped: result.skipped,
                errors: result.errors,
                would_create: result
                    .would_create
                    .into_iter()
                    .map(|c| ProtoGitContributor {
                        name: c.name,
                        email: c.email,
                    })
                    .collect(),
                would_skip: result
                    .would_skip
                    .into_iter()
                    .map(|c| ProtoGitContributor {
                        name: c.name,
                        email: c.email,
                    })
                    .collect(),
                manifest: Some(manifest_to_proto(&full_result.manifest)),
            }))
        }
        Err(e) => Ok(Response::new(SyncUsersResponse {
            success: false,
            error: to_error_json(&req.project_path, &e),
            created: vec![],
            skipped: vec![],
            errors: vec![],
            would_create: vec![],
            would_skip: vec![],
            manifest: None,
        })),
    }
}
