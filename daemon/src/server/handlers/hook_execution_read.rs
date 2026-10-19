use std::path::Path;

use crate::hooks::history::{
    get_hook_execution, list_hook_executions, HookExecutionFilter, HookExecutionRecord,
};
use crate::server::assert_service::assert_initialized;
use crate::server::proto::{
    GetHookExecutionRequest, GetHookExecutionResponse, HookExecution, ListHookExecutionsRequest,
    ListHookExecutionsResponse,
};
use crate::server::structured_error::to_error_json;
use tonic::{Response, Status};

fn record_to_proto(r: HookExecutionRecord) -> HookExecution {
    HookExecution {
        id: r.id,
        timestamp: r.timestamp.to_rfc3339(),
        hook_pattern: r.hook_pattern,
        command: r.command,
        exit_code: r.exit_code.unwrap_or(-1i32),
        stdout: r.stdout,
        stderr: r.stderr,
        duration_ms: r.duration_ms,
        blocked_operation: r.blocked_operation,
        phase: r.phase,
        item_type: r.item_type,
        operation: r.operation,
        item_id: r.item_id.unwrap_or_default(),
        timed_out: r.timed_out,
    }
}

pub async fn list_hook_executions_handler(
    req: ListHookExecutionsRequest,
) -> Result<Response<ListHookExecutionsResponse>, Status> {
    let project_path = Path::new(&req.project_path);
    if let Err(e) = assert_initialized(project_path) {
        return Ok(Response::new(ListHookExecutionsResponse {
            success: false,
            error: to_error_json(&req.project_path, &e),
            ..Default::default()
        }));
    }
    let filter = HookExecutionFilter {
        phase: req.phase,
        item_type: req.item_type,
        operation: req.operation,
        item_id: req.item_id,
        limit: usize::try_from(req.limit).unwrap_or(usize::MAX),
    };
    let executions = list_hook_executions(project_path, &filter)
        .await
        .into_iter()
        .map(record_to_proto)
        .collect();
    Ok(Response::new(ListHookExecutionsResponse {
        success: true,
        error: String::new(),
        executions,
    }))
}

pub async fn get_hook_execution_handler(
    req: GetHookExecutionRequest,
) -> Result<Response<GetHookExecutionResponse>, Status> {
    let project_path = Path::new(&req.project_path);
    if let Err(e) = assert_initialized(project_path) {
        return Ok(Response::new(GetHookExecutionResponse {
            success: false,
            error: to_error_json(&req.project_path, &e),
            ..Default::default()
        }));
    }
    let record = get_hook_execution(project_path, &req.execution_id).await;
    Ok(Response::new(GetHookExecutionResponse {
        success: true,
        error: String::new(),
        found: record.is_some(),
        execution: record.map(record_to_proto),
    }))
}
