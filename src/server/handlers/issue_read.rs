use std::path::Path;

use crate::registry::track_project_async;
use crate::server::proto::{GetNextIssueNumberRequest, GetNextIssueNumberResponse};
use crate::server::structured_error::StructuredError;
use crate::utils::get_centy_path;
use tonic::{Response, Status};

pub async fn get_next_issue_number(
    req: GetNextIssueNumberRequest,
) -> Result<Response<GetNextIssueNumberResponse>, Status> {
    track_project_async(req.project_path.clone());
    let project_path = Path::new(&req.project_path);
    let issues_path = get_centy_path(project_path).join("issues");

    #[allow(deprecated)]
    match crate::item::entities::issue::create::get_next_issue_number(&issues_path).await {
        Ok(issue_number) => Ok(Response::new(GetNextIssueNumberResponse {
            issue_number,
            success: true,
            error: String::new(),
        })),
        Err(e) => Ok(Response::new(GetNextIssueNumberResponse {
            success: false,
            error: StructuredError::new(&req.project_path, "IO_ERROR", e.to_string()).to_json(),
            issue_number: String::new(),
        })),
    }
}
