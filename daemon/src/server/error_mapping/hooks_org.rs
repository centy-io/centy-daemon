use super::ToStructuredError;
impl ToStructuredError for crate::hooks::HookError {
    fn error_code_and_tip(&self) -> (&str, Option<&str>) {
        use crate::hooks::HookError;
        match self {
            HookError::PreHookFailed { .. } => ("HOOK_PRE_FAILED", None),
            HookError::Timeout { .. } => ("HOOK_TIMEOUT", None),
            HookError::ExecutionError(_) => ("HOOK_EXECUTION_ERROR", None),
            HookError::InvalidPattern(_) => ("HOOK_INVALID_PATTERN", None),
            HookError::IoError(_) => ("IO_ERROR", None),
            HookError::JsonError(_) => ("JSON_ERROR", None),
        }
    }
}
