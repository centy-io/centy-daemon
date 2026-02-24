use std::path::Path;

use tonic::{Response, Status};

use crate::hooks::history::{
    get_hook_execution, list_hook_executions, HookExecutionFilter, HookExecutionRecord,
};
use crate::server::proto::{
    GetHookExecutionRequest, GetHookExecutionResponse, HookExecution,
    ListHookExecutionsRequest, ListHookExecutionsResponse,
};

fn record_to_proto(r: HookExecutionRecord) -> HookExecution {
    HookExecution {
        id: r.id,
        timestamp: r.timestamp.to_rfc3339(),
        hook_pattern: r.hook_pattern,
        command: r.command,
        exit_code: r.exit_code.unwrap_or(-1),
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
    let filter = HookExecutionFilter {
        phase: if req.phase.is_empty() {
            None
        } else {
            Some(req.phase)
        },
        item_type: if req.item_type.is_empty() {
            None
        } else {
            Some(req.item_type)
        },
        operation: if req.operation.is_empty() {
            None
        } else {
            Some(req.operation)
        },
        item_id: if req.item_id.is_empty() {
            None
        } else {
            Some(req.item_id)
        },
        limit: if req.limit == 0 {
            None
        } else {
            Some(req.limit)
        },
    };

    let records = list_hook_executions(project_path, &filter).await;
    let executions = records.into_iter().map(record_to_proto).collect();

    Ok(Response::new(ListHookExecutionsResponse { executions }))
}

pub async fn get_hook_execution_handler(
    req: GetHookExecutionRequest,
) -> Result<Response<GetHookExecutionResponse>, Status> {
    let project_path = Path::new(&req.project_path);
    match get_hook_execution(project_path, &req.execution_id).await {
        Some(record) => Ok(Response::new(GetHookExecutionResponse {
            found: true,
            execution: Some(record_to_proto(record)),
        })),
        None => Ok(Response::new(GetHookExecutionResponse {
            found: false,
            execution: None,
        })),
    }
}
