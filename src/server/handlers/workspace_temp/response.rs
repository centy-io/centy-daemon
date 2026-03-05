use crate::server::proto::OpenInTempWorkspaceResponse;
use tonic::Response;

pub(super) fn err_response(
    error: String,
    issue_id: String,
    dn: u32,
    req_cfg: bool,
) -> Response<OpenInTempWorkspaceResponse> {
    Response::new(OpenInTempWorkspaceResponse {
        success: false,
        error,
        workspace_path: String::new(),
        issue_id,
        display_number: dn,
        expires_at: String::new(),
        editor_opened: false,
        requires_status_config: req_cfg,
        workspace_reused: false,
        original_created_at: String::new(),
    })
}
