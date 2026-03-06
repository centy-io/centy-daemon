use super::issue::create_update_issue;
use crate::github_releases::{fetch_releases, is_update_available, latest_stable_version};
use crate::server::proto::{
    CheckForUpdatesRequest, CheckForUpdatesResponse, GitHubRelease as ProtoRelease,
};
use crate::server::structured_error::StructuredError;
use crate::utils::CENTY_VERSION;
use tonic::{Response, Status};

pub async fn check_for_updates(
    req: CheckForUpdatesRequest,
) -> Result<Response<CheckForUpdatesResponse>, Status> {
    let releases = match fetch_releases().await {
        Ok(r) => r,
        Err(e) => {
            return Ok(error_response(&format!("Failed to fetch releases: {e}")));
        }
    };

    let latest = latest_stable_version(&releases).unwrap_or_default();
    let update_available = !latest.is_empty() && is_update_available(CENTY_VERSION, &latest);
    let proto_releases: Vec<ProtoRelease> = releases.iter().map(to_proto_release).collect();

    let (issue_created, issue_id) =
        if update_available && req.create_issue && !req.project_path.is_empty() {
            create_update_issue(&req.project_path, &latest).await
        } else {
            (false, String::new())
        };

    Ok(Response::new(CheckForUpdatesResponse {
        success: true,
        error: String::new(),
        current_version: CENTY_VERSION.to_string(),
        latest_version: latest,
        update_available,
        releases: proto_releases,
        issue_created,
        issue_id,
    }))
}

fn error_response(message: &str) -> Response<CheckForUpdatesResponse> {
    let se = StructuredError::new("", "GITHUB_API_ERROR", message.to_string());
    Response::new(CheckForUpdatesResponse {
        success: false,
        error: se.to_json(),
        current_version: CENTY_VERSION.to_string(),
        latest_version: String::new(),
        update_available: false,
        releases: vec![],
        issue_created: false,
        issue_id: String::new(),
    })
}

fn to_proto_release(r: &crate::github_releases::GitHubRelease) -> ProtoRelease {
    ProtoRelease {
        tag_name: r.tag_name.clone(),
        name: r.name.clone().unwrap_or_default(),
        published_at: r.published_at.clone().unwrap_or_default(),
        prerelease: r.prerelease,
        html_url: r.html_url.clone(),
    }
}
