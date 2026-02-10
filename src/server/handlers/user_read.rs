use std::path::Path;

use crate::registry::track_project_async;
use crate::server::convert_entity::user_to_proto;
use crate::server::proto::{GetUserRequest, GetUserResponse, ListUsersRequest, ListUsersResponse};
use crate::server::structured_error::to_error_json;
use crate::user::{get_user as internal_get_user, list_users as internal_list_users};
use tonic::{Response, Status};

pub async fn get_user(req: GetUserRequest) -> Result<Response<GetUserResponse>, Status> {
    track_project_async(req.project_path.clone());
    let project_path = Path::new(&req.project_path);

    match internal_get_user(project_path, &req.user_id).await {
        Ok(user) => Ok(Response::new(GetUserResponse {
            success: true,
            error: String::new(),
            user: Some(user_to_proto(&user)),
        })),
        Err(e) => Ok(Response::new(GetUserResponse {
            success: false,
            error: to_error_json(&req.project_path, &e),
            user: None,
        })),
    }
}

pub async fn list_users(req: ListUsersRequest) -> Result<Response<ListUsersResponse>, Status> {
    track_project_async(req.project_path.clone());
    let project_path = Path::new(&req.project_path);

    let filter = if req.git_username.is_empty() {
        None
    } else {
        Some(req.git_username.as_str())
    };

    match internal_list_users(project_path, filter, false).await {
        Ok(users) => {
            let total_count = users.len() as i32;
            Ok(Response::new(ListUsersResponse {
                users: users.iter().map(user_to_proto).collect(),
                total_count,
                success: true,
                error: String::new(),
            }))
        }
        Err(e) => Ok(Response::new(ListUsersResponse {
            success: false,
            error: to_error_json(&req.project_path, &e),
            users: vec![],
            total_count: 0,
        })),
    }
}
